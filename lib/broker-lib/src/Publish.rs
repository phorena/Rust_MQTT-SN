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
    flags::{
        flag_qos_level, flags_set, CLEAN_SESSION_FALSE, CLEAN_SESSION_TRUE,
        DUP_FALSE, DUP_TRUE, QOS_LEVEL_0, QOS_LEVEL_1, QOS_LEVEL_2,
        QOS_LEVEL_3, RETAIN_FALSE, RETAIN_TRUE, TOPIC_ID_TYPE_NORMAL,
        TOPIC_ID_TYPE_PRE_DEFINED, TOPIC_ID_TYPE_RESERVED, TOPIC_ID_TYPE_SHORT,
        WILL_FALSE, WILL_TRUE,
    },
    pub_msg_cache::PubMsgCache,
    BrokerLib::MqttSnClient,
    Filter::{get_subscribers_with_topic_id, Subscriber},
    PubAck::PubAck,
    PubRec::PubRec,
    MSG_LEN_PUBACK, MSG_LEN_PUBREC, MSG_TYPE_CONNACK, MSG_TYPE_CONNECT,
    MSG_TYPE_PUBACK, MSG_TYPE_PUBCOMP, MSG_TYPE_PUBLISH, MSG_TYPE_PUBREC,
    MSG_TYPE_PUBREL, MSG_TYPE_SUBACK, MSG_TYPE_SUBSCRIBE, RETURN_CODE_ACCEPTED,
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
    // data: String,
    data: BytesMut,
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
        let publish = Publish {
            len,
            msg_type: MSG_TYPE_PUBLISH,
            flags,
            topic_id,
            msg_id,
            data: BytesMut::new(),
        };
        publish
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
    pub fn rx(
        buf: &[u8],
        size: usize,
        client: &MqttSnClient,
    ) -> Result<(), String> {
        // TODO replace unwrap
        let (publish, read_fixed_len) = Publish::try_read(&buf, size).unwrap();
        dbg!(publish.clone());
        dbg!(publish.clone().data);
        let read_len = read_fixed_len + publish.data.len();

        dbg!((size, read_len));

        let subscriber_vec = get_subscribers_with_topic_id(publish.topic_id);
        dbg!(&subscriber_vec);
        // TODO check QoS, https://www.hivemq.com/blog/mqtt-essentials-
        // part-6-mqtt-quality-of-service-levels/
        if read_len == size {
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
                    let bytes = PubRec::tx(publish.msg_id, client);
                    // PUBREL message doesn't have topic id.
                    // For the time wheel hash, default to 0.
                    let _result = client.schedule_tx.send((
                        client.remote_addr,
                        MSG_TYPE_PUBREL,
                        0,
                        publish.msg_id,
                        bytes,
                    ));
                    // cache the publish message and the subscribers to send when PUBREL is received
                    // from the publisher. The remote_addr and msg_id are used as the key because they
                    // are part the message.
                    let msg_id = publish.msg_id; // copy the msg_id so publish can be used in the hash
                    let cache = PubMsgCache {
                        publish,
                        subscriber_vec,
                    };
                    PubMsgCache::try_insert(
                        (client.remote_addr, msg_id),
                        cache,
                    )?;
                    return Ok(());
                }
                QOS_LEVEL_1 => {
                    // send PUBACK to PUBLISH client
                    PubAck::tx(
                        publish.topic_id,
                        publish.msg_id,
                        RETURN_CODE_ACCEPTED,
                        client,
                    );
                }
                QOS_LEVEL_0 | QOS_LEVEL_3 => {}
                _ => {
                    // Should never happen because flag_qos_level() filters for 4 cases only.
                    {}
                }
            }
            Publish::send_msg_to_subscribers(subscriber_vec, publish, client)?;

            // TODO check dup, likely not dup
            //
            // TODO check retain, likely not retain
            // if retain {
            //   send a message to save the message in the topic db
            // }
            Ok(())
        } else {
            // TODO remove len check
            return Err("len error".to_string());
        }
    }

    /// Publish a message
    /// 1. Format a message with Publish struct.
    /// 2. Serialize into a byte stream.
    /// 3. Send it to the channel.
    /// 4. Schedule retransmit for QoS Level 1 & 2.
    #[inline(always)]
    #[trace]
    pub fn tx(
        topic_id: u16,
        msg_id: u16,
        qos: u8,
        retain: u8,
        data: BytesMut,
        client: &MqttSnClient,
        remote_addr: SocketAddr,
    ) -> Result<(), String> {
        let publish = Publish::new(topic_id, msg_id, qos, retain, data);
        let mut bytes_buf = BytesMut::with_capacity(publish.len as usize);
        publish.try_write(&mut bytes_buf);
        let _result =
            client.transmit_tx.send((remote_addr, bytes_buf.to_owned()));
        dbg!(&qos);
        match qos {
            // For level 1, schedule a message for retransmit,
            // cancel it if receive a PUBACK message.
            QOS_LEVEL_1 => {
                dbg!((&qos, QOS_LEVEL_1));
                let _result = client.schedule_tx.send((
                    remote_addr,
                    MSG_TYPE_PUBACK,
                    topic_id,
                    msg_id,
                    bytes_buf,
                ));
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
                let _result = client.schedule_tx.send((
                    remote_addr,
                    MSG_TYPE_PUBREC,
                    0,
                    msg_id,
                    bytes_buf,
                ));
            }
            // no restransmit for Level 0 & 3.
            QOS_LEVEL_0 => {
                ();
            }
            QOS_LEVEL_3 => {
                ();
            }
            _ => {
                // TODO return error
                ()
            }
        }
        Ok(())
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
            // TODO use Bytes not BytesMut to eliminate clone/copy.
            // TODO error for every subscriber/message
            // Can't return error, because not all subscribers will have error.
            let _result = Publish::tx(
                publish.topic_id,
                publish.msg_id,
                qos,
                RETAIN_FALSE,
                publish.data.clone(),
                client,
                socket_addr,
            );
        }
        Ok(())
    }
}
