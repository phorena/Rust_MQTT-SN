/*
5.4.2 SEARCHGW
Length MsgType Radius
(octet 0) (1) (2)
Table 7: SEARCHGW Message
The SEARCHGW message is broadcasted by a client when it searches for a GW. The broadcast radius of the
SEARCHGW is limited and depends on the density of the clients deployment, e.g. only 1-hop broadcast in case
of a very dense network in which every MQTT-SN client is reachable from each other within 1-hop transmission.
The format of a SEARCHGW message is illustrated in Table 7:
• Length and MsgType: see Section 5.2.
• Radius: the broadcast radius of this message.
The broadcast radius is also indicated to the underlying network layer when MQTT-SN gives this message for
transmission.
*/
use crate::{
    eformat, function, gw_info::GwInfo, multicast, MSG_LEN_SEARCH_GW,
    MSG_TYPE_SEARCH_GW,
};
use bytes::{BufMut, BytesMut};
use custom_debug::Debug;
use getset::{CopyGetters, Getters, MutGetters};
use log::*;
use std::net::SocketAddr;
use std::str;

pub const SEARCH_RADIUS_MAX: u8 = 2;

#[derive(
    Debug, Clone, Getters, /*Setters,*/ MutGetters, CopyGetters, Default,
)]
#[getset(get, set)]
pub struct SearchGw {
    pub len: u8,
    #[debug(format = "0x{:x}")]
    pub msg_type: u8,
    pub radius: u8,
}
impl SearchGw {
    // for client to multicast
    pub fn run(socket_addr: SocketAddr, radius: u8, duration: u16) {
        let mut bytes = BytesMut::with_capacity(MSG_LEN_SEARCH_GW as usize);
        let buf: &[u8] = &[MSG_LEN_SEARCH_GW, MSG_TYPE_SEARCH_GW, radius];
        bytes.put(buf);
        dbg!(&buf);
        multicast::broadcast_loop(bytes.freeze(), socket_addr, duration);
    }
    pub fn recv(
        buf: &[u8],
        size: usize,
        socket_addr: &SocketAddr,
    ) -> Result<(), String> {
        match SearchGw::try_read(buf, size) {
            Some((search_gw, size)) if size == MSG_LEN_SEARCH_GW as usize => {
                info!(
                    "{}: search gw {} with {} radius",
                    socket_addr, search_gw.radius, search_gw.radius
                );
                if search_gw.radius > SEARCH_RADIUS_MAX {
                    error!(
                        "{}: search gw radius {} is too large (max {})",
                        socket_addr, search_gw.radius, SEARCH_RADIUS_MAX
                    );
                }
                // TODO use configure gateway ip address/port.
                if let Err(why) =
                    GwInfo::send(1, "124.0.0.5:61000".to_string(), socket_addr)
                {
                    error!("{}", why);
                }
                Ok(())
            }
            Some((_, size)) => Err(format!(
                "{}: search gw: {} bytes received, but {} bytes expected",
                socket_addr, size, MSG_LEN_SEARCH_GW
            )),
            None => Err(eformat!(socket_addr)),
        }
    }
}
