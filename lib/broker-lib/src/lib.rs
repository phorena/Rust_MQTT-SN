#[warn(non_snake_case)]
#[macro_use]
extern crate arrayref;
#[macro_use]
extern crate lazy_static;

// TODO fix non_snake_case.
#[allow(non_snake_case)]
pub mod BrokerLib;
pub mod advertise;
pub mod conn_ack;
pub mod connect;
pub mod connection;
// pub mod ConnectionDb;
pub mod disconnect;
// pub mod Functions;
// pub mod MainMachineClient;
#[allow(non_snake_case)]
pub mod MsgType;
#[allow(non_snake_case)]
pub mod StateEnum;
#[allow(non_snake_case)]
pub mod StateMachine;
#[allow(non_snake_case)]
pub mod SubscriberDb;
#[allow(non_snake_case)]
pub mod TimingWheel2;
#[allow(non_snake_case)]
pub mod TopicDb;
pub mod filter;
pub mod ping_req;
pub mod ping_resp;
pub mod pub_ack;
pub mod pub_comp;
pub mod pub_rec;
pub mod pub_rel;
pub mod publish;
#[allow(non_snake_case)]
pub mod reg_ack;
pub mod register;
pub mod sub_ack;
pub mod subscribe;
pub mod unsub_ack;
pub mod unsubscribe;
pub mod will_msg;
pub mod will_msg_req;
pub mod will_msg_resp;
pub mod will_msg_upd;
pub mod will_topic;
pub mod will_topic_req;
pub mod will_topic_resp;
pub mod will_topic_upd;
// pub mod client_struct;
pub mod flags;
pub mod gw_info;
pub mod message;
pub mod pub_msg_cache;
pub mod search_gw;

// pub mod BrokerLib;
// #[allow(non_snake_case)]
// pub mod Channels;

pub const MTU: usize = 1500;

pub type TopicIdType = u16;
pub type MsgIdType = u16;

pub type MsgTypeConst = u8;
pub const MSG_TYPE_ADVERTISE: MsgTypeConst = 0x0;
pub const MSG_TYPE_SEARCH_GW: MsgTypeConst = 0x1;
pub const MSG_TYPE_GW_INFO: MsgTypeConst = 0x2;
pub const MSG_TYPE_CONNECT: MsgTypeConst = 0x4;
pub const MSG_TYPE_CONNACK: MsgTypeConst = 0x5;
pub const MSG_TYPE_SUBSCRIBE: MsgTypeConst = 0x12;
pub const MSG_TYPE_SUBACK: MsgTypeConst = 0x13;
pub const MSG_TYPE_UNSUBSCRIBE: MsgTypeConst = 0x14;
pub const MSG_TYPE_UNSUBACK: MsgTypeConst = 0x15;
pub const MSG_TYPE_PUBLISH: MsgTypeConst = 0xC; // should be 0, most popular
pub const MSG_TYPE_PUBACK: MsgTypeConst = 0xD;
pub const MSG_TYPE_PUBCOMP: MsgTypeConst = 0xE;
pub const MSG_TYPE_PUBREC: MsgTypeConst = 0xF;
pub const MSG_TYPE_PUBREL: MsgTypeConst = 0x10;
pub const MSG_TYPE_DISCONNECT: MsgTypeConst = 0x18;
pub const MSG_TYPE_WILL_TOPIC_REQ: MsgTypeConst = 0x06;
pub const MSG_TYPE_WILL_TOPIC: MsgTypeConst = 0x07;
pub const MSG_TYPE_WILL_MSG_REQ: MsgTypeConst = 0x08;
pub const MSG_TYPE_WILL_MSG: MsgTypeConst = 0x09;
pub const MSG_TYPE_WILL_TOPIC_RESP: MsgTypeConst = 0x1B;
pub const MSG_TYPE_WILL_MSG_RESP: MsgTypeConst = 0x1D;
pub const MSG_TYPE_WILL_TOPIC_UPD: MsgTypeConst = 0x1A;
pub const MSG_TYPE_WILL_MSG_UPD: MsgTypeConst = 0x1C;
pub const MSG_TYPE_PINGREQ: MsgTypeConst = 0x16;
pub const MSG_TYPE_PINGRESP: MsgTypeConst = 0x17;
pub const MSG_TYPE_REGISTER: MsgTypeConst = 0x0A;
pub const MSG_TYPE_REGACK: MsgTypeConst = 0x0B;

// TODO fill in the rest
pub const MSG_TYPE_WILLMSGRESP: MsgTypeConst = 0x1D; // 29

// 0x1E-0xFD reserved
pub const MSG_TYPE_ENCAP_MSG: MsgTypeConst = 0xFE;
// XXX not an optimal choice because, array of MsgTypeConst
// must include 256 entries.
// For the 2x2 array [0..6][0..255] states,
// instead of array  [0..6][0..29] states.
//
//

