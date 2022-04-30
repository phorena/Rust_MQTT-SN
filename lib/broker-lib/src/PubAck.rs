use bytes::{BufMut, BytesMut};
use custom_debug::Debug;
use getset::{CopyGetters, Getters, MutGetters};
use std::mem;

use crate::{
    eformat, function,
    BrokerLib::MqttSnClient,
    // flags::{flags_set, flag_qos_level, },
    MSG_LEN_PUBACK,
    MSG_TYPE_PUBACK,
};
#[derive(
    Debug,
    Clone,
    Getters,
    /* Setters,*/ MutGetters,
    CopyGetters,
    Default,
    PartialEq,
)]
#[getset(get, set)]
pub struct PubAck {
    pub len: u8,
    #[debug(format = "0x{:x}")]
    pub msg_type: u8,
    pub topic_id: u16,
    pub msg_id: u16,
    pub return_code: u8,
}

impl PubAck {
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
    fn constraint_return_code(_val: &u8) -> bool {
        //dbg!(_val);
        true
    }
    */

    #[inline(always)]
    pub fn rx(
        buf: &[u8],
        size: usize,
        client: &MqttSnClient,
    ) -> Result<(u16, u16, u8), String> {
        let (pub_ack, read_len) = PubAck::try_read(&buf, size).unwrap();
        dbg!(pub_ack.clone());
        if read_len == MSG_LEN_PUBACK as usize {
            match client.cancel_tx.try_send((
                client.remote_addr,
                pub_ack.msg_type,
                pub_ack.topic_id,
                pub_ack.msg_id,
            )){
                // TODO process return code?
                Ok(()) => return Ok((pub_ack.topic_id, pub_ack.msg_id, pub_ack.return_code)),
                Err(err) => return Err(eformat!(client.remote_addr, err)),
            }
        } else {
            Err(eformat!(client.remote_addr, "len err", read_len))
        }
    }
    #[inline(always)]
    pub fn tx(
        topic_id: u16,
        msg_id: u16,
        return_code: u8,
        client: &MqttSnClient,
    ) {
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
        let msg_id_byte_0 = msg_id as u8;
        let topic_id_byte_0 = topic_id as u8;
        let msg_id_byte_1 = (msg_id >> 8) as u8;
        let topic_id_byte_1 = (topic_id >> 8) as u8;
        // message format
        // PUBACK:[len(0), msg_type(1),
        //         topic_id(2,3), msg_id(4,5),
        //         return_code(6)]
        let mut bytes = BytesMut::with_capacity(MSG_LEN_PUBACK as usize);
        let buf: &[u8] = &[
            MSG_LEN_PUBACK,
            MSG_TYPE_PUBACK,
            topic_id_byte_1,
            topic_id_byte_0,
            msg_id_byte_1,
            msg_id_byte_0,
            return_code,
        ];
        bytes.put(buf);
        let _result = client.transmit_tx.send((client.remote_addr, bytes));
        dbg!(&buf);
    }
}
// NOTE: puback_tx is inlined hard coded for performance.
