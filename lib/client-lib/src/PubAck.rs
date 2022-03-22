use bytes::{BufMut, BytesMut};
use crossbeam::channel::{bounded, unbounded, Receiver, Sender};
use custom_debug::Debug;
use getset::{CopyGetters, Getters, MutGetters, Setters};
use std::mem;
use std::str;
use std::{io, net::SocketAddr, net::SocketAddrV4, sync::Arc, sync::Mutex};

use crate::{
    flags::{
        flag_qos_level, flags_set, CleanSessionConst, DupConst, QoSConst,
        RetainConst, TopicIdTypeConst, WillConst, CLEAN_SESSION_FALSE,
        CLEAN_SESSION_TRUE, DUP_FALSE, DUP_TRUE, QOS_LEVEL_0, QOS_LEVEL_1,
        QOS_LEVEL_2, QOS_LEVEL_3, RETAIN_FALSE, RETAIN_TRUE,
        TOPIC_ID_TYPE_NORNAL, TOPIC_ID_TYPE_PRE_DEFINED,
        TOPIC_ID_TYPE_RESERVED, TOPIC_ID_TYPE_SHORT, WILL_FALSE, WILL_TRUE,
    },
    ClientLib::MqttSnClient,
    Errors::ExoError,
    // flags::{flags_set, flag_qos_level, },
    StateMachine,
    MSG_LEN_PUBACK,
    MSG_LEN_PUBREC,

    MSG_TYPE_CONNACK,
    MSG_TYPE_CONNECT,
    MSG_TYPE_PUBACK,
    MSG_TYPE_PUBCOMP,
    MSG_TYPE_PUBLISH,
    MSG_TYPE_PUBREC,
    MSG_TYPE_PUBREL,
    MSG_TYPE_SUBACK,

    MSG_TYPE_SUBSCRIBE,
    RETURN_CODE_ACCEPTED,
};
#[derive(Debug, Clone, Getters, Setters, MutGetters, CopyGetters, Default)]
#[getset(get, set)]
pub struct PubAck {
    pub len: u8,
    #[debug(format = "0x{:x}")]
    pub msg_type: u8,
    pub topic_id: u16,
    pub msg_id: u16,
    pub return_code: u8,
}

impl PubAck {
    fn constraint_len(_val: &u8) -> bool {
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
    }

#[inline(always)]
pub fn rx(
    buf: &[u8],
    size: usize,
    client: &MqttSnClient,
) -> Result<(u16, u16, u8), ExoError> {
    let (pub_ack, read_len) = PubAck::try_read(&buf, size).unwrap();
    dbg!(pub_ack.clone());
    if read_len == MSG_LEN_PUBACK as usize {
        client.cancel_tx.send((
            client.remote_addr,
            pub_ack.msg_type,
            pub_ack.topic_id,
            pub_ack.msg_id,
        ));
        // TODO process return code?
        Ok((pub_ack.topic_id, pub_ack.msg_id, pub_ack.return_code))
    } else {
        Err(ExoError::LenError(read_len, MSG_LEN_PUBACK as usize))
    }
}

}
// NOTE: puback_tx is inlined hard coded for performance.
