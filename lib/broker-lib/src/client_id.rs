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
    pub fn insert(client_id: Bytes, val: SocketAddr) {
        CLIENT_ID_MAP.lock().unwrap().insert(client_id, val);
    }
    pub fn exists(client_id: &Bytes) -> bool {
        CLIENT_ID_MAP.lock().unwrap().key_exists(client_id)
    }
    pub fn contains(client_id: &Bytes, val: &SocketAddr) -> bool {
        CLIENT_ID_MAP.lock().unwrap().contains(client_id, val)
    }
    pub fn get(client_id: &Bytes) -> Vec<SocketAddr> {
        CLIENT_ID_MAP.lock().unwrap().get(client_id)
    }
    pub fn rev_get(client_id: &SocketAddr) -> Vec<Bytes> {
        CLIENT_ID_MAP.lock().unwrap().rev_get(client_id)
    }
    pub fn delete(client_id: &Bytes) -> Vec<SocketAddr> {
        CLIENT_ID_MAP.lock().unwrap().delete(client_id)
    }
    pub fn rev_delete(client_id: &SocketAddr) -> Vec<Bytes> {
        CLIENT_ID_MAP.lock().unwrap().rev_delete(client_id)
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

    let val = ClientId::exists(&bytes);
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
    let val = ClientId::exists(&bytes);
    dbg!(val);
}