pub const MSG_TYPE_MAX: usize = 256;

pub const STATE_ENUM_LEN: usize = 5;

pub type MsgLenConst = u8;
pub const MSG_LEN_ADVERTISE: MsgLenConst = 5;
pub const MSG_LEN_SEARCH_GW: MsgLenConst = 3;
pub const MSG_LEN_PUBACK: MsgLenConst = 7;
pub const MSG_LEN_PUBREC: MsgLenConst = 4;
pub const MSG_LEN_PUBREL: MsgLenConst = 4;
pub const MSG_LEN_PUBCOMP: MsgLenConst = 4;
pub const MSG_LEN_SUBACK: MsgLenConst = 8;
pub const MSG_LEN_REGACK: MsgLenConst = 7;
pub const MSG_LEN_CONNACK: MsgLenConst = 3;
pub const MSG_LEN_DISCONNECT: MsgLenConst = 2;
pub const MSG_LEN_DISCONNECT_DURATION: MsgLenConst = 4;
pub const MSG_LEN_WILL_TOPIC_REQ: MsgLenConst = 2;
pub const MSG_LEN_WILL_MSG_REQ: MsgLenConst = 2;
pub const MSG_LEN_WILL_TOPIC_RESP: MsgLenConst = 3;
pub const MSG_LEN_WILL_MSG_RESP: MsgLenConst = 3;
pub const MSG_LEN_PINGRESP: MsgLenConst = 2;
pub const MSG_LEN_UNSUBACK: MsgLenConst = 4;

pub const MSG_LEN_GW_INFO_HEADER: MsgLenConst = 3;
pub const MSG_LEN_WILL_TOPIC_HEADER: MsgLenConst = 3;
pub const MSG_LEN_WILL_MSG_HEADER: MsgLenConst = 2;
pub const MSG_LEN_WILL_TOPIC_UPD_HEADER: MsgLenConst = 3;
pub const MSG_LEN_WILL_MSG_UPD_HEADER: MsgLenConst = 2;
pub const MSG_LEN_PUBLISH_HEADER: MsgLenConst = 6;
pub const MSG_LEN_CONNECT_HEADER: MsgLenConst = 6;
pub const MSG_LEN_PINGREQ_HEADER: MsgLenConst = 2;
pub const MSG_LEN_SUBSCRIBE_HEADER: MsgLenConst = 7;
pub const MSG_LEN_UNSUBSCRIBE_HEADER: MsgLenConst = 7;
pub const MSG_LEN_REGISTER_HEADER: MsgLenConst = 6;

type ReturnCodeConst = u8;
const RETURN_CODE_ACCEPTED: ReturnCodeConst = 0;
// const RETURN_CODE_CONGESTION: ReturnCodeConst = 1;
// const RETURN_CODE_INVALID_TOPIC_ID: ReturnCodeConst = 2;
// const RETURN_CODE_NOT_SUPPORTED: ReturnCodeConst = 3;

#[macro_export]
macro_rules! function {
    () => {{
        fn f() {}
        fn type_name_of<T>(_: T) -> &'static str {
            std::any::type_name::<T>()
        }
        let name = type_name_of(f);
        &name[..name.len() - 3]
    }};
}

// dbg macro that prints function name instead of file name.
// https://stackoverflow.com/questions/65946195/understanding-the-dbg-macro-in-rust
#[macro_export]
macro_rules! dbg_fn2 {
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
#[macro_export]
/// MUST ALSO IMPORT function!()
macro_rules! eformat {
    ($exp:expr,$exp2:expr) => {
        format!("{}:{} {} {}", function!(), line!(), $exp, $exp2)
    };
    ($exp:expr) => {
        format!("{}:{} {}", function!(), line!(), $exp)
    };
    ($exp:expr,$exp2:expr,$exp3:expr) => {
        format!("{}:{} {} {} {}", function!(), line!(), $exp, $exp2, $exp3)
    };
    ($exp:expr,$exp2:expr,$exp3:expr,$exp4:expr) => {
        format!(
            "{}:{} {} {} {} {}",
            function!(),
            line!(),
            $exp,
            $exp2,
            $exp3,
            $exp4
        )
    };
}

// dbg macro that prints function name instead of file name.
// https://stackoverflow.com/questions/65946195/understanding-the-dbg-macro-in-rust
#[macro_export]
macro_rules! dbg_fn {
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
        ($($dbg_fn!($val)),+,)
    };
}

#[macro_export]
macro_rules! dbg_buf {
    ($buf:ident, $size:ident) => {
        let mut i: usize = 0;
        eprint!("[{}{}] ", file!(), line!());
        while i < $size {
            eprint!("{:#04X?} ", $buf[i]);
            i += 1;
        }
        eprintln!("");
    };
}
