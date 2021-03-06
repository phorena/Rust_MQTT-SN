/*
5.4.18 UNSUBACK
Length MsgType MsgId
(octet 0) (1) (2-3)
Table 21: UNSUBACK Message
An UNSUBACK message is sent by a GW to acknowledge the receipt and processing of an UNSUBSCRIBE
message. Its format is illustrated in Table 21:
• Length and MsgType: see Section 5.2.
• MsgId: same value as the one contained in the corresponding UNSUBSCRIBE message.
*/
use crate::{
    broker_lib::MqttSnClient, eformat, function, msg_hdr::MsgHeader,
    retransmit::RetransTimeWheel, MSG_LEN_UNSUBACK, MSG_TYPE_UNSUBACK,
};
use bytes::{BufMut, BytesMut};
use custom_debug::Debug;
use getset::{CopyGetters, Getters, MutGetters};
use std::mem;

#[derive(Debug, Clone, Getters, MutGetters, CopyGetters, Default)]
#[getset(get, set)]
pub struct UnsubAck {
    pub len: u8,
    #[debug(format = "0x{:x}")]
    pub msg_type: u8,
    #[debug(format = "0b{:08b}")]
    pub msg_id: u16,
}

impl UnsubAck {
    pub fn recv(
        buf: &[u8],
        size: usize,
        client: &MqttSnClient,
        msg_header: MsgHeader,
    ) -> Result<(), String> {
        let (unsub_ack, read_len) = UnsubAck::try_read(buf, size).unwrap();
        dbg!(unsub_ack.clone());
        let remote_socket_addr = msg_header.remote_socket_addr;

        if read_len == MSG_LEN_UNSUBACK as usize {
            match RetransTimeWheel::cancel_timer(
                remote_socket_addr,
                unsub_ack.msg_type,
                0,
                unsub_ack.msg_id,
            ) {
                Ok(_) => Ok(()),
                Err(err) => Err(err),
            }
        } else {
            Err(eformat!(remote_socket_addr, "size", buf[0]))
        }
    }
    pub fn send(
        client: &MqttSnClient,
        msg_header: MsgHeader,
        msg_id: u16,
    ) -> Result<(), String> {
        let remote_socket_addr = msg_header.remote_socket_addr;
        let mut bytes_buf = BytesMut::with_capacity(MSG_LEN_UNSUBACK as usize);
        let unsub_ack = UnsubAck {
            len: MSG_LEN_UNSUBACK,
            msg_type: MSG_TYPE_UNSUBACK,
            msg_id,
        };
        dbg!(unsub_ack.clone());
        unsub_ack.try_write(&mut bytes_buf);
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
