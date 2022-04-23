// #[derive(Serialize, Deserialize, Debug, Clone)]
// For transfering data between methods
use crate::MessageDb::{MessageDb, /*MessageDbKey, MessageDbValue*/ };
use crate::{SubscriberDb::SubscriberDb, TopicDb::TopicDb};

use bytes::BytesMut;
use std::net::SocketAddr;
#[derive(Debug, Clone)]
pub struct Transfer {
    pub peer: SocketAddr,
    pub topic_id_counter: u16,
    // Use tuple(U,V) for because Vec takes one argument, Vec<T>
    pub egress_buffers: Vec<(SocketAddr, BytesMut)>,
    pub subscriber_db: SubscriberDb,
    // pub connection_db: ConnectionDb,
    pub topic_db: TopicDb,
    pub message_db: MessageDb,
    pub input_bytes: Vec<u8>,
    pub size: usize,
}
