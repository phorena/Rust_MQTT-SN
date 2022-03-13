use bytes::{BufMut, BytesMut};
use custom_debug::Debug;
use getset::{CopyGetters, Getters, MutGetters, Setters};
use std::mem;
use std::str;

#[derive(Debug, Clone, Getters, Setters, MutGetters, CopyGetters, Default)]
#[getset(get, set)]
pub struct Connect {
    pub len: u8,
    #[debug(format = "0x{:x}")]
    pub msg_type: u8,
    #[debug(format = "0b{:08b}")]
    pub flags: u8,
    pub protocol_id: u8,
    pub duration: u16,
    pub client_id: String,
}

// TODO
impl Connect {
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
    pub fn constraint_protocol_id(_val: &u8) -> bool {
        //dbg!(_val);
        true
    }
    pub fn constraint_duration(_val: &u16) -> bool {
        //dbg!(_val);
        true
    }
    pub fn constraint_client_id(_val: &String) -> bool {
        // dbg!(_val);
        true
    }
}
