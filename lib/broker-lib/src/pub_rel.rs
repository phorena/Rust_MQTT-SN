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
    eformat, function, pub_comp::PubComp, pub_msg_cache::PubMsgCache,
    publish::Publish,
    /*
    flags::{flags_set, flag_qos_level, },
    StateMachine,
    MSG_LEN_PUBACK,

    MSG_TYPE_CONNACK,
    MSG_TYPE_CONNECT,
    MSG_TYPE_PUBACK,
    MSG_TYPE_PUBCOMP,
    MSG_TYPE_PUBLISH,
    MSG_TYPE_SUBACK,

    MSG_TYPE_SUBSCRIBE,
    RETURN_CODE_ACCEPTED,
    */
    /*
    flags::{
        flag_qos_level, flags_set, CleanSessionConst, DupConst, QoSConst,
        RetainConst, TopicIdTypeConst, WillConst, CLEAN_SESSION_FALSE,
        CLEAN_SESSION_TRUE, DUP_FALSE, DUP_TRUE, QOS_LEVEL_0, QOS_LEVEL_1,
        QOS_LEVEL_2, QOS_LEVEL_3, RETAIN_FALSE, RETAIN_TRUE,
        TOPIC_ID_TYPE_NORMAL, TOPIC_ID_TYPE_PRE_DEFINED,
        TOPIC_ID_TYPE_RESERVED, TOPIC_ID_TYPE_SHORT, WILL_FALSE, WILL_TRUE,
    },
    */
    BrokerLib::MqttSnClient, MSG_LEN_PUBREL, MSG_TYPE_PUBREL,
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
    ) -> Result<(), String> {
        if buf[0] == MSG_LEN_PUBREL && buf[1] == MSG_TYPE_PUBREL {
            // TODO verify as Big Endian
            let msg_id = buf[3] as u16 + ((buf[2] as u16) << 8);
            // Send PUBCOMP to publisher
            PubComp::send(msg_id, client)?;
            // Send publish message to subscribers.
            match PubMsgCache::remove((client.remote_addr, msg_id)) {
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
            // Cancel the timer for PUBREL
            match client.cancel_tx.send((
                client.remote_addr,
                MSG_TYPE_PUBREL,
                0,
                msg_id,
            )) {
                Ok(_) => Ok(()),
                Err(err) => Err(eformat!(client.remote_addr, err)),
            }
        } else {
            return Err(eformat!(client.remote_addr, "Length", buf[0]));
        }
    }
    #[inline(always)]
    pub fn send(msg_id: u16, client: &MqttSnClient) -> Result<(), String> {
        // faster implementation
        // TODO verify big-endian or little-endian for u16 numbers
        // XXX order of statements performance
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
        match client.transmit_tx.send((client.remote_addr, bytes)) {
            Ok(_) => Ok(()),
            Err(e) => Err(format!(
                "{}-{}:",
                //function!(),
                client.remote_addr,
                e
            )),
        }
    }
}
