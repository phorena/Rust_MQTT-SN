use bytes::{BufMut, BytesMut};
use custom_debug::Debug;
use getset::{CopyGetters, Getters, MutGetters, Setters};
use std::mem;

#[derive(Debug, Clone, Copy, Getters, Setters, MutGetters, CopyGetters, Default)]
#[getset(get, set)]
pub struct PingReq {
    len: u8,
    #[debug(format = "0x{:x}")]
    msg_type: u8,
    client_id: u64,
}

impl PingReq {
    pub fn constraint_len(_val: &u8) -> bool {
        //dbg!(_val);
        true
    }
    pub fn constraint_msg_type(_val: &u8) -> bool {
        //dbg!(_val);
        true
    }
    pub fn constraint_client_id(_val: &u64) -> bool {
        //dbg!(_val);
        true
    }
}
