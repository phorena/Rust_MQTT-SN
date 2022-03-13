use bytes::{BufMut, BytesMut};
use custom_debug::Debug;
use getset::{CopyGetters, Getters, MutGetters, Setters};
use std::mem;
use std::{io, net::SocketAddr, net::SocketAddrV4, sync::Arc, sync::Mutex};
use crossbeam::channel::{bounded, unbounded, Receiver, Sender};
use crate::{
    MSG_TYPE_CONNECT,
    MSG_TYPE_CONNACK,
    MSG_TYPE_PUBLISH,
    MSG_TYPE_PUBACK,
    MSG_TYPE_PUBREC,
    MSG_TYPE_PUBREL,
    MSG_TYPE_PUBCOMP,
    MSG_TYPE_SUBSCRIBE,
    MSG_TYPE_SUBACK,

    MSG_LEN_SUBACK,
    MSG_LEN_PUBREC,

    RETURN_CODE_ACCEPTED,

    flags:: {
        DupConst,
        DUP_FALSE,
        DUP_TRUE,

        QoSConst,
        QOS_LEVEL_0,
        QOS_LEVEL_1,
        QOS_LEVEL_2,
        QOS_LEVEL_3,

        RetainConst,
        RETAIN_FALSE,
        RETAIN_TRUE,

        WillConst,
        WILL_FALSE,
        WILL_TRUE,

        CleanSessionConst,
        CLEAN_SESSION_FALSE,
        CLEAN_SESSION_TRUE,

        TopicIdTypeConst,
        TOPIC_ID_TYPE_NORNAL,
        TOPIC_ID_TYPE_PRE_DEFINED,
        TOPIC_ID_TYPE_SHORT,
        TOPIC_ID_TYPE_RESERVED,

    flags_set, flag_qos_level,
    },
    // flags::{flags_set, flag_qos_level, },
    StateMachine,
    Errors::ExoError,
    ClientLib:: {MqttSnClient,},
};

#[derive(Debug, Clone, Getters, Setters, MutGetters, CopyGetters, Default)]
#[getset(get, set)]
pub struct SubAck {
    pub len: u8,
    #[debug(format = "0x{:x}")]
    pub msg_type: u8,
    #[debug(format = "0b{:08b}")]
    pub flags: u8,
    pub topic_id: u16,
    pub msg_id: u16,
    pub return_code: u8,
}

impl SubAck {
    fn constraint_len(_val: &u8) -> bool {
        //dbg!(_val);
        true
    }
    fn constraint_msg_type(_val: &u8) -> bool {
        //dbg!(_val);
        true
    }
    fn constraint_flags(_val: &u8) -> bool {
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
}

pub fn suback_rx(
    buf: &[u8],
    size: usize,
    client: &MqttSnClient,
    ) -> Result<u16, ExoError> {
    let (sub_ack, read_len) = SubAck::try_read(&buf, size).unwrap();
    dbg!(sub_ack.clone());

    if read_len == MSG_LEN_SUBACK as usize {
        // XXX Cancel the retransmision scheduled.
        //     No topic_id passing to send for now.
        //     because the subscribe message might not contain it.
        //     The retransmision was scheduled with 0.
        client.cancel_tx.send((client.remote_addr, sub_ack.msg_type,
                        0, sub_ack.msg_id));
        // TODO check QoS in flags
        // TODO check flags
        Ok(sub_ack.topic_id)
    } else {
        Err(ExoError::LenError(read_len, MSG_LEN_SUBACK as usize))
    }
}
