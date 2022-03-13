use bytes::{BufMut, BytesMut};
use custom_debug::Debug;
use getset::{CopyGetters, Getters, MutGetters, Setters};

#[derive(Debug, Clone, Copy, Getters, Setters, MutGetters, CopyGetters, Default)]
#[getset(get, set)]
pub struct WillTopicResp {
    pub len: u8,
    #[debug(format = "0x{:x}")]
    pub msg_type: u8,
    pub return_code: u8,
}

impl WillTopicResp {
    pub fn constraint_len(_val: &u8) -> bool {
        //dbg!(_val);
        true
    }
    pub fn constraint_msg_type(_val: &u8) -> bool {
        //dbg!(_val);
        true
    }
    pub fn constraint_return_code(_val: &u8) -> bool {
        //dbg!(_val);
        true
    }
}
