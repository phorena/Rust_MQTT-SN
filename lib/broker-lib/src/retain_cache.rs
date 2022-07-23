/// Cache the retained messages.
/// If the retain flag is set, cache the message in hashmap and save it to the database.
/// The cache is used to send retained messages when a client subscribes to the topic.
/// The messages are also saved and retrieved from MongoDB.
use bytes::Bytes;
use crossbeam::channel::*;
use hashbrown::HashMap;
use log::*;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

use crate::{
    broker_lib::MqttSnClient,
    flags::QoSConst,
    flags::*,
    mongodb::*,
    publish::Publish,
    MsgIdType,
    // eformat,
    // function,
    TopicIdType,
};

#[derive(Debug, Clone)]
pub struct Retain {
    pub qos: QoSConst,
    pub topic_id: TopicIdType,
    pub msg_id: MsgIdType,
    pub payload: Bytes,
}

#[derive(Debug, Clone)]
pub struct RetainCache {
    hash_map: Arc<Mutex<HashMap<TopicIdType, Retain>>>,
    db: Option<RetainDb>,
}
impl RetainCache {
    pub fn new(url: &str) -> Self {
        Self {
            hash_map: Arc::new(Mutex::new(HashMap::new())),
            db: RetainDb::new(url),
        }
    }
    fn insert(&mut self, retain: Retain) {
        let mut hash_map = self.hash_map.lock().unwrap();
        hash_map.insert(retain.topic_id, retain.clone());
        if let Some(db) = &self.db {
            db.upsert(
                retain.qos,
                retain.topic_id,
                "msg_id".to_string(),
                retain.msg_id,
                &retain.payload[..],
            );
        }
    }
    fn get(&mut self, topic_id: &TopicIdType) -> Option<Retain> {
        let mut hash_map = self.hash_map.lock().unwrap();
        match hash_map.get(topic_id) {
            // in the cache, return the value.
            Some(retain) => {
                dbg!(&retain);
                Some(retain.clone())
            }
            None => {
                // not in the cache, get from MongoDB.
                if let Some(db) = &self.db {
                    match db.get_with_topic_id(*topic_id) {
                        Ok(retain_doc) => {
                            dbg!(&retain_doc);
                            // Convert the serde_bytes into slice.
                            let msg = retain_doc.msg.as_slice();
                            dbg!(&msg);
                            let retain = Retain {
                                qos: retain_doc.qos,
                                topic_id: retain_doc.topic_id,
                                msg_id: retain_doc.msg_id,
                                // Convert slice into Bytes.
                                payload: Bytes::copy_from_slice(&msg[..]),
                            };
                            // insert the value into the cache.
                            hash_map.insert(*topic_id, retain.clone());
                            return Some(retain);
                        }
                        Err(_) => {
                            // Not in MongoDB, return None.
                            ();
                        }
                    }
                }
                None
            }
        }
    }
    pub fn run(&mut self, client: MqttSnClient) {
        let client2 = client.clone();
        let mut self2 = self.clone();
        let _sub_thread = thread::spawn(move || loop {
            select! {
                // A subscriber sent socket_addr and topic_id.
                // If the topic_id is in the hash_map, send the message to the subscriber.
                recv(&client2.sub_retain_rx) -> msg => {
                    dbg!(&msg);
                    match msg {
                        Ok((socket_addr, topic_id)) => {
                            if let Some(retain) = self2.get(&topic_id) {
                                dbg!(&retain);
                                let _result = Publish::send(
                                    retain.topic_id,
                                    retain.msg_id,
                                    retain.qos,
                                    RETAIN_FALSE,
                                    retain.payload.clone(),
                                    &client2,
                                    socket_addr,
                                );
                            }
                        }
                        Err(err) => {
                            dbg!(&err);
                            error!("{}", err);
                        }
                    }
                }
                // A publisher sent retain message.
                // Insert the retain message into the cache and db.
                recv(&client2.pub_retain_rx) -> retain => {
                    match retain {
                        Ok(retain) => {
                            dbg!(&retain);
                            self2.insert(retain);
                        }
                        Err(err) => {
                            error!("{}", err);
                        }
                    }
                }
            }
        });
    }
    /*
    pub fn run3(&mut self, client: MqttSnClient) {
        let client2 = client.clone();
        let mut self2 = self.clone();
        let _sub_thread = thread::spawn(move || loop {
            match client2.pub_retain_rx.recv() {
                Ok(retain) => {
                    println!("2000 **************************************************");
                    self2.hash_map.insert(retain.topic_id, retain);
                }
                Err(e) => {
                    eprintln!("{}", e);
                }
            }
        });
        let client2 = client.clone();
        let self2 = self.clone();
        let _pub_thread = thread::spawn(move || loop {
            match client2.sub_retain_rx.recv() {
                Ok((socket_addr, topic_id)) => {
                    if let Some(retain) = self2.hash_map.get(&topic_id) {
                        let _result = Publish::send(
                            retain.topic_id,
                            retain.msg_id,
                            retain.qos,
                            RETAIN_FALSE,
                            retain.payload.clone(),
                            &client2,
                            socket_addr,
                        );
                    }
                }
                Err(e) => {
                    eprintln!("{}", e);
                }
            }
        });
    }
    */
}

#[cfg(test)]
mod test {
    /*
        #[test]
        fn test_retain() {
            use bytes::{BytesMut};
            let buf: &[u8] = &[
                len as u8,
                MSG_TYPE_PUBLISH,
                flags,
                msg_id_byte_0,
                msg_id_byte_1,
                topic_id_byte_0,
                topic_id_byte_1,
            ];
            bytes_buf.put(buf);
    let mut buf = BytesMut::with_capacity(64);

    buf.put(b'h');
    buf.put(b'e');
    buf.put("llo");
            let topic_id = 11;
            let msg_id = 22;
            let mut payload = BytesMut::new();
            payload.put("hello");
            super::Retain::insert(topic_id, msg_id, payload);
            let retain = super::Retain::get(topic_id);
            {
                let retain_map = super::RETAIN_MAP.lock().unwrap();
                dbg!(retain_map);
            }
            dbg!(&retain);
            println!("{:?}", retain.unwrap());

            // second retain
            let topic_id = 111;
            let msg_id = 33;
            let payload = BytesMut::from_static(b"hey");
            super::Retain::insert(topic_id, msg_id, payload);
            let retain = super::Retain::get(topic_id);
            {
                let retain_map = super::RETAIN_MAP.lock().unwrap();
                dbg!(retain_map);
            }
            dbg!(&retain);
            println!("{:?}", retain.unwrap());

            // replace retain
            let topic_id = 11;
            let msg_id = 33;
            let payload = BytesMut::from_static(b"hi");
            super::Retain::insert(topic_id, msg_id, payload);
            let retain = super::Retain::get(topic_id);
            {
                let retain_map = super::RETAIN_MAP.lock().unwrap();
                dbg!(retain_map);
            }
            dbg!(&retain);
            println!("{:?}", retain.unwrap());
        }
        */
}
