/*
5.4.5 CONNACK
Length    MsgType ReturnCode
(octet 0) (1)     (2)
Table 10: CONNACK Message
The CONNACK message is sent by the server in response to a connection request from a client. Its format is
shown in Table 10:
• Length and MsgType: see Section 5.2.
• ReturnCode: encoded according to Table 5
*/
use bytes::{BufMut, BytesMut};
use custom_debug::Debug;
use getset::{CopyGetters, Getters, MutGetters /* Setters */};

use crate::{
    broker_lib::MqttSnClient,
    eformat,
    function,
    retransmit::RetransTimeWheel,
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
    Debug,
    Clone,
    Copy,
    Getters,
    // Setters,
    MutGetters,
    CopyGetters,
    Default,
    PartialEq,
)]
#[getset(get, set)]
/// ConnAck message type has 3 bytes, doesn't need MsgHeader and Body.
pub struct ConnAck {
    pub len: u8,
    #[debug(format = "0x{:x}")]
    pub msg_type: u8,
    pub return_code: u8, // use enum for print
}

impl ConnAck {
    /*
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
    */
    #[inline(always)]
    pub fn recv(
        buf: &[u8],
        size: usize,
        client: &MqttSnClient,
    ) -> Result<(), String> {
        let (conn_ack, read_len) = ConnAck::try_read(&buf, size).unwrap();
        dbg!(conn_ack.clone());
        if read_len == MSG_LEN_CONNACK as usize {
            RetransTimeWheel::cancel_timer(
                client.remote_addr,
                conn_ack.msg_type,
                0,
                0,
            )?;
            dbg!("connack cancel timer");
            Ok(())
        } else {
            Err(eformat!("len err", read_len))
        }
    }

    #[inline(always)]
    pub fn send(client: &MqttSnClient, return_code: u8) -> Result<(), String> {
        let connack = ConnAck {
            len: MSG_LEN_CONNACK,
            msg_type: MSG_TYPE_CONNACK,
            return_code,
        };
        let mut bytes_buf = BytesMut::with_capacity(MSG_LEN_CONNACK as usize);
        dbg!(connack.clone());
        connack.try_write(&mut bytes_buf);
        dbg!(bytes_buf.clone());
        // transmit to network
        match client
            .transmit_tx
            .try_send((client.remote_addr, bytes_buf.to_owned()))
        {
            Ok(()) => Ok(()),
            Err(err) => Err(eformat!(client.remote_addr, err)),
        }
    }
}
