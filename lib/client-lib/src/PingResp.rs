use crate::{
    ClientLib::MqttSnClient, Errors::ExoError, MSG_LEN_PINGRESP,
    MSG_TYPE_PINGRESP,
};
use bytes::{BufMut, BytesMut};
use custom_debug::Debug;
use getset::{CopyGetters, Getters, MutGetters, Setters};

#[derive(
    Debug, Clone, Copy, Getters, Setters, MutGetters, CopyGetters, Default,
)]
#[getset(get, set)]
pub struct PingResp {
    pub len: u8,
    #[debug(format = "0x{:x}")]
    pub msg_type: u8,
}

impl PingResp {
    fn constraint_len(_val: &u8) -> bool {
        //dbg!(_val);
        true
    }
    fn constraint_msg_type(_val: &u8) -> bool {
        //dbg!(_val);
        true
    }
}
pub fn tx(client: &MqttSnClient) {
    let pingresp = PingResp {
        len: MSG_LEN_PINGRESP,
        msg_type: MSG_TYPE_PINGRESP,
    };
    let mut bytes_buf = BytesMut::with_capacity(MSG_LEN_PINGRESP as usize);
    dbg!(pingresp.clone());
    pingresp.try_write(&mut bytes_buf);
    dbg!(bytes_buf.clone());
    dbg!(client.remote_addr);
    // transmit to network
    client
        .transmit_tx
        .send((client.remote_addr, bytes_buf.to_owned()));
}
pub fn rx(
    buf: &[u8],
    size: usize,
    client: &MqttSnClient,
) -> Result<(), ExoError> {
    let (ping_resp, read_len) = PingResp::try_read(&buf, size).unwrap();
    dbg!(ping_resp.clone());
    if read_len == MSG_LEN_PINGRESP as usize {
        client
            .cancel_tx
            .send((client.remote_addr, ping_resp.msg_type, 0, 0));
        Ok(())
    } else {
        Err(ExoError::LenError(read_len, MSG_LEN_PINGRESP as usize))
    }
}
