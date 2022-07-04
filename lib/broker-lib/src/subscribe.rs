/*

The SUBSCRIBE message is used by a client to subscribe to a certain topic name. Its format is illustrated in
Table 19:
• Length and MsgType: see Section 5.2.
• Flags:
– DUP: same as MQTT, indicates whether message is sent for first time or not.
– QoS: same as MQTT, contains the requested QoS level for this topic.
– Retain: not used
– Will: not used
– CleanSession: not used
– TopicIdType: indicates the type of information included at the end of the message, namely “0b00”
topic name, “0b01” pre-defined topic id, “0b10” short topic name, and “0b11” reserved.
• MsgId: should be coded such that it can be used to identify the corresponding SUBACK message.
• TopicName or TopicId: contains topic name, topic id, or short topic name as indicated in the TopicIdType
field.

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
    broker_lib::MqttSnClient, eformat, filter::*, flags::*, function,
    msg_hdr::*, publish::Publish, retain::Retain, retransmit::RetransTimeWheel,
    sub_ack::SubAck, MSG_TYPE_SUBACK, MSG_TYPE_SUBSCRIBE, RETURN_CODE_ACCEPTED,
};

#[derive(
    Debug, Clone, Getters, MutGetters, CopyGetters, Default, PartialEq,
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
        Subscribe {
            len,
            msg_type: MSG_TYPE_SUBSCRIBE,
            flags,
            msg_id,
            topic_name, // TODO use enum for topic_name or topic_id
                        //          bb,
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
    fn constraint_msg_id(_val: &u16) -> bool {
        //dbg!(_val);
        true
    }
    fn constraint_topic_name(_val: &String) -> bool {
        //dbg!(_val);
        true
    }
    fn constraint_bb(_val: &BytesMut) -> bool {
        //dbg!(_val);
        true
    }
    */

    // TODO error checking and return
    #[inline(always)]
    #[trace]
    pub fn send(
        topic: String,
        msg_id: u16,
        qos: u8,
        retain: u8,
        client: &MqttSnClient,
        msg_header: MsgHeader,
    ) -> Result<(), String> {
        let subscribe = Subscribe::new(qos, retain, msg_id, topic);
        let remote_socket_addr = msg_header.remote_socket_addr;
        dbg!(&subscribe);
        let mut bytes_buf = BytesMut::with_capacity(subscribe.len as usize);
        subscribe.try_write(&mut bytes_buf);
        // transmit to network
        if let Err(err) = client
            .egress_tx
            .try_send((remote_socket_addr, bytes_buf.to_owned()))
        {
            return Err(eformat!(remote_socket_addr, err));
        }
        match RetransTimeWheel::schedule_timer(
            remote_socket_addr,
            MSG_TYPE_SUBACK,
            0,
            0,
            1,
            bytes_buf,
        ) {
            Ok(()) => Ok(()),
            Err(err) => Err(err),
        }
    }

    #[inline(always)]
    #[trace]
    pub fn recv(
        buf: &[u8],
        size: usize,
        client: &MqttSnClient,
        msg_header: MsgHeader,
    ) -> Result<(), String> {
        // TODO replace unwrap
        let (subscribe, read_fixed_len) = match msg_header.header_len {
            MsgHeaderLenEnum::Short => Subscribe::try_read(buf, size).unwrap(),
            MsgHeaderLenEnum::Long => {
                Subscribe::try_read(&buf[2..], size - 2).unwrap()
            }
        };
        let remote_socket_addr = msg_header.remote_socket_addr;
        dbg!(subscribe.clone());
        dbg!(subscribe.clone().topic_name);
        let read_len = read_fixed_len + subscribe.topic_name.len();

        dbg!((size, read_len));
        dbg!(flag_topic_id_type(subscribe.flags));

        // TODO check QoS, https://www.hivemq.com/blog/mqtt-essentials-
        // part-6-mqtt-quality-of-service-levels/
        if read_len == size {
            match flag_topic_id_type(subscribe.flags) {
                TOPIC_ID_TYPE_NORMAL => {
                    // Normal topic type(string): assign topic_id from existing
                    // or new.
                    let topic_id = try_insert_topic_name(subscribe.topic_name)?;
                    subscribe_with_topic_id(
                        remote_socket_addr,
                        topic_id,
                        flag_qos_level(subscribe.flags),
                    )?;
                    dbg!(topic_id);
                    // Because only QoS flag is used and other flags are not used,
                    // return the same flags as received.
                    SubAck::send(
                        client,
                        msg_header,
                        subscribe.flags,
                        topic_id,
                        subscribe.msg_id,
                        RETURN_CODE_ACCEPTED,
                    )?;
                    return Ok(());
                }
                TOPIC_ID_TYPE_PRE_DEFINED => {
                    // Pre-defined topic type(u16/2 bytes) in the topic_id field.
                    // The struct has topic_name field only. We have to convert it to
                    // topic_id.
                    let id = subscribe.topic_name.chars().as_str();
                    dbg!(id);
                    dbg!(id.len());
                    if id.len() != 2 {
                        return Err(eformat!(
                            remote_socket_addr,
                            "Invalid topic_name length: {}",
                            id.len()
                        ));
                    }
                    let mut topic_id: u16 = 0;
                    for char in id.chars() {
                        topic_id = (topic_id << 8) + char as u16;
                    }
                    dbg!(topic_id);
                    // Pre-defined topic type(integer): save remote_addr and
                    // topic_id to the hash map.
                    subscribe_with_topic_id(
                        remote_socket_addr,
                        topic_id,
                        flag_qos_level(subscribe.flags),
                    )?;
                    dbg!(topic_id);
                    SubAck::send(
                        client,
                        msg_header,
                        subscribe.flags,
                        topic_id,
                        subscribe.msg_id,
                        RETURN_CODE_ACCEPTED,
                    )?;
                    dbg!(topic_id);
                    if let Some(msg) = Retain::get(topic_id) {
                        dbg!(topic_id);
                        Publish::send(
                            msg.topic_id,
                            msg.msg_id,
                            msg.qos,
                            RETAIN_FALSE,
                            msg.payload,
                            client,
                            remote_socket_addr,
                        )?;
                    }
                    return Ok(());
                }
                TOPIC_ID_TYPE_SHORT => {
                    dbg!(flag_topic_id_type(subscribe.flags));
                    return Err(eformat!(
                        remote_socket_addr,
                        "topic Id short topic name not supported"
                    ));
                }
                TOPIC_ID_TYPE_RESERVED => {
                    dbg!(flag_topic_id_type(subscribe.flags));
                    return Err(eformat!(
                        remote_socket_addr,
                        "topic Id reserved type"
                    ));
                }
                _ => {
                    dbg!(flag_topic_id_type(subscribe.flags));
                    return Err(eformat!(
                        remote_socket_addr,
                        "topic Id unknown type"
                    ));
                }
            };
        } else {
            // TODO clean up, length check is not needed,
            // if it's check else where, it's not needed here.
            return Err(eformat!(remote_socket_addr, "wrong size"));
        }
    }
}
