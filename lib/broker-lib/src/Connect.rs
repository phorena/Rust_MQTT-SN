use bytes::{BufMut, BytesMut};
use custom_debug::Debug;
use getset::{CopyGetters, Getters, MutGetters, Setters};
use std::mem;
use std::str;

use crate::{
    /*
    flags::{
        flag_is_clean_session,
        flag_is_will,
        flag_qos_level, flags_set,
        CleanSessionConst, DupConst, QoSConst, RetainConst, TopicIdTypeConst,
        WillConst, CLEAN_SESSION_FALSE, CLEAN_SESSION_TRUE, DUP_FALSE,
        DUP_TRUE, QOS_LEVEL_0, QOS_LEVEL_1, QOS_LEVEL_2, QOS_LEVEL_3,
        RETAIN_FALSE, RETAIN_TRUE, TOPIC_ID_TYPE_NORNAL,
        TOPIC_ID_TYPE_PRE_DEFINED, TOPIC_ID_TYPE_RESERVED, TOPIC_ID_TYPE_SHORT,
        WILL_FALSE, WILL_TRUE,
    },
        */
    message::MsgHeader,
    /*
    Errors::ExoError,
    MSG_LEN_PUBACK,
    MSG_LEN_PUBREC,

    MSG_TYPE_PUBACK,
    MSG_TYPE_PUBCOMP,
    MSG_TYPE_PUBLISH,
    MSG_TYPE_PUBREC,
    MSG_TYPE_PUBREL,
    MSG_TYPE_SUBACK,

    MSG_TYPE_SUBSCRIBE,
    RETURN_CODE_ACCEPTED,
    */
    BrokerLib::MqttSnClient,
    ConnAck::ConnAck,
    Connection::Connection,
    // flags::{flags_set, flag_qos_level, },
    MSG_TYPE_CONNACK,
    MSG_TYPE_CONNECT,
    RETURN_CODE_ACCEPTED,
};

/// Connect and Connect4 are for sending CONNECT messages with different header lengths.
#[derive(
    Debug, Clone, Getters, Setters, MutGetters, CopyGetters, Default, PartialEq,
)]
#[getset(get, set)]
pub struct Connect {
    pub len: u8,
    #[debug(format = "0x{:x}")]
    pub msg_type: u8,
    #[debug(format = "0b{:08b}")]
    pub flags: u8,
    pub protocol_id: u8,
    pub duration: u16,
    pub client_id: String,
}

#[derive(
    Debug, Clone, Getters, MutGetters, CopyGetters, Default, PartialEq,
)]
#[getset(get, set)]
/// Connect message type with 4 bytes header.
pub struct Connect4 {
    pub one: u8,
    pub len: u16,
    #[debug(format = "0x{:x}")]
    pub msg_type: u8,
    #[debug(format = "0b{:08b}")]
    pub flags: u8,
    pub protocol_id: u8,
    pub duration: u16,
    pub client_id: String,
}

/// Body is for parsing the body of a CONNECT message, length is parsed be the caller.
#[derive(
    Debug, Clone, Getters, MutGetters, CopyGetters, Default, PartialEq,
)]
#[getset(get, set)]
struct Body {
    #[debug(format = "0b{:08b}")]
    pub flags: u8,
    pub protocol_id: u8,
    pub duration: u16,
    pub client_id: String,
}

// TODO
impl Connect {
    pub fn constraint_len(_val: &u8) -> bool {
        //dbg!(_val);
        true
    }
    pub fn constraint_msg_type(_val: &u8) -> bool {
        //dbg!(_val);
        true
    }
    pub fn constraint_flags(_val: &u8) -> bool {
        //dbg!(_val);
        true
    }
    pub fn constraint_protocol_id(_val: &u8) -> bool {
        //dbg!(_val);
        true
    }
    pub fn constraint_duration(_val: &u16) -> bool {
        //dbg!(_val);
        true
    }
    pub fn constraint_client_id(_val: &String) -> bool {
        // dbg!(_val);
        true
    }

    // TODO error checking and return
    pub fn tx(
        client_id: String,
        duration: u16,
        client: &MqttSnClient,
    ) -> Result<(), String> {
        let len = client_id.len() + 6;
        // TODO check for 250 & 1400
        if len < 250 {
            let connect = Connect {
                len: len as u8,
                msg_type: MSG_TYPE_CONNECT,
                flags: 0b00000100,
                protocol_id: 1,
                duration,
                client_id,
            };
            let mut bytes_buf = BytesMut::with_capacity(len);
            // serialize the con_ack struct into byte(u8) array for the network.
            // serialize the con_ack struct into byte(u8) array for the network.
            dbg!(connect.clone());
            dbg!((bytes_buf.clone(), &connect));
            connect.try_write(&mut bytes_buf);
            dbg!(bytes_buf.clone());
            // transmit to network
            let _result = client
                .transmit_tx
                .send((client.remote_addr, bytes_buf.to_owned()));
            // schedule retransmit
            let _result = client.schedule_tx.send((
                client.remote_addr,
                MSG_TYPE_CONNACK,
                0,
                0,
                bytes_buf,
            ));
            return Ok(());
        } else if len < 1400 {
            let connect = Connect4 {
                one: 1,
                len: len as u16,
                msg_type: MSG_TYPE_CONNECT,
                flags: 0b00000100,
                protocol_id: 1,
                duration,
                client_id,
            };
            let mut bytes_buf = BytesMut::with_capacity(len);
            // serialize the con_ack struct into byte(u8) array for the network.
            // serialize the con_ack struct into byte(u8) array for the network.
            dbg!(connect.clone());
            dbg!((bytes_buf.clone(), &connect));
            connect.try_write(&mut bytes_buf);
            dbg!(bytes_buf.clone());
            // transmit to network
            let _result = client
                .transmit_tx
                .send((client.remote_addr, bytes_buf.to_owned()));
            // schedule retransmit
            let _result = client.schedule_tx.send((
                client.remote_addr,
                MSG_TYPE_CONNACK,
                0,
                0,
                bytes_buf,
            ));
            return Ok(());
        } else {
            return Err(String::from("client_id too long"));
        }
    }

    #[inline(always)]
    pub fn rx(
        buf: &[u8],
        size: usize,
        client: &mut MqttSnClient,
        header: MsgHeader,
    ) -> Result<(), String> {
        let body: Body;
        let _read_fixed_len;
        if header.header_len == 2 {
            // TODO replace unwrap
            (body, _read_fixed_len) = Body::try_read(&buf[2..], size).unwrap();
        } else {
            (body, _read_fixed_len) = Body::try_read(&buf[4..], size).unwrap();
        }
        dbg!(body.clone());
        Connection::try_insert(client.remote_addr, body.flags, body.duration)?;
        ConnAck::tx(client, RETURN_CODE_ACCEPTED);
        Ok(())
    }
}
