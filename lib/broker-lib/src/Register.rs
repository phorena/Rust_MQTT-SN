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
use std::mem;
use std::str;

use crate::{
    eformat,
    function,
    message::{MsgHeader, MsgHeaderEnum},
    BrokerLib::MqttSnClient,
    // flags::{flags_set, flag_qos_level, },
    MSG_LEN_REGISTER_HEADER,
    MSG_TYPE_REGISTER,
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
    pub fn rx(
        buf: &[u8],
        size: usize,
        client: &MqttSnClient,
        msg_header: MsgHeader,
    ) -> Result<(), String> {
        let register: Register;
        let _read_fixed_len: usize;
        match msg_header.header_len {
            MsgHeaderEnum::Short => {
                let (register, _read_fixed_len) =
                    Register::try_read(&buf, size).unwrap();
                // TODO verify
                // client.register_topic_id(register.topic_name, register.topic_id)?;
            }
            MsgHeaderEnum::Long => {
                let (register, _read_fixed_len) =
                    Register::try_read(&buf[3..], size).unwrap();
                // TODO verify
                // client.register_topic_id(register.topic_name, register.topic_id)?;
            }
        }
        Ok(())
    }
}
