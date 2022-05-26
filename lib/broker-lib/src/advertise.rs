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
extern crate socket2;

use std::io;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, UdpSocket};
use std::time::Duration;

use socket2::{Domain, Protocol, SockAddr, Socket, Type};

pub const PORT: u16 = 7645;
lazy_static! {
    pub static ref IPV4: IpAddr = Ipv4Addr::new(224, 0, 0, 123).into();
    pub static ref IPV6: IpAddr = Ipv6Addr::new(0xFF02, 0, 0, 0, 0, 0, 0, 0x0123).into();
}
use crate::{
    broker_lib::MqttSnClient, eformat, function, MSG_LEN_ADVERTISE,
    MSG_TYPE_ADVERTISE,
};
use bytes::{BufMut, BytesMut};
use custom_debug::Debug;
use getset::{CopyGetters, Getters, MutGetters};
use log::*;
use std::mem;

fn multicast_socket(addr: &SocketAddr) -> io::Result<UdpSocket> {
    dbg!(addr);
    let domain = if addr.is_ipv4() {
        Domain::ipv4()
    } else {
        return Err(io::Error::new(io::ErrorKind::Other, "V6 not supported"));
    };
    if !addr.ip().is_multicast() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "Not a multicast address",
        ));
    }
    let socket = Socket::new(domain, Type::dgram(), Some(Protocol::udp()))?;
    // read timeouts so that we don't hang waiting for packets
    socket.set_read_timeout(Some(Duration::from_millis(100)))?;
    socket.set_multicast_if_v4(&Ipv4Addr::new(0, 0, 0, 0))?;
    socket.bind(&SockAddr::from(SocketAddr::new(
        Ipv4Addr::new(0, 0, 0, 0).into(),
        0,
    )))?;
    // convert to standard UDP sockets
    Ok(socket.into_udp_socket())
}

fn multicast_loop(addr: SocketAddr) -> io::Result<()> {
    dbg!(addr);
    let socket = multicast_socket(&addr).expect("failed to create sender");
    let message = b"Hello from client!!!!";
    let _join_handle = std::thread::Builder::new()
        .name(format!("server"))
        .spawn(move || {
            // while true {
            socket.send_to(message, &addr).expect("failed to send");
            // std::thread::sleep(Duration::from_millis(1000));
            // }
        })
        .unwrap();

    Ok(())
}

#[derive(
    Debug, Clone, Getters, /*Setters,*/ MutGetters, CopyGetters, Default,
)]
#[getset(get, set)]
pub struct Advertise {
    pub len: u8,
    #[debug(format = "0x{:x}")]
    pub msg_type: u8,
    pub gw_id: u8,
    pub duration: u16,
}
impl Advertise {
    pub fn send(
        gw_id: u8,
        duration: u16,
        client: &MqttSnClient,
    ) -> Result<(), String> {
        let duration_0 = (duration >> 8) as u8;
        let duration_1 = duration as u8;
        let mut bytes = BytesMut::with_capacity(MSG_LEN_ADVERTISE as usize);
        let buf: &[u8] = &[
            MSG_LEN_ADVERTISE,
            MSG_TYPE_ADVERTISE,
            gw_id,
            duration_0,
            duration_1,
        ];
        bytes.put(buf);
        dbg!(&buf);
        match client.transmit_tx.try_send((client.remote_addr, bytes)) {
            Ok(()) => Ok(()),
            Err(err) => return Err(eformat!(client.remote_addr, err)),
        }
    }
    pub fn recv(
        buf: &[u8],
        size: usize,
        client: &MqttSnClient,
    ) -> Result<(), String> {
        let (advertise, _read_fixed_len) =
            Advertise::try_read(buf, size).unwrap();
        info!(
            "{}: advertise {} with {} id",
            client.remote_addr, advertise.gw_id, advertise.duration
        );
        Ok(())
    }
}
