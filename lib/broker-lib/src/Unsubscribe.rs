/*
An UNSUBSCRIBE message is sent by the client to the GW to unsubscribe from named topics. Its format is
illustrated in Table 19:
• Length and MsgType: see Section 5.2.
• Flags:
– DUP: not used.
– QoS: not used.
– Retain: not used.
– Will: not used
– CleanSession: not used
– TopicIdType: indicates the type of information included at the end of the message, namely “0b00”
topic name, “0b01” pre-defined topic id, “0b10” short topic name, and “0b11” reserved.
• MsgId: should be coded such that it can be used to identify the corresponding SUBACK message.
• TopicName or TopicId: contains topic name, pre-defined topic id, or short topic name as indicated in the
TopicIdType field.

Length    MsgType Flags MsgId TopicName or TopicId
(octet 0) (1)     (2)   (3-4) (5:n) or (5-6)
Table 19: SUBSCRIBE and UNSUBSCRIBE Messages

*/
use bytes::{BufMut, BytesMut};
use custom_debug::Debug;
use getset::{CopyGetters, Getters, MutGetters};
use std::mem;
use std::str;

extern crate trace_caller;
use trace_caller::trace;

use crate::{
    eformat,
    //     StateMachine,
    flags::{
        flag_topic_id_type,
        flags_set,
        CLEAN_SESSION_FALSE,
        DUP_FALSE,
        TOPIC_ID_TYPE_NORMAL,
        TOPIC_ID_TYPE_PRE_DEFINED,
        TOPIC_ID_TYPE_RESERVED,
        TOPIC_ID_TYPE_SHORT,
        // CleanSessionConst, DupConst, QoSConst, RetainConst, TopicIdTypeConst,
        // WillConst, CLEAN_SESSION_TRUE,
        // DUP_TRUE, QOS_LEVEL_0, QOS_LEVEL_1, QOS_LEVEL_2, QOS_LEVEL_3,
        // RETAIN_FALSE, RETAIN_TRUE,
        // WILL_TRUE,
        WILL_FALSE,
    },
    function,
    message::{MsgHeader, MsgHeaderEnum},
    BrokerLib::MqttSnClient,
    // Connection::connection_filter_insert,
    Filter::{
        // global_filter_insert,
        unsubscribe_with_topic_id,
        unsubscribe_with_topic_name,
    },
    MSG_LEN_UNSUBSCRIBE_HEADER,
    MSG_TYPE_UNSUBACK,
    MSG_TYPE_UNSUBSCRIBE,
};

#[derive(Debug, Clone, Getters, MutGetters, CopyGetters, Default)]
#[getset(get, set)]
pub struct Unsubscribe {
    pub len: u8,
    #[debug(format = "0x{:x}")]
    pub msg_type: u8,
    #[debug(format = "0b{:08b}")]
    pub flags: u8,
    pub msg_id: u16,
    pub topic_name: String, // TODO use enum for topic_name or topic_id
}

impl Unsubscribe {
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
    fn constraint_msg_id(_val: &u16) -> bool {
        //dbg!(_val);
        true
    }
    fn constraint_topic_name(_val: &String) -> bool {
        //dbg!(_val);
        true
    }
    */
    pub fn new(qos: u8, retain: u8, msg_id: u16, topic_name: String) -> Self {
        let len = (topic_name.len() + 5) as u8;
        let mut bb = BytesMut::new();
        bb.put_slice(topic_name.as_bytes());
        let flags = flags_set(
            DUP_FALSE,
            qos,
            retain,
            WILL_FALSE,          // not used
            CLEAN_SESSION_FALSE, // not used
            TOPIC_ID_TYPE_NORMAL,
        ); // default for now
        Unsubscribe {
            len,
            msg_type: MSG_TYPE_UNSUBSCRIBE,
            flags,
            msg_id,
            topic_name, // TODO use enum for topic_name or topic_id
        }
    }
    pub fn rx(
        buf: &[u8],
        size: usize,
        client: &MqttSnClient,
        msg_header: MsgHeader,
    ) -> Result<(), String> {
        let unsubscribe: Unsubscribe;
        let _read_fixed_len: usize;
        match msg_header.header_len {
            MsgHeaderEnum::Short =>
            // TODO replace unwrap
            {
                (unsubscribe, _read_fixed_len) =
                    Unsubscribe::try_read(&buf, size).unwrap()
            }
            MsgHeaderEnum::Long =>
            // TODO replace unwrap
            // For the 4-byte header, parse the body ignoring the first 2 bytes and
            // don't use the length field for the unsubscribe struct.
            // Use the length field from the msg_header.
            {
                (unsubscribe, _read_fixed_len) =
                    Unsubscribe::try_read(&buf[3..], size).unwrap()
            }
        }
        dbg!(unsubscribe.clone());
        match flag_topic_id_type(unsubscribe.flags) {
            TOPIC_ID_TYPE_NORMAL => {
                unsubscribe_with_topic_name(
                    client.remote_addr,
                    unsubscribe.topic_name,
                )?;
            }
            TOPIC_ID_TYPE_PRE_DEFINED => {
                match unsubscribe.topic_name.parse::<u16>() {
                    Ok(topic_id) => {
                        dbg!(topic_id);
                        unsubscribe_with_topic_id(
                            client.remote_addr,
                            topic_id,
                        )?;
                        return Ok(());
                    }
                    Err(err) => {
                        return Err(eformat!(
                            client.remote_addr,
                            "error parsing topic_id",
                            err,
                            unsubscribe.topic_name
                        ));
                    }
                }
            }
            TOPIC_ID_TYPE_SHORT => {
                return Err(eformat!(
                    client.remote_addr,
                    "topic Id short topic name not supported"
                ));
            }
            TOPIC_ID_TYPE_RESERVED => {
                return Err(eformat!(
                    client.remote_addr,
                    "topic Id reserved type"
                ));
            }
            _ => {
                return Err(eformat!(
                    client.remote_addr,
                    "topic Id unknown type"
                ));
            }
        }
        Ok(())
    }
    #[inline(always)]
    #[trace]
    pub fn tx(
        topic: String,
        msg_id: u16,
        qos: u8,
        retain: u8,
        client: &MqttSnClient,
    ) -> Result<(), String> {
        if topic.len() + (MSG_LEN_UNSUBSCRIBE_HEADER as usize) < 256 {
            let unsubscribe = Unsubscribe::new(qos, retain, msg_id, topic);
            dbg!(&unsubscribe);
            let mut bytes_buf =
                BytesMut::with_capacity(unsubscribe.len as usize);
            unsubscribe.try_write(&mut bytes_buf);
            // transmit to network
            if let Err(err) = client
                .transmit_tx
                .try_send((client.remote_addr, bytes_buf.to_owned()))
            {
                return Err(eformat!(client.remote_addr, err));
            }
            // schedule retransmit
            // Unsuback returns the msg_id, but not topic_id.
            match client.schedule_tx.try_send((
                client.remote_addr,
                MSG_TYPE_UNSUBACK,
                0,
                msg_id,
                bytes_buf,
            )) {
                Ok(_) => Ok(()),
                Err(err) => Err(eformat!(client.remote_addr, err)),
            }
        } else {
            Err(eformat!(client.remote_addr, "topic name too long"))
        }
    }
}
