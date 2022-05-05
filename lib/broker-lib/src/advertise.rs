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
    MSG_LEN_ADVERTISE,
    MSG_TYPE_ADVERTISE,
};
use bytes::{BufMut, BytesMut};
use custom_debug::Debug;
use getset::{CopyGetters, Getters, MutGetters};
use log::*;
use std::mem;

#[derive(
    Debug, Clone, Getters, /*Setters,*/ MutGetters, CopyGetters, Default,
)]
#[getset(get, set)]
pub struct Advertise {
    pub len: u8,
    #[debug(format = "0x{:x}")]
    pub msg_type: u8,
    pub gw_id: u8,
    pub duration: u16,
}
impl Advertise {
    pub fn send(
        gw_id: u8,
        duration: u16,
        client: &MqttSnClient,
    ) -> Result<(), String> {
        // faster implementation
        // TODO verify big-endian or little-endian for u16 numbers
        // XXX order of statements performance
        let duration_0 = duration as u8;
        let duration_1 = (duration >> 8) as u8;
        // message format
        // PUBACK:[len(0), msg_type(1), msg_id(2,3)]
        let mut bytes = BytesMut::with_capacity(MSG_LEN_ADVERTISE as usize);
        let buf: &[u8] = &[
            MSG_LEN_ADVERTISE,
            MSG_TYPE_ADVERTISE,
            gw_id,
            duration_0,
            duration_1,
        ];
        bytes.put(buf);
        // TODO replace BytesMut with Bytes to eliminate clone as copy
        dbg!(&buf);
        match client
            .transmit_tx
            .try_send((client.remote_addr, bytes.to_owned()))
        {
            Ok(()) => Ok(()),
            Err(err) => return Err(eformat!(client.remote_addr, err)),
        }
    }
    pub fn recv(
        buf: &[u8],
        size: usize,
        client: &MqttSnClient,
    ) -> Result<(), String> {
        let (advertise, read_fixed_len) =
            Advertise::try_read(buf, size).unwrap();
        info!(
            "{}: advertise {} with {} id",
            client.remote_addr, advertise.gw_id, advertise.duration
        );
        Ok(())
    }
}
