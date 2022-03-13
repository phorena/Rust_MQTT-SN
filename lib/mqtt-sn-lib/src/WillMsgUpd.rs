use bytes::{BufMut, BytesMut};
use custom_debug::Debug;
use getset::{CopyGetters, Getters, MutGetters, Setters};
use std::str;

#[derive(Debug, Clone, Getters, Setters, MutGetters, CopyGetters, Default)]
#[getset(get, set)]
pub struct WillMsgUpd {
    len: u8,
    #[debug(format = "0x{:x}")]
    msg_type: u8,
    will_topic: String, // TODO use enum for topic_name or topic_id
}

impl WillMsgUpd {
    pub fn constraint_len(_val: &u8) -> bool {
        //dbg!(_val);
        true
    }
    pub fn constraint_msg_type(_val: &u8) -> bool {
        //dbg!(_val);
        true
    }
    pub fn constraint_will_topic(_val: &String) -> bool {
        //dbg!(_val);
        true
    }
}
