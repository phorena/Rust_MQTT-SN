/*
5.4.21 DISCONNECT
Length    MsgType Duration (optional)
(octet 0) (1)     (2-3)
Table 24: DISCONNECT Message
The format of the DISCONNECT message is illustrated in Table 24:
• Length and MsgType: see Section 5.2.
• Duration: contains the value of the sleep timer; this field is optional and is included by a “sleeping” client
that wants to go the “asleep” state, see Section 6.14 for further details.
As with MQTT, the DISCONNECT message is sent by a client to indicate that it wants to close the connection.
The gateway will acknowledge the receipt of that message by returning a DISCONNECT to the client. A server or
gateway may also sends a DISCONNECT to a client, e.g. in case a gateway, due to an error, cannot map a received
message to a client. Upon receiving such a DISCONNECT message, a client should try to setup the connection
again by sending a CONNECT message to the gateway or server. In all these cases the DISCONNECT message
does not contain the Duration field.
A DISCONNECT message with a Duration field is sent by a client when it wants to go to the “asleep” state.
The receipt of this message is also acknowledged by the gateway by means of a DISCONNECT message (without
a duration field).
*/

use bytes::{BufMut, BytesMut};
use custom_debug::Debug;
use getset::{CopyGetters, Getters, MutGetters};
use std::mem;

use crate::{
    connection::Connection,
    eformat,
    function,
    broker_lib::MqttSnClient,
    MSG_LEN_DISCONNECT,
    MSG_LEN_DISCONNECT_DURATION,
    // flags::{flags_set, flag_qos_level, },
    MSG_TYPE_DISCONNECT,
};

#[derive(
    Debug,
    Clone,
    Copy,
    Getters,
    /*Setters,*/ MutGetters,
    CopyGetters,
    Default,
)]
#[getset(get, set)]
pub struct Disconnect {
    len: u8,
    #[debug(format = "0x{:x}")]
    msg_type: u8,
}

#[derive(
    Debug,
    Clone,
    Copy,
    Getters,
    /*Setters,*/ MutGetters,
    CopyGetters,
    Default,
)]
#[getset(get, set)]
pub struct DisconnectDuration {
    len: u8,
    #[debug(format = "0x{:x}")]
    msg_type: u8,
    duration: u16,
}
impl Disconnect {
    pub fn recv(
        buf: &[u8],
        size: usize,
        client: &MqttSnClient,
    ) -> Result<(), String> {
        if size == MSG_LEN_DISCONNECT as usize {
            let (disconnect, _read_len) =
                Disconnect::try_read(buf, size).unwrap();
            dbg!(disconnect.clone());
            Connection::db();
            Connection::remove(client.remote_addr)?;
            Connection::db();
            Disconnect::send(client)?;
            Ok(())
        } else if size == MSG_LEN_DISCONNECT_DURATION as usize {
            // TODO: implement DisconnectDuration
            let (disconnect_duration, _read_len) =
                DisconnectDuration::try_read(buf, size).unwrap();
            dbg!(disconnect_duration.clone());
            Connection::remove(client.remote_addr)?;
            Disconnect::send(client)?;
            Ok(())
        } else {
            Err(eformat!("len err", size))
        }
    }

    pub fn send(client: &MqttSnClient) -> Result<(), String> {
        let disconnect = Disconnect {
            len: MSG_LEN_DISCONNECT as u8,
            msg_type: MSG_TYPE_DISCONNECT,
        };
        let mut bytes_buf =
            BytesMut::with_capacity(MSG_LEN_DISCONNECT as usize);
        dbg!(disconnect.clone());
        disconnect.try_write(&mut bytes_buf);
        dbg!(bytes_buf.clone());
        dbg!(client.remote_addr);
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
