use bytes::Bytes;
use crossbeam::channel::*;
use hashbrown::HashMap;
use log::*;
use std::sync::Mutex;
use std::thread;

use crate::{
    broker_lib::MqttSnClient,
    flags::QoSConst,
    flags::*,
    publish::Publish,
    MsgIdType,
    // eformat,
    // function,
    TopicIdType,
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
    hash_map: HashMap<TopicIdType, Retain>,
}
impl RetainCache {
    pub fn new() -> Self {
        Self {
            hash_map: HashMap::new(),
        }
    }
    pub fn insert(&mut self, retain: Retain) {
        self.hash_map.insert(retain.topic_id, retain);
    }
    pub fn get(&self, topic_id: &TopicIdType) -> Option<&Retain> {
        self.hash_map.get(topic_id).clone()
    }
    pub fn run2(&mut self, client: MqttSnClient) {
        let client2 = client.clone();
        let mut self2 = self.clone();
        let _sub_thread = thread::spawn(move || loop {
            select! {
                recv(&client2.sub_retain_rx) -> msg => {
                    println!("3000 **************************************************");
                    dbg!(&msg);
                    match msg {
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
                        Err(err) => {
                            error!("{}", err);
                        }
                    }
                }
                recv(&client2.pub_retain_rx) -> retain => {
                    match retain {
                        Ok(retain) => {
                            println!("3001 **************************************************");
                            dbg!(&retain);
                            self2.hash_map.insert(retain.topic_id, retain);
                        }
                        Err(err) => {
                            error!("{}", err);
                        }
                    }
                }
            }
        });
    }
    pub fn run(&mut self, client: MqttSnClient) {
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
