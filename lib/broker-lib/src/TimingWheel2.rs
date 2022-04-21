#![warn(rust_2018_idioms)]
use bytes::BytesMut;
use chrono::{Datelike, Local, Timelike};
use core::fmt::Debug;
use core::hash::Hash;
use crossbeam::channel::{Receiver, Sender};
use std::collections::HashMap;
use std::thread;
use std::time::Duration;
use std::{net::SocketAddr, sync::Arc, sync::Mutex};

use trace_var::trace_var;

// TODO move to utility lib
macro_rules! function {
    () => {{
        fn f() {}
        fn type_name_of<T>(_: T) -> &'static str {
            std::any::type_name::<T>()
        }
        let name = type_name_of(f);

        // Find and cut the rest of the path
        match &name[..name.len() - 3].rfind(':') {
            Some(pos) => &name[pos + 1..name.len() - 3],
            None => &name[..name.len() - 3],
        }
    }};
}

// dbg macro that prints function name instead of file name.
// https://stackoverflow.com/questions/65946195/understanding-the-dbg-macro-in-rust
/*
macro_rules! dbg2 {
    () => {
        $crate::eprintln!("[{}:{}]", function!(), line!());
    };
    ($val:expr $(,)?) => {
        // Use of `match` here is intentional because it affects the lifetimes
        // of temporaries - https://stackoverflow.com/a/48732525/1063961
        match $val {
            tmp => {

                // Added the timestamp
                let now = chrono::Local::now();
                eprint!(
                    "{:02}-{:02} {:02}:{:02}:{:02}.{:09}",
                    now.month(),
                    now.day(),
                    now.hour(),
                    now.minute(),
                    now.second(),
                    now.nanosecond(),
                    );
                // replace file!() with function!()
                eprintln!("<{}:{}> {} = {:#?}",
                          function!(), line!(), stringify!($val), &tmp);
                tmp
            }
        }
    };
    ($($val:expr),+ $(,)?) => {
        ($($dbg!($val)),+,)
    };
}
*/

#[derive(Debug, Clone)]
struct Slot<KEY: Debug + Clone> {
    pub entries: Arc<Mutex<Vec<(KEY, usize)>>>,
}

