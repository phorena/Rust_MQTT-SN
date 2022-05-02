#[warn(non_snake_case)]
#[macro_use]
extern crate arrayref;
#[macro_use]
extern crate lazy_static;

// TODO fix non_snake_case.
#[allow(non_snake_case)]
pub mod Advertise;
#[allow(non_snake_case)]
pub mod BrokerLib;
#[allow(non_snake_case)]
pub mod ConnAck;
#[allow(non_snake_case)]
pub mod Connect;
#[allow(non_snake_case)]
pub mod Connection;
// pub mod ConnectionDb;
#[allow(non_snake_case)]
pub mod Disconnect;
// pub mod Functions;
// pub mod MainMachineClient;
#[allow(non_snake_case)]
pub mod Filter;
#[allow(non_snake_case)]
pub mod MsgType;
#[allow(non_snake_case)]
pub mod PingReq;
#[allow(non_snake_case)]
pub mod PingResp;
#[allow(non_snake_case)]
pub mod PubAck;
#[allow(non_snake_case)]
pub mod PubComp;
#[allow(non_snake_case)]
pub mod PubRec;
#[allow(non_snake_case)]
pub mod PubRel;
#[allow(non_snake_case)]
pub mod Publish;
#[allow(non_snake_case)]
pub mod RegAck;
#[allow(non_snake_case)]
pub mod Register;
#[allow(non_snake_case)]
pub mod StateEnum;
#[allow(non_snake_case)]
pub mod StateMachine;
#[allow(non_snake_case)]
pub mod SubAck;
#[allow(non_snake_case)]
pub mod Subscribe;
#[allow(non_snake_case)]
pub mod SubscriberDb;
#[allow(non_snake_case)]
pub mod TimingWheel2;
#[allow(non_snake_case)]
pub mod TopicDb;
#[allow(non_snake_case)]
pub mod UnsubAck;
#[allow(non_snake_case)]
pub mod Unsubscribe;
#[allow(non_snake_case)]
pub mod WillMsg;
#[allow(non_snake_case)]
pub mod WillMsgReq;
#[allow(non_snake_case)]
pub mod WillMsgResp;
#[allow(non_snake_case)]
pub mod WillMsgUpd;
#[allow(non_snake_case)]
pub mod WillTopic;
#[allow(non_snake_case)]
pub mod WillTopicReq;
#[allow(non_snake_case)]
pub mod WillTopicResp;
#[allow(non_snake_case)]
pub mod WillTopicUpd;
// pub mod client_struct;
pub mod flags;
pub mod message;
pub mod pub_msg_cache;

// pub mod BrokerLib;
#[allow(non_snake_case)]
pub mod Channels;

pub const MTU: usize = 1500;

pub type TopicIdType = u16;
pub type MsgIdType = u16;

pub type MsgTypeConst = u8;
pub const MSG_TYPE_CONNECT: MsgTypeConst = 0x4;
pub const MSG_TYPE_CONNACK: MsgTypeConst = 0x5;
pub const MSG_TYPE_SUBSCRIBE: MsgTypeConst = 0x12;
pub const MSG_TYPE_SUBACK: MsgTypeConst = 0x13;
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
pub const MSG_LEN_PUBACK: MsgLenConst = 7;
pub const MSG_LEN_PUBREC: MsgLenConst = 4;
pub const MSG_LEN_PUBREL: MsgLenConst = 4;
pub const MSG_LEN_PUBCOMP: MsgLenConst = 4;
pub const MSG_LEN_SUBACK: MsgLenConst = 8;
pub const MSG_LEN_CONNACK: MsgLenConst = 3;
pub const MSG_LEN_DISCONNECT: MsgLenConst = 2;
pub const MSG_LEN_DISCONNECT_DURATION: MsgLenConst = 4;
pub const MSG_LEN_WILL_TOPIC_REQ: MsgLenConst = 2;
pub const MSG_LEN_WILL_MSG_REQ: MsgLenConst = 2;
pub const MSG_LEN_WILL_TOPIC_RESP: MsgLenConst = 3;
pub const MSG_LEN_WILL_MSG_RESP: MsgLenConst = 3;
pub const MSG_LEN_WILL_TOPIC_HEADER: MsgLenConst = 3;
pub const MSG_LEN_WILL_MSG_HEADER: MsgLenConst = 2;
pub const MSG_LEN_WILL_TOPIC_UPD_HEADER: MsgLenConst = 3;
pub const MSG_LEN_WILL_MSG_UPD_HEADER: MsgLenConst = 2;
pub const MSG_LEN_PUBLISH_HEADER: MsgLenConst = 6;
pub const MSG_LEN_CONNECT_HEADER: MsgLenConst = 6;
pub const MSG_LEN_PINGREQ_HEADER: MsgLenConst = 2;
pub const MSG_LEN_PINGRESP: MsgLenConst = 2;

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
