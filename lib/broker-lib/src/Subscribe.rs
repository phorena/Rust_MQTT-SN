use bytes::{BufMut, BytesMut};
use custom_debug::Debug;
use getset::{CopyGetters, Getters, MutGetters, Setters};
use std::mem;
use std::str;

extern crate trace_caller;
use trace_caller::trace;

use crate::{
    //     StateMachine,
    flags::{
        flag_qos_level,
        flags_set,
        CLEAN_SESSION_FALSE,
        DUP_FALSE,
        TOPIC_ID_TYPE_NORNAL,
        // CleanSessionConst, DupConst, QoSConst, RetainConst, TopicIdTypeConst,
        // WillConst, CLEAN_SESSION_TRUE,
        // DUP_TRUE, QOS_LEVEL_0, QOS_LEVEL_1, QOS_LEVEL_2, QOS_LEVEL_3,
        // RETAIN_FALSE, RETAIN_TRUE,
        // TOPIC_ID_TYPE_PRE_DEFINED, TOPIC_ID_TYPE_RESERVED, TOPIC_ID_TYPE_SHORT,
        // WILL_TRUE,
        WILL_FALSE,
    },
    BrokerLib::MqttSnClient,
    Connection::connection_filter_insert,
    Errors::ExoError,
    Filter::global_filter_insert,
    SubAck::SubAck,
    MSG_TYPE_SUBACK,
    MSG_TYPE_SUBSCRIBE,
};

#[derive(
    Debug, Clone, Getters, Setters, MutGetters, CopyGetters, Default, PartialEq,
)]
#[getset(get, set)]
pub struct Subscribe {
    pub len: u8,
    #[debug(format = "0x{:x}")]
    pub msg_type: u8,
    #[debug(format = "0b{:08b}")]
    pub flags: u8,
    pub msg_id: u16,
    pub topic_name: String, // TODO use enum for topic_name or topic_id
                            //     pub bb: BytesMut,
}

impl Subscribe {
    pub fn new(topic_name: String, msg_id: u16, qos: u8, retain: u8) -> Self {
        let len = (topic_name.len() + 5) as u8;
        let mut bb = BytesMut::new();
        bb.put_slice(topic_name.as_bytes());
        let flags = flags_set(
            DUP_FALSE,
            qos,
            retain,
            WILL_FALSE,          // not used
            CLEAN_SESSION_FALSE, // not used
            TOPIC_ID_TYPE_NORNAL,
        ); // default for now
        let subscribe = Subscribe {
            len,
            msg_type: MSG_TYPE_SUBSCRIBE,
            flags,
            msg_id,
            topic_name, // TODO use enum for topic_name or topic_id
                        //          bb,
        };
        subscribe
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
    fn constraint_msg_id(_val: &u16) -> bool {
        //dbg!(_val);
        true
    }
    fn constraint_topic_name(_val: &String) -> bool {
        //dbg!(_val);
        true
    }
    /*
    fn constraint_bb(_val: &BytesMut) -> bool {
        //dbg!(_val);
        true
    }
    */

    // TODO error checking and return
    #[trace]
    pub fn tx(
        topic: String,
        msg_id: u16,
        qos: u8,
        retain: u8,
        client: &MqttSnClient,
    ) {
        let subscribe = Subscribe::new(topic, msg_id, qos, retain);
        dbg!(&subscribe);
        let mut bytes_buf = BytesMut::with_capacity(subscribe.len as usize);
        subscribe.try_write(&mut bytes_buf);
        client
            .transmit_tx
            .send((client.remote_addr, bytes_buf.to_owned()));
        client.schedule_tx.send((
            client.remote_addr,
            MSG_TYPE_SUBACK,
            0,
            0,
            bytes_buf,
        ));
        // TODO return Result
    }

    #[inline(always)]
    #[trace]
    pub fn rx(
        buf: &[u8],
        size: usize,
        client: &MqttSnClient,
    ) -> Result<(), String> {
        // TODO replace unwrap
        let (subscribe, read_fixed_len) =
            Subscribe::try_read(&buf, size).unwrap();
        dbg!(subscribe.clone());
        dbg!(subscribe.clone().topic_name);
        let read_len = read_fixed_len + subscribe.topic_name.len();

        dbg!((size, read_len));

        // TODO check QoS, https://www.hivemq.com/blog/mqtt-essentials-
        // part-6-mqtt-quality-of-service-levels/
        if read_len == size {
            connection_filter_insert(
                &subscribe.topic_name[..],
                client.remote_addr,
            )?;
            global_filter_insert(
                &subscribe.topic_name[..],
                client.remote_addr,
            )?;
            match flag_qos_level(subscribe.flags) {
                // TODO topic_id & return_code need values
                QOS_LEVEL_1 => {
                    SubAck::tx(
                        client,
                        subscribe.flags,
                        888,
                        subscribe.msg_id,
                        99,
                    );
                }
                QOS_LEVEL_2 => {
                    SubAck::tx(
                        client,
                        subscribe.flags,
                        888,
                        subscribe.msg_id,
                        99,
                    );
                }
                _ => {} // do nothing for QoS levels 0 & 3.
            }
            Ok(())
        } else {
            // TODO clean up, length check is not needed,
            // if it's check else where, it's not needed here.
            return Err("wrong size".to_string());
        }
    }
}
