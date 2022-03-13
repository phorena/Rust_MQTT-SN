#![warn(rust_2018_idioms)]
#[macro_use]
use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;
use std::sync::atomic::{AtomicU8, Ordering};
use std::{hint, thread};
use std::net::UdpSocket;
use std::{io, net::SocketAddr, net::SocketAddrV4, sync::Arc, sync::Mutex};
use std::collections::HashMap;
use core::fmt::Debug;
use core::hash::Hash;
use std::time::{Duration, SystemTime};

use bytes::{Bytes, BufMut, BytesMut};
use nanoid::nanoid;
use log::*;
use simplelog::*;
use arr_macro::arr;

/// Use of const u8 instead of enum:
/// 1. Portability
/// 2. In Rust converting from enum to number is simple,
///    but from number to enum is complicated, requires match/if.
/// 3. u8 allows flexibility, but enum is safer.
/// 4. Performance

type StateTypeConst  = u8;
const STATE_ACTIVE:StateTypeConst       = 0;
const STATE_DISCONNECT:StateTypeConst   = 1;
const STATE_CONNECT_SENT:StateTypeConst = 2;
const STATE_LOST:StateTypeConst         = 3;
const STATE_SLEEP:StateTypeConst        = 4;
const STATE_AWAKE:StateTypeConst        = 5;

const STATE_MAX: usize                  = 6;

type MsgTypeConst = u8;
const MSG_TYPE_CONNECT:MsgTypeConst     = 0x4;
const MSG_TYPE_CONNACK:MsgTypeConst     = 0x5;
const MSG_TYPE_SUBSCRIBE:MsgTypeConst   = 0x12;
const MSG_TYPE_SUBACK:MsgTypeConst      = 0x13;
const MSG_TYPE_PUBLISH:MsgTypeConst     = 0xC; // should be 0, most popular
const MSG_TYPE_PUBACK:MsgTypeConst      = 0xD;

// TODO fill in the rest
const MSG_TYPE_WILLMSGRESP:MsgTypeConst = 0x1D; // 29


// 0x1E-0xFD reserved
const MSG_TYPE_ENCAP_MSG:MsgTypeConst   = 0xFE;
// XXX not an optimal choice because, array of MsgTypeConst
// must include 256 entries.
// For the 2x2 array [0..6][0..255] states,
// instead of array  [0..6][0..29] states.

const MSG_TYPE_MAX:usize                = 256;


type DupConst = u8;
const DUP_FALSE:DupConst = 0b_0_00_0_0_0_00;
const DUP_TRUE:DupConst  = 0b_1_00_0_0_0_00;

type QoSConst = u8;
const    QOS_LEVEL_0:QoSConst = 0b_0_00_0_0_0_00;
const    QOS_LEVEL_1:QoSConst = 0b_0_01_0_0_0_00;
const    QOS_LEVEL_2:QoSConst = 0b_0_10_0_0_0_00;
const    QOS_LEVEL_3:QoSConst = 0b_0_11_0_0_0_00;

type RetainConst = u8;
const RETAIN_FALSE:RetainConst = 0b_0_00_0_0_0_00;
const RETAIN_TRUE:RetainConst  = 0b_0_00_1_0_0_00;

type WillConst = u8;
const WILL_FALSE:WillConst = 0b_0_00_0_0_0_00;
const WILL_TRUE:WillConst  = 0b_0_00_0_1_0_00;

type CleanSessionConst = u8;
const CLEAN_SESSION_FALSE:CleanSessionConst = 0b_0_00_0_0_0_00;
const CLEAN_SESSION_TRUE:CleanSessionConst  = 0b_0_00_0_0_1_00;

type TopicIdTypeConst = u8;
const   TOPIC_ID_TYPE_NORNAL:TopicIdTypeConst       = 0b_0_00_0_0_0_00;
const   TOPIC_ID_TYPE_PRE_DEFINED:TopicIdTypeConst  = 0b_0_00_0_0_0_01;
const   TOPIC_ID_TYPE_SHORT:TopicIdTypeConst        = 0b_0_00_0_0_0_10;
const   TOPIC_ID_TYPE_RESERVED:TopicIdTypeConst    = 0b_0_00_0_0_0_11;

