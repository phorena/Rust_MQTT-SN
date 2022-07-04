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
    broker_lib::MqttSnClient, conn_ack::ConnAck, connection::Connection,
    eformat, function, msg_hdr::MsgHeader, MSG_LEN_WILL_MSG_HEADER,
    MSG_TYPE_WILL_MSG, RETURN_CODE_ACCEPTED,
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
    msg: String,
}

#[derive(Debug, Clone, Getters, MutGetters, CopyGetters, Default)]
#[getset(get, set)]
struct WillMsg4 {
    // NOTE: no pub
    one: u8,
    len: u16,
    #[debug(format = "0x{:x}")]
    msg_type: u8,
    msg: String,
}

impl WillMsg {
    pub fn recv(
        buf: &[u8],
        size: usize,
        client: &MqttSnClient,
        msg_header: MsgHeader,
    ) -> Result<(), String> {
        let remote_socket_addr = msg_header.remote_socket_addr;
        if size < 256 {
            let (will, mut len) = WillMsg::try_read(buf, size).unwrap();
            len += will.msg.len() as usize;
            if size == len as usize {
                Connection::update_will_msg(remote_socket_addr, will.msg)?;
                ConnAck::send(client, msg_header, RETURN_CODE_ACCEPTED)?;
                Ok(())
            } else {
                Err(eformat!(
                    remote_socket_addr,
                    "2-bytes len not supported",
                    size
                ))
            }
        } else if size < 1400 {
            let (will, mut len) = WillMsg4::try_read(buf, size).unwrap();
            len += will.msg.len() as usize;
            if size == len as usize && will.one == 1 {
                Connection::update_will_msg(remote_socket_addr, will.msg)?;
                ConnAck::send(client, msg_header, RETURN_CODE_ACCEPTED)?;
                Ok(())
            } else {
                Err(eformat!(
                    remote_socket_addr,
                    "4-bytes len not supported",
                    size
                ))
            }
        } else {
            Err(eformat!(remote_socket_addr, "msg too long", size))
        }
    }
    pub fn send(
        msg: String,
        client: &MqttSnClient,
        msg_header: MsgHeader,
    ) -> Result<(), String> {
        let len: usize = MSG_LEN_WILL_MSG_HEADER as usize + msg.len() as usize;
        let mut bytes = BytesMut::with_capacity(len);
        let remote_socket_addr = msg_header.remote_socket_addr;
        if len < 256 {
            let will = WillMsg {
                len: len as u8,
                msg_type: MSG_TYPE_WILL_MSG,
                msg,
            };
            will.try_write(&mut bytes);
        } else if len < 1400 {
            let will = WillMsg4 {
                one: 1,
                len: len as u16,
                msg_type: MSG_TYPE_WILL_MSG,
                msg,
            };
            will.try_write(&mut bytes);
        } else {
            return Err(eformat!(remote_socket_addr, "len err", len));
        }
        match client
            .egress_tx
            .try_send((remote_socket_addr, bytes.to_owned()))
        {
            Ok(()) => Ok(()),
            Err(err) => Err(eformat!(remote_socket_addr, err)),
        }
    }
}
