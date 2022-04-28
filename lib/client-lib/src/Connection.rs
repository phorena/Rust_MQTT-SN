use crate::Filter::Filter;
use log::*;
use rand::Rng;
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::v1::{Context, Timestamp};
use uuid::{Error, Uuid};

macro_rules! function {
    () => {{
        fn f() {}
        fn type_name_of<T>(_: T) -> &'static str {
            std::any::type_name::<T>()
        }
        let name = type_name_of(f);
        &name[..name.len() - 3]
    }};
}

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
            return Err(msg.to_string());
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

#[derive(Debug, Clone)]
pub struct Connection {
    socket_addr: SocketAddr,
    clean: bool,
    // TODO Struct Will
    will: u8,
    state: u8,
    duration: u16,
    filter: Filter,
}

impl Connection {
    pub fn new(socket_addr: SocketAddr, duration: u16) -> Result<Self, String> {
        let conn = Connection {
            clean: true,
            will: 0,
            state: 0,
            socket_addr,
            duration,
            filter: Filter::new(),
        };
        Ok(conn)
    }
}

/// Connection is stored in a HashMap indexed by ConnId because the SocketAddr of the client
/// can change.
/// Context is used to generate a unique ConnId for each client.
#[derive(Debug, Clone)]
pub struct ConnHashMap {
    context: u16,
    local_addr: SocketAddr,
    socket_addr_hashmap: HashMap<SocketAddr, ConnId>,
    id_hashmap: HashMap<ConnId, Connection>,
}

impl ConnHashMap {
    pub fn new(context: u16, local_addr: SocketAddr) -> Self {
        ConnHashMap {
            context,
            local_addr,
            socket_addr_hashmap: HashMap::new(),
            id_hashmap: HashMap::new(),
        }
    }
    pub fn insert(&mut self, conn: Connection) -> Result<(), String> {
        let socket_addr = conn.socket_addr;
        if !self.socket_addr_hashmap.contains_key(&socket_addr) {
            let id = generate_conn_id(socket_addr, self.context)?;
            self.id_hashmap.insert(id, conn);
            self.socket_addr_hashmap.insert(socket_addr, id);
            return Ok(());
        } else {
            return Err(format!(
                "{}: socket_addr: {} already exists",
                function!(),
                socket_addr
            ));
        }
    }
    /// Get connection by socket_addr
    pub fn socket_addr_get(
        &self,
        socket_addr: SocketAddr,
    ) -> Option<Connection> {
        let id = self.socket_addr_hashmap.get(&socket_addr)?;
        self.id_hashmap.get(id).cloned()
    }
    /// Get connection by id
    pub fn id_get(&self, id: ConnId) -> Option<Connection> {
        self.id_hashmap.get(&id).cloned()
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_conn_hashmap() {
        use rand::Rng;

        let random_bytes = rand::thread_rng().gen::<[u8; 6]>();
        println!("{:?}", random_bytes);
        use std::net::{IpAddr, Ipv6Addr, SocketAddr};
        let socket = "127.0.0.1:0".parse::<SocketAddr>().unwrap();
        let mut conn_hashmap = super::ConnHashMap::new(11, socket);
        let socket = "127.0.0.1:1200".parse::<SocketAddr>().unwrap();
        let connection = super::Connection::new(socket, 0).unwrap();
        let result = conn_hashmap.insert(connection);
        assert!(result.is_ok());
        dbg!(result);
        let connection = super::Connection::new(socket, 0).unwrap();
        let result = conn_hashmap.insert(connection);
        assert!(!result.is_ok());
        dbg!(result);
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
/*
running 1 test
[src/Connection.rs:176] id = Err(
    "ipv6: ::ffff:0.0.0.1, segments: [0, 0, 0, 0, 0, 65535, 0, 1] not supported",
)
[src/Connection.rs:181] id = Ok(
    98e73c4e-bbae-11ec-8000-7f00000104b0,
)
[src/Connection.rs:186] id = Ok(
    98e73d83-bbae-11ec-8001-7f00000104b0,
)
[src/Connection.rs:191] id = Ok(
    98e73e70-bbae-11ec-8000-7f00000204b0,
)
test Connection::test::test_generate_uuid ... ok

[src/Connection.rs:181] id = Ok(
    d8d577b2-bbae-11ec-8000-7f00000104b0,
)
[src/Connection.rs:186] id = Ok(
    d8d57930-bbae-11ec-8001-7f00000104b0,
)
[src/Connection.rs:191] id = Ok(
    d8d57a14-bbae-11ec-8000-7f00000204b0,
*/
