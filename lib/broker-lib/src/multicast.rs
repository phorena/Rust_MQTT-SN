// Copyright 2018 Benjamin Fry <benjaminfry@me.com>
extern crate lazy_static;
extern crate socket2;

use crate::{broker_lib::MqttSnClient, eformat, function};

use std::io;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, UdpSocket};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Barrier};
use std::thread::{self, JoinHandle};
use std::time::Duration;
use bytes::{Bytes};

use socket2::{Domain, Protocol, SockAddr, Socket, Type};

pub const PORT: u16 = 7645;
pub const SOCKET_READ_TIMEOUT_MS: u64 = 100;
lazy_static! {
    pub static ref IPV4: IpAddr = Ipv4Addr::new(224, 0, 0, 123).into();
    pub static ref IPV6: IpAddr =
        Ipv6Addr::new(0xFF02, 0, 0, 0, 0, 0, 0, 0x0123).into();
}
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

pub fn advertise_broadcast_loop(
    bytes: Bytes,
    addr: SocketAddr,
    duration: u64,
) {
    dbg!(addr);
    let socket = multicast_socket(&addr).expect("failed to create sender");
    let duration = duration * 1000; // convert sec to ms
    let _join_handle = std::thread::Builder::new()
        .name(function!().to_string())
        .spawn(move || {
            loop {
                // TODO relplace expect()
                socket.send_to(&bytes[..], &addr).expect("failed to send");
                std::thread::sleep(Duration::from_millis(duration));
            }
        })
        .unwrap();
}

// this will be common for all our sockets
fn new_socket(addr: &SocketAddr) -> io::Result<Socket> {
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

fn multicast_listener(
    response: &'static str,
    client_done: Arc<AtomicBool>,
    addr: SocketAddr,
) -> JoinHandle<()> {
    // A barrier to not start the client test code until after the server is running
    let server_barrier = Arc::new(Barrier::new(2));
    let client_barrier = Arc::clone(&server_barrier);

    let join_handle = std::thread::Builder::new()
        .name(function!().to_string())
        .spawn(move || {
            // socket creation will go here...
            let listener = multicast_bind(addr).unwrap();
            println!("{}:server: joined: {}", response, addr);

            server_barrier.wait();
            println!("{}:server: is ready", response);

            let mut counter = 0;
            // We'll be looping until the client indicates it is done.
            while !client_done.load(std::sync::atomic::Ordering::Relaxed) {
                counter += 1;
                dbg!(counter);
                // test receive and response code will go here...
                let mut buf = [0u8; 64]; // receive buffer

                // we're assuming failures were timeouts, the client_done loop will stop us
                match listener.recv_from(&mut buf) {
                    Ok((len, remote_addr)) => {
                        let data = &buf[..len];

                        println!(
                            "{}:server: got data: {} from: {}",
                            response,
                            String::from_utf8_lossy(data),
                            remote_addr
                        );

                        // create a socket to send the response
                        let responder = new_socket(&remote_addr)
                            .expect("failed to create responder")
                            .into_udp_socket();

                        // we send the response that was set at the method beginning
                        responder
                            .send_to(response.as_bytes(), &remote_addr)
                            .expect("failed to respond");

                        println!(
                            "{}:server: sent response to: {}",
                            response, remote_addr
                        );
                    }
                    Err(err) => {
                        // recv timeout, keep looping
                        if &err.kind() == &io::ErrorKind::WouldBlock {
                            std::thread::sleep(Duration::from_millis(10));
                        } else {
                            // other errors
                            // TODO change to error!()
                            println!(
                                "{}:server: failed to receive: {}",
                                response, err
                            );
                        }
                    }
                }
            }
            println!("{}:server: client is done", response);
        })
        .unwrap();

    client_barrier.wait();
    join_handle
}
fn multicast_bind(addr: SocketAddr) -> io::Result<UdpSocket> {
    let ip_addr = addr.ip();
    if !ip_addr.is_multicast() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "Not a multicast IP address",
        ));
    }
    let domain = if addr.is_ipv4() {
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
        IpAddr::V6(ref addr_v6) => {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "IPV6 is not supported",
            ));
        }
    };
    socket.bind(&socket2::SockAddr::from(addr))?;
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

/// Our generic test over different IPs
fn test_multicast(test: &'static str, addr: IpAddr) {
    assert!(addr.is_multicast());
    dbg!(addr);
    let addr = SocketAddr::new(addr, PORT);
    dbg!(addr);
    dbg!(addr.ip());

    let client_done = Arc::new(AtomicBool::new(false));
    let notify = NotifyServer(Arc::clone(&client_done));

    multicast_listener(test, client_done, addr);

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

/*
#[test]
fn test_ipv6_multicast() {
    test_multicast("ipv6", *IPV6);
}
*/
