use bytes::{BufMut, BytesMut};
use custom_debug::Debug;
use getset::{CopyGetters, Getters, MutGetters, Setters};
use std::mem;
use std::str;

#[derive(Debug, Clone, Getters, Setters, MutGetters, CopyGetters, Default)]
#[getset(get, set)]
pub struct Publish {
    len: u8,
    #[debug(format = "0x{:x}")]
    msg_type: u8,
    #[debug(format = "0b{:08b}")]
    flags: u8,
    pub topic_id: u16,
    msg_id: u16,
    data: String,
}

impl Publish {
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
    pub fn constraint_topic_id(_val: &u16) -> bool {
        //dbg!(_val);
        true
    }
    pub fn constraint_msg_id(_val: &u16) -> bool {
        //dbg!(_val);
        true
    }
    pub fn constraint_data(_val: &String) -> bool {
        //dbg!(_val);
        true
    }
}