type ReturnCodeConst = u8;
const RETURN_CODE_ACCEPTED:ReturnCodeConst          = 0;
const RETURN_CODE_CONGESTION:ReturnCodeConst        = 1;
const RETURN_CODE_INVALID_TOPIC_ID:ReturnCodeConst  = 2;
const RETURN_CODE_NOT_SUPPORTED:ReturnCodeConst     = 3;

// use DTLS::dtls_client::DtlsClient;
use client_lib::{
//    ConnectionDb::ConnectionDb,
//    SubscriberDb::SubscriberDb,
//    Advertise::Advertise,
//    Transfer::Transfer,MTU,
//    TopicDb::TopicDb,
//    MessageDb::MessageDb,
    Functions::{
        process_input,
        connect2,
        verify_puback2,
        verify_suback2,
        verify_connack2,
        verify_publish2,
        publish, subscribe},
    Subscribe::Subscribe,
    Publish::Publish,
    MsgType::MsgType,
    client_struct::ClientStruct,
};


fn generate_client_id() -> String {
    format!("exofense/{}", nanoid!())
}


static NTHREADS: i32 = 3;

// TODO move to utility lib
macro_rules! function {
    () => {{
        fn f() {}
        fn type_name_of<KEY>(_: KEY) -> &'static str {
            std::any::type_name::<KEY>()
        }
        let name = type_name_of(f);
        &name[..name.len() - 3]
    }};
}

macro_rules! dbg_buf {
    ($buf:ident, $size:ident) => {
        let mut i: usize = 0;
        eprint!("[{}:{}] ", function!(), line!());
        while i < $size {
            eprint!("{:#04X?} ", $buf[i]);
            i += 1;
        }
        eprintln!("");
    };
}

