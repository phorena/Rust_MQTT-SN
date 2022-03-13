use bytes::{BufMut, BytesMut};
use custom_debug::Debug;
use getset::{CopyGetters, Getters, MutGetters, Setters};
use std::str;

#[derive(Debug, Clone, Getters, Setters, MutGetters, CopyGetters, Default)]
#[getset(get, set)]
pub struct GwInfo {
    len: u8,
    #[debug(format = "0x{:x}")]
    msg_type: u8,
    #[debug(format = "0b{:08b}")]
    gw_id: u8,
    gw_add: String,
}

// TODO
impl GwInfo {
    pub fn constraint_len(_val: &u8) -> bool {
        //dbg!(_val);
        true
    }
    pub fn constraint_msg_type(_val: &u8) -> bool {
        //dbg!(_val);
        true
    }
    pub fn constraint_gw_id(_val: &u8) -> bool {
        //dbg!(_val);
        true
    }
    pub fn constraint_gw_add(_val: &String) -> bool {
        // dbg!(_val);
        true
    }
}