use bytes::{BufMut, BytesMut};
use custom_debug::Debug;
use getset::{CopyGetters, Getters, MutGetters, Setters};
use std::mem;
use std::str;

use crate::{
    flags::flags_set,
    //     StateMachine,
    flags::{
        CleanSessionConst, DupConst, QoSConst, RetainConst, TopicIdTypeConst,
        WillConst, CLEAN_SESSION_FALSE, CLEAN_SESSION_TRUE, DUP_FALSE,
        DUP_TRUE, QOS_LEVEL_0, QOS_LEVEL_1, QOS_LEVEL_2, QOS_LEVEL_3,
        RETAIN_FALSE, RETAIN_TRUE, TOPIC_ID_TYPE_NORNAL,
        TOPIC_ID_TYPE_PRE_DEFINED, TOPIC_ID_TYPE_RESERVED, TOPIC_ID_TYPE_SHORT,
        WILL_FALSE, WILL_TRUE,
    },
    ClientLib::MqttSnClient,
    MSG_TYPE_CONNACK,
    MSG_TYPE_CONNECT,
    MSG_TYPE_PUBACK,
    MSG_TYPE_PUBLISH,
    MSG_TYPE_SUBACK,

    MSG_TYPE_SUBSCRIBE,
};

#[derive(Debug, Clone, Getters, Setters, MutGetters, CopyGetters, Default)]
#[getset(get, set)]
pub struct Subscribe {
    pub len: u8,
    #[debug(format = "0x{:x}")]
    pub msg_type: u8,
    #[debug(format = "0b{:08b}")]
    pub flags: u8,
    pub msg_id: u16,
    pub topic_name: String, // TODO use enum for topic_name or topic_id
}

impl Subscribe {
    pub fn new(topic_name: String, msg_id: u16, qos: u8, retain: u8) -> Self {
        let len = (topic_name.len() + 5) as u8;
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
}

// TODO error checking and return
pub fn subscribe_tx(
    topic: String,
    msg_id: u16,
    qos: u8,
    retain: u8,
    client: &MqttSnClient,
) {
    let subscribe = Subscribe::new(topic, msg_id, qos, retain);
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
