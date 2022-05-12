/*
5.4.12 PUBLISH
Length    MsgType Flags TopicId MsgId Data
(octet 0) (1)     (2)   (3-4)   (5-6) (7:n)
Table 16: PUBLISH Message
This message is used by both clients and gateways to publish data for a certain topic. Its format is illustrated
in Table 16:
• Length and MsgType: see Section 5.2.
• Flags:
– DUP: same as MQTT, indicates whether message is sent for the first time or not.
– QoS: same as MQTT, contains the QoS level for this PUBLISH message.
– Retain: same as MQTT, contains the Retain flag.
– Will: not used
– CleanSession: not used
– TopicIdType: indicates the type of the topic id contained in the TopicId field.
• TopicId: contains the topic id value or the short topic name for which the data is published.
• MsgId: same meaning as the MQTT “Message ID”; only relevant in case of QoS levels 1 and 2, otherwise
coded 0x0000.
• Data: the published data.
*/
#![allow(unused_imports)]
use bytes::{BufMut, BytesMut};
use custom_debug::Debug;
use getset::{CopyGetters, Getters, MutGetters};
use std::mem;
use std::net::SocketAddr;
use std::str;

extern crate trace_caller;
use hashbrown::HashMap;
use std::sync::Mutex;
use trace_caller::trace;

use crate::{
    broker_lib::MqttSnClient,
    eformat,
    filter::{get_subscribers_with_topic_id, Subscriber},
    flags::{
        flag_is_retain, flag_qos_level, flags_set, CLEAN_SESSION_FALSE,
        CLEAN_SESSION_TRUE, DUP_FALSE, DUP_TRUE, QOS_LEVEL_0, QOS_LEVEL_1,
        QOS_LEVEL_2, QOS_LEVEL_3, RETAIN_FALSE, RETAIN_TRUE,
        TOPIC_ID_TYPE_NORMAL, TOPIC_ID_TYPE_PRE_DEFINED,
        TOPIC_ID_TYPE_RESERVED, TOPIC_ID_TYPE_SHORT, WILL_FALSE, WILL_TRUE,
    },
    function,
    message::{MsgHeader, MsgHeaderEnum},
    pub_ack::PubAck,
    pub_msg_cache::PubMsgCache,
    pub_rec::PubRec,
    retain::Retain,
    MSG_LEN_PUBACK, MSG_LEN_PUBLISH_HEADER, MSG_LEN_PUBREC, MSG_TYPE_CONNACK,
    MSG_TYPE_CONNECT, MSG_TYPE_PUBACK, MSG_TYPE_PUBCOMP, MSG_TYPE_PUBLISH,
    MSG_TYPE_PUBREC, MSG_TYPE_PUBREL, MSG_TYPE_SUBACK, MSG_TYPE_SUBSCRIBE,
    RETURN_CODE_ACCEPTED,
};

#[derive(Debug, Clone, Default)]
pub struct PublishRecv {
    pub topic_id: u16,
    pub msg_id: u16,
    pub data: String,
}

// TODO 3 bytes message length. use macros
/*
#[derive(Debug, Clone, Getters, Setters, MutGetters, CopyGetters, Default)]
#[getset(get, set)]
pub struct Publish3Bytes {
    first_octet: u8, // Must be 0x1 for 3 bytes length.
    len: u16,        // Next 2 bytes indicate the length, up 64K bytes.
    #[debug(format = "0x{:x}")]
    msg_type: u8,
    #[debug(format = "0b{:08b}")]
    flags: u8,
    topic_id: u16,
    msg_id: u16,
    // data: String,
    data: BytesMut,
}
*/

#[derive(
    Debug, Clone, Getters, MutGetters, CopyGetters, Default, PartialEq, Hash, Eq,
)]
#[getset(get, set)]
pub struct Publish {
    len: u8,
    #[debug(format = "0x{:x}")]
    msg_type: u8,
    #[debug(format = "0b{:08b}")]
    flags: u8,
    topic_id: u16,
    msg_id: u16,
    data: BytesMut, // TODO: use Bytes.
}

impl Publish {
    pub fn new(
        topic_id: u16,
        msg_id: u16,
        qos: u8,
        retain: u8,
        data: BytesMut,
    ) -> Self {
        let len = (data.len() + 7) as u8;
        let flags = flags_set(
            DUP_FALSE,
            qos,
            retain,
            WILL_FALSE,          // not used
            CLEAN_SESSION_FALSE, // not used
            TOPIC_ID_TYPE_NORMAL,
        ); // default for now
        Publish {
            len,
            msg_type: MSG_TYPE_PUBLISH,
            flags,
            topic_id,
            msg_id,
            data: BytesMut::new(),
        }
    }

