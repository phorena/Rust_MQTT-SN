use crate::{
    broker_lib::MqttSnClient, connection::Connection, connection::StateEnum2,
    eformat, function,
};
use core::fmt::Debug;
use core::hash::Hash;
use hashbrown::HashMap;
use log::*;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use trace_var::trace_var;

#[derive(Debug, Clone, Hash)]
pub struct KeepAliveKey {
    addr: SocketAddr,
}

#[derive(Debug, Clone)]
struct KeepAliveVal {
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

static SLEEP_DURATION: usize = 100;
static MAX_SLOT: usize = (1000 / SLEEP_DURATION) * 64 * 2;

// TODO use lazy_static for easy access from any code without
// attaching to a structure.
lazy_static! {
    static ref CURRENT_COUNTER: AtomicU64 = AtomicU64::new(0);
    static ref SLOT_VEC: Mutex<Vec<Slot>> =
        Mutex::new(Vec::with_capacity(MAX_SLOT));
    static ref TIME_WHEEL_MAP: Mutex<HashMap<SocketAddr, KeepAliveVal>> =
        Mutex::new(HashMap::new());
}

// TODO only for retransmit timing wheel.
// The initial duration is set to TIME_WHEEL_INIT_DURATION, but can be
// changed to reflect the network the client is on, (LAN or WAN),
// or the latency pattern.

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

/// Timing wheel for keep alive.
/// The wheel is divided into MAX_SLOT slots.
/// Each slot is a vector of SocketAddr.
/// The data is stored in a HashMap indexed by the SocketAddr.
pub struct KeepAliveTimeWheel {}

impl KeepAliveTimeWheel {
    pub fn init() {
        let mut slot_vec = SLOT_VEC.lock().unwrap();
        for _ in 0..MAX_SLOT {
            slot_vec.push(Slot::new());
        }
    }
    /// Schedule a keep alive event for a connection.
    /// Insert the connection address(key) into the corresponding slot.
    /// Insert data into the TIME_WHEEL_MAP.
    #[inline(always)]
    // #[trace_var(index, slot, hash)]
    pub fn schedule(key: SocketAddr, conn_duration: u16) -> Result<(), String> {
        // store the key in a slot of the timing wheel
        // TODO XXX change value 10 to a constant
        let conn_duration = conn_duration * 10;
        let cur_counter = CURRENT_COUNTER.load(Ordering::Relaxed) as usize;
        let index = (cur_counter + conn_duration as usize) % MAX_SLOT;
        match TIME_WHEEL_MAP.try_lock() {
            Ok(mut time_wheel_map) => {
                time_wheel_map.insert(
                    key,
                    KeepAliveVal {
                        latest_counter: cur_counter,
                        conn_duration: conn_duration,
                    },
                );
            }
            Err(why) => {
                return Err(eformat!(why.to_string()));
            }
        }
        match SLOT_VEC.try_lock() {
            Ok(mut slot_vec) => {
                let slot = &mut slot_vec[index];
                match slot.entries.try_lock() {
                    Ok(mut entries) => {
                        entries.push(key);
                    }
                    Err(why) => {
                        // unwind: remove the inserted key from the time_wheel_map
                        if let None =
                            TIME_WHEEL_MAP.lock().unwrap().remove(&key)
                        {
                            return Err(eformat!("key not found"));
                        }
                        return Err(eformat!(why.to_string()));
                    }
                }
            }
            Err(why) => {
                // unwind: remove the inserted key from the time_wheel_map
                if let None = TIME_WHEEL_MAP.lock().unwrap().remove(&key) {
                    return Err(eformat!("key not found"));
                }
                return Err(eformat!(why.to_string()));
            }
        }
        return Ok(());
    }
    /// Cancel a keep alive event.
    /// Call when it received a DISCONNECT message from the sender.
    #[inline(always)]
    #[trace_var(index, slot, hash, vec)]
    pub fn cancel(socket_addr: &SocketAddr) -> Result<(), String> {
        match TIME_WHEEL_MAP.try_lock() {
            Ok(mut time_wheel_map) => {
                match time_wheel_map.remove(socket_addr) {
                    Some(_) => Ok(()),
                    None => Err(eformat!(socket_addr)),
                }
            }
            Err(why) => Err(eformat!(socket_addr, why.to_string())),
        }
    }
    /// Reschedule a keep alive event when it received a message from the sender.
    /// Modify the latest_counter in the TIME_WHEEL_MAP to the current counter.
    #[inline(always)]
    #[trace_var(index, slot, hash, vec)]
    pub fn reschedule(socket_addr: SocketAddr) -> Result<(), String> {
        let latest_counter = CURRENT_COUNTER.load(Ordering::Relaxed) as usize;
        match TIME_WHEEL_MAP.try_lock() {
            Ok(mut time_wheel_map) => {
                match time_wheel_map.get_mut(&socket_addr) {
                    Some(conn) => {
                        dbg!(&conn);
                        dbg!(&latest_counter);
                        conn.latest_counter = latest_counter;
                        dbg!(&latest_counter);
                        dbg!(&conn);
                        Ok(())
                    }
                    None => Err(eformat!(socket_addr, "not found.")),
                }
            }
            Err(why) => Err(eformat!(socket_addr, why.to_string())),
        }
    }
    /// When the address(key) is expired in the timing wheel, it compare the latest_counter
    /// with the current counter. If the latest_counter is less than the current counter,
    /// the address(key) is expired. Otherwise, put it back to a new slot.
    pub fn run(client: MqttSnClient) {
        // When the keep_alive timing wheel entry is accessed,
        // this code determines if the connection is expired.
        // If the hash entry has been updated to a new counter,
        // then reschedule the connection in the timing wheel.
        //
        // TODO replace lock with try_lock
        let _keep_alive_expire_thread = thread::spawn(move || {
            loop {
                // The sleep() has to be outside of the mutex lock block for
                // the lock to be unlocked while the thread is sleeping.
                thread::sleep(Duration::from_millis(SLEEP_DURATION as u64));
                {
                    let cur_counter: usize;
                    cur_counter = CURRENT_COUNTER
                        .fetch_add(1, Ordering::Relaxed)
                        as usize;
                    let index = cur_counter % MAX_SLOT;
                    // dbg!(&cur_slot);
                    // dbg!(cur_counter);
                    let slot_vec = SLOT_VEC.lock().unwrap();
                    let mut slot = slot_vec[index].entries.lock().unwrap();
                    let mut time_wheel_map = TIME_WHEEL_MAP.lock().unwrap();
                    // process the expired connections
                    while let Some(socket_addr) = slot.pop() {
                        dbg!(index);
                        dbg!(socket_addr);
                        if let Some(conn) = time_wheel_map.get(&socket_addr) {
                            dbg!(&conn);
                            let new_counter = conn.latest_counter as usize
                                + conn.conn_duration as usize;
                            dbg!(&conn);
                            if new_counter > cur_counter {
                                // Not expired, reschedule
                                // The new duration starts from the latest_counter,
                                // not the cur_counter. Subtract cur_counter is needed.
                                let mut new_index = new_counter % MAX_SLOT;
                                dbg!(&conn);
                                if new_index == index {
                                    // Can't lock the same slot twice
                                    // Even without lock, push() to the same slot will be popped
                                    // in the while loop, so it's an infinite loop.
                                    // Use the next slot instead.
                                    new_index = (index + 1) % MAX_SLOT;
                                }
                                let mut new_slot =
                                    slot_vec[new_index].entries.lock().unwrap();
                                new_slot.push(socket_addr);
                            } else {
                                // Client timeout, move from ACTIVE to LOST state.
                                // MQTT-SN 1.2 spec page 25
                                // The entry was pop() from the timing wheel slot.
                                //    client_reschedule.set_state(STATE_LOST);
                                // Remove it from the hashmap.
                                // TODO XXX change connection state to LOST.
                                // remove socket_add from keep alive HashMap
                                if let Some(conn) =
                                    time_wheel_map.remove(&socket_addr)
                                {
                                    dbg!(&conn);
                                    dbg!(&time_wheel_map);
                                    dbg!(&socket_addr);
                                    match Connection::update_state(
                                        &socket_addr,
                                        StateEnum2::LOST,
                                    ) {
                                        Ok(_) => {
                                            let _result =
                                                Connection::publish_will(
                                                    &socket_addr,
                                                    &client,
                                                );
                                        }
                                        Err(why) => {
                                            error!(
                                                "{}",
                                                eformat!(
                                                    socket_addr,
                                                    why.to_string()
                                                )
                                            );
                                        }
                                    }
                                }
                                info!("Connection Timeout: {:?}", socket_addr);
                            }
                        }
                    }
                }
            }
        });
    }
}
