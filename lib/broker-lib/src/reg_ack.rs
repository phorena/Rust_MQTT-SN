/*
5.4.11 REGACK
Length    MsgType TopicId MsgId ReturnCode
(octet 0) (1)     (2,3)   (4,5) (6)
Table 15: REGACK Message

The REGACK message is sent by a client or by a GW as an acknowledgment to the receipt and processing of
a REGISTER message. Its format is illustrated in Table 15:
• Length and MsgType: see Section 5.2.
• TopicId: the value that shall be used as topic id in the PUBLISH messages;
• MsgId: same value as the one contained in the corresponding REGISTER message.
• ReturnCode: “accepted”, or rejection reason.
*/
use bytes::{BufMut, BytesMut};
use custom_debug::Debug;
use getset::{CopyGetters, Getters, MutGetters};
use std::mem;

use crate::{
    broker_lib::MqttSnClient, eformat, function, msg_hdr::MsgHeader,
    retransmit::RetransTimeWheel, MSG_LEN_REGACK, MSG_TYPE_REGACK,
};

#[derive(Debug, Clone, Getters, MutGetters, CopyGetters, Default)]
#[getset(get, set)]
pub struct RegAck {
    pub len: u8,
    #[debug(format = "0x{:x}")]
    pub msg_type: u8,
    pub topic_id: u16,
    pub msg_id: u16,
    pub return_code: u8,
}
impl RegAck {
    pub fn recv(
        buf: &[u8],
        size: usize,
        client: &MqttSnClient,
        msg_header: MsgHeader,
    ) -> Result<(), String> {
        let (reg_ack, read_len) = RegAck::try_read(buf, size).unwrap();
        dbg!(reg_ack.clone());

        let remote_socket_addr = msg_header.remote_socket_addr;
        if read_len == MSG_LEN_REGACK as usize {
            match RetransTimeWheel::cancel_timer(
                remote_socket_addr,
                reg_ack.msg_type,
                reg_ack.topic_id,
                reg_ack.msg_id,
            ) {
                Ok(()) => Ok(()),
                Err(err) => Err(err),
            }
        } else {
            Err(eformat!(remote_socket_addr, "size", buf[0]))
        }
    }
    pub fn send(
        topic_id: u16,
        msg_id: u16,
        return_code: u8,
        client: &MqttSnClient,
        msg_header: MsgHeader,
    ) -> Result<(), String> {
        let remote_socket_addr = msg_header.remote_socket_addr;
        let reg_ack = RegAck {
            len: MSG_LEN_REGACK,
            msg_type: MSG_TYPE_REGACK,
            topic_id,
            msg_id,
            return_code,
        };
        let mut bytes_buf = BytesMut::with_capacity(MSG_LEN_REGACK as usize);
        dbg!(reg_ack.clone());
        reg_ack.try_write(&mut bytes_buf);
        dbg!(bytes_buf.clone());
        dbg!(remote_socket_addr);
        // transmit to network
        match client
            .egress_tx
            .try_send((remote_socket_addr, bytes_buf.to_owned()))
        {
            Ok(_) => Ok(()),
            Err(err) => Err(eformat!(remote_socket_addr, err)),
        }
    }
}