impl<KEY: Debug + Clone> Slot<KEY> {
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
pub struct TimingWheel2<KEY: Debug + Clone, VAL: Debug + Clone> {
    max_slot: usize,       // (1000 / sleep_duration) * 64 * 2;
    sleep_duration: usize, // in milli seconds
    default_duration: usize,
    cur_counter: usize,
    slot_vec: Vec<Slot<KEY>>,
    hash: Arc<Mutex<HashMap<KEY, VAL>>>,
}

impl<KEY: Eq + Hash + Debug + Clone, VAL: Debug + Clone>
    TimingWheel2<KEY, VAL>
{
    #[trace_var(max_slot, slot_vec, default_duration)]
    pub fn new(sleep_duration: usize, default_duration_ms: usize) -> Self {
        dbg!((sleep_duration, default_duration_ms));
        // verify sleep_duration (milli seconds) 1..1000.
        let max_slot = (1000 / sleep_duration) * 64 * 2;
        let mut slot_vec: Vec<Slot<KEY>> = Vec::with_capacity(max_slot);
        // verify default_duration (milli seconds) 1..1000.
        let default_duration = default_duration_ms / sleep_duration;

        // Must use for loop to initialize
        // let slot_vec:Vec<Slot<KEY>> = vec![Vec<Slot<KEY>>, max_slot] didn'KEY work.
        // It allocates one Vec<Slot<KEY>> and all entries point to it.
        for _ in 0..max_slot {
            slot_vec.push(Slot::new());
            // slot_hash.push(Arc::new(Mutex::new(HashMap::new())));
        }
        TimingWheel2 {
            max_slot,
            sleep_duration,
            default_duration,
            slot_vec,
            cur_counter: 0,
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
        key: KEY,
        val: VAL,
        duration: usize,
    ) -> Result<(), (String, KEY)> {
        // store the key in a slot of the timing wheel
        let index = (self.cur_counter + duration) % self.max_slot;
        let slot = &self.slot_vec[index];
        // TODO replace unwrap
        let mut hash = self.hash.lock().unwrap();
        let mut vec = slot.entries.lock().unwrap();
        hash.insert(key.clone(), val);
        vec.push((key, duration));
        return Ok(());
    }

    // Reschedule for later expiration, but not changes to the hashmap.
    #[trace_var(index, slot, vec)]
    #[inline(always)]
    fn reschedule(
        &mut self,
        key: KEY,
        duration: usize,
    ) -> Result<(), (String, KEY)> {
        dbg!((&key, duration, self.hash.lock(), self.cur_counter));
        // store the key in a slot of the timing wheel
        let index = (self.cur_counter + duration) % self.max_slot;
        let slot = &self.slot_vec[index];
        // TODO replace unwrap
        let mut vec = slot.entries.lock().unwrap();
        vec.push((key, duration));
        return Ok(());
    }

    #[inline(always)]
    fn get_cur_counter(&mut self) -> usize {
        self.cur_counter
    }

    #[inline(always)]
    fn get_hash(&mut self, key: KEY) -> Result<VAL, Option<(String, KEY)>> {
        match self.hash.lock() {
            Ok(hash) => match hash.get(&key) {
                Some(val) => {
                    dbg!((&key, val));
                    Ok(val.to_owned())
                }
                None => Err(None),
            },
            _ => {
                let string = format!("[{}:hash.lock()]", function!());
                Err(Some((string, key)))
            }
        }
    }

    /// remove the data from the hash only
    /// the expire() won't return data from the slot if it has been deleted
    #[inline(always)]
    #[trace_var(hash)]
    fn cancel(&mut self, key: KEY) -> Result<(), (String, KEY)> {
        let mut hash = self.hash.lock().unwrap();
        match hash.remove(&key) {
            Some(_val) => {
                // dbg!((key, val));
                Ok(())
            }
            None => {
                let string = format!("[{}:hash.remove()]", function!());
                Err((string, key))
            }
        }
    }

    /// remove the data from the hash only
    /// the expire() won't return data from the slot if it has been deleted
    #[inline(always)]
    fn reset(&mut self, key: KEY) -> Result<(), (String, KEY)> {
        let hash = self.hash.lock().unwrap();
        match hash.get(&key) {
            Some(_val) => {
                // dbg!((key, val));
                Ok(())
            }
            None => {
                let string = format!("[{}:hash.remove()]", function!());
                Err((string, key))
            }
        }
    }
    /// expire() is called every TIME_WHEEL_SLEEP_DURATION to iterate all
    /// the entries in the slot.
    /// If the new duration is greater than the maximun duration
    /// (TIME_WHEEL_MAX_SLOTS)
    /// remove it from the hashmap, else reschedule to the new duration.
    // #[trace_var (next_slot_index, result_vec, result)]
    #[inline(always)]
    fn expire(&mut self) -> Vec<(KEY, VAL)> {
        // select and lock the current time slot in the Vec
        let cur_slot = &self.slot_vec[self.cur_counter % self.max_slot];
        // returning data store in this vector
        let mut result_vec = Vec::new();
        // TODO replace unwrap()
        let mut cur_slot_lock = cur_slot.entries.lock().unwrap();
        // iterate for each entry in the selected Vec
        while let Some(top) = cur_slot_lock.pop() {
            let mut hash = self.hash.lock().unwrap();
            // dbg!((top.clone(), SystemTime::now()));
            // exponetial backup is inside the expire (lib)
            // the caller doesn't have to do it
            let duration = top.1 * 2;
            let retrans_hdr = top.0;
            if duration < self.max_slot {
                // reschedule, don't remove hash entry
                match hash.get(&retrans_hdr) {
                    Some(result) => {
                        // dbg!((retrans_hdr.clone(), result.clone()));
                        // insert entry into next slot
                        let next_slot_index =
                            (self.cur_counter + duration) % self.max_slot;
                        // gather results
                        result_vec
                            .push((retrans_hdr.clone(), result.to_owned()));
                        let next_slot = &self.slot_vec[next_slot_index];

                        let mut next_slot_lock =
                            next_slot.entries.lock().unwrap();
                        // dbg!(retrans_hdr.clone());
                        next_slot_lock.push((retrans_hdr, duration));
                        // TODO send it back inside the loop,
                        // eliminate return loop need the to use
                        // RetransmitHeader to access the address
                        // can't access it from generic KEY
                    }
                    // It's empty, the hash value is canceled
                    _ => (),
                }
            } else {
                // timeout duration is over the limit, remove hash entry
                // XXX Because generic is used, we can't access the internal of
                // the KEY and VAL.
                // TODO need to detect connect() timeout.
                match hash.remove(&retrans_hdr) {
                    Some(result) => {
                        dbg!((retrans_hdr.clone(), result.clone()));
                        /*
                        if retrans_hdr.msg_type == MSG_TYPE_CONNACK {
                            panic!("connect(): timeout; destination: {:?}",
                                retrans_hdr.addr);
                        }
                        */
                        // gather results
                        // result_vec.push((retrans_hdr.clone(), result.to_owned()));
                    }
                    // It's empty, the hash value is canceled
                    _ => (),
                }
            }
        }
        // next slot with overflow back to 0
        self.cur_counter = self.cur_counter + 1;
        // TODO test preload this slot in CPU cache?
        result_vec
    }

    /// expire() is called every TIME_WHEEL_SLEEP_DURATION to iterate all the entries
    /// in the slot.
    /// If the new duration is greater than the maximun duration (TIME_WHEEL_MAX_SLOTS)
    /// remove it from the hashmap, else reschedule to the new duration.
    #[inline(always)]
    // fn keep_alive_expire(&mut self) -> Vec<(KEY, VAL)>
    fn keep_alive_expire(&mut self) -> &Slot<KEY> {
        // select and lock the current time slot in the Vec
        let cur_slot = &self.slot_vec[self.cur_counter % self.max_slot];
        self.cur_counter = self.cur_counter + 1;
        return cur_slot;
    }
}

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
pub struct RetransmitHeader {
    pub addr: SocketAddr,
    pub msg_type: u8,
    pub topic_id: u16, // for pub and sub, default 0
    pub msg_id: u16,   // for pub and sub, default 0
}

#[derive(Debug, Clone)]
pub struct RetransmitData {
    pub bytes: BytesMut,
}

#[derive(Debug, Clone)]
struct KeepAliveData {
    conn_duration: u16,
    latest_counter: u32,
}

#[derive(Debug, Clone)]
pub struct RetransTimeWheel {
    schedule_tx: Sender<(SocketAddr, u8, u16, u16, BytesMut)>,
    pub schedule_rx: Receiver<(SocketAddr, u8, u16, u16, BytesMut)>,
    cancel_tx: Sender<(SocketAddr, u8, u16, u16)>,
    pub cancel_rx: Receiver<(SocketAddr, u8, u16, u16)>,
    transmit_tx: Sender<(SocketAddr, BytesMut)>,
    transmit_rx: Receiver<(SocketAddr, BytesMut)>,
    wheel: TimingWheel2<RetransmitHeader, RetransmitData>,
}

impl RetransTimeWheel {
    pub fn new(
        sleep_duration: usize,
        default_duration_ms: usize,
        schedule_tx: Sender<(SocketAddr, u8, u16, u16, BytesMut)>,
        schedule_rx: Receiver<(SocketAddr, u8, u16, u16, BytesMut)>,
        cancel_tx: Sender<(SocketAddr, u8, u16, u16)>,
        cancel_rx: Receiver<(SocketAddr, u8, u16, u16)>,
        transmit_tx: Sender<(SocketAddr, BytesMut)>,
        transmit_rx: Receiver<(SocketAddr, BytesMut)>,
    ) -> Self {
        let wheel = TimingWheel2::new(sleep_duration, default_duration_ms);
        RetransTimeWheel {
            schedule_tx,
            schedule_rx,
            cancel_rx,
            cancel_tx,
            transmit_tx,
            transmit_rx,
            wheel,
        }
    }
    pub fn run(self) {
        let sleep_duration = self.wheel.sleep_duration;
        let default_duration = self.wheel.default_duration;
        let schedule_rx = self.schedule_rx.clone();
        let cancel_rx = self.cancel_rx.clone();
        let transmit_tx = self.transmit_tx.clone();
        let w = Arc::new(Mutex::new(self.wheel));
        let rx_wheel = w.clone();
        let expire_wheel = w.clone();
        let cancel_wheel = w.clone();
        // receive messages for cancel
        let cancel_thread = thread::spawn(move || {
            loop {
                match cancel_rx.recv() {
                    // XXX
                    Ok((addr, msg_type, topic_id, msg_id)) => {
                        println!("-----------------------------");
                        let retrans_hdr = RetransmitHeader {
                            addr,
                            msg_type,
                            topic_id,
                            msg_id,
                        };
                        dbg!(retrans_hdr);
                        {
                            // XXX lock an entry in the array, instead of the wheel?
                            let mut cancel_wheel_lock =
                                cancel_wheel.lock().unwrap();
                            let _result = cancel_wheel_lock.cancel(retrans_hdr);
                        }
                    }
                    Err(why) => {
                        // XXX thread panic, but the rest still run
                        // no more sender, nothing to recv
                        // unlikely, but need to verify
                        println!(
                            "==============timing_wheel_rx_thread: {}",
                            why
                        );
                        panic!("panic");
                        // break;
                    }
                }
            }
        });
        // receive messages for schedule
        let schedule_thread = thread::spawn(move || {
            loop {
                match schedule_rx.recv() {
                    // XXX
                    Ok((addr, msg_type, topic_id, msg_id, bytes)) => {
                        println!("&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&");
                        dbg!(bytes.clone());
                        let data = RetransmitData { bytes };
                        let retrans_hdr = RetransmitHeader {
                            addr,
                            msg_type,
                            topic_id,
                            msg_id,
                        };
                        dbg!(retrans_hdr);
                        dbg!(default_duration);
                        {
                            // XXX lock an entry in the array, instead of the wheel?
                            let mut rx_wheel_lock = rx_wheel.lock().unwrap();
                            let _result = rx_wheel_lock.schedule(
                                retrans_hdr,
                                data,
                                default_duration,
                            );
                        }
                    }
                    Err(why) => {
                        // XXX thread panic, but the rest still run
                        // no more sender, nothing to recv
                        // unlikely, but need to verify
                        println!(
                            "==============timing_wheel_rx_thread: {}",
                            why
                        );
                        panic!("panic");
                        // break;
                    }
                }
            }
        });
        // timing wheel expire checks for timeout messages
        let expire_thread = thread::spawn(move || {
            // tokio::spawn(async move {
            loop {
                // use the block to allow lock() to automatically
                // unlock at the end on the block
                {
                    let mut expire_wheel_lock = expire_wheel.lock().unwrap();
                    let v = expire_wheel_lock.expire();
                    // TODO this lock is too long
                    for ack in v {
                        // dbg!(ack.clone());
                        let (retrans_hdr, data) = ack;
                        dbg!((retrans_hdr.addr, data.bytes.clone()));
                        let _result =
                            transmit_tx.send((retrans_hdr.addr, data.bytes));
                    }
                    // let len = socket_tx3.send_to(b"hello", remote_addr);
                }
                // dbg!(sleep_duration);
                thread::sleep(Duration::from_millis(sleep_duration as u64));
            }
        });
    }
}
