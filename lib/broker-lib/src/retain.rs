use bytes::Bytes;
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
    pub payload: Bytes,
}

impl Retain {
    pub fn new(
        topic_id: TopicIdType,
        msg_id: MsgIdType,
        payload: Bytes,
    ) -> Self {
        Self {
            topic_id,
            msg_id,
            payload,
        }
    }
    pub fn insert(topic_id: TopicIdType, msg_id: MsgIdType, payload: Bytes) {
        let mut retain_map = GLOBAL_RETAIN_MAP.lock().unwrap();
        // if the topic_id is already in the map, replace the old retain with the new one
        // TODO check error
        retain_map.insert(topic_id, Retain::new(topic_id, msg_id, payload));
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
    use bytes::Bytes;
    #[test]
    fn test_retain() {
        let topic_id = 11;
        let msg_id = 22;
        let payload = Bytes::from_static(b"hello");
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
        let payload = Bytes::from_static(b"hey");
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
        let payload = Bytes::from_static(b"hi");
        super::Retain::insert(topic_id, msg_id, payload);
        let retain = super::Retain::get(topic_id);
        {
            let retain_map = super::GLOBAL_RETAIN_MAP.lock().unwrap();
            dbg!(retain_map);
        }
        dbg!(&retain);
        println!("{:?}", retain.unwrap());
    }
}
