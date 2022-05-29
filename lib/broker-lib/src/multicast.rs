/// *NOTE: socket_addr is different from socket2::SocketAddr.
/// * use socket2::SockAddr::from(socket_addr) to convert.
extern crate socket2;

use crate::{function, search_gw::SearchGw};

use bytes::Bytes;
use log::*;
use std::io;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, UdpSocket};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Duration;

use socket2::{Domain, Protocol, SockAddr, Socket, Type};

pub const PORT: u16 = 7645;
pub const SOCKET_READ_TIMEOUT_MS: u64 = 100;
lazy_static! {
    pub static ref IPV4: IpAddr = Ipv4Addr::new(224, 0, 0, 123).into();
    pub static ref IPV6: IpAddr =
        Ipv6Addr::new(0xFF02, 0, 0, 0, 0, 0, 0, 0x0123).into();
}
fn multicast_socket(multicast_addr: &SocketAddr) -> io::Result<UdpSocket> {
    dbg!(multicast_addr);
    let domain = if multicast_addr.is_ipv4() {
        Domain::ipv4()
    } else {
        return Err(io::Error::new(io::ErrorKind::Other, "V6 not supported"));
    };
    if !multicast_addr.ip().is_multicast() {
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

pub fn broadcast_loop(
    bytes: Bytes,
    multicast_addr: SocketAddr,
    duration_sec: u16,
) {
    dbg!(multicast_addr);
    let socket =
        multicast_socket(&multicast_addr).expect("failed to create sender");
    let duration_ms = duration_sec as u64 * 1000;
    let _join_handle = std::thread::Builder::new()
        .name(function!().to_string())
        .spawn(move || {
            loop {
                // TODO relplace expect()
                socket
                    .send_to(&bytes[..], &multicast_addr)
                    .expect("failed to send");
                std::thread::sleep(Duration::from_millis(duration_ms));
            }
        })
        .unwrap();
}

// this will be common for all our sockets
pub fn new_udp_socket(addr: &SocketAddr) -> io::Result<Socket> {
    let domain = if addr.is_ipv4() {
        Domain::ipv4()
    } else {
        // Domain::ipv6()
        return Err(io::Error::new(io::ErrorKind::Other, "V6 not supported"));
    };

    let socket = Socket::new(domain, Type::dgram(), Some(Protocol::udp()))?;

    // read timeouts, don't hang waiting for packets
    socket.set_read_timeout(Some(Duration::from_millis(
        SOCKET_READ_TIMEOUT_MS,
    )))?;

    Ok(socket)
}

pub fn listen_loop(multicast_addr: SocketAddr) -> JoinHandle<()> {
    let join_handle = std::thread::Builder::new()
        .name(function!().to_string())
        .spawn(move || {
            // socket creation will go here...
            let listener = multicast_bind(multicast_addr).unwrap();
            println!("server: joined: {}", multicast_addr);

            // use while loop to check for condition
            loop {
                // test receive and response code will go here...
                let mut buf = [0u8; 1400]; // receive buffer

                // we're assuming failures were timeouts, the client_done loop will stop us
                match listener.recv_from(&mut buf) {
                    Ok((len, remote_addr)) => {
                        let data = &buf[..len];
                        if let Err(why) =
                            SearchGw::recv(data, len, &remote_addr)
                        {
                            error!("{:?}", why);
                        }
                    }
                    Err(err) => {
                        // recv timeout, keep looping
                        if &err.kind() == &io::ErrorKind::WouldBlock {
                            std::thread::sleep(Duration::from_millis(10));
                        } else {
                            // other errors
                            error!("failed to receive: {}", err);
                        }
                    }
                }
            }
        })
        .unwrap();
    join_handle
}
fn multicast_bind(multicast_addr: SocketAddr) -> io::Result<UdpSocket> {
    let ip_addr = multicast_addr.ip();
    if !ip_addr.is_multicast() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "Not a multicast IP address",
        ));
    }
    let domain = if multicast_addr.is_ipv4() {
        Domain::ipv4()
    } else {
        // Domain::ipv6()
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "IPV6 is not supported",
        ));
    };

    let socket = Socket::new(domain, Type::dgram(), Some(Protocol::udp()))?;

    // read timeouts so that we don't hang waiting for packets
    socket.set_read_timeout(Some(Duration::from_millis(100)))?;

    match ip_addr {
        IpAddr::V4(ref addr_v4) => {
            dbg!(addr_v4);
            socket.join_multicast_v4(addr_v4, &Ipv4Addr::new(0, 0, 0, 0))?;
        }
        IpAddr::V6(ref _addr_v6) => {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "IPV6 is not supported",
            ));
        }
    };
    socket.bind(&socket2::SockAddr::from(multicast_addr))?;
    // convert to standard UDP sockets
    Ok(socket.into_udp_socket())
}

/// This will guarantee we always tell the server to stop
struct NotifyServer(Arc<AtomicBool>);
impl Drop for NotifyServer {
    fn drop(&mut self) {
        self.0.store(true, Ordering::Relaxed);
    }
}

/*
/// Our generic test over different IPs
fn test_multicast(test: &'static str, addr: IpAddr) {
    assert!(addr.is_multicast());
    dbg!(addr);
    let addr = SocketAddr::new(addr, PORT);
    dbg!(addr);
    dbg!(addr.ip());

    let client_done = Arc::new(AtomicBool::new(false));
    let notify = NotifyServer(Arc::clone(&client_done));

    // multicast_listener(test, client_done, addr);

    // client test code send and receive code after here
    println!("{}:client: running", test);

    let message = b"Hello from client!";

    let new_addr = addr.clone();
    // multicast_loop(new_addr);
    // create the sending socket
    let socket = multicast_socket(&addr).expect("could not create sender!");
    socket.send_to(message, &addr).expect("could not send_to!");

    std::thread::sleep(Duration::from_millis(1000));
    let mut buf = [0u8; 64]; // receive buffer

    match socket.recv_from(&mut buf) {
        Ok((len, remote_addr)) => {
            let data = &buf[..len];
            let response = String::from_utf8_lossy(data);

            println!("{}:client: got data: {}", test, response);

            // verify it's what we expected
            assert_eq!(test, response);
        }
        Err(err) => {
            println!("{}:client: had a problem: {}", test, err);
            assert!(false);
        }
    }

    std::thread::sleep(Duration::from_millis(1000));
    // multicast_loop(new_addr);
    // make sure we don't notify the server until the end of the client test
    drop(notify);
}

#[test]
fn test_ipv4_multicast() {
    test_multicast("ipv4 hello", *IPV4);
}
*/
/*
#[test]
fn test_ipv6_multicast() {
    test_multicast("ipv6", *IPV6);
}
*/
