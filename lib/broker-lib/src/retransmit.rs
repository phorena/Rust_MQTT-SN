use core::fmt::Debug;
use core::hash::Hash;
use hashbrown::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::thread;

//use bytes::{BufMut, Bytes, BytesMut};
use log::*;

use chrono::{Datelike, Local, Timelike};
// use crossbeam::channel::{bounded, unbounded, Receiver, Sender};
use std::time::Duration;
use trace_var::trace_var;

use crate::{
    broker_lib::MqttSnClient, connection::Connection, eformat, function,
};

#[derive(Debug, Clone, Hash)]
pub struct KeepAliveKey {
    addr: SocketAddr,
}

#[derive(Debug, Clone)]
pub struct KeepAliveVal {
    latest_counter: usize,
    conn_duration: u16,
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

static SLEEP_DURATION: usize = 100; // in millisec
static MAX_SLOT: usize = (1000 / SLEEP_DURATION) * 64 * 2;

lazy_static! {
    static ref CUR_COUNTER: Mutex<u64> = Mutex::new(0);
    static ref SLOT_VEC: Mutex<Vec<Slot>> =
        Mutex::new(Vec::with_capacity(MAX_SLOT));
    static ref TIME_WHEEL_MAP: Mutex<HashMap<SocketAddr, KeepAliveVal>> =
        Mutex::new(HashMap::new());
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
// static TIME_WHEEL_DEFAULT_DURATION_MS: usize = 300;

#[derive(Debug, Clone)]
pub struct KeepAliveTimeWheel {
    max_slot: usize,       // (1000 / sleep_duration) * 64 * 2;
    sleep_duration: usize, // in milli seconds
    cur_counter: Arc<Mutex<usize>>,
    slot_vec: Vec<Slot>,
    hash: Arc<Mutex<HashMap<SocketAddr, KeepAliveVal>>>,
}

impl KeepAliveTimeWheel {
    pub fn new() -> Self {
        // verify sleep_duration (milli seconds) 1..1000.
        let sleep_duration = 1000; // 1 sec
                                   // let max_slot = (1000 / sleep_duration) * 64 * 2;
        let max_slot = 16;
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
            cur_counter: Arc::new(Mutex::new(0)),
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
    pub fn schedule(
        &mut self,
        key: SocketAddr,
        conn_duration: u16,
    ) -> Result<(), String> {
        // store the key in a slot of the timing wheel
        let cur_counter = *self.cur_counter.lock().unwrap();
        let index = (cur_counter + conn_duration as usize) % self.max_slot;
        let slot = &self.slot_vec[index];
        // TODO replace unwrap
        let mut hash = self.hash.lock().unwrap();
        let mut vec = slot.entries.lock().unwrap();
        let val = KeepAliveVal {
            latest_counter: index,
            conn_duration,
        };
        hash.insert(key, val);
        vec.push(key);
        dbg!(index);
        dbg!(vec);
        dbg!(hash);
        return Ok(());
    }

    #[inline(always)]
    #[trace_var(index, slot, hash, vec)]
    pub fn reschedule(
        &mut self,
        socket_addr: SocketAddr,
    ) -> Result<(), String> {
        let mut hash = self.hash.lock().unwrap();
        match hash.get_mut(&socket_addr) {
            Some(conn) => {
                dbg!(&conn);
                dbg!(&self.cur_counter);
                conn.latest_counter = *self.cur_counter.lock().unwrap();
                dbg!(&conn);
                Ok(())
            }
            None => Err(eformat!(socket_addr, "not found.")),
        }
    }

    pub fn run(self, client: MqttSnClient) {
        // When the keep_alive timing wheel entry is accessed,
        // this code determines if the connection is expired.
        // If the hash entry has been updated to a new counter,
        // then reschedule the connection in the timing wheel.
        //
        let _keep_alive_expire_thread = thread::spawn(move || {
            let sleep_duration = self.sleep_duration as u64;
            loop {
                let cur_counter = *self.cur_counter.lock().unwrap();
                let cur_slot = &self.slot_vec[cur_counter % self.max_slot];
                // dbg!(&cur_slot);
                dbg!(&cur_counter);
                match cur_slot.entries.lock() {
                    Ok(mut slot) => {
                        // iterate all client in the slot
                        while let Some(socket_addr) = slot.pop() {
                            dbg!(&socket_addr);
                            let mut hash_entry_lock = self.hash.lock().unwrap();
                            if let Some(conn) =
                                hash_entry_lock.get(&socket_addr)
                            {
                                dbg!(&conn);
                                let new_counter = conn.latest_counter as usize
                                    + conn.conn_duration as usize;
                                if new_counter > cur_counter {
                                    // not expired, reschedule
                                    // The new duration starts from the latest_counter,
                                    // not the cur_counter. Subtract cur_counter is needed.
                                    let index = new_counter % self.max_slot;

                                    let slot = &self.slot_vec[index];
                                    // TODO replace unwrap
                                    let mut vec = slot.entries.lock().unwrap();
                                    vec.push(socket_addr);
                                    dbg!(index);
                                    dbg!(vec);
                                } else {
                                    // Client timeout, move from ACTIVE to LOST state.
                                    // MQTT-SN 1.2 spec page 25
                                    // The entry was pop() from the timing wheel slot.
                                    //    client_reschedule.set_state(STATE_LOST);
                                    // Remove it from the hashmap.
                                    // TODO XXX change connection state to LOST.

                                    // remove socket_add from keep alive HashMap
                                    if let Some(conn) =
                                        hash_entry_lock.remove(&socket_addr)
                                    {
                                        dbg!(&conn);
                                        match Connection::remove(socket_addr) {
                                            Ok(conn) => {
                                                let _result =
                                                    conn.publish_will(&client);
                                            }
                                            Err(_) => {
                                                // TODO
                                            }
                                        }
                                        dbg!(&self.cur_counter);
                                    }
                                    info!(
                                        "Connection Timeout: {:?}",
                                        socket_addr
                                    );
                                }
                            }
                        }
                    }
                    Err(_) => {}
                }
                *self.cur_counter.lock().unwrap() += 1;

                // The sleep() has to be outside of the mutex lock block for
                // the lock to be unlocked while the thread is sleeping.
                thread::sleep(Duration::from_millis(sleep_duration));
            }
        });
    }
}
