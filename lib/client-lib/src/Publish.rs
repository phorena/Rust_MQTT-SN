#![allow(unused_imports)]
use bytes::{BufMut, BytesMut};
use custom_debug::Debug;
use getset::{CopyGetters, Getters, MutGetters, Setters};
use std::mem;
use std::str;

use crate::{
    flags::{
        flag_qos_level, flags_set,
        CLEAN_SESSION_FALSE, CLEAN_SESSION_TRUE,
        DUP_FALSE, DUP_TRUE, QOS_LEVEL_0, QOS_LEVEL_1,
        QOS_LEVEL_2, QOS_LEVEL_3, RETAIN_FALSE, RETAIN_TRUE,
        TOPIC_ID_TYPE_NORNAL, TOPIC_ID_TYPE_PRE_DEFINED,
        TOPIC_ID_TYPE_RESERVED, TOPIC_ID_TYPE_SHORT, WILL_FALSE, WILL_TRUE,
    },
    ClientLib::MqttSnClient,
    Errors::ExoError,
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

#[derive(Debug, Clone, Getters, Setters, MutGetters, CopyGetters, Default)]
#[getset(get, set)]
pub struct Publish {
    len: u8,
    #[debug(format = "0x{:x}")]
    msg_type: u8,
    #[debug(format = "0b{:08b}")]
    flags: u8,
    topic_id: u16,
    msg_id: u16,
    data: String,
}

impl Publish {
    pub fn new(
        topic_id: u16,
        msg_id: u16,
        qos: u8,
        retain: u8,
        data: String,
    ) -> Self {
        let len = (data.len() + 7) as u8;
        let flags = flags_set(
            DUP_FALSE,
            qos,
            retain,
            WILL_FALSE,          // not used
            CLEAN_SESSION_FALSE, // not used
            TOPIC_ID_TYPE_NORNAL,
        ); // default for now
        let publish = Publish {
            len,
            msg_type: MSG_TYPE_PUBLISH,
            flags,
            topic_id,
            msg_id,
            data,
        };
        publish
    }

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
    fn constraint_data(_val: &String) -> bool {
        //dbg!(_val);
        true
    }
}

#[inline(always)]
pub fn publish_rx(
    buf: &[u8],
    size: usize,
    client: &MqttSnClient,
) -> Result<(), ExoError> {
    // TODO replace unwrap
    let (publish, read_fixed_len) = Publish::try_read(&buf, size).unwrap();
    dbg!(publish.clone());
    dbg!(publish.clone().data);
    let read_len = read_fixed_len + publish.data.len();

    dbg!((size, read_len));

    // TODO check QoS, https://www.hivemq.com/blog/mqtt-essentials-
    // part-6-mqtt-quality-of-service-levels/
    if read_len == size {
        match flag_qos_level(publish.flags) {
            QOS_LEVEL_1 => {
                /* slow implementation
                   let mut bytes_buf = BytesMut::with_capacity(7);
                   let puback_bytes = PubAck {
                   len: 7,
                   msg_type: MsgType::PUBACK as u8,
                   topic_id: publish.topic_id,
                   msg_id: publish.msg_id,
                   return_code: RETURN_CODE_ACCEPTED
                   };
                   puback_bytes.try_write(&mut bytes_buf);
                   dbg!(&bytes_buf);
                // let amt = socket.send(&bytes_buf[..]);
                */

                // faster implementation
                // TODO verify big-endian or little-endian for u16 numbers
                // XXX order of statements performance
                let msg_id_byte_0 = publish.msg_id as u8;
                let topic_id_byte_0 = publish.topic_id as u8;
                let msg_id_byte_1 = (publish.msg_id >> 8) as u8;
                let topic_id_byte_1 = (publish.topic_id >> 8) as u8;
                // message format
                // PUBACK:[len(0), msg_type(1),
                //         topic_id(2,3), msg_id(4,5),
                //         return_code(6)]
                let mut bytes =
                    BytesMut::with_capacity(MSG_LEN_PUBACK as usize);
                let buf: &[u8] = &[
                    MSG_LEN_PUBACK,
                    MSG_TYPE_PUBACK,
                    topic_id_byte_1,
                    topic_id_byte_0,
                    msg_id_byte_1,
                    msg_id_byte_0,
                    RETURN_CODE_ACCEPTED,
                ];
                bytes.put(buf);
                client.transmit_tx.send((client.remote_addr, bytes));
                dbg!(&buf);
            }
            QOS_LEVEL_2 => {
                // 4-way handshake for QoS level 2 message for the RECEIVER.
                // 1. Received PUBLISH message.
                // 2. Reply with PUBREC,
                //      schedule for restransmit,
                //      expect PUBREL
                // 3. Receive PUBREL - in PubRel module
                //      reply with PUBCOMP
                //      cancel restransmit of PUBREC
                let msg_id_byte_0 = publish.msg_id as u8;
                let msg_id_byte_1 = (publish.msg_id >> 8) as u8;
                // PUBREC:[len(0),msg_type(1),
                //         msg_id(2,3)]
                let mut bytes =
                    BytesMut::with_capacity(MSG_LEN_PUBACK as usize);
                let buf: &[u8] = &[
                    MSG_LEN_PUBREC,
                    MSG_TYPE_PUBREC,
                    msg_id_byte_1,
                    msg_id_byte_0,
                ];
                // TODO change to channel
                bytes.put(buf);
                client.transmit_tx.send((client.remote_addr, bytes.clone()));
                dbg!(&buf);
                // PUBREL message doesn't have topic id.
                // For the time wheel hash, default to 0.

                client.schedule_tx.send((
                    client.remote_addr,
                    MSG_TYPE_PUBREL,
                    0,
                    publish.msg_id,
                    bytes,
                ));
            }
            _ => {} // do nothing for QoS levels 0 & 3.
        }

        // TODO check dup, likely not dup
        //
        // TODO check retain, likely not retain
        // if retain {
        //   send a message to save the message in the topic db
        // }
        client.subscribe_tx.send(publish);
        Ok(())
    } else {
        return Err(ExoError::LenError(read_len, size));
    }
}
/// Publish a message
/// 1. Format a message with Publish struct.
/// 2. Serialize into a byte stream.
/// 3. Send it to the channel.
/// 4. Schedule retransmit for QoS Level 1 & 2.
#[inline(always)]
pub fn publish_tx(
    topic_id: u16,
    msg_id: u16,
    qos: u8,
    retain: u8,
    data: String,
    client: &MqttSnClient,
) -> Result<(), ExoError> {
    let publish = Publish::new(topic_id, msg_id, qos, retain, data);
    let mut bytes_buf = BytesMut::with_capacity(publish.len as usize);
    publish.try_write(&mut bytes_buf);
    client
        .transmit_tx
        .send((client.remote_addr, bytes_buf.to_owned()));
    dbg!(&qos);
    match qos {
        // For level 1, schedule a message for retransmit,
        // cancel it if receive a PUBACK message.
        QOS_LEVEL_1 => {
            dbg!((&qos, QOS_LEVEL_1));
            client.schedule_tx.send((
                client.remote_addr,
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
            client.schedule_tx.send((
                client.remote_addr,
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
