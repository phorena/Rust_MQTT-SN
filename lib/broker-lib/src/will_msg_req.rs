/*
5.4.8 WILLMSGREQ
The WILLMSGREQ message is sent by the GW to request a client for sending the Will message. Its format is
shown in Table 11: it has only a header and no variable part.
*/
use bytes::{BufMut, BytesMut};
use custom_debug::Debug;
use getset::{CopyGetters, Getters, MutGetters};

use crate::{
    broker_lib::MqttSnClient, eformat, function, msg_hdr::MsgHeader,
    MSG_LEN_WILL_MSG_REQ, MSG_TYPE_WILL_MSG_REQ,
};

#[derive(Debug, Clone, Copy, Getters, MutGetters, CopyGetters, Default)]
#[getset(get, set)]
pub struct WillMsgReq {
    pub len: u8,
    #[debug(format = "0x{:x}")]
    pub msg_type: u8,
}

impl WillMsgReq {
    /*
    fn constraint_len(_val: &u8) -> bool {
        //dbg!(_val);
        true
    }
    fn constraint_msg_type(_val: &u8) -> bool {
        //dbg!(_val);
        true
    }
    */
    pub fn recv(
        buf: &[u8],
        size: usize,
        client: &MqttSnClient,
        msg_header: MsgHeader,
    ) -> Result<(), String> {
        if size == MSG_LEN_WILL_MSG_REQ as usize
            && buf[0] == MSG_LEN_WILL_MSG_REQ
        {
            Ok(())
        } else {
            let remote_socket_addr = msg_header.remote_socket_addr;
            Err(eformat!(remote_socket_addr, "len err", size))
        }
    }

    pub fn send(
        client: &MqttSnClient,
        msg_header: MsgHeader,
    ) -> Result<(), String> {
        let will = WillMsgReq {
            len: MSG_LEN_WILL_MSG_REQ as u8,
            msg_type: MSG_TYPE_WILL_MSG_REQ,
        };
        let remote_socket_addr = msg_header.remote_socket_addr;
        let mut bytes = BytesMut::with_capacity(MSG_LEN_WILL_MSG_REQ as usize);
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
