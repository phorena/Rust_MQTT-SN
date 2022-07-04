/*
5.4.14 PUBREC, PUBREL, and PUBCOMP
Length MsgType MsgId
(octet 0) (1) (2-3)
Table 18: PUBREC, PUBREL, and PUBCOMP Messages
As with MQTT, the PUBREC, PUBREL, and PUBCOMP messages are used in conjunction with a PUBLISH
message with QoS level 2. Their format is illustrated in Table 18:
• Length and MsgType: see Section 5.2.
• MsgId: same value as the one contained in the corresponding PUBLISH message.
*/
use bytes::{BufMut, BytesMut};
use custom_debug::Debug;
use getset::{CopyGetters, Getters, MutGetters};
use std::mem;

use crate::{
    broker_lib::MqttSnClient,
    eformat,
    function,
    msg_hdr::MsgHeader,
    retransmit::RetransTimeWheel,
    // flags::{flags_set, flag_qos_level, },
    MSG_LEN_PUBCOMP,

    MSG_TYPE_PUBCOMP,
};
#[derive(Debug, Clone, Getters, MutGetters, CopyGetters, Default)]
#[getset(get, set)]
pub struct PubComp {
    pub len: u8,
    #[debug(format = "0x{:x}")]
    pub msg_type: u8,
    pub msg_id: u16,
}

impl PubComp {
    /*
    fn constraint_len(_val: &u8) -> bool {
        //dbg!(_val);
        true
    }
    fn constraint_msg_type(_val: &u8) -> bool {
        //dbg!(_val);
        true
    }
    fn constraint_msg_id(_val: &u16) -> bool {
        //dbg!(_val);
        true
    }
    */
    #[inline(always)]
    pub fn send(
        msg_id: u16,
        client: &MqttSnClient,
        msg_header: MsgHeader,
    ) -> Result<(), String> {
        // faster implementation
        // TODO verify big-endian or little-endian for u16 numbers
        // XXX order of statements performance
        let msg_id_byte_0 = msg_id as u8;
        let msg_id_byte_1 = (msg_id >> 8) as u8;
        // message format
        // PUBACK:[len(0), msg_type(1), msg_id(2,3)]
        let mut bytes = BytesMut::with_capacity(MSG_LEN_PUBCOMP as usize);
        let remote_socket_addr = msg_header.remote_socket_addr;
        let buf: &[u8] = &[
            MSG_LEN_PUBCOMP,
            MSG_TYPE_PUBCOMP,
            msg_id_byte_1,
            msg_id_byte_0,
        ];
        bytes.put(buf);
        match client.egress_tx.try_send((remote_socket_addr, bytes)) {
            Ok(()) => Ok(()),
            Err(err) => Err(eformat!(remote_socket_addr, err)),
        }
    }
    #[inline(always)]
    pub fn recv(
        buf: &[u8],
        size: usize,
        client: &MqttSnClient,
        msg_header: MsgHeader,
    ) -> Result<u16, String> {
        let remote_socket_addr = msg_header.remote_socket_addr;
        if buf[0] == MSG_LEN_PUBCOMP
            && buf[1] == MSG_TYPE_PUBCOMP
            && size == MSG_LEN_PUBCOMP as usize
        {
            // TODO verify as Big Endian
            let msg_id = buf[3] as u16 + ((buf[2] as u16) << 8);
            RetransTimeWheel::cancel_timer(
                remote_socket_addr,
                MSG_TYPE_PUBCOMP,
                0,
                msg_id,
            )?;
            Ok(msg_id)
        } else {
            Err(eformat!(remote_socket_addr, "size", buf[0]))
        }
    }
}
