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
pub mod DebugFunctions;
#[allow(non_snake_case)]
pub mod Disconnect;
#[allow(non_snake_case)]
pub mod Errors;
// pub mod Functions;
// pub mod MainMachineClient;
#[allow(non_snake_case)]
pub mod Filter;
#[allow(non_snake_case)]
pub mod MessageDb;
#[allow(non_snake_case)]
pub mod MsgType;
#[allow(non_snake_case)]
pub mod PingReq;
#[allow(non_snake_case)]
pub mod PingResp;
#[allow(non_snake_case)]
pub mod PubAck;
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
pub mod Transfer;
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
pub mod client_struct;
pub mod flags;
// pub mod BrokerLib;
#[allow(non_snake_case)]
pub mod Channels;

pub const MTU: usize = 1500;

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

type ReturnCodeConst = u8;
const RETURN_CODE_ACCEPTED: ReturnCodeConst = 0;
// const RETURN_CODE_CONGESTION: ReturnCodeConst = 1;
// const RETURN_CODE_INVALID_TOPIC_ID: ReturnCodeConst = 2;
// const RETURN_CODE_NOT_SUPPORTED: ReturnCodeConst = 3;