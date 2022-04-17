use bytes::{BufMut, BytesMut};
use custom_debug::Debug;
use getset::{CopyGetters, Getters, MutGetters, Setters};

use crate::{
    ClientLib::MqttSnClient,
    Errors::ExoError,
    // flags::{flags_set, flag_qos_level, },
    MSG_LEN_CONNACK,

    MSG_TYPE_CONNACK,
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

#[derive(
    Debug, Clone, Copy, Getters, Setters, MutGetters, CopyGetters, Default,
)]
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
    pub fn rx(
        buf: &[u8],
        size: usize,
        client: &MqttSnClient,
    ) -> Result<(), ExoError> {
        let (conn_ack, read_len) = ConnAck::try_read(&buf, size).unwrap();
        dbg!(conn_ack.clone());
        if read_len == MSG_LEN_CONNACK as usize {
            client.cancel_tx.send((
                client.remote_addr,
                conn_ack.msg_type,
                0,
                0,
            ));
            Ok(())
        } else {
            Err(ExoError::LenError(read_len, MSG_LEN_CONNACK as usize))
        }
    }

    // TODO error checking and return
    pub fn tx(client: &MqttSnClient, return_code: u8) {
        let connack = ConnAck {
            len: MSG_LEN_CONNACK,
            msg_type: MSG_TYPE_CONNACK,
            return_code,
        };
        let mut bytes_buf = BytesMut::with_capacity(MSG_LEN_CONNACK as usize);
        dbg!(connack.clone());
        connack.try_write(&mut bytes_buf);
        dbg!(bytes_buf.clone());
        dbg!(client.remote_addr);
        // transmit to network
        client
            .transmit_tx
            .send((client.remote_addr, bytes_buf.to_owned()));
    }
}
