use bytes::{BufMut, BytesMut};
use custom_debug::Debug;
use getset::{CopyGetters, Getters, MutGetters, Setters};
use std::mem;

#[derive(Debug, Clone, Copy, Getters, Setters, MutGetters, CopyGetters, Default)]
#[getset(get, set)]
pub struct Disconnect {
    pub len: u8,
    #[debug(format = "0x{:x}")]
    pub msg_type: u8,
    pub duration: u16,
}

impl Disconnect {
    pub fn constraint_len(_val: &u8) -> bool {
        //dbg!(_val);
        true
    }
    pub fn constraint_msg_type(_val: &u8) -> bool {
        //dbg!(_val);
        true
    }
    pub fn constraint_duration(_val: &u16) -> bool {
        //dbg!(_val);
        true
    }
}
