use crate::{
    ClientLib::MqttSnClient, /*Errors::ExoError,*/ MSG_LEN_WILLMESSAGEREQ,
    MSG_TYPE_WILLMESSAGEREQ,
};
use bytes::{BufMut, BytesMut};
use custom_debug::Debug;
use getset::{CopyGetters, Getters, MutGetters /*Setters*/};

macro_rules! function {
    () => {{
        fn f() {}
        fn type_name_of<T>(_: T) -> &'static str {
            std::any::type_name::<T>()
        }
        let name = type_name_of(f);
        &name[..name.len() - 3]
    }};
}
#[derive(
    Debug,
    Clone,
    Copy,
    Getters,
    /*Setters,*/ MutGetters,
    CopyGetters,
    Default,
)]
#[getset(get, set)]
pub struct WillMsgReq {
    pub len: u8,
    #[debug(format = "0x{:x}")]
    pub msg_type: u8,
}

impl WillMsgReq {
    /*fn constraint_len(_val: &u8) -> bool {
        //dbg!(_val);
        true
    }
    fn constraint_msg_type(_val: &u8) -> bool {
        //dbg!(_val);
        true
    }*/
}

pub fn tx(client: &MqttSnClient) {
    let willmsgreq = WillMsgReq {
        len: MSG_LEN_WILLMESSAGEREQ,
        msg_type: MSG_TYPE_WILLMESSAGEREQ,
    };
    let mut bytes_buf =
        BytesMut::with_capacity(MSG_LEN_WILLMESSAGEREQ as usize);
    dbg!(willmsgreq.clone());
    willmsgreq.try_write(&mut bytes_buf);
    dbg!(bytes_buf.clone());
    dbg!(client.remote_addr);
    // transmit to network
    let _result = client
        .transmit_tx
        .send((client.remote_addr, bytes_buf.to_owned()));
}
pub fn rx(
    buf: &[u8],
    size: usize,
    client: &MqttSnClient,
) -> Result<(), String> {
    let (will_msg_req, read_len) = WillMsgReq::try_read(&buf, size).unwrap();
    dbg!(will_msg_req.clone());
    if read_len == MSG_LEN_WILLMESSAGEREQ as usize {
        let _result = client.cancel_tx.send((
            client.remote_addr,
            will_msg_req.msg_type,
            0,
            0,
        ));
        Ok(())
    } else {
        // Err(ExoError::LenError(
        //     read_len,
        //     MSG_LEN_WILLMESSAGEREQ as usize,
        // ))
        Err(format!(
            "{} {}: Length Error: {}.",
            function!(),
            read_len,
            MSG_LEN_WILLMESSAGEREQ
        ))
    }
}
