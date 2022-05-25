use crate::{broker_lib::MqttSnClient, eformat, function};
use bytes::BytesMut;
// use core::fmt::Debug;
use core::hash::Hash;
use custom_debug::Debug;
use hashbrown::HashMap;
use log::*;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use trace_var::trace_var;

/// RetransmitHeader is stored in:
/// (1) HashMap for cancellation from an ACK
/// (2) timing wheel slots for timeouts.
/// When the wheel reads a slot, it iterates all entries in the vector.
/// Using the RetransmitHeader to get/remove the RetransmitData
/// in the HashMap.
/// If the new duration is greater that the maximum timeout period
/// the HashMap entry will be removed.
/// To cancel a scheduled event, remove the HashMap entry with the
/// RetransmitHeader. Don't need to remove the entry in the slot
/// because the slot entry lookup ignores missing HashMap entries.
#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
struct RetransmitHeader {
    pub addr: SocketAddr,
    #[debug(format = "0x{:x}")]
    pub msg_type: u8,
    pub topic_id: u16, // for pub and sub, default 0
    pub msg_id: u16,   // for pub and sub, default 0
}

#[derive(Debug, Clone)]
struct RetransmitData {
    pub bytes: BytesMut, // TODO use Bytes instead.
}

#[derive(Debug, Clone)]
struct Slot {
    pub entries: Arc<Mutex<Vec<(RetransmitHeader, u16)>>>,
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
    static ref TIME_WHEEL_MAP: Mutex<HashMap<RetransmitHeader, RetransmitData>> =
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
pub struct RetransTimeWheel {}

impl RetransTimeWheel {
    pub fn init() {
        let mut slot_vec = SLOT_VEC.lock().unwrap();
        for _ in 0..MAX_SLOT {
            slot_vec.push(Slot::new());
        }
    }

    // The initial duration is set to TIME_WHEEL_INIT_DURATION, but can be
    // changed to reflect the network the client is on, (LAN or WAN),
    // or the latency pattern.
    #[inline(always)]
    pub fn schedule_timer(
        addr: SocketAddr,
        msg_type: u8,
        topic_id: u16,
        msg_id: u16,
        duration: u16,
        bytes: BytesMut,
    ) -> Result<(), String> {
        // store the retrans_hdr in a slot of the timing wheel
        // TODO XXX change value 10 to a constant
        let retrans_hdr = RetransmitHeader {
            addr,
            msg_type,
            topic_id,
            msg_id,
        };
        let val = RetransmitData { bytes };
        let duration = duration * 10;
        let cur_counter = CURRENT_COUNTER.load(Ordering::Relaxed) as usize;
        let index = (cur_counter + duration as usize) % MAX_SLOT;
        match TIME_WHEEL_MAP.try_lock() {
            Ok(mut map) => {
                map.insert(retrans_hdr, val);
            }
            Err(why) => {
                return Err(eformat!(retrans_hdr, why.to_string()));
            }
        }
        match SLOT_VEC.try_lock() {
            Ok(mut slot_vec) => {
                let slot = &mut slot_vec[index];
                match slot.entries.try_lock() {
                    Ok(mut entries) => {
                        entries.push((retrans_hdr, duration));
                    }
                    Err(why) => {
                        // unwind: remove the inserted retrans_hdr from the map
                        if let None =
                            TIME_WHEEL_MAP.lock().unwrap().remove(&retrans_hdr)
                        {
                            return Err(eformat!(retrans_hdr, "key not found"));
                        }
                        return Err(eformat!(retrans_hdr, why.to_string()));
                    }
                }
            }
            Err(why) => {
                // unwind: remove the inserted retrans_hdr from the map
                if let None =
                    TIME_WHEEL_MAP.lock().unwrap().remove(&retrans_hdr)
                {
                    return Err(eformat!("key not found"));
                }
                return Err(eformat!(why.to_string()));
            }
        }
        return Ok(());
    }
    /// Reschedule a keep alive event when it received a message from the sender.
    /// Modify the latest_counter in the TIME_WHEEL_MAP to the current counter.
    #[inline(always)]
    #[trace_var(index, slot, hash, vec)]
    pub fn cancel_timer(
        addr: SocketAddr,
        msg_type: u8,
        topic_id: u16,
        msg_id: u16,
    ) -> Result<(), String> {
        let retrans_hdr = RetransmitHeader {
            addr,
            msg_type,
            topic_id,
            msg_id,
        };
        match TIME_WHEEL_MAP.try_lock() {
            Ok(mut map) => {
                if let None = map.remove(&retrans_hdr) {
                    return Err(eformat!(retrans_hdr, "not found."));
                }
                Ok(())
            }
            Err(why) => Err(eformat!(retrans_hdr, why.to_string())),
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
        let _retrans_expire_thread = thread::spawn(move || {
            loop {
                // The sleep() has to be outside of the mutex lock block for
                // the lock to be unlocked while the thread is sleeping.
                thread::sleep(Duration::from_millis(SLEEP_DURATION as u64));
                {
                    let cur_counter: usize;
                    cur_counter = CURRENT_COUNTER
                        .fetch_add(1, Ordering::Relaxed)
                        as usize;
                    // dbg!(&cur_slot);
                    // dbg!(cur_counter);
                    let slot_vec = SLOT_VEC.lock().unwrap();
                    let index = cur_counter % MAX_SLOT;
                    let mut slot = slot_vec[index].entries.lock().unwrap();
                    let mut map = TIME_WHEEL_MAP.lock().unwrap();
                    // process the expired connections
                    while let Some((retrans_hdr, mut duration)) = slot.pop() {
                        dbg!(index);
                        duration *= 2;
                        dbg!((duration, MAX_SLOT));
                        if duration < (MAX_SLOT as u16) {
                            // not expired, reschedule to new slot, don't remove hash entry
                            if let Some(retrans_data) = map.get(&retrans_hdr) {
                                let mut new_index = (cur_counter
                                    + duration as usize)
                                    % MAX_SLOT;
                        dbg!((new_index, index));
                                if new_index == index {
                                    // Can't lock the same slot twice
                                    // Even without lock, push() to the same slot will be popped
                                    // in the while loop, so it's an infinite loop.
                                    // Use the next slot instead.
                                    new_index = (index + 1) % MAX_SLOT;
                                }
                        dbg!((new_index, index));
                                let mut new_slot =
                                    slot_vec[new_index].entries.lock().unwrap();
                                new_slot.push((retrans_hdr, duration));
                                // Retransmit the message to the receiver.
                                if let Err(err) = client.transmit_tx.send((
                                    retrans_hdr.addr,
                                    retrans_data.bytes.clone(),
                                )) {
                                    error!("{:?} {:?}", err, retrans_hdr);
                        dbg!((new_index, index));
                                }
                        dbg!(retrans_hdr);
                            }
                        } else {
                            // The connection is expired, remove the hash entry
                            map.remove(&retrans_hdr);
                            info!("Retransmit Timeout: {:?}", retrans_hdr);
                        }
                    }
                }
            }
        });
    }
}
