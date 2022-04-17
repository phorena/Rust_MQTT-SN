use bytes::{BufMut, BytesMut};
use custom_debug::Debug;
use getset::{CopyGetters, Getters, MutGetters, Setters};

#[derive(
    Debug, Clone, Copy, Getters, Setters, MutGetters, CopyGetters, Default,
)]
#[getset(get, set)]
pub struct WillTopicReq {
    pub len: u8,
    #[debug(format = "0x{:x}")]
    pub msg_type: u8,
}

impl WillTopicReq {
    fn constraint_len(_val: &u8) -> bool {
        //dbg!(_val);
        true
    }
    fn constraint_msg_type(_val: &u8) -> bool {
        //dbg!(_val);
        true
    }
}
