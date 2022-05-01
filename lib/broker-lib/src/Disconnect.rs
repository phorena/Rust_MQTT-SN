use bytes::{BufMut, BytesMut};
use custom_debug::Debug;
use getset::{CopyGetters, Getters, MutGetters};
use std::mem;

use crate::{
    eformat,
    function,
    BrokerLib::MqttSnClient,
    Connection::Connection,
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

/*
impl Disconnect {
    fn constraint_len(_val: &u8) -> bool {
        //dbg!(_val);
        true
    }
    fn constraint_msg_type(_val: &u8) -> bool {
        //dbg!(_val);
        true
    }
    fn constraint_duration(_val: &u16) -> bool {
        //dbg!(_val);
        true
    }
}
*/

impl Disconnect {
    pub fn rx(
        buf: &[u8],
        size: usize,
        client: &MqttSnClient,
    ) -> Result<(), String> {
        if size == MSG_LEN_DISCONNECT as usize {
            let (disconnect, _read_len) =
                Disconnect::try_read(&buf, size).unwrap();
            dbg!(disconnect.clone());
            Connection::db();
            Connection::remove(client.remote_addr)?;
            Connection::db();
            Disconnect::tx(client)?;
            return Ok(());
        } else if size == MSG_LEN_DISCONNECT_DURATION as usize {
            // TODO: implement DisconnectDuration
            let (disconnect_duration, _read_len) =
                DisconnectDuration::try_read(&buf, size).unwrap();
            dbg!(disconnect_duration.clone());
            Connection::remove(client.remote_addr)?;
            Disconnect::tx(client)?;
            return Ok(());
        } else {
            return Err(eformat!("len err", size));
        }
    }

    pub fn tx(client: &MqttSnClient) -> Result<(), String> {
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
            Ok(()) => return Ok(()),
            Err(err) => return Err(eformat!(client.remote_addr, err)),
        }
    }
}
