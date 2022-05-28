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
    broker_lib::MqttSnClient, multicast, MSG_LEN_ADVERTISE, MSG_TYPE_ADVERTISE,
};
use bytes::{BufMut, BytesMut};
use custom_debug::Debug;
use getset::{CopyGetters, Getters, MutGetters};
use log::*;
use std::mem;
use std::net::{SocketAddr, UdpSocket};

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
    pub fn run(socket_addr: SocketAddr, gw_id: u8, duration: u16) {
        let duration_0 = (duration >> 8) as u8;
        let duration_1 = duration as u8;
        let mut bytes = BytesMut::with_capacity(MSG_LEN_ADVERTISE as usize);
        let buf: &[u8] = &[
            MSG_LEN_ADVERTISE,
            MSG_TYPE_ADVERTISE,
            gw_id,
            duration_0,
            duration_1,
        ];
        bytes.put(buf);
        dbg!(&buf);
        multicast::broadcast_loop(bytes.freeze(), socket_addr, duration);
    }
    pub fn recv(
        buf: &[u8],
        size: usize,
        client: &MqttSnClient,
    ) -> Result<(), String> {
        let (advertise, _read_fixed_len) =
            Advertise::try_read(buf, size).unwrap();
        info!(
            "{}: advertise {} with {} id",
            client.remote_addr, advertise.gw_id, advertise.duration
        );
        Ok(())
    }
}
