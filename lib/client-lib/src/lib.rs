#[warn(non_snake_case)]
#[macro_use]
extern crate arrayref;
pub mod Advertise;
pub mod ClientLib;
pub mod ConnAck;
pub mod Connect;
// pub mod ConnectionDb;
pub mod DebugFunctions;
pub mod Disconnect;
pub mod Errors;
// pub mod Functions;
// pub mod MainMachineClient;
pub mod MessageDb;
pub mod MsgType;
pub mod PingReq;
pub mod PingResp;
pub mod PubAck;
pub mod PubRec;
pub mod PubRel;
pub mod Publish;
pub mod RegAck;
pub mod Register;
pub mod StateEnum;
pub mod StateMachine;
pub mod SubAck;
pub mod Subscribe;
pub mod SubscriberDb;
pub mod TimingWheel2;
pub mod TopicDb;
pub mod Transfer;
pub mod UnsubAck;
pub mod Unsubscribe;
pub mod WillMsg;
pub mod WillMsgReq;
pub mod WillMsgResp;
pub mod WillMsgUpd;
pub mod WillTopic;
pub mod WillTopicReq;
pub mod WillTopicResp;
pub mod WillTopicUpd;
pub mod client_struct;
pub mod flags;
// pub mod BrokerLib;
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
pub const MSG_TYPE_PINGRESP: MsgTypeConst = 0x17;
// 0x1E-0xFD reserved
pub const MSG_TYPE_ENCAP_MSG: MsgTypeConst = 0xFE;
// XXX not an optimal choice because, array of MsgTypeConst
// must include 256 entries.
// For the 2x2 array [0..6][0..255] states,
// instead of array  [0..6][0..29] states.
//
//

pub const MSG_TYPE_MAX: usize = 256;

pub const StateEnumLen: usize = 5;

pub type MsgLenConst = u8;
pub const MSG_LEN_PUBACK: MsgLenConst = 7;
pub const MSG_LEN_PUBREC: MsgLenConst = 4;
pub const MSG_LEN_PUBREL: MsgLenConst = 4;
pub const MSG_LEN_PUBCOMP: MsgLenConst = 4;
pub const MSG_LEN_SUBACK: MsgLenConst = 8;
pub const MSG_LEN_CONNACK: MsgLenConst = 3;
pub const MSG_LEN_PINGRESP: MsgLenConst = 2;

type ReturnCodeConst = u8;
const RETURN_CODE_ACCEPTED: ReturnCodeConst = 0;
const RETURN_CODE_CONGESTION: ReturnCodeConst = 1;
const RETURN_CODE_INVALID_TOPIC_ID: ReturnCodeConst = 2;
const RETURN_CODE_NOT_SUPPORTED: ReturnCodeConst = 3;