    /*
    fn constraint_len(_val: &u8) -> bool {
        //dbg!(_val);
        true
    }
    fn constraint_msg_type(_val: &u8) -> bool {
        //dbg!(_val);
        true
    }
    fn constraint_flags(_val: &u8) -> bool {
        //dbg!(_val);
        true
    }
    fn constraint_topic_id(_val: &u16) -> bool {
        //dbg!(_val);
        true
    }
    fn constraint_msg_id(_val: &u16) -> bool {
        //dbg!(_val);
        true
    }
    fn constraint_data(_val: &BytesMut) -> bool {
        //dbg!(_val);
        true
    }
    */

    #[inline(always)]
    pub fn recv(
        buf: &[u8],
        size: usize,
        client: &MqttSnClient,
        msg_header: MsgHeader,
    ) -> Result<(), String> {
        let (publish, _read_fixed_len) = match msg_header.header_len {
            MsgHeaderEnum::Short => Publish::try_read(buf, size).unwrap(),
            MsgHeaderEnum::Long => {
                // * NOTE: don't use publish.len from this arm, because the
                // * shift to eliminate the need the long struct.
                // * Use the len from the msg_header.
                Publish::try_read(&buf[2..], size - 2).unwrap()
            }
        };
        dbg!((size, _read_fixed_len));
        dbg!(publish.clone());
        let subscriber_vec = get_subscribers_with_topic_id(publish.topic_id);
        dbg!(&subscriber_vec);
        // TODO check QoS, https://www.hivemq.com/blog/mqtt-essentials-
        // part-6-mqtt-quality-of-service-levels/
        match flag_qos_level(publish.flags) {
            QOS_LEVEL_2 => {
                // 4-way handshake for QoS level 2 message for the RECEIVER.
                // 1. Received PUBLISH message.
                // 2. Reply with PUBREC,
                //      schedule for restransmit,
                //      expect PUBREL
                // 3. Receive PUBREL - in PubRel module
                //      reply with PUBCOMP
                //      cancel restransmit of PUBREC
                // 4. Send PUBLISH message to subscribers from PUBREL.rx.

                //dbg!(&client);
                let bytes = PubRec::send(publish.msg_id, client)?;
                // PUBREL message doesn't have topic id.
                // For the time wheel hash, default to 0.
                if let Err(err) = client.schedule_tx.try_send((
                    client.remote_addr,
                    MSG_TYPE_PUBREL,
                    0,
                    publish.msg_id,
                    bytes,
                )) {
                    return Err(eformat!(client.remote_addr, err));
                }
                // cache the publish message and the subscribers to send when PUBREL is received
                // from the publisher. The remote_addr and msg_id are used as the key because they
                // are part the message.
                let msg_id = publish.msg_id; // copy the msg_id so publish can be used in the hash
                let cache = PubMsgCache {
                    publish,
                    subscriber_vec,
                };
                PubMsgCache::try_insert((client.remote_addr, msg_id), cache)?;
                return Ok(());
            }
            QOS_LEVEL_1 => {
                // send PUBACK to PUBLISH client
                PubAck::send(
                    publish.topic_id,
                    publish.msg_id,
                    RETURN_CODE_ACCEPTED,
                    client,
                )?;
            }
            QOS_LEVEL_0 => {}
            QOS_LEVEL_3 => {
                return Err(eformat!(
                    client.remote_addr,
                    "QoS level 3 is not supported"
                ));
            }
            _ => {
                // Should never happen because flag_qos_level() filters for 4 cases only.
                {}
            }
        }
        if flag_is_retain(publish.flags) {
            Retain::insert(
                flag_qos_level(publish.flags),
                publish.topic_id,
                publish.msg_id,
                publish.data.clone(),
            );
        }
        Publish::send_msg_to_subscribers(subscriber_vec, publish, client)?;

        // TODO check dup, likely not dup
        //
        // TODO check retain, likely not retain
        // if retain {
        //   send a message to save the message in the topic db
        // }
        Ok(())
    }

