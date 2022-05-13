use crate::{eformat, function};
// use log::*;
// use rand::Rng;
use bytes::Bytes;
use hashbrown::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::v1::{Context, Timestamp};
use uuid::Uuid;

pub type ConnId = Uuid;

/// Generate a new UUID
/// Use timestamp with nanoseconds precision
/// Use socket_addr, 6 bytes
/// Use Context for the namespace, etc.
pub fn generate_conn_id(
    socket_addr: SocketAddr,
    context_num: u16,
) -> Result<ConnId, String> {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    let time_stamp_nanos = since_the_epoch.as_nanos() as u32;
    let time_stamp_secs = since_the_epoch.as_secs();
    let context = Context::new(context_num);
    let time_stamp =
        Timestamp::from_unix(&context, time_stamp_secs, time_stamp_nanos);
    let ip4_bytes: [u8; 4];
    let port_bytes: [u8; 2] = socket_addr.port().to_be_bytes();

    match socket_addr.ip() {
        IpAddr::V4(ip4) => ip4_bytes = ip4.octets(),
        IpAddr::V6(ip6) => {
            let msg = format!(
                "ipv6: {}, segments: {:?} not supported",
                ip6,
                ip6.segments()
            );
            return Err(msg);
        }
    }
    let socket_addr_bytes: [u8; 6] = [
        ip4_bytes[0],
        ip4_bytes[1],
        ip4_bytes[2],
        ip4_bytes[3],
        port_bytes[0],
        port_bytes[1],
    ];

    // TODO error handling
    let uuid = match Uuid::new_v1(time_stamp, &socket_addr_bytes) {
        Ok(uuid) => uuid,
        Err(e) => return Err(format!("{}", e)),
    };
    // dbg!((&context, time_stamp, uuid));
    Ok(uuid)
}

lazy_static! {
    // TODO add comments!!!
    static ref CONN_HASHMAP: Mutex<HashMap<SocketAddr, Connection>> =
        Mutex::new(HashMap::new());

    // TODO: for connection migration, when the client has a new socket_addr,
    //       use the ConnId to locate the connection.
    static ref CONN_ID_HASHMAP: Mutex<HashMap<ConnId, SocketAddr>> =
        Mutex::new(HashMap::new());
}

/// A connection is CURRENT network connection a client connects to the server.
/// The filter is used to delete its global filters when the client disconnects
/// or unsubscribes.
/// In the future, the client might be able to connect to multiple servers and
/// move to a different network connection.

// TODO: remove later
// #[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Connection {
    socket_addr: SocketAddr,
    flags: u8,
    protocol_id: u8,
    duration: u16,
    client_id: Bytes,
    // TODO Struct Will
    _will: u8,
    _state: u8,
    will_topic: Bytes, // NOTE: this is a Bytes, not a BytesMut.
    will_message: Bytes,
}

