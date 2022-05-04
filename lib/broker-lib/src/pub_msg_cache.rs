/// Cache for published messages
use hashbrown::HashMap;
use std::sync::Mutex;

use crate::MsgIdType;

use crate::Filter::Subscriber;
use crate::Publish::Publish;

use crate::{eformat, function};
use std::net::SocketAddr;

lazy_static! {
    static ref PUB_MSG_CACHE: Mutex<HashMap<(SocketAddr, MsgIdType), PubMsgCache>> =
        Mutex::new(HashMap::new());
}

#[derive(Debug, Clone)]
pub struct PubMsgCache {
    pub publish: Publish,
    pub subscriber_vec: Vec<Subscriber>,
}

impl PubMsgCache {
    /// Cache for publish messages and subscribers for the PUBREL message.
    /// For QoS 2, the broker waits for the PUBREL message before sending the PUBCOMP message
    /// to the publisher, then send the PUBLISH message to the subscribers.
    /// Note: publisher are the sender and subscribers are receivers of the message.
    /// Note: QoS 2 is a four-way handshake. The broker has to complete the handshake before sending
    /// the PUBLISH message to the subscribers.
    pub fn try_insert(
        key: (SocketAddr, MsgIdType),
        value: PubMsgCache,
    ) -> Result<(), String> {
        let mut pub_cache = PUB_MSG_CACHE.lock().unwrap();
        match pub_cache.try_insert(key, value) {
            Ok(_) => return Ok(()),
            Err(_e) => return Err(eformat!(key.0, key.1, "already exists.")),
        };
    }

    pub fn remove(key: (SocketAddr, MsgIdType)) -> Option<PubMsgCache> {
        // mut is needed to remove the entry.
        let mut pub_cache = PUB_MSG_CACHE.lock().unwrap();
        let val = pub_cache.remove(&key)?;
        Some(val)
    }

    pub fn get(key: (SocketAddr, MsgIdType)) -> Option<PubMsgCache> {
        let pub_cache = PUB_MSG_CACHE.lock().unwrap();
        let val = pub_cache.get(&key)?;
        // need to clone the value because the value is borrowed.
        Some(val.clone())
    }
}