    /// Publish a message
    /// 1. Format a message with Publish struct.
    /// 2. Serialize into a byte stream.
    /// 3. Send it to the channel.
    /// 4. Schedule retransmit for QoS Level 1 & 2.
    #[inline(always)]
    #[trace]
    pub fn send(
        topic_id: u16,
        msg_id: u16,
        qos: u8,
        retain: u8,
        data: BytesMut,
        client: &MqttSnClient, // contains the address of the publisher
        remote_addr: SocketAddr, // address of the subscriber
    ) -> Result<(), String> {
        let len = data.len() + MSG_LEN_PUBLISH_HEADER as usize;
        let mut bytes_buf = BytesMut::with_capacity(len);
        // TODO verify that this is correct
        let flags = flags_set(
            DUP_FALSE,
            qos,
            retain,
            WILL_FALSE,          // not used
            CLEAN_SESSION_FALSE, // not used
            TOPIC_ID_TYPE_NORMAL,
        ); // default for now

        // TODO verify big-endian or little-endian for u16 numbers
        // XXX order of statements performance
        let msg_id_byte_1 = msg_id as u8;
        let topic_id_byte_1 = topic_id as u8;
        let msg_id_byte_0 = (msg_id >> 8) as u8;
        let topic_id_byte_0 = (topic_id >> 8) as u8;

        if len < 256 {
            let buf: &[u8] = &[
                len as u8,
                MSG_TYPE_PUBLISH,
                flags,
                msg_id_byte_0,
                msg_id_byte_1,
                topic_id_byte_0,
                topic_id_byte_1,
            ];
            bytes_buf.put(buf);
        } else if len < 1400 {
            let buf: &[u8] = &[
                1,
                (len >> 8) as u8,
                len as u8,
                MSG_TYPE_PUBLISH,
                flags,
                msg_id_byte_0,
                msg_id_byte_1,
                topic_id_byte_0,
                topic_id_byte_1,
            ];
            bytes_buf.put(buf);
        } else {
            return Err(eformat!(client.remote_addr, "len too long", len));
        }
        bytes_buf.put(data);
        // TODO: let bytes = bytes_buf.freeze(); // no copy on clone.

        dbg!(&qos);
        match qos {
            // For level 1, schedule a message for retransmit,
            // cancel it if receive a PUBACK message.
            QOS_LEVEL_1 => {
                dbg!((&qos, QOS_LEVEL_1));
                if let Err(why) = client.schedule_tx.try_send((
                    remote_addr,
                    MSG_TYPE_PUBACK,
                    0,
                    msg_id,
                    bytes_buf.clone(),
                )) {
                    return Err(eformat!(client.remote_addr, why));
                };
            }
            QOS_LEVEL_2 => {
                // 4-way handshake for QoS level 2 message for the SENDER.
                // 1. Send a PUBLISH message.
                // 2. Schedule for restransmit,
                //      expect PUBREC
                // 3. Receive PUBREC - in PubRec module
                //      reply with PUBREL
                //      schedule restransmit
                //      expect PUBCOMP
                //      cancel restransmit of PUBLISH
                // 4. Receive PUBCOMP - in PubComp module
                //      cancel retransmit of PUBREL
                // PUBREC message doesn't have topic id.
                // For the time wheel hash, default to 0.
                dbg!(&qos);
                if let Err(why) = client.schedule_tx.try_send((
                    remote_addr,
                    MSG_TYPE_PUBREC,
                    0,
                    msg_id,
                    bytes_buf.clone(),
                )) {
                    return Err(eformat!(client.remote_addr, why));
                };
            }
            // no restransmit for Level 0 & 3.
            QOS_LEVEL_0 | QOS_LEVEL_3 => {}
            _ => {
                // TODO return error
            }
        }
        // transmit message to remote address
        match client.transmit_tx.try_send((remote_addr, bytes_buf)) {
            Ok(_) => Ok(()),
            Err(why) => Err(eformat!(client.remote_addr, why)),
        }
    }
    /// send PUBLISH messages to subscribers
    pub fn send_msg_to_subscribers(
        subscriber_vec: Vec<Subscriber>,
        publish: Publish,
        client: &MqttSnClient,
    ) -> Result<(), String> {
        // send PUBLISH messages to subscribers
        for subscriber in subscriber_vec {
            let socket_addr = subscriber.socket_addr;
            let qos = subscriber.qos;
            dbg!(&socket_addr);
            // Can't return error, because not all subscribers will have error.
            // TODO error for every subscriber/message
            // TODO use Bytes not BytesMut to eliminate clone/copy.
            // TODO new tx method to reduce have try_write() run once for every subscriber.
            Publish::send(
                publish.topic_id,
                publish.msg_id,
                qos,
                RETAIN_FALSE,
                publish.data.clone(),
                client,
                socket_addr,
            )?;
        }
        Ok(())
    }
}
