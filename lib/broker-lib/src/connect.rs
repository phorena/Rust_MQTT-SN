/*
5.4.4 CONNECT
Length MsgType Flags ProtocolId Duration ClientId
(octet 0) (1) (2) (3) (4,5) (6:n)
Table 9: CONNECT Message
The CONNECT message is sent by a client to setup a connection. Its format is shown in Table 9:
• Length and MsgType: see Section 5.2.
• Flags:
– DUP, QoS, Retain, TopicIdType: not used.
– Will: if set, indicates that client is requesting for Will topic and Will message prompting;
– CleanSession: same meaning as with MQTT, however extended for Will topic and Will message (see
Section 6.3).
• ProtocolId: corresponds to the “Protocol Name” and “Protocol Version” of the MQTT CONNECT message.
• Duration: same as with MQTT, contains the value of the Keep Alive timer.
• ClientId: same as with MQTT, contains the client id which is a 1-23 character long string which uniquely
identifies the client to the server.
5.4.5 CONNACK
*/
use bytes::{BufMut, Bytes, BytesMut};
use custom_debug::Debug;
use getset::{CopyGetters, Getters, MutGetters};
use std::mem;
use std::str;

use crate::{
    conn_ack::ConnAck,
    connection::Connection,
    eformat,
    function,
    message::{MsgHeader, MsgHeaderEnum},
    BrokerLib::MqttSnClient,
    MSG_LEN_CONNECT_HEADER,
    // flags::{flags_set, flag_qos_level, },
    MSG_TYPE_CONNACK,
    MSG_TYPE_CONNECT,
    RETURN_CODE_ACCEPTED,
};

/// Connect and Connect4 are for sending CONNECT messages with different header lengths.
#[derive(
    Debug, Clone, Getters, MutGetters, CopyGetters, Default, PartialEq,
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
    pub client_id: Bytes,
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
    pub client_id: Bytes,
}

impl Connect {
    #[inline(always)]
    pub fn send(
        flags: u8,
        protocol_id: u8,
        duration: u16,
        client_id: Bytes,
        client: &MqttSnClient,
    ) -> Result<(), String> {
        let len = client_id.len() + MSG_LEN_CONNECT_HEADER as usize;
        if len < 256 {
            let connect = Connect {
                len: len as u8,
                msg_type: MSG_TYPE_CONNECT,
                flags,
                protocol_id,
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
        // TODO check size 1400
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
            Err(eformat!(client.remote_addr, "client_id too long"))
        }
    }

    #[inline(always)]
    pub fn recv(
        buf: &[u8],
        size: usize,
        client: &mut MqttSnClient,
        header: MsgHeader,
    ) -> Result<(), String> {
        match header.header_len {
            MsgHeaderEnum::Short => {
                // TODO check size vs len
                let (connect, _read_fixed_len) =
                    Connect::try_read(buf, size).unwrap();
                dbg!(&connect);
                Connection::try_insert(
                    client.remote_addr,
                    connect.flags,
                    connect.protocol_id,
                    connect.duration,
                    connect.client_id,
                )?;
            }
            MsgHeaderEnum::Long => {
                let (connect, _read_fixed_len) =
                    Connect4::try_read(buf, size).unwrap();
                dbg!(&connect);
                Connection::try_insert(
                    client.remote_addr,
                    connect.flags,
                    connect.protocol_id,
                    connect.duration,
                    connect.client_id,
                )?;
            }
        }
        ConnAck::send(client, RETURN_CODE_ACCEPTED)?;
        Ok(())
    }
}
