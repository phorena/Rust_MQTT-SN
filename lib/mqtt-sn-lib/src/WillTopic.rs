use bytes::{BufMut, BytesMut};
use custom_debug::Debug;
use getset::{CopyGetters, Getters, MutGetters, Setters};
use std::str;

#[derive(Debug, Clone, Getters, Setters, MutGetters, CopyGetters, Default)]
#[getset(get, set)]
pub struct WillTopic {
    len: u8,
    #[debug(format = "0x{:x}")]
    msg_type: u8,
    #[debug(format = "0b{:08b}")]
    flags: u8,
    will_topic: String, // TODO use enum for topic_name or topic_id
}

impl WillTopic {
    pub fn constraint_len(_val: &u8) -> bool {
        //dbg!(_val);
        true
    }
    pub fn constraint_msg_type(_val: &u8) -> bool {
        //dbg!(_val);
        true
    }
    pub fn constraint_flags(_val: &u8) -> bool {
        //dbg!(_val);
        true
    }
    pub fn constraint_will_topic(_val: &String) -> bool {
        //dbg!(_val);
        true
    }
}
