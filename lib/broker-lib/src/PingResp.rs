use crate::{
    eformat, function, BrokerLib::MqttSnClient, MSG_LEN_PINGRESP,
    MSG_TYPE_PINGRESP,
};
use bytes::{BufMut, BytesMut};
use custom_debug::Debug;
use getset::{CopyGetters, Getters, MutGetters};

#[derive(
    Debug,
    Clone,
    Copy,
    Getters,
    //   Setters,
    MutGetters,
    CopyGetters,
    Default,
    PartialEq,
)]
#[getset(get, set)]
pub struct PingResp {
    pub len: u8,
    #[debug(format = "0x{:x}")]
    pub msg_type: u8,
}

impl PingResp {
    pub fn rx(
        buf: &[u8],
        size: usize,
        client: &mut MqttSnClient,
    ) -> Result<(), String> {
        if size == MSG_LEN_PINGRESP as usize && buf[0] == MSG_LEN_PINGRESP {
            // TODO update ping timer.
            return Ok(());
        } else {
            return Err(eformat!(client.remote_addr, "len err", size));
        }
    }
    pub fn tx(client: &mut MqttSnClient) -> Result<(), String> {
        let buf: &[u8] = &[MSG_LEN_PINGRESP, MSG_TYPE_PINGRESP];
        let bytes = BytesMut::from(buf);
        match client
            .transmit_tx
            .try_send((client.remote_addr, bytes.to_owned()))
        {
            Ok(()) => return Ok(()),
            Err(err) => return Err(eformat!(client.remote_addr, err)),
        }
    }
}
