use bytes::{BufMut, BytesMut};
use custom_debug::Debug;
use getset::{CopyGetters, Getters, MutGetters, Setters};

use crate::{
    ClientLib::MqttSnClient,
    Errors::ExoError,
    // flags::{flags_set, flag_qos_level, },
    MSG_LEN_WILLTOPICRESP,

    MSG_TYPE_WILLTOPICRESP,
};

//changes 
#[derive(Debug, thiserror::Error)]
pub enum WillTopicRespError {
    #[error("WillTopicResp Rejection: {0}")]
    WillTopicRespRejection(u8),
    #[error("WillTopicResp Unknown Code: {0}")]
    WillTopicRespUnknownCode(u8),
    #[error("WillTopicResp Wrong Message Type: {0}")]
    WillTopicRespWrongMessageType(u8),
}

#[derive(
    Debug, Clone, Copy, Getters, Setters, MutGetters, CopyGetters, Default,
)]
#[getset(get, set)]
pub struct WillTopicResp {
    pub len: u8,
    #[debug(format = "0x{:x}")]
    pub msg_type: u8,
    pub return_code: u8,
}

impl WillTopicResp {
    fn constraint_len(_val: &u8) -> bool {
        //dbg!(_val);
        true
    }
    fn constraint_msg_type(_val: &u8) -> bool {
        //dbg!(_val);
        true
    }
    fn constraint_return_code(_val: &u8) -> bool {
        //dbg!(_val);
        true
    }
    //changes 
    pub fn rx(
        buf: &[u8],
        size: usize,
        client: &MqttSnClient,
    ) -> Result<(), ExoError> {
        let (will_topicresp, read_len) = WillTopicResp::try_read(&buf, size).unwrap();
        dbg!(will_topicresp.clone());
        if read_len == MSG_LEN_WILLTOPICRESP as usize {
            client.cancel_tx.send((
                client.remote_addr,
                will_topicresp.msg_type,
                0,
                0,
            ));
            Ok(())
        } else {
            Err(ExoError::LenError(read_len, MSG_LEN_WILLTOPICRESP as usize))
        }
    }

    // TODO error checking and return
    pub fn tx(client: &MqttSnClient, return_code: u8) {
        let willtopicresp = WillTopicResp {
            len: MSG_LEN_WILLTOPICRESP,
            msg_type: MSG_TYPE_WILLTOPICRESP,
            return_code,
        };
        let mut bytes_buf = BytesMut::with_capacity(MSG_LEN_WILLTOPICRESP as usize);
        dbg!(willtopicresp.clone());
        willtopicresp.try_write(&mut bytes_buf);
        dbg!(bytes_buf.clone());
        dbg!(client.remote_addr);
        // transmit to network
        client
            .transmit_tx
            .send((client.remote_addr, bytes_buf.to_owned()));
    }
}

