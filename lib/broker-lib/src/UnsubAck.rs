use bytes::{BufMut, BytesMut};
use custom_debug::Debug;
use getset::{CopyGetters, Getters, MutGetters};
use std::mem;

#[derive(Debug, Clone, Getters, MutGetters, CopyGetters, Default)]
#[getset(get, set)]
pub struct UnsubAck {
    pub len: u8,
    #[debug(format = "0x{:x}")]
    pub msg_type: u8,
    #[debug(format = "0b{:08b}")]
    pub msg_id: u16,
}

impl UnsubAck {
/*
    fn constraint_len(_val: &u8) -> bool {
        //dbg!(_val);
        true
    }
    fn constraint_msg_type(_val: &u8) -> bool {
        //dbg!(_val);
        true
    }
    fn constraint_msg_id(_val: &u16) -> bool {
        //dbg!(_val);
        true
    }
    */
}
