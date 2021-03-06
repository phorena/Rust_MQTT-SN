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
    broker_lib::MqttSnClient, eformat, function, msg_hdr::MsgHeader,
    pub_comp::PubComp, pub_msg_cache::PubMsgCache, publish::Publish,
    retransmit::RetransTimeWheel, MSG_LEN_PUBREL, MSG_TYPE_PUBREL,
};

#[derive(
    Debug, Clone, Getters, MutGetters, CopyGetters, Default, PartialEq,
)]
#[getset(get, set)]
pub struct PubRel {
    pub len: u8,
    #[debug(format = "0x{:x}")]
    pub msg_type: u8,
    pub msg_id: u16,
}

impl PubRel {
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
    pub fn recv(
        buf: &[u8],
        _size: usize,
        client: &MqttSnClient,
        msg_header: MsgHeader,
    ) -> Result<(), String> {
        let remote_socket_addr = msg_header.remote_socket_addr;
        if buf[0] == MSG_LEN_PUBREL && buf[1] == MSG_TYPE_PUBREL {
            // TODO verify as Big Endian
            let msg_id = buf[3] as u16 + ((buf[2] as u16) << 8);
            // Send PUBCOMP to publisher
            PubComp::send(msg_id, client, msg_header)?;
            // Send publish message to subscribers.
            match PubMsgCache::remove((remote_socket_addr, msg_id)) {
                Some(pub_msg_cache) => {
                    dbg!(&pub_msg_cache);
                    Publish::send_msg_to_subscribers(
                        pub_msg_cache.subscriber_vec,
                        pub_msg_cache.publish,
                        client,
                    )?;
                }
                None => {
                    // TODO return error or no subscribers?
                    {}
                }
            }
            match RetransTimeWheel::cancel_timer(
                remote_socket_addr,
                MSG_TYPE_PUBREL,
                0,
                msg_id,
            ) {
                Ok(()) => Ok(()),
                Err(err) => Err(err),
            }
        } else {
            return Err(eformat!(remote_socket_addr, "Length", buf[0]));
        }
    }
    #[inline(always)]
    pub fn send(
        msg_id: u16,
        client: &MqttSnClient,
        msg_header: MsgHeader,
    ) -> Result<(), String> {
        // faster implementation
        // TODO verify big-endian or little-endian for u16 numbers
        // XXX order of statements performance
        let remote_socket_addr = msg_header.remote_socket_addr;
        let msg_id_byte_0 = msg_id as u8;
        let msg_id_byte_1 = (msg_id >> 8) as u8;
        // message format
        // PUBACK:[len(0), msg_type(1), msg_id(2,3)]
        let mut bytes = BytesMut::with_capacity(MSG_LEN_PUBREL as usize);
        let buf: &[u8] = &[
            MSG_LEN_PUBREL,
            MSG_TYPE_PUBREL,
            msg_id_byte_1,
            msg_id_byte_0,
        ];
        bytes.put(buf);
        match client.egress_tx.send((remote_socket_addr, bytes)) {
            Ok(_) => Ok(()),
            Err(e) => Err(format!(
                "{}-{}:",
                //function!(),
                remote_socket_addr,
                e
            )),
        }
    }
}
