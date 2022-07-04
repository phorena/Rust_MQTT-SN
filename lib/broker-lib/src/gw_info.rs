/*
5.4.3 GWINFO
Length MsgType GwId GwAdd*
(octet 0) (1) (2) (3:n)
(*) only present if message is sent by a client
Table 8: GWINFO Message
The GWINFO message is sent as response to a SEARCHGW message using the broadcast service of the
underlying layer, with the radius as indicated in the SEARCHGW message. If sent by a GW, it contains only the
id of the sending GW; otherwise, if sent by a client, it also includes the address of the GW, see Table 8:
• Length and MsgType: see Section 5.2.
• GwId: the id of a GW.
• GwAdd: address of the indicated GW; optional, only included if message is sent by a client.
Like the SEARCHGW message the broadcast radius for this message is also indicated to the underlying
network layer when MQTT-SN gives this message for transmission.
*/
use crate::{
    broker_lib::MqttSnClient, eformat, function, msg_hdr::MsgHeader, multicast,
    multicast::new_udp_socket, MSG_LEN_GW_INFO_HEADER, MSG_TYPE_GW_INFO,
};
use bytes::{BufMut, BytesMut};
use custom_debug::Debug;
use getset::{CopyGetters, Getters, MutGetters};
use log::*;
use std::net::SocketAddr;
use std::str; // NOTE: needed for MutGetters

#[derive(
    // NOTE: must include std::str for MutGetters
    Debug,
    Clone,
    Getters,
    CopyGetters,
    MutGetters,
    Default,
    PartialEq,
    Hash,
    Eq,
)]
#[getset(get, set)]
pub struct GwInfo {
    pub len: u8,
    #[debug(format = "0x{:x}")]
    pub msg_type: u8,
    pub gw_id: u8,
    pub gw_addr: String,
}
impl GwInfo {
    pub fn run(socket_addr: SocketAddr) {
        multicast::gw_info_listen_loop(socket_addr);
    }
    pub fn send(
        gw_id: u8,
        gw_addr: String,
        socket_addr: &SocketAddr,
    ) -> Result<(), String> {
        let len = MSG_LEN_GW_INFO_HEADER as usize + gw_addr.len() as usize;
        if len > 255 {
            return Err(format!("gw_addr too long: {}", len));
        }
        // *NOTE*: this return value can be cached.
        let mut bytes = BytesMut::with_capacity(len);
        let buf: &[u8] = &[len as u8, MSG_TYPE_GW_INFO, gw_id];
        bytes.put(buf);
        bytes.put(gw_addr.as_bytes());
        dbg!(&bytes);
        match new_udp_socket(socket_addr) {
            Ok(udp_socket) => {
                match udp_socket
                    .send_to(&bytes[..], &socket2::SockAddr::from(*socket_addr))
                {
                    Ok(size) if size == len => Ok(()),
                    Ok(size) => Err(format!(
                        "send_to: {} bytes sent, but {} bytes expected",
                        size, len
                    )),
                    Err(err) => return Err(eformat!(socket_addr, err)),
                }
            }
            Err(err) => Err(eformat!(socket_addr, err)),
        }
    }
    pub fn recv(
        buf: &[u8],
        size: usize,
        client: &MqttSnClient,
        msg_header: MsgHeader,
    ) -> Result<(), String> {
        let (gw_info, _read_fixed_len) = GwInfo::try_read(buf, size).unwrap();
        info!(
            "{}: {} with {}",
            msg_header.remote_socket_addr, gw_info.gw_id, gw_info.gw_addr
        );
        Ok(())
    }
}
