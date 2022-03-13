use num_derive::FromPrimitive;
use num_enum::IntoPrimitive;

#[derive(FromPrimitive, IntoPrimitive, PartialEq, Eq, Hash, Debug, Copy, Clone)]
#[repr(u8)]
pub enum MsgType {
    ADVERTISE,
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
