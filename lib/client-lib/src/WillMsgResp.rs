use crate::{
    ClientLib::MqttSnClient,
    Errors::ExoError,
    // flags::{flags_set, flag_qos_level, },
    MSG_LEN_WILLMSGRESP,

    MSG_TYPE_WILLMSGRESP,
};
use bytes::{BufMut, BytesMut};
use custom_debug::Debug;
use getset::{CopyGetters, Getters, MutGetters, Setters};

#[derive(
    Debug, Clone, Copy, Getters, Setters, MutGetters, CopyGetters, Default,
)]
#[getset(get, set)]
pub struct WillMsgResp {
    pub len: u8,
    #[debug(format = "0x{:x}")]
    pub msg_type: u8,
    pub return_code: u8,
}

impl WillMsgResp {
    fn constraint_len(_val: &u8) -> bool {
        //dbg!(_val);
        true
    }
    fn constraint_msg_type(_val: &u8) -> bool {
        //dbg!(_val);
        true
    }
    fn constraint_return_code(_val: &u8) -> bool {
        //dbg!(_val);
        true
    }
    pub fn rx(
        buf: &[u8],
        size: usize,
        client: &MqttSnClient,
    ) -> Result<(), ExoError> {
        let (willmsgresp, read_len) =
            WillMsgResp::try_read(&buf, size).unwrap();
        dbg!(willmsgresp.clone());
        if read_len == MSG_LEN_WILLMSGRESP as usize {
            client.cancel_tx.send((
                client.remote_addr,
                willmsgresp.msg_type,
                0,
                0,
            ));
            Ok(())
        } else {
            Err(ExoError::LenError(read_len, MSG_LEN_WILLMSGRESP as usize))
        }
    }
    // TODO error checking and return
    pub fn tx(client: &MqttSnClient, return_code: u8) {
        let willmsgresp = WillMsgResp {
            len: MSG_LEN_WILLMSGRESP,
            msg_type: MSG_TYPE_WILLMSGRESP,
            return_code,
        };
        let mut bytes_buf =
            BytesMut::with_capacity(MSG_LEN_WILLMSGRESP as usize);
        dbg!(willmsgresp.clone());
        willmsgresp.try_write(&mut bytes_buf);
        dbg!(bytes_buf.clone());
        dbg!(client.remote_addr);
        // transmit to network
        client
            .transmit_tx
            .send((client.remote_addr, bytes_buf.to_owned()));
    }
}
