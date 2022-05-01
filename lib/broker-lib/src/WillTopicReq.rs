use crate::{
    eformat, function, BrokerLib::MqttSnClient, MSG_LEN_WILL_TOPIC_REQ,
    MSG_TYPE_WILL_TOPIC_REQ,
};
use bytes::{BufMut, BytesMut};
use custom_debug::Debug;
use getset::{CopyGetters, Getters, MutGetters};
#[derive(Debug, Clone, Copy, Getters, MutGetters, CopyGetters, Default)]
#[getset(get, set)]
pub struct WillTopicReq {
    pub len: u8,
    #[debug(format = "0x{:x}")]
    pub msg_type: u8,
}

impl WillTopicReq {
    /*
    fn constraint_len(_val: &u8) -> bool {
        //dbg!(_val);
        true
    }
    fn constraint_msg_type(_val: &u8) -> bool {
        //dbg!(_val);
        true
    }
    */
    pub fn rx(
        buf: &[u8],
        size: usize,
        client: &MqttSnClient,
    ) -> Result<(), String> {
        if size == MSG_LEN_WILL_TOPIC_REQ as usize
            && buf[0] == MSG_LEN_WILL_TOPIC_REQ
        {
            return Ok(());
        } else {
            return Err(eformat!(client.remote_addr, "len err", size));
        }
    }

    pub fn tx(client: &MqttSnClient) -> Result<(), String> {
        let will = WillTopicReq {
            len: MSG_LEN_WILL_TOPIC_REQ as u8,
            msg_type: MSG_TYPE_WILL_TOPIC_REQ,
        };
        let mut bytes =
            BytesMut::with_capacity(MSG_LEN_WILL_TOPIC_REQ as usize);
        dbg!(will.clone());
        will.try_write(&mut bytes);
        dbg!(bytes.clone());
        dbg!(client.remote_addr);
        // transmit to network
        match client
            .transmit_tx
            .try_send((client.remote_addr, bytes.to_owned()))
        {
            Ok(()) => return Ok(()),
            Err(err) => return Err(eformat!(client.remote_addr, err)),
        }
    }
}
