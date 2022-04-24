use bytes::{BufMut, BytesMut};
use custom_debug::Debug;
use getset::{CopyGetters, Getters, MutGetters};
use std::str;

#[derive(Debug, Clone, Getters, MutGetters, CopyGetters, Default)]
#[getset(get, set)]
pub struct WillMsgUpd {
    len: u8,
    #[debug(format = "0x{:x}")]
    msg_type: u8,
    will_topic: String, // TODO use enum for topic_name or topic_id
}

impl WillMsgUpd {
    /*
    fn constraint_len(_val: &u8) -> bool {
        //dbg!(_val);
        true
    }
    fn constraint_msg_type(_val: &u8) -> bool {
        //dbg!(_val);
        true
    }
    fn constraint_will_topic(_val: &String) -> bool {
        //dbg!(_val);
        true
    }
    */
}
