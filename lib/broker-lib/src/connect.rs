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

6.2 Client’s Connection Setup
As with MQTT, a MQTT-SN client needs to setup a connection to a GW before it can exchange information with
that GW. The procedure for setting up a connection with a GW is illustrated in Fig. 3, in which it is assumed that
the client requests the gateway to prompt for the transfer of Will topic and Will message. This request is indicated
by setting the Will flag of the CONNECT message. The client then sends these two pieces of information to the
GW upon receiving the corresponding request messages WILLTOPICREQ and WILLMSGREQ. The procedure
is terminated with the CONNACK message sent by the GW.
If Will flag is not set then the GW answers directly with a CONNACK message.
In case the GW could not accept the connection request (e.g. because of congestion or it does not support a
feature indicated in the CONNECT message), the GW returns a CONNACK message with the rejection reason.
*/
use bytes::{BufMut, Bytes, BytesMut};
use custom_debug::Debug;
use getset::{CopyGetters, Getters, MutGetters};
use std::mem;
use std::str;

use crate::{
    broker_lib::MqttSnClient,
    conn_ack::ConnAck,
    connection::Connection,
    dbg_buf, eformat,
    flags::flag_is_will,
    function,
    keep_alive::KeepAliveTimeWheel,
    msg_hdr::{MsgHeader, MsgHeaderLenEnum},
    retransmit::RetransTimeWheel,
    will_topic_req::WillTopicReq,
    MSG_LEN_CONNECT_HEADER, MSG_TYPE_CONNACK, MSG_TYPE_CONNECT,
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

/// Connect and Connect4 are for sending CONNECT messages with different header lengths.
#[derive(
    Debug, Clone, Getters, MutGetters, CopyGetters, Default, PartialEq,
)]
#[getset(get, set)]
pub struct Connect4 {
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
        msg_header: MsgHeader,
    ) -> Result<(), String> {
        let len = client_id.len() + MSG_LEN_CONNECT_HEADER as usize;
        let remote_addr = msg_header.remote_socket_addr;
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
                .egress_tx
                .try_send((remote_addr, bytes_buf.to_owned()))
            {
                return Err(eformat!(remote_addr, err));
            }
            RetransTimeWheel::schedule_timer(
                remote_addr,
                MSG_TYPE_CONNACK,
                0,
                0,
                1,
                bytes_buf,
            )?;
            return Ok(());
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
                .egress_tx
                .try_send((remote_addr, bytes_buf.to_owned()))
            {
                return Err(eformat!(remote_addr, err));
            }
            RetransTimeWheel::schedule_timer(
                remote_addr,
                MSG_TYPE_CONNACK,
                0,
                0,
                1,
                bytes_buf,
            )?;
            return Ok(());
        } else {
            Err(eformat!(remote_addr, "client_id too long"))
        }
    }

    #[inline(always)]
    pub fn recv(
        buf: &[u8],
        size: usize,
        client: &mut MqttSnClient,
        msg_header: MsgHeader,
    ) -> Result<(), String> {
        dbg_buf!(buf, size);
        let (connect, _read_fixed_len) = match msg_header.header_len {
            MsgHeaderLenEnum::Short => Connect::try_read(buf, size).unwrap(),
            MsgHeaderLenEnum::Long => {
                // *NOTE* The len is no long valid. Use msg_header.len instead.
                Connect::try_read(&buf[2..], size - 2).unwrap()
            }
        };
        // TODO check size vs len
        // dbg!(msg_header);
        dbg!(&connect);
        // Create a new connection will messages and conn_ack messages.
        let remote_addr = msg_header.remote_socket_addr;
        Connection::try_insert(
            remote_addr,
            connect.flags,
            connect.protocol_id,
            connect.duration,
            connect.client_id,
        )?;
        KeepAliveTimeWheel::schedule(remote_addr, connect.duration)?;
        if flag_is_will(connect.flags) {
            // Client set the Will Flag, so the GW must send a Will Topic Request message.
            WillTopicReq::send(client, msg_header)?;
        } else {
            // Client did not set the Will Flag, so the GW must send a Connect Ack message.
            ConnAck::send(client, msg_header, RETURN_CODE_ACCEPTED)?;
        }
        Ok(())
    }
}
