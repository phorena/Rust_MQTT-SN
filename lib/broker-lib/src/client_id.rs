/// Client Id BisetMap stores client id and its socket addresses.
use bisetmap::BisetMap;
use bytes::Bytes;
use std::net::SocketAddr;
use std::sync::Mutex;

lazy_static! {
    static ref CLIENT_ID_MAP: Mutex<BisetMap<Bytes, SocketAddr>> =
        Mutex::new(BisetMap::new());
}

#[derive(Debug, Clone)]
pub struct ClientId {}

impl ClientId {
    pub fn insert(key: Bytes, val: SocketAddr) {
        let cache = CLIENT_ID_MAP.lock().unwrap();
        cache.insert(key, val);
    }
    pub fn key_exists(key: &Bytes) -> bool {
        CLIENT_ID_MAP.lock().unwrap().key_exists(key)
    }
    pub fn contains(key: &Bytes, val: &SocketAddr) -> bool {
        CLIENT_ID_MAP.lock().unwrap().contains(key, val)
    }
    pub fn get(key: &Bytes) -> Vec<SocketAddr> {
        let cache = CLIENT_ID_MAP.lock().unwrap();
        cache.get(key)
    }
    pub fn rev_get(key: &SocketAddr) -> Vec<Bytes> {
        let cache = CLIENT_ID_MAP.lock().unwrap();
        cache.rev_get(key)
    }
    pub fn delete(key: &Bytes) -> Vec<SocketAddr> {
        let cache = CLIENT_ID_MAP.lock().unwrap();
        cache.delete(key)
    }
    pub fn rev_delete(key: &SocketAddr) -> Vec<Bytes> {
        let cache = CLIENT_ID_MAP.lock().unwrap();
        cache.rev_delete(key)
    }
    pub fn debug() {
        let cache = CLIENT_ID_MAP.lock().unwrap();
        dbg!(&cache);
    }
}
#[cfg(test)]
#[test]
fn test_client_id() {
    use std::net::SocketAddr;

    let socket = "127.0.0.1:1200".parse::<SocketAddr>().unwrap();
    let socket2 = "127.0.0.2:1200".parse::<SocketAddr>().unwrap();
    let bytes = Bytes::from(&b"hello"[..]);
    ClientId::insert(bytes.clone(), socket);
    ClientId::insert(bytes.clone(), socket2);

    ClientId::debug();
    let sock_vec = ClientId::get(&bytes);
    dbg!(sock_vec);

    let val = ClientId::key_exists(&bytes);
    dbg!(val);

    ClientId::debug();
    let id_vec = ClientId::rev_delete(&socket);
    dbg!(id_vec);

    ClientId::debug();
    let id_vec = ClientId::rev_get(&socket);
    dbg!(id_vec);

    let sock_vec = ClientId::delete(&bytes);
    dbg!(sock_vec);
    ClientId::debug();
    let val = ClientId::key_exists(&bytes);
    dbg!(val);
}
