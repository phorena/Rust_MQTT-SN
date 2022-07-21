/// Cache the retained messages.
/// If the retain flag is set, the message is cached.
/// The cache is used to send retained messages when a client subscribes to the topic.
/// The messages are also saved and retrieved from MongoDB.

use bytes::Bytes;
use crossbeam::channel::*;
use hashbrown::HashMap;
use log::*;
use std::sync::Mutex;
use std::thread;
use std::sync::Arc;

use crate::{
    broker_lib::MqttSnClient,
    flags::QoSConst,
    flags::*,
    publish::Publish,
    MsgIdType,
    // eformat,
    // function,
    TopicIdType,
    mongodb::*,
};

lazy_static! {
    pub static ref RETAIN_MAP: Mutex<HashMap<TopicIdType, Retain>> =
        Mutex::new(HashMap::new());
}

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
    db: RetainDb,
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
        self.db.upsert(retain.qos, retain.topic_id,
         "msg_id".to_string(), retain.msg_id, &retain.payload[..]);
    }
    fn get(&mut self, topic_id: &TopicIdType) -> Option<Retain> {
        let mut hash_map = self.hash_map.lock().unwrap();
        match hash_map.get(topic_id) {
            Some(retain) => {
                dbg!(&retain);
                let retain2 = Retain {
                    qos: retain.qos,
                    topic_id: retain.topic_id,
                    msg_id: retain.msg_id,
                    payload: retain.payload.clone(),
                };
                dbg!(&retain2);
                Some(retain2)
            }
            None => {
                match self.db.get_with_topic_id(*topic_id){
                    Ok(retain_doc) => {
                        dbg!(&retain_doc);
                        // let retain_doc2 = retain_doc.clone();
                        // let msg = Bytes::from(&msg2[..]);
                        // dbg!(msg);
                        let msg = retain_doc.msg.clone().into_vec();
                        dbg!(&msg);
                        let msg = retain_doc.msg.as_slice();
                        dbg!(&msg);
                        let retain = Retain {
                            qos: retain_doc.qos,
                            topic_id: retain_doc.topic_id,
                            msg_id: retain_doc.msg_id,
                            payload: Bytes::copy_from_slice(&msg[..]),
                            // payload: Bytes::from("hello"),
                        };
                        // let hash_map = &mut self.hash_map.clone();
                        // let mut hash_map = self.hash_map.lock().unwrap();
                        hash_map.insert(*topic_id, retain.clone());
        dbg!((topic_id, retain.topic_id));
        let result = hash_map.get(topic_id);
        dbg!(&result);
                        dbg!(&retain);
                        dbg!(&hash_map);
                        return Some(retain);
                    },
                    Err(e) => {
                        error!("{}", e);
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
                // A message from subscriber with socket_addr and topic_id.
                // If the topic_id is in the hash_map, send the message to the subscriber.
                recv(&client2.sub_retain_rx) -> msg => {
                    println!("3000 **************************************************");
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
                // A message from subscriber with retain message.
                // Insert the retain message into the hash_map.
                recv(&client2.pub_retain_rx) -> retain => {
                    match retain {
                        Ok(retain) => {
                            println!("3001 **************************************************");
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
