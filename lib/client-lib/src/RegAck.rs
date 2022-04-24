use bytes::{BufMut, BytesMut};
use custom_debug::Debug;
use getset::{CopyGetters, Getters, MutGetters /*Setters*/};
use std::mem;

use crate::{
    ClientLib::MqttSnClient, /*Errors::ExoError,*/ MSG_LEN_REGACK,
    MSG_TYPE_REGACK,
};
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
    Debug, Clone, Getters, /*Setters,*/ MutGetters, CopyGetters, Default,
)]
#[getset(get, set)]
pub struct RegAck {
    pub len: u8,
    #[debug(format = "0x{:x}")]
    pub msg_type: u8,
    pub topic_id: u16,
    pub msg_id: u16,
    pub return_code: u8,
}

impl RegAck {
    /*fn constraint_len(_val: &u8) -> bool {
        //dbg!(_val);
        true
    }
    fn constraint_msg_type(_val: &u8) -> bool {
        //dbg!(_val);
        true
    }
    fn constraint_topic_id(_val: &u16) -> bool {
        //dbg!(_val);
        true
    }
    fn constraint_msg_id(_val: &u16) -> bool {
        //dbg!(_val);
        true
    }
    fn constraint_return_code(_val: &u8) -> bool {
        //dbg!(_val);
        true
    }*/

    #[inline(always)]
    pub fn rx(
        buf: &[u8],
        size: usize,
        client: &MqttSnClient,
    ) -> Result<(u16, u16, u8), String> {
        let (reg_ack, read_len) = RegAck::try_read(&buf, size).unwrap();
        dbg!(reg_ack.clone());

        if read_len == MSG_LEN_REGACK as usize {
            let _result = client.cancel_tx.send((
                client.remote_addr,
                reg_ack.msg_type,
                reg_ack.topic_id,
                reg_ack.msg_id,
            ));

            Ok((reg_ack.topic_id, reg_ack.msg_id, reg_ack.return_code))
        } else {
            //Err(ExoError::LenError(read_len, MSG_LEN_REGACK as usize))
            Err(format!(
                "{} {}: Length Error: {}.",
                function!(),
                read_len,
                MSG_LEN_REGACK
            ))
        }
    }

    #[inline(always)]
    pub fn tx(
        topic_id: u16,
        msg_id: u16,
        return_code: u8,
        client: &MqttSnClient,
    ) {
        let reg_ack = RegAck {
            len: MSG_LEN_REGACK,
            msg_type: MSG_TYPE_REGACK,
            topic_id,
            msg_id,
            return_code,
        };
        dbg!(reg_ack.clone());
        let mut bytes = BytesMut::with_capacity(MSG_LEN_REGACK as usize);
        reg_ack.try_write(&mut bytes);
        dbg!(bytes.clone());
        dbg!(client.remote_addr);
        let _result = client.transmit_tx.send((client.remote_addr, bytes));
    }
}
