use core::fmt::Debug;
use core::hash::Hash;
use hashbrown::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::{hint, thread};

use bytes::{BufMut, Bytes, BytesMut};
use log::*;
use simplelog::*;

use chrono::{Datelike, Local, Timelike};
use crossbeam::channel::{bounded, unbounded, Receiver, Sender};
use std::time::Duration;
use trace_var::trace_var;

#[derive(Debug, Clone, Hash)]
pub struct KeepAliveKey {
    addr: SocketAddr,
}

#[derive(Debug, Clone)]
pub struct KeepAliveVal {
    conn_duration: u16,
    latest_counter: u32,
}

lazy_static! {
    pub static ref KEEP_ALIVE_VAL_MAP: Mutex<HashMap<SocketAddr, KeepAliveVal>> =
        Mutex::new(HashMap::new());
    pub static ref KEEP_ALIVE_TIME_WHEEL: Vec<Mutex<Vec<KeepAliveKey>>> =
        Vec::new();
}

#[derive(Debug, Clone)]
struct Slot {
    pub entries: Arc<Mutex<Vec<SocketAddr>>>,
}

impl Slot {
    pub fn new() -> Self {
        Slot {
            entries: Arc::new(Mutex::new(Vec::new())),
        }
    }
}
// clients use 100 milli seconds
// brokers use 10 milli seconds
/// static TIME_WHEEL_SLEEP_DURATION:usize = 100; // in milli seconds
// For 64 seconds at 100 ms sleeping interval, we need 10 * 64 slots, (1000/100 = 10)
// For 64 seconds at 10 ms sleeping interval, we need 100 * 64 slots, (1000/10 = 100)

// The maximum timeout duration is 64 seconds
// The last one might be over 64, rounding up.
/// static TIME_WHEEL_MAX_SLOTS:usize = (1000 / TIME_WHEEL_SLEEP_DURATION) * 64 * 2;
// Initial timeout duration is 300 ms
static TIME_WHEEL_DEFAULT_DURATION_MS: usize = 300;

#[derive(Debug, Clone)]
struct KeepAliveTimeWheel {
    max_slot: usize,       // (1000 / sleep_duration) * 64 * 2;
    sleep_duration: usize, // in milli seconds
    cur_counter: usize,
    slot_vec: Vec<Slot>,
    hash: Arc<Mutex<HashMap<SocketAddr, KeepAliveVal>>>,
}

impl KeepAliveTimeWheel {
    pub fn new() -> Self {
        // verify sleep_duration (milli seconds) 1..1000.
        let sleep_duration = 500; // .5 sec
        let max_slot = (1000 / sleep_duration) * 64 * 2;
        let mut slot_vec: Vec<Slot> = Vec::with_capacity(max_slot);
        // verify default_duration (milli seconds) 1..1000.

        // Must use for loop to initialize
        // let slot_vec:Vec<Slot<KEY>> = vec![Vec<Slot<KEY>>, max_slot] didn'KEY work.
        // It allocates one Vec<Slot<KEY>> and all entries point to it.
        for _ in 0..max_slot {
            slot_vec.push(Slot::new());
            // slot_hash.push(Arc::new(Mutex::new(HashMap::new())));
        }
        KeepAliveTimeWheel {
            max_slot,
            sleep_duration,
            cur_counter: 0,
            slot_vec,
            // slot_hash,
            hash: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    // The initial duration is set to TIME_WHEEL_INIT_DURATION, but can be
    // changed to reflect the network the client is on, (LAN or WAN),
    // or the latency pattern.
    #[inline(always)]
    #[trace_var(index, slot, hash, vec)]
    fn schedule(
        &mut self,
        key: SocketAddr,
        val: KeepAliveVal,
        duration: usize,
    ) -> Result<(), String> {
        // store the key in a slot of the timing wheel
        let index = (self.cur_counter + duration) % self.max_slot;
        let slot = &self.slot_vec[index];
        // TODO replace unwrap
        let mut hash = self.hash.lock().unwrap();
        let mut vec = slot.entries.lock().unwrap();
        hash.insert(key, val);
        vec.push(key);
        dbg!(vec);
        dbg!(hash);
        return Ok(());
    }

    pub fn _run(mut self) {
        // When the keep_alive timing wheel entry is expired,
        // this code determines if the connection is expired.
        // If the hash entry has been updated to a new counter,
        // then reschedule the connection.
        let _keep_alive_expire_thread = thread::spawn(move || {
            let sleep_duration = self.sleep_duration as u64;
            loop {
                let cur_slot = &self.slot_vec[self.cur_counter % self.max_slot];
                self.cur_counter += 1;
                dbg!(&cur_slot);
                match cur_slot.entries.lock() {
                    Ok(mut slot) => {
                        while let Some(slot_entry) = slot.pop() {
                            dbg!(&slot_entry);
                            let hash_entry_lock = self.hash.lock().unwrap();
                            let hash_entry = hash_entry_lock.get(&slot_entry);
                            dbg!(&hash_entry);
                        }
                    }
                    Err(_) => {}
                }

                /*
                {
                    let mut wheel = keep_alive_time_wheel2.lock().unwrap();
                    let mut wheel2 = wheel.clone();
                    // get vec for current time slot
                    let slot = wheel.keep_alive_expire();
                    dbg_fn!(slot);
                    let cur_slot_lock = slot.entries.lock();
                    match cur_slot_lock {
                        Ok(mut slot) => {
                            dbg_fn!(&slot);
                            let cur_counter = wheel2.cur_counter;
                            // iterate entries in the vec
                            while let Some(top) = slot.pop() {
                                dbg_fn!(top);
                                // use the addr as the key to hashmap
                                let addr = top.0;
                                match wheel2.get_hash(addr) {
                                    Ok(conn) => {
                                        dbg_fn!(conn.clone());
                                        let conn = conn.lock().unwrap();
                                        let new_counter =
                                            conn.latest_counter as usize + conn.conn_duration as usize;
                                        if new_counter > cur_counter {
                                            // not expired, reschedule
                                            // The new duration starts from the latest_counter,
                                            // not the cur_counter. Subtract cur_counter is needed.
                                            let conn_duration = new_counter - wheel2.cur_counter;
                                            wheel2.reschedule(addr, conn_duration);
                                        } else {
                                            // Client timeout, move from ACTIVE to LOST state.
                                            // MQTT-SN 1.2 spec page 25
                                            // The entry was pop() from the timing wheel slot.
                                            client_reschedule.set_state(STATE_LOST);
                                            // Remove it from the hashmap.
                                            wheel2.cancel(addr);
                                            info!("Connection lost: {:?}", addr);
                                        }
                                    }
                                    Err(result) => {
                                        match result {
                                            None => {
                                                // entry doesn't exist
                                                error!("keep_alive get_hash(): None {:?}", addr);
                                            }
                                            Some((str, addr)) => {
                                                error!("{}:{:?}", str, addr);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        _ => (),
                    }
                }
                */
                // The sleep() has to be outside of the mutex lock block for
                // the lock to be unlocked while the thread is sleeping.
                thread::sleep(Duration::from_millis(sleep_duration));
            }
        });
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_keep_alive() {
        use std::net::SocketAddr;
        let socket = "127.0.0.1:1200".parse::<SocketAddr>().unwrap();
        let val = super::KeepAliveVal {
            conn_duration: 10,
            latest_counter: 20,
        };
        let mut tw = super::KeepAliveTimeWheel::new();
        tw.schedule(socket, val, 100);
    }
}
