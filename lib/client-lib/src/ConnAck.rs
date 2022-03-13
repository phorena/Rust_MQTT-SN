use bytes::{BufMut, BytesMut};
use custom_debug::Debug;
use getset::{CopyGetters, Getters, MutGetters, Setters};

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
    MSG_LEN_CONNACK,

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

#[derive(Debug, thiserror::Error)]
pub enum ConnAckError {
    #[error("ConnAck Rejection: {0}")]
    ConnAckRejection(u8),
    #[error("ConnAck Unknown Code: {0}")]
    ConnAckUnknownCode(u8),
    #[error("ConnAck Wrong Message Type: {0}")]
    ConnAckWrongMessageType(u8),
}

#[derive(Debug, Clone, Copy, Getters, Setters,
    MutGetters, CopyGetters, Default,)]
#[getset(get, set)]
pub struct ConnAck {
    pub len: u8,
    #[debug(format = "0x{:x}")]
    pub msg_type: u8,
    pub return_code: u8, // use enum for print
}

impl ConnAck {
    fn constraint_len(_val: &u8) -> bool {
        //dbg!(_val);
        true
    }
    fn constraint_msg_type(val: &u8) -> Result<(), ConnAckError> {
        match *val {
            0x00 => {
                // XXX Ok(())
                Err(ConnAckError::ConnAckRejection(*val))
            }
            0x01..=0x03 => Err(ConnAckError::ConnAckRejection(*val)),
            _ => Err(ConnAckError::ConnAckUnknownCode(*val)),
        }
    }
    fn constraint_return_code(_val: &u8) -> bool {
        //dbg!(_val);
        true
    }
}

pub fn connack_rx(
    buf: &[u8],
    size: usize,
    client: &MqttSnClient,
) -> Result<(), ExoError> {
    let (conn_ack, read_len) = ConnAck::try_read(&buf, size).unwrap();
    dbg!(conn_ack.clone());
    if read_len == MSG_LEN_CONNACK as usize {
        client.cancel_tx.send((client.remote_addr,
                                 conn_ack.msg_type, 0, 0));
        Ok(())
    } else {
        Err(ExoError::LenError(read_len, MSG_LEN_SUBACK as usize))
    }
}
