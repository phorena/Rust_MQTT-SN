/*
5.4.19 PINGREQ
Length MsgType ClientId (optional)
(octet 0) (1) (2:n)
Table 22: PINGREQ Message
As with MQTT, the PINGREQ message is an ”are you alive” message that is sent from or received by a
connected client. Its format is illustrated in Table 22:
c Copyright IBM Corporation 1999, 2013. All rights reserved. 16
• Length and MsgType: see Section 5.2.
• ClientId: contains the client id; this field is optional and is included by a “sleeping” client when it goes
to the “awake” state and is waiting for messages sent by the server/gateway, see Section 6.14 for further
details.
*/

use bytes::{BufMut, BytesMut};
use custom_debug::Debug;
use getset::{CopyGetters, Getters, MutGetters};
use std::mem;
use std::str; // NOTE: needed for MutGetters

use crate::{
    eformat, function,
    message::{MsgHeader, MsgHeaderEnum},
    ping_resp::PingResp,
    BrokerLib::MqttSnClient,
    MSG_LEN_PINGREQ_HEADER, MSG_TYPE_PINGREQ,
};

#[derive(Debug, Clone, Getters, MutGetters, CopyGetters, Default)]
#[getset(get, set)]
pub struct PingReq {
    len: u8,
    #[debug(format = "0x{:x}")]
    msg_type: u8,
    client_id: String,
}

#[derive(Debug, Clone, Getters, MutGetters, CopyGetters, Default)]
#[getset(get, set)]
struct PingReq4 {
    // NOTE: no pub
    one: u8,
    len: u16,
    #[debug(format = "0x{:x}")]
    msg_type: u8,
    client_id: String,
}

impl PingReq {
    #[inline(always)]
    pub fn recv(
        buf: &[u8],
        size: usize,
        client: &mut MqttSnClient,
        header: MsgHeader,
    ) -> Result<(), String> {
        match header.header_len {
            MsgHeaderEnum::Short => {
                // TODO update ping timer.
                let (_ping_req, _read_fixed_len) =
                    PingReq::try_read(buf, size).unwrap();
            }
            MsgHeaderEnum::Long => {
                // TODO update ping timer.
                let (_ping_req, _read_fixed_len) =
                    PingReq4::try_read(buf, size).unwrap();
            }
        }
        PingResp::send(client)?;
        Ok(())
    }
    pub fn send(
        client_id: String,
        client: &mut MqttSnClient,
    ) -> Result<(), String> {
        let len = client_id.len() + MSG_LEN_PINGREQ_HEADER as usize;
        let mut bytes = BytesMut::with_capacity(len);
        if len < 256 {
            let ping_req = PingReq {
                len: len as u8,
                msg_type: MSG_TYPE_PINGREQ,
                client_id,
            };
            ping_req.try_write(&mut bytes);
            match client
                .transmit_tx
                .try_send((client.remote_addr, bytes.to_owned()))
            {
                Ok(_) => Ok(()),
                Err(err) => Err(eformat!(client.remote_addr, err)),
            }
        } else {
            Err(eformat!(client.remote_addr, "len too long", len))
        }
    }
}
