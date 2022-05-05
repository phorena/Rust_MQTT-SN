/*
5.4.23 WILLMSGUPD
Length    MsgType WillMsg
(octet 0) (1)     (2:n)
Table 26: WILLMSGUPD Message

The WILLMSGUPD message is sent by a client to update its Will message stored in the GW/server. Its format
is shown in Table 26:
*/
use bytes::{BufMut, BytesMut};
use custom_debug::Debug;
use getset::{CopyGetters, Getters, MutGetters};
use std::mem;
use std::str;

use crate::{
    connection::Connection, eformat, function, will_msg_resp::WillMsgResp,
    BrokerLib::MqttSnClient, MSG_LEN_WILL_MSG_HEADER, MSG_TYPE_WILL_MSG,
    RETURN_CODE_ACCEPTED,
};

#[derive(Debug, Clone, Getters, MutGetters, CopyGetters, Default)]
#[getset(get, set)]
pub struct WillMsgUpd {
    len: u8,
    #[debug(format = "0x{:x}")]
    msg_type: u8,
    will_msg: String,
}
#[derive(Debug, Clone, Getters, MutGetters, CopyGetters, Default)]
#[getset(get, set)]
struct WillMsgUpd4 {
    // NOTE: no pub
    one: u8,
    len: u16,
    #[debug(format = "0x{:x}")]
    msg_type: u8,
    will_msg: String,
}

impl WillMsgUpd {
    pub fn recv(
        buf: &[u8],
        size: usize,
        client: &MqttSnClient,
    ) -> Result<(), String> {
        if size < 256 {
            let (will, len) = WillMsgUpd::try_read(buf, size).unwrap();
            if size == len as usize {
                Connection::update_will_msg(client.remote_addr, will.will_msg)?;
                WillMsgResp::send(RETURN_CODE_ACCEPTED, client)?;
                Ok(())
            } else {
                Err(eformat!(
                    client.remote_addr,
                    "2-bytes len not supported", // TODO wrong message.
                    size
                ))
            }
        } else if size < 1400 {
            let (will, len) = WillMsgUpd4::try_read(buf, size).unwrap();
            if size == len as usize && will.one == 1 {
                Connection::update_will_msg(client.remote_addr, will.will_msg)?;
                WillMsgResp::send(RETURN_CODE_ACCEPTED, client)?;
                Ok(())
            } else {
                Err(eformat!(
                    client.remote_addr,
                    "4-bytes len not supported", // TODO wrong message.
                    size
                ))
            }
        } else {
            Err(eformat!(client.remote_addr, "len not supported", size))
        }
    }
    pub fn send(will_msg: String, client: &MqttSnClient) -> Result<(), String> {
        let len: usize =
            MSG_LEN_WILL_MSG_HEADER as usize + will_msg.len() as usize;
        if len < 256 {
            let will = WillMsgUpd {
                len: len as u8,
                msg_type: MSG_TYPE_WILL_MSG,
                will_msg,
            };
            let mut bytes = BytesMut::with_capacity(len);
            will.try_write(&mut bytes);
            match client
                .transmit_tx
                .try_send((client.remote_addr, bytes.to_owned()))
            {
                Ok(()) => Ok(()),
                Err(err) => Err(eformat!(client.remote_addr, err)),
            }
        } else if len < 1400 {
            let will = WillMsgUpd4 {
                one: 1,
                len: len as u16,
                msg_type: MSG_TYPE_WILL_MSG,
                will_msg,
            };
            let mut bytes = BytesMut::with_capacity(len);
            will.try_write(&mut bytes);
            match client
                .transmit_tx
                .try_send((client.remote_addr, bytes.to_owned()))
            {
                Ok(()) => Ok(()),
                Err(err) => Err(eformat!(client.remote_addr, err)),
            }
        } else {
            Err(eformat!(client.remote_addr, "len err", len))
        }
    }
}
