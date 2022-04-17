use crate::Filter::Filter;
// use log::*;
// use rand::Rng;
use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::v1::{Context, Timestamp};
use uuid::Uuid;

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
            socket_addr,
            clean: true,
            will: 0,
            state: 0,
            duration,
            filter: Filter::new(),
        };
        Ok(conn)
    }
}

lazy_static! {
    /// Store ConnectionID with SocketAddr indexing
    /// because SocketAddr can change when the client reconnects.
    static ref SOCKET_ADDR_HASHMAP: Mutex<HashMap<SocketAddr, ConnId>> =
        Mutex::new(HashMap::new());
    /// Store Connection with ConnId indexing.
    static ref ID_HASHMAP: Mutex<HashMap<ConnId, Connection>> =
        Mutex::new(HashMap::new());
}

// Insert a new connection into the hashmaps.
pub fn connection_hashmap_insert(conn: Connection) -> Result<(), String> {
    let socket_addr = conn.socket_addr;
    let mut socket_addr_hashmap = SOCKET_ADDR_HASHMAP.lock().unwrap();
    let mut id_hashmap = ID_HASHMAP.lock().unwrap();
    if !socket_addr_hashmap.contains_key(&socket_addr) {
        // TODO change context 999 to a global value
        let id = generate_conn_id(socket_addr, 999)?;
        id_hashmap.insert(id, conn);
        socket_addr_hashmap.insert(socket_addr, id);
        return Ok(());
    } else {
        return Err(format!(
            "{}: socket_addr: {} already exists",
            function!(),
            socket_addr
        ));
    }
}

/// Insert a filter into the connection hashmap.
pub fn connection_filter_insert(
    socket_addr: SocketAddr,
    filter: String,
) -> Result<ConnId, String> {
    let socket_addr_hashmap = SOCKET_ADDR_HASHMAP.lock().unwrap();
    let mut id_hashmap = ID_HASHMAP.lock().unwrap();
    let id = socket_addr_hashmap.get(&socket_addr).unwrap();
    dbg!(id);
    let conn = id_hashmap.get_mut(&id).unwrap();
    dbg!(conn.clone());
    dbg!(filter.clone());
    conn.filter.insert(&filter[..], *id);
    dbg!(conn);
    Ok(*id)
}

#[cfg(test)]
mod test {
    #[test]
    fn test_conn_hashmap() {
        use std::net::{IpAddr, Ipv4Addr, SocketAddr};
        use std::thread;

        let socket = "127.0.0.1:1200".parse::<SocketAddr>().unwrap();
        let connection = super::Connection::new(socket, 0).unwrap();
        let result = super::connection_hashmap_insert(connection).unwrap();
        dbg!(result);
        let socket = "127.0.0.1:1200".parse::<SocketAddr>().unwrap();
        let connection = super::Connection::new(socket, 0).unwrap();
        let result = super::connection_hashmap_insert(connection);
        assert!(!result.is_ok());
        dbg!(result);
        let socket = "127.0.0.2:1200".parse::<SocketAddr>().unwrap();
        let connection = super::Connection::new(socket, 0).unwrap();
        let result = super::connection_hashmap_insert(connection).unwrap();
        dbg!(result);
        dbg!(super::ID_HASHMAP.lock().unwrap());
        dbg!(super::SOCKET_ADDR_HASHMAP.lock().unwrap());
        let socket = "127.0.0.1:1200".parse::<SocketAddr>().unwrap();
        let id = super::SOCKET_ADDR_HASHMAP
            .lock()
            .unwrap()
            .get(&socket)
            .unwrap()
            .to_owned();
        let socket = "127.0.0.5:1200".parse::<SocketAddr>().unwrap();
        let connection = super::Connection::new(socket, 0).unwrap();
        *super::ID_HASHMAP.lock().unwrap().get_mut(&id).unwrap() = connection;
        dbg!(super::ID_HASHMAP.lock().unwrap());

        let handle = thread::spawn(move || {
            let socket = "127.0.0.7:1200".parse::<SocketAddr>().unwrap();
            let connection = super::Connection::new(socket, 0).unwrap();
            let result = super::connection_hashmap_insert(connection).unwrap();
            dbg!(result);
        });
        handle.join().unwrap();
        dbg!(super::ID_HASHMAP.lock().unwrap());
        /*
        let random_bytes = rand::thread_rng().gen::<[u8; 6]>();
        println!("{:?}", random_bytes);
        use std::net::{IpAddr, Ipv6Addr, SocketAddr};
        let socket = "127.0.0.1:0".parse::<SocketAddr>().unwrap();
        // let mut conn_hashmap = super::ConnHashMap::new(11, socket);
        let socket = "127.0.0.1:1200".parse::<SocketAddr>().unwrap();
        let connection = super::Connection::new(socket, 0).unwrap();
        // let result = conn_hashmap.insert(connection);
        assert!(result.is_ok());
        dbg!(result);
        let connection = super::Connection::new(socket, 0).unwrap();
        let result = conn_hashmap.insert(connection);
        assert!(!result.is_ok());
        dbg!(result);
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
