/*
5.4.25 WILLMSGRESP
The WILLMSGRESP message is sent by a GW to acknowledge the receipt and processing of an WILLMSGUPD
message. Its format is illustrated in Table 27:
• Length and MsgType: see Section 5.2.
• ReturnCode: “accepted”, or rejection reason
*/
use bytes::{BufMut, BytesMut};
use custom_debug::Debug;
use getset::{CopyGetters, Getters, MutGetters};

use crate::{
    eformat, function, broker_lib::MqttSnClient, ReturnCodeConst,
    MSG_LEN_WILL_MSG_RESP, MSG_TYPE_WILL_MSG_RESP,
};
#[derive(Debug, Clone, Copy, Getters, MutGetters, CopyGetters, Default)]
#[getset(get, set)]
pub struct WillMsgResp {
    pub len: u8,
    #[debug(format = "0x{:x}")]
    pub msg_type: u8,
    pub return_code: u8,
}

impl WillMsgResp {
    pub fn recv(
        buf: &[u8],
        size: usize,
        client: &MqttSnClient,
    ) -> Result<(), String> {
        if size == MSG_LEN_WILL_MSG_RESP as usize
            && buf[0] == MSG_LEN_WILL_MSG_RESP
        {
            Ok(())
        } else {
            Err(eformat!(client.remote_addr, "len err", size))
        }
    }
    pub fn send(
        return_code: ReturnCodeConst,
        client: &MqttSnClient,
    ) -> Result<(), String> {
        let will = WillMsgResp {
            len: MSG_LEN_WILL_MSG_RESP as u8,
            msg_type: MSG_TYPE_WILL_MSG_RESP,
            return_code,
        };
        let mut bytes = BytesMut::with_capacity(MSG_LEN_WILL_MSG_RESP as usize);
        dbg!(will.clone());
        will.try_write(&mut bytes);
        dbg!(bytes.clone());
        dbg!(client.remote_addr);
        // transmit to network
        match client
            .transmit_tx
            .try_send((client.remote_addr, bytes.to_owned()))
        {
            Ok(()) => Ok(()),
            Err(err) => Err(eformat!(client.remote_addr, err)),
        }
    }
}
