use crate::{eformat, function};
use custom_debug::Debug;

// Variable length message type:
// Connect, Publish, Subscribe, WillTopic, WillMsg, WillTopicUpd, WillMsgUpd, ForwardEncap,

#[derive(Debug, Copy, Clone)]
pub struct MsgHeader {
    pub len: u16,
    pub header_len: u8,
    #[debug(format = "0x{:x}")]
    pub msg_type: u8,
}

/*
From MQTT-SN v1.2 spec.
The Length field is either 1- or 3-octet long and specifies the total number of octets contained in the message
(including the Length field itself).
If the first octet of the Length field is coded “0x01” then the Length field is 3-octet long; in this case, the two
following octets specify the total number of octets of the message (most-significant octet first). Otherwise, the
Length field is only 1-octet long and specifies itself the total number of octets contained in the message.
The 3-octet format allows the encoding of message lengths up to 65535 octets. Messages with lengths smaller
than 256 octets may use the shorter 1-octet format.
Note that because MQTT-SN does not support message fragmentation and reassembly, the maximum message
length that could be used in a network is governed by the maximum packet size that is supported by that network,
and not by the maximum length that could be encoded by MQTT-SN.
*/

impl MsgHeader {
    pub fn try_read(buf: &[u8], size: usize) -> Result<MsgHeader, String> {
        let len;
        let msg_type;
        let mut header_len = 2;
        if size >= 3 {
            // Determine 2 or 4 byte header.
            if buf[0] != 1 {
                len = buf[0] as u16;
                msg_type = buf[1] as u8;
            } else {
                len = (buf[1] as u16) << 8 | buf[2] as u16;
                msg_type = buf[3] as u8;
                header_len = 4;
            }
            if size == len as usize {
                return Ok(MsgHeader {
                    len,
                    header_len,
                    msg_type,
                });
            }
            return Err(eformat!(
                //" Message length doesn't match size",
                len, size
            ));
        } else {
            return Err(eformat!("Message is too short", size));
        }
    }
}

/*
pub enum MsgBody {
    Connect(Connect),
    Connack(Connack),
    WillTopicReq(WillTopicReq),
    WillTopic(WillTopic),
    WillMsgReq(WillMsgReq),
    WillMsg(WillMsg),
    Register(Register),
    Regack(Regack),
    Publish(Publish),
    Puback(Puback),
    Pubcomp(Pubcomp),
    Pubrec(Pubrec),
    Pubrel(Pubrel),
    Subscribe(Subscribe),
    Suback(Suback),
    Unsubscribe(Unsubscribe),
    Unsuback(Unsuback),
    Pingreq(Pingreq),
    Pingresp(Pingresp),
    Disconnect(Disconnect),
    WillTopicUpd(WillTopicUpd),
    WillTopicResp(WillTopicResp),
    WillMsgUpd(WillMsgUpd),
    WillMsgResp(WillMsgResp),
    MsgTypeErr(MsgTypeErr),
}

pub Msg {
    header: MsgHeader,
    body: MsgBody,
    buf: [u8],
}

impl Msg {
    pub fn new(msg_type: MsgType, body: MsgBody) -> Self {
        Msg {
            header: MsgHeader {
                len: 0,
                msg_type: msg_type,
                buf: [0; 2],
            },
            body: body,
        }
    }
    pub fn try_read(&mut self, buf: [u8], size: usize) -> Result<Msg, String> {
        if size < 3 {
                return Err(eformat!( "Message is too short"));
        }
        // Determine 2 or 4 byte header.
        let mut msg_type_index = 1;
        if buf[0] != 1 {
            self.header.len = buf[0] as u16;
            self.header.msg_type = MsgType::from_u8(buf[1]).unwrap();
        } else {
            self.header.len = (buf[1] as u16) << 8 | buf[2] as u16;
            self.header.msg_type = MsgType::from_u8(buf[3]).unwrap();
            header_len = 3;
        }
        if size != self.header.len as usize {
            return Err(format!(
                " Message length doesn't match size",
                self.header.len, size
            ));
        }
        self.buf = buf;


        msg_type = MsgType::from_u8(buf[msg_type_index]).unwrap();
        match msg_type {
            MsgType::CONNECT => {
                body = MsgBody::Connect(Connect::try_read(buf, len as usize));
            }
            MsgType::CONNACK => {
                body = MsgBody::Connack(Connack::try_read(buf, len as usize));
            }
            MsgType::WILLTOPICREQ => {
                body = MsgBody::WillTopicReq(WillTopicReq::try_read(buf, len as usize));
            }
            MsgType::WILLTOPIC => {
                body = MsgBody::WillTopic(WillTopic::try_read(buf, len as usize));
            }
            MsgType::WILLMSGREQ => {
                body = MsgBody::WillMsgReq(WillMsgReq::try_read(buf, len as usize));
            }
            MsgType::WILLMSG => {
                body = MsgBody::WillMsg(WillMsg::try_read(buf, len as usize));
            }
            MsgType::REGISTER => {
                body = MsgBody::Register(Register::try_read(buf, len as usize));
            }
            MsgType::REGACK => {
                body = MsgBody::Regack(Regack::try_read(buf, len as usize));
            }
            MsgType::PUBLISH => {
                body
            }
}

*/

#[cfg(test)]
mod test {
    #[test]
    fn test_msg_header_read() {
        let msg_header = super::MsgHeader::try_read(&[1, 2, 3, 4], 4);
        dbg!(msg_header);
        let msg_header = super::MsgHeader::try_read(&[4, 2, 3, 4], 4);
        dbg!(msg_header);
        let mut bytes: [u8; 1024] = [0; 1024];
        bytes[0] = 1;
        bytes[1] = 1;
        bytes[2] = 0;
        bytes[3] = 0xf;
        dbg!(bytes.len());
        let msg_header = super::MsgHeader::try_read(&bytes, 256).unwrap();
        dbg!(msg_header);
        dbg!(&bytes[3..]);
        dbg!(&bytes[5..]);
    }
}
