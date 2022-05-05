use crate::{
    connection::Connection, eformat, function, will_topic_resp::WillTopicResp,
    BrokerLib::MqttSnClient, MSG_LEN_WILL_TOPIC_UPD_HEADER,
    MSG_TYPE_WILL_TOPIC_UPD, RETURN_CODE_ACCEPTED,
};
use bytes::{BufMut, BytesMut};
use custom_debug::Debug;
use getset::{CopyGetters, Getters, MutGetters};
use std::mem;
use std::str;

#[derive(Debug, Clone, Getters, MutGetters, CopyGetters, Default)]
#[getset(get, set)]
pub struct WillTopicUpd {
    len: u8,
    #[debug(format = "0x{:x}")]
    msg_type: u8,
    #[debug(format = "0b{:08b}")]
    flags: u8,
    will_topic: String,
}
#[derive(Debug, Clone, Getters, MutGetters, CopyGetters, Default)]
#[getset(get, set)]
struct WillTopicUpd4 {
    // NOTE: no pub
    one: u8,
    len: u16,
    #[debug(format = "0x{:x}")]
    msg_type: u8,
    #[debug(format = "0b{:08b}")]
    flags: u8,
    will_topic: String,
}

impl WillTopicUpd {
    pub fn recv(
        buf: &[u8],
        size: usize,
        client: &MqttSnClient,
    ) -> Result<(), String> {
        if size < 256 {
            let (will, len) = WillTopicUpd::try_read(buf, size).unwrap();
            if size == len as usize {
                Connection::update_will_topic(
                    client.remote_addr,
                    will.will_topic,
                )?;
                WillTopicResp::send(RETURN_CODE_ACCEPTED, client)?;
                Ok(())
            } else {
                Err(eformat!(
                    client.remote_addr,
                    "2-bytes len not supported",
                    size
                ))
            }
        } else if size < 1400 {
            let (will, len) = WillTopicUpd4::try_read(buf, size).unwrap();
            if size == len as usize && will.one == 1 {
                Connection::update_will_topic(
                    client.remote_addr,
                    will.will_topic,
                )?;
                WillTopicResp::send(RETURN_CODE_ACCEPTED, client)?;
                Ok(())
            } else {
                Err(eformat!(
                    client.remote_addr,
                    "4-bytes len not supported",
                    size
                ))
            }
        } else {
            Err(eformat!(client.remote_addr, "len too big", size))
        }
    }
    pub fn send(
        flags: u8,
        will_topic: String,
        client: &MqttSnClient,
    ) -> Result<(), String> {
        let len: usize =
            MSG_LEN_WILL_TOPIC_UPD_HEADER as usize + will_topic.len() as usize;
        if len < 256 {
            let will = WillTopicUpd {
                len: len as u8,
                msg_type: MSG_TYPE_WILL_TOPIC_UPD,
                flags,
                will_topic,
            };
            let mut bytes = BytesMut::with_capacity(len);
            will.try_write(&mut bytes);
            match client
                .transmit_tx
                .try_send((client.remote_addr, bytes.to_owned()))
            {
                Ok(()) => Ok(()),
                Err(err) => Err(eformat!(client.remote_addr, err)),
            }
        } else if len < 1400 {
            let will = WillTopicUpd4 {
                one: 1,
                len: len as u16,
                msg_type: MSG_TYPE_WILL_TOPIC_UPD,
                flags,
                will_topic,
            };
            let mut bytes = BytesMut::with_capacity(len);
            will.try_write(&mut bytes);
            match client
                .transmit_tx
                .try_send((client.remote_addr, bytes.to_owned()))
            {
                Ok(()) => Ok(()),
                Err(err) => Err(eformat!(client.remote_addr, err)),
            }
        } else {
            Err(eformat!(client.remote_addr, "len err", len))
        }
    }
}
