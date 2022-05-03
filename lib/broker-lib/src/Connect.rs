use bytes::{BufMut, BytesMut, Bytes};
use custom_debug::Debug;
use getset::{CopyGetters, Getters, MutGetters, Setters};
use std::mem;
use std::str;

use crate::{
    eformat,
    function,
    message::MsgHeader,
    BrokerLib::MqttSnClient,
    ConnAck::ConnAck,
    Connection::Connection,
    MSG_LEN_CONNECT_HEADER,
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
struct Connect4 {
    // NOTE: no pub
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
    pub client_id: Bytes,
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
        let len = client_id.len() + MSG_LEN_CONNECT_HEADER as usize;
        if len < 256 {
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
            if let Err(err) = client
                .transmit_tx
                .try_send((client.remote_addr, bytes_buf.to_owned()))
            {
                return Err(eformat!(client.remote_addr, err));
            }
            // schedule retransmit
            match client.schedule_tx.try_send((
                client.remote_addr,
                MSG_TYPE_CONNACK,
                0,
                0,
                bytes_buf,
            )) {
                Ok(_) => Ok(()),
                Err(err) => Err(eformat!(client.remote_addr, err)),
            }
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
            if let Err(err) = client
                .transmit_tx
                .try_send((client.remote_addr, bytes_buf.to_owned()))
            {
                return Err(eformat!(client.remote_addr, err));
            }
            // schedule retransmit
            match client.schedule_tx.try_send((
                client.remote_addr,
                MSG_TYPE_CONNACK,
                0,
                0,
                bytes_buf,
            )) {
                Ok(_) => Ok(()),
                Err(err) => Err(eformat!(client.remote_addr, err)),
            }
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
            dbg!(&body);
        } else {
            (body, _read_fixed_len) = Body::try_read(&buf[4..], size).unwrap();
        }
        dbg!(body.clone());
        Connection::try_insert(client.remote_addr, body.flags, body.protocol_id, body.duration, body.client_id)?;
        ConnAck::tx(client, RETURN_CODE_ACCEPTED)?;
        Ok(())
    }
}