// dbg macro that prints function name instead of file name.
// https://stackoverflow.com/questions/65946195/understanding-the-dbg-macro-in-rust
macro_rules! dbg_fn {
    () => {
        $crate::eprintln!("[{}:{}]", function!(), line!());
    };
    ($val:expr $(,)?) => {
        // Use of `match` here is intentional because it affects the lifetimes
        // of temporaries - https://stackoverflow.com/a/48732525/1063961
        match $val {
            tmp => {
                // replace file!() with function!()
                eprintln!("[{}:{}] {} = {:#?}",
                    function!(), line!(), stringify!($val), &tmp);
                tmp
            }
        }
    };
    ($($val:expr),+ $(,)?) => {
        ($($dbg_fn!($val)),+,)
    };
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
struct RetransmitHeader {
    addr: SocketAddr,
    msg_type: u8,
    topic_id: u16, // for pub and sub, default 0
    msg_id: u16, // for pub and sub, default 0
}

#[derive(Debug, Clone)]
struct RetransmitData {
    bytes: BytesMut,
}

#[derive(Debug, Clone)]
struct Slot<KEY: Debug + Clone> {
    pub entries: Arc<Mutex<Vec<(KEY, usize)>>>
}

impl<KEY: Debug + Clone> Slot<KEY> {
    pub fn new() -> Self {
        Slot {
            entries: Arc::new(Mutex::new(Vec::new()))
        }
    }
}

// clients use 100 milli seconds
// brokers use 10 milli seconds
static TIME_WHEEL_SLEEP_DURATION:usize = 100; // in milli seconds
// For 64 seconds at 100 ms sleeping interval, we need 10 * 64 slots, (1000/100 = 10)
// For 64 seconds at 10 ms sleeping interval, we need 100 * 64 slots, (1000/10 = 100)

// The maximum timeout duration is 64 seconds
// The last one might be over 64, rounding up.
static TIME_WHEEL_MAX_SLOTS:usize = (1000 / TIME_WHEEL_SLEEP_DURATION) * 64 * 2;
// Initial timeout duration is 300 ms
static TIME_WHEEL_INIT_DURATION:usize = 300 / TIME_WHEEL_SLEEP_DURATION;


#[derive(Debug, Clone)]
struct TimingWheel <KEY: Debug + Clone, VAL: Debug + Clone> {
    max_slot: usize,
    slot_index: usize,
    slot_vec: Vec<Slot<KEY>>,
    hash: Arc<Mutex<HashMap<KEY, VAL>>>,
}

impl<KEY: Eq + Hash + Debug + Clone, VAL: Debug + Clone> TimingWheel<KEY, VAL> {
    pub fn new(max_slot: usize) -> Self {
        let mut slot_vec:Vec<Slot<KEY>> = Vec::with_capacity(max_slot);
        // Must use for loop to initialize
        // let slot_vec:Vec<Slot<KEY>> = vec![Vec<Slot<KEY>>, max_slot] didn'KEY work.
        // It allocates one Vec<Slot<KEY>> and all entries point to it.
        for _ in 0..max_slot {
            slot_vec.push(Slot::new());
        }
        TimingWheel {
            max_slot,
            slot_vec,
            slot_index: 0,
            hash: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    // The initial duration is set to TIME_WHEEL_INIT_DURATION, but can be
    // changed to reflect the network the client is on, (LAN or WAN),
    // or the latency pattern.
    #[inline(always)]
    fn schedule(&mut self,
              key: KEY,
              val: VAL,
              duration: usize) -> Result<(), (KEY, usize)>
    {
        // store the key in a slot of the timing wheel
        let slot_index = (self.slot_index + duration) % self.max_slot;
        let slot = &self.slot_vec[slot_index];
        let mut hash = self.hash.lock().unwrap();
        let mut vec = slot.entries.lock().unwrap();
        // store the data the hash
        hash.insert(key.clone(), val);
        // use tuple to add duration and avoid define new struct
        dbg_fn!((key.clone(), duration, SystemTime::now()));
        vec.push((key, duration));
        return Ok(());
    }

    #[inline(always)]
    fn cancel(&mut self,
              key: KEY) -> Result<(), KEY>
    {
        let mut hash = self.hash.lock().unwrap();
        // remove the data from the hash only
        // the expire() won't return data from the slot if it has been deleted
        match hash.remove(&key) {
            Some(result) => {
                dbg!((key, result));
                Ok(())
            }
            None => Err(key)
        }
    }

    /// expire() is called every TIME_WHEEL_SLEEP_DURATION to iterate all the entries
    /// in the slot.
    /// If the new duration is greater than the maximun duration (TIME_WHEEL_MAX_SLOTS)
    /// remove it from the hashmap, else reschedule to the new duration.
    #[inline(always)]
    fn expire(&mut self) -> Vec<(KEY, VAL)>
    {
        // select and lock the current time slot in the Vec
        let cur_slot = &self.slot_vec[self.slot_index];
        // returning data store in this vector
        let mut result_vec = Vec::new();
        // TODO replace unwrap()
        let mut cur_slot_lock = cur_slot.entries.lock().unwrap();
        // iterate for each entry in the selected Vec
        while let Some(top) = cur_slot_lock.pop() {
            dbg_fn!((top.clone(), SystemTime::now()));
            let hash_hdr = top.0;
            // exponetial backup is inside the expire (lib)
            // the caller doesn't have to do it
            let duration = top.1 * 2;
            { // Locking the hashmap as short as possible
                let mut hash = self.hash.lock().unwrap();
                if duration < TIME_WHEEL_MAX_SLOTS {
                    // reschedule, don't remove hash entry
                    match hash.get(&hash_hdr) {
                        Some(result) => {
                            // dbg!((hash_hdr.clone(), result.clone()));

                            // insert entry into next slot
                            let next_slot_index = (self.slot_index +
                                                   duration) % self.max_slot;
                            // gather results
                            result_vec.push((hash_hdr.clone(), result.to_owned()));
                            let next_slot = &self.slot_vec[next_slot_index];

                            let mut next_slot_lock = next_slot.
                                entries.lock().unwrap();
                            dbg_fn!(hash_hdr.clone());
                            next_slot_lock.push((hash_hdr, duration));
                            // TODO send it back inside the loop, eliminate return loop
                            // need the to use RetransmitHeader to access the address
                            // can't access it from generic KEY
                        }
                        // TODO clean up
                        None => println!("+++++++++++++++++++empty.")
                    }
                } else {
                    // timeout duration is over the limit, remove hash entry
                    match hash.remove(&hash_hdr) {
                        Some(result) => {
                            // dbg!((hash_hdr.clone(), result.clone()));
                            // gather results
                            result_vec.push((hash_hdr.clone(), result.to_owned()));
                        }
                        // TODO clean up
                        None => println!("+++++++++++++++++++empty.")
                    }
                }
            }
        }
        // next slot with overflow back to 0
        self.slot_index = (self.slot_index + 1) % self.max_slot;
        // TODO test preload this slot in CPU cache?
        result_vec
    }

    fn print(&mut self)
    {
        let slot = &self.slot_vec[0];
        println!("+++++++++++++++++++{:?}", slot);
        let slot = &self.slot_vec[1];
        println!("+++++++++++++++++++{:?}", slot);
        let slot = &self.slot_vec[2];
        println!("+++++++++++++++++++{:?}", slot);
        let slot = &self.slot_vec[3];
        println!("+++++++++++++++++++{:?}", slot);
    }
}



#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
enum StateEnum {
    Active,
    Initial,
    Connect,
    ConnSent,
    Disconnect,
}

const StateEnumLen: usize = 5;

#[derive(Debug, Clone)]
struct CacheMsgType  {
  messages: [Option<Bytes>; MSG_TYPE_MAX],
}

impl CacheMsgType {
    pub fn new() -> Self {
        let messages: [Option<Bytes>; 256] =
            arr![None; 256];
        CacheMsgType {
            messages,
        }
    }
}

#[derive(Debug, Clone)]
struct StateMachine  {
  states: [[Option<StateTypeConst>; MSG_TYPE_MAX] ; STATE_MAX],
}

// TODO
// convert states types into enum
// use 'as usize' to convert.
// don't need to convert msg_type because it fits into u8
impl StateMachine {
    pub fn new() -> Self {
        let mut state_machine = StateMachine {
            states:[[None; MSG_TYPE_MAX]; STATE_MAX],
        };
        // Initialize the state machine.
        // Use insert() to ensure the input & output states
        // are StateEnum and msg_type is u8.
        // TODO insert more states
        state_machine.insert(STATE_DISCONNECT,
                             MSG_TYPE_CONNECT,
                             STATE_CONNECT_SENT);

        state_machine.insert(STATE_CONNECT_SENT,
                             MSG_TYPE_CONNACK,
                             STATE_ACTIVE);

        state_machine.insert(STATE_ACTIVE,
                             MSG_TYPE_PUBLISH,
                             STATE_ACTIVE);
        state_machine
    }

    fn insert(&mut self, cur_state: StateTypeConst,
              msg_type: u8, next_state: StateTypeConst){
        self.states[cur_state as usize][msg_type as usize]
            = Some(next_state);
    }

    // transition (cur_state, input_message) -> next_state
    #[inline(always)]
    pub fn transition(&self, cur_state: StateTypeConst, msg_type: u8)
        -> Option<StateTypeConst> {
        self.states[cur_state as usize][msg_type as usize]
    }
}

#[derive(Debug)]
struct ClientStruct2 {
    // for performance, use lockfree structure
    state: Arc<AtomicU8>,
    addr: SocketAddr,
    broker: UdpSocket,
    timing_wheel: TimingWheel<RetransmitHeader, RetransmitData>,
}

impl ClientStruct2 {
    pub fn new(broker: UdpSocket, timing_wheel: TimingWheel<RetransmitHeader, RetransmitData>)-> Self {
        ClientStruct2 {
            state: Arc::new(AtomicU8::new(STATE_DISCONNECT)),
            addr: "127.0.0.1:60000".parse::<SocketAddr>().unwrap(),
            broker,
            timing_wheel,
        }
    }
    #[inline(always)]
    pub fn state_transition(&self, state_machine: &StateMachine, msg_type: u8)
        -> Option<u8> {
            // get cur_state
            let cur_state = self.state.load(Ordering::SeqCst);
            // transition to new state
            dbg_fn!((cur_state, msg_type));
            // TODO replace unwrap()
            let new_state = state_machine.transition(cur_state, msg_type).unwrap();
            // Save new result, might fail because another thread changed
            // the value, but very unlikely.
            // TODO check return value
            self.state.compare_exchange(cur_state, new_state,
                                        Ordering::Acquire,
                                        Ordering::Relaxed);
            Some(new_state)
        }
    #[inline(always)]
    pub fn get_state(&self) -> u8 {
        self.state.load(Ordering::SeqCst)
    }
}


#[derive(Debug, Clone, Copy)]
struct Flags;

impl Flags {
    pub fn new()-> Self {
        Flags
    }
    #[inline(always)]
    pub fn is_dup(&self, input: u8) -> bool {
        (input & 0b1_0000000) != 0
    }
    #[inline(always)]
    pub fn qos_level(&self, input: u8) -> QoSConst {
        input & 0b0_11_00000
    }
    #[inline(always)]
    pub fn is_retain(&self, input: u8) -> bool {
        (input & 0b000_1_0000) != 0
    }
    #[inline(always)]
    pub fn is_will(&self, input: u8) -> bool {
        (input & 0b0000_1_000) != 0
    }
    #[inline(always)]
    pub fn is_clean_session(&self, input: u8) -> bool {
        (input & 0b00000_1_00) != 0
    }
    #[inline(always)]
    pub fn sess_id_type(&self, input: u8) -> TopicIdTypeConst {
        input & 0b11
    }
    #[inline(always)]
    pub fn set(&self,
               dup: DupConst,
               qos: QoSConst,
               retain: RetainConst,
               will: WillConst,
               clean_session: CleanSessionConst,
               topic_id_type: TopicIdTypeConst) -> u8 {
        dup | qos | retain | will | clean_session | topic_id_type
    }
    #[inline(always)]
    pub fn set_dup(&self, bytes: &[u8], dup: DupConst) -> u8 {
        dup | bytes[2]
    }
}


fn foo(i: u8) -> u8 {
    i+1
}
fn bar(i: u8) -> u8 {
    i+2
}

fn main() {
    // function in an array example
    let functions: Vec<fn(u8) -> u8> = vec![foo, bar];
    println!("foo() = {}, bar() = {}", functions[0](2), functions[1](3));





    init_logging();

    let flags=Flags::new();
    let set_flags = flags.set(
        DUP_TRUE,
        QOS_LEVEL_1,
        RETAIN_TRUE,
        WILL_TRUE,
        CLEAN_SESSION_TRUE,
        TOPIC_ID_TYPE_PRE_DEFINED );

    println!("set_flags: {:#b}", set_flags);
    match flags.qos_level(set_flags) {
        QOS_LEVEL_0 => {
            dbg_fn!("level 0");
        },
        QOS_LEVEL_1 => {
            dbg_fn!("level 1");
        },
        QOS_LEVEL_2 => {
            dbg_fn!("level 2");
        },
        QOS_LEVEL_3 => {
            dbg_fn!("level 3");
        },
        _ => {
            unreachable!();
        }
    }
    dbg_fn!(flags.is_dup(set_flags));
    dbg_fn!(flags.is_retain(set_flags));
    dbg_fn!(flags.is_will(set_flags));
    dbg_fn!(flags.is_clean_session(set_flags));
    match flags.sess_id_type(0b_0_01_0_0_0_10){
        SESSION_ID_TYPE_NORNAL => {
            dbg_fn!("nornal");
        },
        SESSION_ID_TYPE_PRE_DEFINED => {
            dbg_fn!("PreDefined");
        },
        SESSION_ID_TYPE_SHORT => {
            dbg_fn!("Short");
        },
        SESSION_ID_TYPE_UNDEFINED => {
            dbg_fn!("Undefined");
        },
        _ => {
            unreachable!();
        }
    }
    let return_code = 0;
    match return_code {
        RETURN_CODE_ACCEPTED => {
            dbg_fn!("accepted");
        },
        RETURN_CODE_CONGESTION => {
            dbg_fn!("congestion");
        },
        RETURN_CODE_INVALID_TOPIC_ID => {
            dbg_fn!("Invalide topic id");
        },
        RETURN_CODE_NOT_SUPPORTED => {
            dbg_fn!("not supported");
        },
        _ => {
            error!("return code reserved: {:?}", return_code);
        }
    }


    let state_machine = StateMachine::new();
    let r = state_machine.transition(STATE_DISCONNECT, MSG_TYPE_CONNECT);
    dbg_fn!(r);
    let r = state_machine.transition(STATE_CONNECT_SENT, MSG_TYPE_CONNACK);
    dbg_fn!(r);


    let state_machine = StateMachine::new();

    // TODO replace unwrap
    let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
    let (channel_tx, channel_rx):
        (Sender<(SocketAddr, BytesMut)>, Receiver<(SocketAddr, BytesMut)>)
         = mpsc::channel();

    let socket2 = socket.try_clone().expect("couldn't clone the socket");
    let socket3 = socket.try_clone().expect("couldn't clone the socket");
    let socket4 = socket.try_clone().expect("couldn't clone the socket");
    let socket_connect = socket.try_clone().expect("couldn't clone the socket");
    let socket_wheel = socket.try_clone().expect("couldn't clone the socket");
    let expire_socket = socket.try_clone().expect("couldn't clone the socket");
    let subscribe_socket = socket.try_clone().expect("couldn't clone the socket");
    let publish_socket = socket.try_clone().expect("couldn't clone the socket");
    let client_socket = socket.try_clone().expect("couldn't clone the socket");

    /// XXX need to change?
    let remote_addr = "127.0.0.1:60000".parse::<SocketAddr>().unwrap();

    let client_id = generate_client_id();

    let w:TimingWheel<RetransmitHeader, RetransmitData> = TimingWheel::new(TIME_WHEEL_MAX_SLOTS);

    let w2 = Arc::new(Mutex::new(w));
    let w3 = w2.clone();
    let w4 = w2.clone();
    let w5 = w2.clone();
    let w6 = w2.clone();
    let connect_wheel = w2.clone();
    let subsribe_wheel = w2.clone();
    let publish_wheel = w2.clone();
    let recv_wheel = w2.clone();
    
    let tw:TimingWheel<RetransmitHeader, RetransmitData> = TimingWheel::new(TIME_WHEEL_MAX_SLOTS);

    let client = ClientStruct2::new(client_socket, tw);
    client.state_transition(&state_machine, MSG_TYPE_CONNECT);
    // let mut state = *client.cur_state.get_mut();
    dbg_fn!(&client);

    let client_arc = Arc::new(client);
    let client_recv = Arc::clone(&client_arc);
    let client3 = Arc::clone(&client_arc);


    let recv_thread = thread::spawn(move || {
        let mut buf = [0; 1500];
        client_recv.state_transition(&state_machine, MSG_TYPE_CONNECT);
        loop {
            match socket.recv_from(&mut buf) {
                Ok ((size, addr)) => {
                    let msg_type = buf[1] as u8;
                    let mut retrans_hdr = RetransmitHeader {
                        addr,
                        msg_type,
                        topic_id: 0,
                        msg_id: 0,
                    };

                    // received a PUBLISH message
                    // send PUBACK if necessary
                    if msg_type == MsgType::PUBLISH as u8 {
                        match verify_publish2(&buf, size, &socket2, &remote_addr) {
                            Ok((topic_id, msg_id, data)) => {
                                info!("topic_id: {:?} {:?} {:?}", topic_id, msg_id, data);
                                dbg_fn!(data);
                            },
                            Err(why) => error!("Publish {:?}", why),
                        }
                        continue;
                    };
                    // for QoS 1, 2. Not 0 and 3.
                    if msg_type == MsgType::PUBACK as u8 {
                        match verify_puback2(&buf, size) {
                            Ok((topic_id, msg_id)) => {
                                info!("recv_from: {:?} PUBACK", addr);
                                client_recv.state_transition(&state_machine, msg_type);
                                let mut wheel = recv_wheel.lock().unwrap();
                                retrans_hdr.topic_id = topic_id;
                                retrans_hdr.msg_id = msg_id;
                                wheel.cancel(retrans_hdr);
                            },
                            Err(why) => error!("PubAck {:?}", why),
                        }
                        continue;
                    };
                    if msg_type == MsgType::SUBACK as u8 {
                        match verify_suback2(&buf, size) {
                            Ok(msg_id) => {
                                info!("msg_id: {:?}", msg_id);
                                retrans_hdr.msg_id = msg_id;
                                let mut wheel = recv_wheel.lock().unwrap();
                                wheel.cancel(retrans_hdr);
                            },
                            Err(why) => error!("SubAck {:?}", why),
                        }
                        continue;
                    };
                    if msg_type == MsgType::CONNACK as u8 {
                        info!("recv_from: {:?} CONNACK", addr);
                        match verify_connack2(&buf, size) {
                            Ok(_) => {
                                // received connack
                                {
                                    let mut wheel = recv_wheel.lock().unwrap();
                                    wheel.cancel(retrans_hdr);
                                }
                                dbg_fn!(&client_recv);
                                client_recv.state_transition(&state_machine, MSG_TYPE_CONNACK);
                                dbg_fn!(&client_recv);
                            },
                            Err(why) => error!("ConnAck {:?}", why),
                        }
                        continue;
                    };
                }

                Err (why) => {
                    error!("{}", why);
                }
            }
        }
    });

    let connect_thread = thread::spawn(move || {
        let bytes = connect2(client_id);
        socket_connect.send_to(&bytes.to_owned(), remote_addr);
        let data = RetransmitData {
            bytes,
        };
        let retrans_hdr = RetransmitHeader {
            addr: remote_addr.clone(),
            msg_type: MsgType::CONNACK as u8,
            topic_id: 0,
            msg_id: 0,
        };
        {
            let mut wheel = connect_wheel.lock().unwrap();
            wheel.schedule(retrans_hdr, data, TIME_WHEEL_INIT_DURATION);
        }
    });

    let channel_rx_thread = thread::spawn(move || {
        loop {
            match channel_rx.recv(){
                Ok ((addr, bytes)) => {
                    socket4.send_to(&bytes[..], addr);
                }
                Err(why) => {
                    println!("{}", why);
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
                let mut ww = w3.lock().unwrap();
                let v =  ww.expire();
                // TODO this lock is too long
                for ack in v {
                    dbg!(ack.clone());
                    let (retrans_hdr, data) = ack;
                    let hdr:RetransmitHeader = retrans_hdr;
                    let addr = hdr.addr;
                    let data:RetransmitData = data;
                    let data_gram = data.bytes;
                    dbg!((addr, data_gram.clone()));
                    // TODO use mpsc
                    socket3.send_to(&data_gram, remote_addr);
            }
            // let len = socket_tx3.send_to(b"hello", remote_addr);

                }
            thread::sleep(Duration::from_millis(TIME_WHEEL_SLEEP_DURATION as u64));
        }
    });


    // Wait for CONNACK response to change the state to ACTIVE.
    while client3.get_state() != STATE_ACTIVE {
        hint::spin_loop();
    }

    let subscribe_thread = thread::spawn(move || {
        // Wait for the other thread to release the lock

        // Wait for CONNACK response to change the state to ACTIVE.
        while client3.get_state() != STATE_ACTIVE {
            hint::spin_loop();
        }
        let msg_id:u16 = 1;
        // Send SUBSCRIBE
        let bytes = subscribe("hello".to_string(), 1);
        subscribe_socket.send_to(&bytes.to_owned(), remote_addr);
        let data = RetransmitData {
            bytes,
        };
        let retrans_hdr = RetransmitHeader {
            addr: remote_addr.clone(),
            msg_type: MsgType::SUBACK as u8,
            topic_id: 0,
            msg_id,
        };
        {
            let mut wheel = subsribe_wheel.lock().unwrap();
            wheel.schedule(retrans_hdr, data, TIME_WHEEL_INIT_DURATION);
        }
    });

    let publish_thread = thread::spawn(move || {
        let mut msg_id = 0;
        let topic_id = 1;
        let qos_level = QOS_LEVEL_1;
        let flags=Flags::new();
        loop {
            let sub_msg = format!("recursive_  {:?}", msg_id);
            msg_id = (msg_id + 1) % 64000;
            let mut bytes = publish(topic_id, msg_id, &sub_msg,
                                qos_level, RETAIN_TRUE,
                                TOPIC_ID_TYPE_NORNAL);

            publish_socket.send_to(&bytes, remote_addr);
            // For QoS level 1 or 2, restransmit if the receiver doesn't ack before timeout.
            // Set the duplicate flag.
            if qos_level == QOS_LEVEL_1 || qos_level == QOS_LEVEL_2 {
                flags.set_dup(&bytes, DUP_TRUE);
                // dbg_fn!(&bytes);
                let retrans_hdr = RetransmitHeader {
                    addr: remote_addr.clone(),
                    msg_type: MsgType::PUBLISH as u8,
                    topic_id,
                    msg_id,
                };
                let data = RetransmitData {
                    bytes,
                };
                /*
                   {
                   let mut wheel = publish_wheel.lock().unwrap();
                   wheel.schedule(retrans_hdr, data, TIME_WHEEL_INIT_DURATION);
                   }
                 */
            }


            /*
            // let sub_bytes = subscribe(sub_msg, msg_id);
            // let buf2 = publish(1, msg_id, String::from_utf8_lossy(&sub_bytes[..]).to_string(), 0);
            {
                //let mut ww = w2.lock().unwrap();
                //ww.schedule(wait2, data2, TIME_WHEEL_INIT_DURATION);
                //    ww.schedule(wait3, data3, TIME_WHEEL_INIT_DURATION + 100);
                //    ww.schedule(wait4, data4, TIME_WHEEL_INIT_DURATION + 500);
            }
            */
            thread::sleep(Duration::from_millis(2000));
            /*
               {
               let mut ww = w2.lock().unwrap();
               ww.cancel(wait2);
               }
               */

            // break;
        }
    });

    /*
    // Channels have two endpoints: the `Sender<T>` and the `Receiver<T>`,
    // where `T` is the type of the message to be transferred
    // (type annotation is superfluous)
    let (tx, rx): (Sender<(i32, String)>, Receiver<(i32, String)>) = mpsc::channel();
    let mut children = Vec::new();

    for id in 0..NTHREADS {
        // The sender endpoint can be copied
        let thread_tx = tx.clone();

        // Each thread will send its id via the channel
        let child = thread::spawn(move || {
            // The thread takes ownership over `thread_tx`
            // Each thread queues a message in the channel
            thread_tx.send((id, "hello".to_owned())).unwrap();

            // Sending is a non-blocking operation, the thread will continue
            // immediately after sending its message
            println!("thread {} finished", id);
        });

        children.push(child);
    }

    // Here, all the messages are collected
    let mut ids = Vec::with_capacity(NTHREADS as usize);
    for _ in 0..NTHREADS {
        // The `recv` method picks a message from the channel
        // `recv` will block the current thread if there are no messages available
        ids.push(rx.recv());
    }

    // Wait for the threads to complete any remaining work
    for child in children {
        child.join().expect("oops! the child thread panicked");
    }

    // Show the order in which the messages were sent
    println!("{:?}", ids);
    */
    connect_thread.join().expect("The sender thread has panicked");
    subscribe_thread.join().expect("The sender thread has panicked");
    recv_thread.join().expect("The sender thread has panicked");
    expire_thread.join().expect("The sender thread has panicked");
    publish_thread.join().expect("The sender thread has panicked");
    channel_rx_thread.join().expect("The sender thread has panicked");
}

fn init_logging() {
    TermLogger::init(
        LevelFilter::Info,
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )
    .unwrap();
}
