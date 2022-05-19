/*
5.4.10 REGISTER
The REGISTER message is sent by a client to a GW for requesting a topic id value for the included topic name.
It is also sent by a GW to inform a client about the topic id value it has assigned to the included topic name. Its
format is illustrated in Table 14:
• Length and MsgType: see Section 5.2.
• TopicId: if sent by a client, it is coded 0x0000 and is not relevant; if sent by a GW, it contains the topic id
value assigned to the topic name included in the TopicName field;
• MsgId: should be coded such that it can be used to identify the corresponding REGACK message.
• TopicName: contains the topic name.

Length    MsgType TopicId MsgId TopicName
(octet 0) (1)     (2,3)   (4:5) (6:n)
Table 14: REGISTER Message
*/
use bytes::{BufMut, BytesMut};
use custom_debug::Debug;
use getset::{CopyGetters, Getters, MutGetters};
use log::*;
use std::mem;
use std::str;

use crate::{
    broker_lib::MqttSnClient,
    eformat,
    filter::get_topic_id_with_topic_name,
    function,
    msg_hdr::{MsgHeader, MsgHeaderEnum},
    reg_ack::RegAck,
    // flags::{flags_set, flag_qos_level, },
    MSG_LEN_REGISTER_HEADER,
    MSG_TYPE_REGACK,
    MSG_TYPE_REGISTER,
    RETURN_CODE_ACCEPTED,
    RETURN_CODE_INVALID_TOPIC_ID,
};
#[derive(Debug, Clone, Getters, MutGetters, CopyGetters, Default)]
#[getset(get, set)]
pub struct Register {
    pub len: u8,
    #[debug(format = "0x{:x}")]
    pub msg_type: u8,
    pub topic_id: u16,
    pub msg_id: u16,
    pub topic_name: String, // TODO use enum for topic_name or topic_id
}

impl Register {
    /*
    fn constraint_len(_val: &u8) -> bool {
        //dbg!(_val);
        true
    }
    fn constraint_msg_type(_val: &u8) -> bool {
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
    fn constraint_topic_name(_val: &String) -> bool {
        //dbg!(_val);
        true
    }
    */
    pub fn recv(
        buf: &[u8],
        size: usize,
        client: &MqttSnClient,
        msg_header: MsgHeader,
    ) -> Result<(), String> {
        let register: Register;
        let _read_fixed_len: usize;
        match msg_header.header_len {
            MsgHeaderEnum::Short => {
                (register, _read_fixed_len) =
                    Register::try_read(buf, size).unwrap();
            }
            MsgHeaderEnum::Long => {
                (register, _read_fixed_len) =
                    Register::try_read(&buf[3..], size).unwrap();
            }
        }
        match get_topic_id_with_topic_name(register.topic_name) {
            Some(topic_id) => {
                RegAck::send(
                    topic_id,
                    register.msg_id,
                    RETURN_CODE_ACCEPTED,
                    client,
                )?;
            }
            None => {
                RegAck::send(
                    0,
                    register.msg_id,
                    RETURN_CODE_INVALID_TOPIC_ID,
                    client,
                )?;
            }
        };
        Ok(())
    }
    pub fn send(
        topic_id: u16,
        msg_id: u16,
        topic_name: String,
        client: &MqttSnClient,
    ) -> Result<(), String> {
        // new way to format a message
        let len = MSG_LEN_REGISTER_HEADER as usize + topic_name.len() as usize;
        let mut buf = BytesMut::with_capacity(len);
        // TODO optimize by initializing an array of header fields
        // then buf.put_slice().
        if len < 256 {
            // 2-byte header
            buf.put_u8(len as u8);
        } else if len < 1400 {
            // 4-byte header
            buf.put_u8(1);
            buf.put_u16(len as u16);
        } else {
            return Err(eformat!("len is too big", len));
        }
        buf.put_u8(MSG_TYPE_REGISTER);
        buf.put_u16(topic_id);
        buf.put_u16(msg_id);
        buf.put_slice(topic_name.as_bytes());
        // transmit to network
        // transmit message to remote address
        if let Err(err) = client
            .transmit_tx
            .try_send((client.remote_addr, buf.to_owned()))
        {
            return Err(eformat!(client.remote_addr, err));
        }
        match client.schedule_tx.try_send((
            client.remote_addr,
            MSG_TYPE_REGACK,
            topic_id,
            msg_id,
            buf,
        )) {
            Ok(()) => Ok(()),
            Err(err) => Err(eformat!(client.remote_addr, err)),
        }
    }
}
