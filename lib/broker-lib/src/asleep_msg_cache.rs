use crate::publish::Publish;
use bisetmap::BisetMap;
use std::net::SocketAddr;
/// Cache for published messages
use std::sync::Mutex;

lazy_static! {
    static ref ASLEEP_MSG_CACHE: Mutex<BisetMap<SocketAddr, Publish>> =
        Mutex::new(BisetMap::new());
}

#[derive(Debug, Clone)]
pub struct AsleepMsgCache {}

impl AsleepMsgCache {
    // Don't need vec of Publish because BisetMap allows the same key with different
    // values. HashMap would require a Vec of Publish, one key maps to one value.
    pub fn insert(key: SocketAddr, value: Publish) {
        let cache = ASLEEP_MSG_CACHE.lock().unwrap();
        cache.insert(key, value);
    }

    // returns all the Publish objects with the key.
    pub fn delete(key: SocketAddr) -> Vec<Publish> {
        let cache = ASLEEP_MSG_CACHE.lock().unwrap();
        cache.delete(&key)
    }
    pub fn debug() {
        let cache = ASLEEP_MSG_CACHE.lock().unwrap();
        dbg!(&cache);
    }
}
#[cfg(test)]
#[test]
fn test_asleep_cache() {
    use bytes::BytesMut;
    use std::net::SocketAddr;

    let socket = "127.0.0.1:1200".parse::<SocketAddr>().unwrap();
    let socket2 = "127.0.0.2:1200".parse::<SocketAddr>().unwrap();
    let bytes = BytesMut::from(&b"hello"[..]);
    let p = Publish::new(22, 22, 1, 3, bytes.clone());
    AsleepMsgCache::insert(socket, p);
    let p = Publish::new(11, 11, 1, 3, bytes.clone());
    AsleepMsgCache::insert(socket, p);
    let p = Publish::new(33, 33, 1, 3, bytes.clone());
    AsleepMsgCache::insert(socket2, p);
    let p = Publish::new(55, 55, 1, 3, bytes);
    AsleepMsgCache::insert(socket2, p);

    AsleepMsgCache::debug();
    let msg_vec = AsleepMsgCache::delete(socket);
    dbg!(msg_vec);
    AsleepMsgCache::debug();
}
