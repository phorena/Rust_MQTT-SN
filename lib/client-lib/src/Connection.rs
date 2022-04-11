use crate::Filter::Filter;
use std::collections::HashMap;
use std::net::SocketAddr;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct ConnId {
    id: String,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]

pub struct Connection {
    clean: bool,
    // TODO Struct Will
    will: u8,
    state: u8,
    id: ConnId,
    socket_addr: SocketAddr,
    duration: u16,
}

impl Connection {
    pub fn new(socket_addr: SocketAddr, id: ConnId, duration: u16) -> Self {
        Connection {
            clean: true,
            will: 0,
            state: 0,
            id,
            socket_addr,
            duration,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConnHashSocketAddr {
    connections: HashMap<SocketAddr, Connection>,
}

impl ConnHashSocketAddr {
    pub fn new() -> Self {
        ConnHashSocketAddr {
            connections: HashMap::new(),
        }
    }
    pub fn insert(&mut self, socket_addr: SocketAddr, connection: Connection) {
        self.connections.insert(socket_addr, connection);
    }
    pub fn remove(&mut self, socket_addr: &SocketAddr) {
        self.connections.remove(&socket_addr);
    }
}

/// The connection can migrate to another source IP address.
/// Use this hashmap to locate the existing connection.
#[derive(Debug, Clone)]
pub struct ConnHashId {
    connections: HashMap<ConnId, SocketAddr>,
}
