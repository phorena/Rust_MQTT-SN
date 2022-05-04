/*
5.4.1 ADVERTISE
Length    MsgType GwId Duration
(octet 0) (1)     (2)  (3,4)
Table 6: ADVERTISE Message
The ADVERTISE message is broadcasted periodically by a gateway to advertise its presence. The time
interval until the next broadcast time is indicated in the Duration field of this message. Its format is illustrated in
Table 6:
• Length and MsgType: see Section 5.2.
• GwId: the id of the gateway which sends this message.
• Duration: time interval until the next ADVERTISE is broadcasted by this gateway
*/
use crate::{
    eformat,
    function,
    BrokerLib::MqttSnClient,
    MSG_LEN_SEARCH_GW,
    MSG_TYPE_SEARCH_GW,
};
use bytes::{BufMut, BytesMut};
use custom_debug::Debug;
use getset::{CopyGetters, Getters, MutGetters};
use log::*;
use std::str;

#[derive(
    Debug, Clone, Getters, /*Setters,*/ MutGetters, CopyGetters, Default,
)]
#[getset(get, set)]
pub struct SearchGw {
    pub len: u8,
    #[debug(format = "0x{:x}")]
    pub msg_type: u8,
    pub radius: u8,
}
impl SearchGw {
    pub fn send(
        radius: u8,
        client: &MqttSnClient,
    ) -> Result<(), String> {
        let mut bytes = BytesMut::with_capacity(MSG_LEN_SEARCH_GW as usize);
        let buf: &[u8] = &[
            MSG_LEN_SEARCH_GW,
            MSG_TYPE_SEARCH_GW,
            radius
        ];
        bytes.put(buf);
        // TODO replace BytesMut with Bytes to eliminate clone as copy
        dbg!(&buf);
        match client
            .transmit_tx
            .try_send((client.remote_addr, bytes.to_owned()))
        {
            Ok(()) => return Ok(()),
            Err(err) => return Err(eformat!(client.remote_addr, err)),
        }
    }
    pub fn recv(
        buf: &[u8],
        size: usize,
        client: &MqttSnClient,
    ) -> Result<(), String> {
        let (search_gw, _read_fixed_len) =
            SearchGw::try_read(buf, size).unwrap();
        info!(
            "{}: search gw {} ",
            client.remote_addr, search_gw.radius
        );
        Ok(())
    }
}
