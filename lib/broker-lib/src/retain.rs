use bytes::BytesMut;
use hashbrown::HashMap;
use std::sync::Mutex;

use crate::{
    MsgIdType,
    // eformat,
    // function,
    TopicIdType,
};

lazy_static! {
    pub static ref GLOBAL_RETAIN_MAP: Mutex<HashMap<TopicIdType, Retain>> =
        Mutex::new(HashMap::new());
}

#[derive(Debug, Clone)]
pub struct Retain {
    pub topic_id: TopicIdType,
    pub msg_id: MsgIdType,
    pub payload: BytesMut,
}

impl Retain {
    pub fn new(
        topic_id: TopicIdType,
        msg_id: MsgIdType,
        payload: BytesMut,
    ) -> Self {
        Self {
            topic_id,
            msg_id,
            payload,
        }
    }
    pub fn insert(topic_id: TopicIdType, msg_id: MsgIdType, payload: BytesMut) {
        let mut retain_map = GLOBAL_RETAIN_MAP.lock().unwrap();
        // if the topic_id is already in the map, replace the old retain with the new one
        // TODO check error
        retain_map.insert(topic_id, Retain::new(topic_id, msg_id, payload));
        dbg!(&retain_map);
    }
    pub fn get(topic_id: TopicIdType) -> Option<Retain> {
        let retain_map = GLOBAL_RETAIN_MAP.lock().unwrap();
        match retain_map.get(&topic_id) {
            Some(retain) => Some(retain.clone()),
            None => None,
        }
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
                let retain_map = super::GLOBAL_RETAIN_MAP.lock().unwrap();
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
                let retain_map = super::GLOBAL_RETAIN_MAP.lock().unwrap();
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
                let retain_map = super::GLOBAL_RETAIN_MAP.lock().unwrap();
                dbg!(retain_map);
            }
            dbg!(&retain);
            println!("{:?}", retain.unwrap());
        }
        */
}
