use bytes::{BufMut, BytesMut};
use custom_debug::Debug;
use getset::{CopyGetters, Getters, MutGetters, Setters};
use std::mem;

#[derive(Debug, Clone, Getters, Setters, MutGetters, CopyGetters, Default)]
#[getset(get, set)]
pub struct SubAck {
    pub len: u8,
    #[debug(format = "0x{:x}")]
    pub msg_type: u8,
    #[debug(format = "0b{:08b}")]
    pub flags: u8,
    pub topic_id: u16,
    pub msg_id: u16,
    pub return_code: u8,
}

impl SubAck {
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
    pub fn constraint_return_code(_val: &u8) -> bool {
        //dbg!(_val);
        true
    }
}
