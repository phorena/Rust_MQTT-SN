/*
5.4.20 PINGRESP
Length MsgType
(octet 0) (1)
Table 23: PINGRESP Message
As with MQTT, a PINGRESP message is the response to a PINGREQ message and means ”yes I am alive”.
Keep Alive messages flow in either direction, sent either by a connected client or the gateway. Its format is
illustrated in Table 23: it has only a header and no variable part.
Moreover, a PINGRESP message is sent by a gateway to inform a sleeping client that it has no more buffered
messages for that client, see Section 6.14 for further details.
*/

use crate::{
    broker_lib::MqttSnClient, eformat, function, msg_hdr::MsgHeader,
    MSG_LEN_PINGRESP, MSG_TYPE_PINGRESP,
};
use bytes::{BufMut, BytesMut};
use custom_debug::Debug;
use getset::{CopyGetters, Getters, MutGetters};

#[derive(
    Debug,
    Clone,
    Copy,
    Getters,
    //   Setters,
    MutGetters,
    CopyGetters,
    Default,
    PartialEq,
)]
#[getset(get, set)]
pub struct PingResp {
    pub len: u8,
    #[debug(format = "0x{:x}")]
    pub msg_type: u8,
}

impl PingResp {
    pub fn recv(
        buf: &[u8],
        size: usize,
        client: &MqttSnClient,
        msg_header: MsgHeader,
    ) -> Result<(), String> {
        let remote_socket_addr = msg_header.remote_socket_addr;
        if size == MSG_LEN_PINGRESP as usize && buf[0] == MSG_LEN_PINGRESP {
            // TODO update ping timer.
            Ok(())
        } else {
            Err(eformat!(remote_socket_addr, "len err", size))
        }
    }
    pub fn send(
        client: &MqttSnClient,
        msg_header: MsgHeader,
    ) -> Result<(), String> {
        let remote_socket_addr = msg_header.remote_socket_addr;
        let buf: &[u8] = &[MSG_LEN_PINGRESP, MSG_TYPE_PINGRESP];
        let bytes = BytesMut::from(buf);
        match client.egress_tx.try_send((remote_socket_addr, bytes)) {
            Ok(()) => Ok(()),
            Err(err) => Err(eformat!(remote_socket_addr, err)),
        }
    }
}
