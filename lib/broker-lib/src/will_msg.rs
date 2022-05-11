/*
5.4.9 WILLMSG
Length MsgType WillMsg
(octet 0) (1) (2:n)
Table 13: WILLMSG Message
The WILLMSG message is sent by a client as response to a WILLMSGREQ for transferring its Will message
to the GW. Its format is shown in Table 13:
• Length and MsgType: see Section 5.2.
• WillMsg: contains the Will message.
*/
use crate::{
    broker_lib::MqttSnClient, connection::Connection, eformat, function,
    MSG_LEN_WILL_MSG_HEADER, MSG_TYPE_WILL_MSG,
};
use bytes::{BufMut, BytesMut};
use custom_debug::Debug;
use getset::{CopyGetters, Getters, MutGetters};
use std::mem;
use std::str;

#[derive(Debug, Clone, Getters, MutGetters, CopyGetters, Default)]
#[getset(get, set)]
pub struct WillMsg {
    len: u8,
    #[debug(format = "0x{:x}")]
    msg_type: u8,
    will_msg: String,
}

#[derive(Debug, Clone, Getters, MutGetters, CopyGetters, Default)]
#[getset(get, set)]
struct WillMsg4 {
    // NOTE: no pub
    one: u8,
    len: u16,
    #[debug(format = "0x{:x}")]
    msg_type: u8,
    will_msg: String,
}

impl WillMsg {
    pub fn recv(
        buf: &[u8],
        size: usize,
        client: &MqttSnClient,
    ) -> Result<(), String> {
        if size < 256 {
            let (will, len) = WillMsg::try_read(buf, size).unwrap();
            if size == len as usize {
                Connection::update_will_msg(client.remote_addr, will.will_msg)?;
                Ok(())
            } else {
                Err(eformat!(
                    client.remote_addr,
                    "2-bytes len not supported",
                    size
                ))
            }
        } else if size < 1400 {
            let (will, len) = WillMsg4::try_read(buf, size).unwrap();
            if size == len as usize && will.one == 1 {
                Connection::update_will_msg(client.remote_addr, will.will_msg)?;
                Ok(())
            } else {
                Err(eformat!(
                    client.remote_addr,
                    "4-bytes len not supported",
                    size
                ))
            }
        } else {
            Err(eformat!(client.remote_addr, "msg too long", size))
        }
    }
    pub fn send(will_msg: String, client: &MqttSnClient) -> Result<(), String> {
        let len: usize =
            MSG_LEN_WILL_MSG_HEADER as usize + will_msg.len() as usize;
        let mut bytes = BytesMut::with_capacity(len);
        if len < 256 {
            let will = WillMsg {
                len: len as u8,
                msg_type: MSG_TYPE_WILL_MSG,
                will_msg,
            };
            will.try_write(&mut bytes);
        } else if len < 1400 {
            let will = WillMsg4 {
                one: 1,
                len: len as u16,
                msg_type: MSG_TYPE_WILL_MSG,
                will_msg,
            };
            will.try_write(&mut bytes);
        } else {
            return Err(eformat!(client.remote_addr, "len err", len));
        }
        match client
            .transmit_tx
            .try_send((client.remote_addr, bytes.to_owned()))
        {
            Ok(()) => Ok(()),
            Err(err) => Err(eformat!(client.remote_addr, err)),
        }
    }
}
