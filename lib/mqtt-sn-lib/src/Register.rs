use bytes::{BufMut, BytesMut};
use custom_debug::Debug;
use getset::{CopyGetters, Getters, MutGetters, Setters};
use std::mem;
use std::str;

#[derive(Debug, Clone, Getters, Setters, MutGetters, CopyGetters, Default)]
#[getset(get, set)]
pub struct Register {
    len: u8,
    #[debug(format = "0x{:x}")]
    pub msg_type: u8,
    topic_id: u16,
    pub msg_id: u16,
    pub topic_name: String, // TODO use enum for topic_name or topic_id
}

impl Register {
    pub fn constraint_len(_val: &u8) -> bool {
        //dbg!(_val);
        true
    }
    pub fn constraint_msg_type(_val: &u8) -> bool {
        //dbg!(_val);
        true
    }
    pub fn constraint_topic_id(_val: &u16) -> bool {
        //dbg!(_val);
        true
    }
    pub fn constraint_msg_id(_val: &u16) -> bool {
        //dbg!(_val);
        true
    }
    pub fn constraint_topic_name(_val: &String) -> bool {
        //dbg!(_val);
        true
    }
}
