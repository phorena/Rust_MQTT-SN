#[derive( Debug, Copy, Clone,)]
pub struct MsgHeader {
    len: u16,
    header_len: u8,
    msg_type: u8,
}

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

impl MsgHeader {
    pub fn try_read(buf: &[u8], size: usize) -> Result<MsgHeader, String> {
        let mut len = 0;
        let mut header_len = 2;
        let mut msg_type = 0xff;
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
            return Err(format!(
                "{}: Message length({}) doesn't match size({})",
                function!(),
                len,
                size,
            ));
        } else {
            return Err(format!("{}: Message is too short", function!(),));
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
                return Err(format!(
                    "{}: Message is too short",
                    function!(),
                ));
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
                "{}: Message length({}) doesn't match size({})",
                function!(), self.header.len, size,
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
        let mut bytes:[u8;1024] = [0; 1024];
        bytes[0] = 1;
        bytes[1] = 1;
        bytes[2] = 0;
        bytes[3] = 0xf;
        dbg!(bytes.len());
        let msg_header = super::MsgHeader::try_read(&bytes, 256).unwrap();
        dbg!(msg_header);
        let len = msg_header.len as usize;
        dbg!(&bytes[3..]);
        dbg!(&bytes[5..]);
    }
}