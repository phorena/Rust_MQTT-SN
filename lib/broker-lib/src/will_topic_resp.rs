/*
5.4.24 WILLTOPICRESP
Length MsgType ReturnCode
(octet 0) (1) (2)
Table 27: WILLTOPICRESP and WILLMSGRESP Messages
The WILLTOPICRESP message is sent by a GW to acknowledge the receipt and processing of an WILLTOPICUPD message. Its format is illustrated in Table 27:
• Length and MsgType: see Section 5.2.
• ReturnCode: “accepted”, or rejection reason
*/
use crate::{
    broker_lib::MqttSnClient, eformat, function, msg_hdr::MsgHeader,
    ReturnCodeConst, MSG_LEN_WILL_TOPIC_RESP, MSG_TYPE_WILL_TOPIC_RESP,
};
use bytes::{BufMut, BytesMut};
use custom_debug::Debug;
use getset::{CopyGetters, Getters, MutGetters};
#[derive(Debug, Clone, Copy, Getters, MutGetters, CopyGetters, Default)]
#[getset(get, set)]
pub struct WillTopicResp {
    pub len: u8,
    #[debug(format = "0x{:x}")]
    pub msg_type: u8,
    pub return_code: u8,
}

impl WillTopicResp {
    pub fn recv(
        buf: &[u8],
        size: usize,
        client: &MqttSnClient,
        msg_header: MsgHeader,
    ) -> Result<(), String> {
        let remote_socket_addr = msg_header.remote_socket_addr;
        if size == MSG_LEN_WILL_TOPIC_RESP as usize
            && buf[0] == MSG_LEN_WILL_TOPIC_RESP
        {
            // TODO cancel timer.
            Ok(())
        } else {
            Err(eformat!(remote_socket_addr, "len err", size))
        }
    }

    pub fn send(
        return_code: ReturnCodeConst,
        client: &MqttSnClient,
        msg_header: MsgHeader,
    ) -> Result<(), String> {
        let remote_socket_addr = msg_header.remote_socket_addr;
        let will = WillTopicResp {
            len: MSG_LEN_WILL_TOPIC_RESP as u8,
            msg_type: MSG_TYPE_WILL_TOPIC_RESP,
            return_code,
        };
        let mut bytes =
            BytesMut::with_capacity(MSG_LEN_WILL_TOPIC_RESP as usize);
        dbg!(will.clone());
        will.try_write(&mut bytes);
        dbg!(bytes.clone());
        dbg!(remote_socket_addr);
        // transmit to network
        match client
            .egress_tx
            .try_send((remote_socket_addr, bytes.to_owned()))
        {
            Ok(()) => Ok(()),
            Err(err) => Err(eformat!(remote_socket_addr, err)),
        }
    }
}