impl Connection {
    pub fn new(
        socket_addr: SocketAddr,
        flags: u8,
        protocol_id: u8,
        duration: u16,
        client_id: Bytes,
    ) -> Result<Self, String> {
        let conn = Connection {
            socket_addr,
            flags,
            _will: 0,
            _state: 0,
            protocol_id,
            duration,
            client_id,
            will_topic: Bytes::new(),
            will_message: Bytes::new(),
        };
        Ok(conn)
    }
    pub fn try_insert(
        socket_addr: SocketAddr,
        flags: u8,
        protocol_id: u8,
        duration: u16,
        client_id: Bytes,
    ) -> Result<(), String> {
        let mut conn_hashmap = CONN_HASHMAP.lock().unwrap();
        let conn = Connection {
            socket_addr,
            flags,
            protocol_id,
            duration,
            client_id,
            _will: 0,
            _state: 0,
            will_topic: Bytes::new(),
            will_message: Bytes::new(),
        };
        match conn_hashmap.try_insert(socket_addr, conn) {
            Ok(_) => Ok(()),
            Err(e) => Err(eformat!(e.entry.key(), "already exists.")),
        }
    }
    pub fn contains_key(socket_addr: SocketAddr) -> bool {
        let conn_hashmap = CONN_HASHMAP.lock().unwrap();
        conn_hashmap.contains_key(&socket_addr)
    }
    pub fn remove(socket_addr: SocketAddr) -> Result<Connection, String> {
        let mut conn_hashmap = CONN_HASHMAP.lock().unwrap();
        match conn_hashmap.remove(&socket_addr) {
            Some(val) => Ok(val),
            None => Err(eformat!(socket_addr, "not found.")),
        }
    }
    // *Note* to an existing connection
    pub fn update_will_topic(
        socket_addr: SocketAddr,
        topic: String,
    ) -> Result<(), String> {
        let mut conn_hashmap = CONN_HASHMAP.lock().unwrap();
        match conn_hashmap.get_mut(&socket_addr) {
            Some(conn) => {
                conn.will_topic = Bytes::from(topic);
                Ok(())
            }
            None => Err(eformat!(socket_addr, "not found.")),
        }
    }
    pub fn update_will_msg(
        socket_addr: SocketAddr,
        message: String,
    ) -> Result<(), String> {
        let mut conn_hashmap = CONN_HASHMAP.lock().unwrap();
        match conn_hashmap.get_mut(&socket_addr) {
            Some(conn) => {
                conn.will_message = Bytes::from(message);
                Ok(())
            }
            None => Err(eformat!(socket_addr, "not found.")),
        }
    }
    pub fn db() {
        let conn_hashmap = CONN_HASHMAP.lock().unwrap();
        dbg!(conn_hashmap);
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_conn_hashmap() {

        /*
        use std::net::SocketAddr;
        // insert first connection
        let socket = "127.0.0.1:1200".parse::<SocketAddr>().unwrap();
        let connection = super::Connection::new(socket, 0).unwrap();
        let result = super::connection_insert(connection);
        assert!(result.is_ok());

        // insert duplicate, should fail.
        let socket = "127.0.0.1:1200".parse::<SocketAddr>().unwrap();
        let connection = super::Connection::new(socket, 0).unwrap();
        let result = super::connection_insert(connection);
        assert!(!result.is_ok());
        dbg!(result);

        // insert different socket_addr, should succeed.
        let socket = "127.0.0.2:1200".parse::<SocketAddr>().unwrap();
        let connection = super::Connection::new(socket, 0).unwrap();
        let result = super::connection_insert(connection);
        assert!(result.is_ok());
        dbg!(super::CONN_HASHMAP.lock().unwrap());

        // insert concrete topic to existing socket_addr/connection, should succeed.
        let socket = "127.0.0.2:1200".parse::<SocketAddr>().unwrap();
        let result = super::connection_filter_insert("test", socket);
        assert!(result.is_ok());
        dbg!(super::CONN_HASHMAP.lock().unwrap());

        // insert filter to non-existing socket_addr, should fail.
        let socket_new = "127.0.0.99:1200".parse::<SocketAddr>().unwrap();
        let result = super::connection_filter_insert("test", socket_new);
        assert!(!result.is_ok());
        dbg!(result);

        // insert duplicate filter to existing socket_addr, should fail.
        let socket = "127.0.0.2:1200".parse::<SocketAddr>().unwrap();
        let result = super::connection_filter_insert("test", socket);
        assert!(!result.is_ok());
        dbg!(result);

        // insert wildcard filter to existing socket_addr/connection, should succeed.
        let socket = "127.0.0.2:1200".parse::<SocketAddr>().unwrap();
        let result = super::connection_filter_insert("test/#", socket);
        assert!(result.is_ok());
        dbg!(super::CONN_HASHMAP.lock().unwrap());
        */
    }
    #[test]
    fn test_generate_uuid() {
        use std::net::{IpAddr, Ipv6Addr, SocketAddr};

        let socket = SocketAddr::new(
            IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 65535, 0, 1)),
            8080,
        );
        let id = super::generate_conn_id(socket, 0);
        assert!(!id.is_ok());
        dbg!(id);

        let socket = "127.0.0.1:1200".parse::<SocketAddr>().unwrap();
        let id = super::generate_conn_id(socket, 0);
        assert!(id.is_ok());
        dbg!(id);

        let socket = "127.0.0.1:1200".parse::<SocketAddr>().unwrap();
        let id = super::generate_conn_id(socket, 1);
        assert!(id.is_ok());
        dbg!(id);

        let socket = "127.0.0.2:1200".parse::<SocketAddr>().unwrap();
        let id = super::generate_conn_id(socket, 0);
        assert!(id.is_ok());
        dbg!(id);
    }
}
