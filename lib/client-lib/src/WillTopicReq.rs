use bytes::{BufMut, BytesMut};
use custom_debug::Debug;
use getset::{CopyGetters, Getters, MutGetters, Setters};

use crate::{
    ClientLib::MqttSnClient,
    Errors::ExoError,
    // flags::{flags_set, flag_qos_level, },
    MSG_LEN_WILLTOPICREQ,

    MSG_TYPE_WILLTOPICREQ,
};

#[derive(
    Debug, Clone, Copy, Getters, Setters, MutGetters, CopyGetters, Default,
)]


#[getset(get, set)]
pub struct WillTopicReq {
    pub len: u8,
    #[debug(format = "0x{:x}")]
    pub msg_type: u8,
}

impl WillTopicReq {
    fn constraint_len(_val: &u8) -> bool {
        //dbg!(_val);
        true
    }
    fn constraint_msg_type(_val: &u8) -> bool {
        //dbg!(_val);
        true
    }
    pub fn tx(client: &MqttSnClient) {
        let willtopicreq = WillTopicReq {
            len: MSG_LEN_WILLTOPICREQ,
            msg_type: MSG_TYPE_WILLTOPICREQ,
        };
        let mut bytes_buf = BytesMut::with_capacity(MSG_LEN_WILLTOPICREQ as usize);
        dbg!(willtopicreq.clone());
        willtopicreq.try_write(&mut bytes_buf);
        dbg!(bytes_buf.clone());
        dbg!(client.remote_addr);
        // transmit to network
        client
            .transmit_tx
            .send((client.remote_addr, bytes_buf.to_owned()));
    }

    pub fn rx(
            buf: &[u8],
            size: usize,
            client: &MqttSnClient,
        ) -> Result<(), ExoError> {
            let (will_topic_req, read_len) = WillTopicReq::try_read(&buf, size).unwrap();
            dbg!(will_topic_req.clone());
            if read_len == MSG_LEN_WILLTOPICREQ as usize {
                client.cancel_tx.send((
                    client.remote_addr,
                    will_topic_req.msg_type,
                    0,
                    0,
                ));
                Ok(())
            } else {
                Err(ExoError::LenError(read_len, MSG_LEN_WILLTOPICREQ as usize))
            }
        }

}


