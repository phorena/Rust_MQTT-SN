/*
5.4.22 WILLTOPICUPD
Length MsgType Flags WillTopic
(octet 0) (1) (2) (3:n)
Table 25: WILLTOPICUPD Message
c Copyright IBM Corporation 1999, 2013. All rights reserved. 17
The WILLTOPICUPD message is sent by a client to update its Will topic name stored in the GW/server. Its
format is shown in Table 25:
• Length and MsgType: see Section 5.2.
• Flags:
– DUP: not used.
– QoS: same as MQTT, contains the Will QoS
– Retain: same as MQTT, contains the Will Retain flag
– Will: not used
– CleanSession: not used
– TopicIdType: not used.
• WillTopic: contains the Will topic name.
An empty WILLTOPICUPD message is a WILLTOPICUPD message without Flags and WillTopic field (i.e.
it is exactly 2 octets long). It is used by a client to delete its Will topic and Will message stored in the GW/server.
*/
use crate::{
    broker_lib::MqttSnClient, connection::Connection, eformat, function,
    will_topic_resp::WillTopicResp, MSG_LEN_WILL_TOPIC_UPD_HEADER,
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
