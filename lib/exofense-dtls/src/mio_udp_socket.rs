use mio::net::UdpSocket;
//use std::net::ToSocketAddrs;
use mio::{Events, Poll, Token};
use std::io;
use std::net::SocketAddr;

pub struct MioUdpSocket {
    socket: UdpSocket,
    poll: Poll,
    events: Events,
}

const UDP_SOCKET: Token = Token(0);

impl MioUdpSocket {
    // pub async fn new(&self) -> io::Result<usize>
    // {
    //     self.poll = Poll::new().unwrap();
    //     self.events = Events::with_capacity(1);
    // }

    pub async fn bind(addr: SocketAddr) -> Self {
        Self {
            socket: UdpSocket::bind(addr).unwrap(),
            poll: Poll::new().unwrap(),
            events: Events::with_capacity(1),
        }
    }
    // pub async fn bind<A: ToSocketAddrs>(&self,addr: A) -> io::Result<UdpSocket>{
    //     let socket=self.socket.bind(addr);
    //     Ok(socket)
    // }
    pub async fn connect(&self, addr: SocketAddr) -> io::Result<()> {
        self.socket.connect(addr)
    }

    pub async fn recv(&self, buf: &mut [u8]) -> io::Result<usize> {
        self.socket.recv(buf)
    }

    pub async fn recv_from(&mut self, buf: &mut [u8]) -> io::Result<(usize, SocketAddr)> {
        //self.socket.recv_from(buf)

        // let mut poll = Poll::new()?;
        // let mut events = Events::with_capacity(1);
        loop {
            // Poll to check if we have events waiting for us.
            self.poll.poll(&mut self.events, None)?;

            // Process each event.
            for event in self.events.iter() {
                // Validate the token we registered our socket with,
                // in this example it will only ever be one but we
                // make sure it's valid none the less.
                match event.token() {
                    UDP_SOCKET => loop {
                        // In this loop we receive all packets queued for the socket.
                        return self.socket.recv_from(buf);
                    },
                    _ => {
                        log::warn!("Got event for unexpected token: {:?}", event);
                    }
                }
            }
        }
    }

    pub async fn send(&self, buf: &[u8]) -> io::Result<usize> {
        self.socket.send(buf)
    }

    pub async fn send_to(&self, buf: &[u8], target: SocketAddr) -> io::Result<usize> {
        self.socket.send_to(buf, target)
    }

    pub async fn local_addr(&self) -> io::Result<SocketAddr> {
        self.socket.local_addr()
    }

    pub async fn remote_addr(&self) -> Option<SocketAddr> {
        log::warn!("remote_addr method for MioUdpSocket is not defined");
        None
    }

    pub async fn close(&self) -> io::Result<()> {
        Ok(())
    }
}
