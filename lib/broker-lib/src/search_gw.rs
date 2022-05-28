/*
5.4.1 ADVERTISE
Length    MsgType GwId Duration
(octet 0) (1)     (2)  (3,4)
Table 6: ADVERTISE Message
The ADVERTISE message is broadcasted periodically by a gateway to advertise its presence. The time
interval until the next broadcast time is indicated in the Duration field of this message. Its format is illustrated in
Table 6:
• Length and MsgType: see Section 5.2.
• GwId: the id of the gateway which sends this message.
• Duration: time interval until the next ADVERTISE is broadcasted by this gateway
*/
use crate::{
    eformat, function, gw_info::GwInfo, MSG_LEN_SEARCH_GW, MSG_TYPE_SEARCH_GW,
};
use bytes::{BufMut, BytesMut};
use custom_debug::Debug;
use getset::{CopyGetters, Getters, MutGetters};
use log::*;
use socket2::{Domain, Protocol, SockAddr, Socket, Type};
use std::net::{SocketAddr, UdpSocket};
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
    // for client to send request to broker.
    pub fn send(
        radius: u8,
        socket_addr: &SocketAddr,
        udp_socket: UdpSocket,
    ) -> Result<(), String> {
        let mut bytes = BytesMut::with_capacity(MSG_LEN_SEARCH_GW as usize);
        let buf: &[u8] = &[MSG_LEN_SEARCH_GW, MSG_TYPE_SEARCH_GW, radius];
        bytes.put(buf);
        dbg!(&buf);
        match udp_socket.send_to(&bytes[..], socket_addr) {
            Ok(size) if size == MSG_LEN_SEARCH_GW as usize => Ok(()),
            Ok(size) => Err(format!(
                "send_to: {} bytes sent, but {} bytes expected",
                size, MSG_LEN_SEARCH_GW
            )),
            Err(err) => return Err(eformat!(socket_addr, err)),
        }
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
