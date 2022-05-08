/*
5.4.16 SUBACK
Length MsgType Flags TopicId MsgId ReturnCode
(octet 0) (1) (2) (3,4) (5,6) (7)
Table 20: SUBACK Message
The SUBACK message is sent by a gateway to a client as an acknowledgment to the receipt and processing of
a SUBSCRIBE message. Its format is illustrated in Table 20:
• Length and MsgType: see Section 5.2.
• Flags:
– DUP: not used.
– QoS: same as MQTT, contains the granted QoS level.
– Retain: not used.
– Will: not used
– CleanSession: not used
– TopicIdType: not used
• TopicId: in case of “accepted” the value that will be used as topic id by the gateway when sending PUBLISH
messages to the client (not relevant in case of subscriptions to a short topic name or to a topic name which
contains wildcard characters)
• MsgId: same value as the one contained in the corresponding SUBSCRIBE message.
• ReturnCode: “accepted”, or rejection reason.
*/
use crate::{
    eformat, function, broker_lib::MqttSnClient, MSG_LEN_SUBACK, MSG_TYPE_SUBACK,
};
use bytes::{BufMut, BytesMut};
use custom_debug::Debug;
use getset::{CopyGetters, Getters, MutGetters};
use std::mem;

#[derive(
    Debug, Clone, Getters, MutGetters, CopyGetters, Default, PartialEq,
)]
#[getset(get, set)]
pub struct SubAck {
    pub len: u8,
    #[debug(format = "0x{:x}")]
    pub msg_type: u8,
    #[debug(format = "0b{:08b}")]
    pub flags: u8,
    pub topic_id: u16,
    pub msg_id: u16,
    pub return_code: u8,
}

impl SubAck {
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
        fn constraint_return_code(_val: &u8) -> bool {
            //dbg!(_val);
            true
        }
    */
    pub fn recv(
        buf: &[u8],
        size: usize,
        client: &MqttSnClient,
    ) -> Result<u16, String> {
        let (sub_ack, read_len) = SubAck::try_read(buf, size).unwrap();
        dbg!(sub_ack.clone());

        if read_len == MSG_LEN_SUBACK as usize {
            // XXX Cancel the retransmision scheduled.
            //     No topic_id passing to send for now.
            //     because the subscribe message might not contain it.
            //     The retransmision was scheduled with 0.
            match client.cancel_tx.try_send((
                client.remote_addr,
                sub_ack.msg_type,
                0,
                sub_ack.msg_id,
            )) {
                Ok(_) => Ok(sub_ack.topic_id),
                Err(err) => Err(eformat!(client.remote_addr, err)),
            }
            // TODO check QoS in flags
            // TODO check flags
        } else {
            Err(eformat!(client.remote_addr, "size", buf[0]))
        }
    }

    // TODO error checking and return
    pub fn send(
        client: &MqttSnClient,
        flags: u8,
        topic_id: u16,
        msg_id: u16,
        return_code: u8,
    ) -> Result<(), String> {
        let sub_ack = SubAck {
            len: MSG_LEN_SUBACK,
            msg_type: MSG_TYPE_SUBACK,
            flags,
            topic_id,
            msg_id,
            return_code,
        };
        let mut bytes_buf = BytesMut::with_capacity(MSG_LEN_SUBACK as usize);
        dbg!(sub_ack.clone());
        sub_ack.try_write(&mut bytes_buf);
        dbg!(bytes_buf.clone());
        dbg!(client.remote_addr);
        // transmit to network
        match client
            .transmit_tx
            .try_send((client.remote_addr, bytes_buf.to_owned()))
        {
            Ok(_) => Ok(()),
            Err(err) => Err(eformat!(client.remote_addr, err)),
        }
    }
}
