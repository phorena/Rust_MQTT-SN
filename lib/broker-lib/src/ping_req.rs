/*
5.4.19 PINGREQ
Length MsgType ClientId (optional)
(octet 0) (1) (2:n)
Table 22: PINGREQ Message

As with MQTT, the PINGREQ message is an ”are you alive” message that is sent from or received by a
connected client. Its format is illustrated in Table 22:
• Length and MsgType: see Section 5.2.
• ClientId: contains the client id; this field is optional and is included by a “sleeping” client when it goes
to the “awake” state and is waiting for messages sent by the server/gateway, see Section 6.14 for further
details.

The sleep timer is stopped when the server/gateway receives a PINGREQ from the client. Like the CONNECT
message, this PINGREQ message contains the Client Id. The identified client is then in the awake state. If the
server/gateway has buffered messages for the client, it will sends these messages to the client. The transfer of
messages to the client is closed by the server/gateway by means of a PINGRESP message, i.e. the server/gateway
will consider the client as asleep and restart the sleep timer again after having sent the PINGRESP message.
If the server/gateway does not have any messages buffered for the client, it answers immediately with a
PINGRESP message, returns the client back to the asleep state, and restarts the sleep timer for that client.

*/

use bytes::{BufMut, BytesMut};
use custom_debug::Debug;
use getset::{CopyGetters, Getters, MutGetters};
use std::mem;
use std::str; // NOTE: needed for MutGetters

use crate::{
    broker_lib::MqttSnClient, eformat, function, msg_hdr::MsgHeader,
    msg_hdr::*, ping_resp::PingResp, MSG_LEN_PINGREQ_HEADER, MSG_TYPE_PINGREQ,
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
        client: &MqttSnClient,
        msg_header: MsgHeader,
    ) -> Result<(), String> {
        match msg_header.header_len {
            MsgHeaderLenEnum::Short => {
                // TODO update ping timer.
                let (_ping_req, _read_fixed_len) =
                    PingReq::try_read(buf, size).unwrap();
            }
            MsgHeaderLenEnum::Long => {
                // TODO update ping timer.
                let (_ping_req, _read_fixed_len) =
                    PingReq4::try_read(buf, size).unwrap();
            }
        }
        PingResp::send(client, msg_header)?;
        Ok(())
    }
    #[inline(always)]
    pub fn send(
        client_id: String,
        client: &mut MqttSnClient,
        msg_header: MsgHeader,
    ) -> Result<(), String> {
        let remote_socket_addr = msg_header.remote_socket_addr;
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
                .egress_tx
                .try_send((remote_socket_addr, bytes.to_owned()))
            {
                Ok(_) => Ok(()),
                Err(err) => Err(eformat!(remote_socket_addr, err)),
            }
        } else {
            Err(eformat!(remote_socket_addr, "len too long", len))
        }
    }
}
