// use num;
use num_derive::FromPrimitive;
use num_enum::IntoPrimitive;

//#[macro_use]
//extern crate num_derive;

#[derive(
    FromPrimitive, IntoPrimitive, PartialEq, Eq, Hash, Debug, Copy, Clone,
)]
#[allow(non_camel_case_types)]
#[repr(u8)]
pub enum MsgType {
    ADVERTISE = 0,
    SEARCHGW,
    GWINFO,
    // Reserved0x03,
    CONNECT = 0x04,
    CONNACK,
    WILLTOPICREQ,
    WILLTOPIC,
    WILLMSGREQ,
    WILLMSG,
    REGISTER,
    REGACK,
    PUBLISH,
    PUBACK,
    PUBCOMP,
    PUBREC,
    PUBREL,
    // Reserved0x11,
    SUBSCRIBE = 0x12,
    SUBACK,
    UNSUBSCRIBE,
    UNSUBACK,
    PINGREQ,
    PINGRESP,
    DISCONNECT,
    // Reserved0x19,
    WILLTOPICUPD = 0x1A,
    WILLTOPICRESP,
    WILLMSGUPD,
    WILLMSGRESP,
    MSG_TYPE_ERR = 0xFF,
}
