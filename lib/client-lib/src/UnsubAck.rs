use crate::{
    ClientLib::MqttSnClient, Errors::ExoError, MSG_LEN_UNSUBACK,
    MSG_TYPE_UNSUBACK,
};
use bytes::{BufMut, BytesMut};
use custom_debug::Debug;
use getset::{CopyGetters, Getters, MutGetters, Setters};
use std::mem;
#[derive(Debug, Clone, Getters, Setters, MutGetters, CopyGetters, Default)]
#[getset(get, set)]
pub struct UnsubAck {
    pub len: u8,
    #[debug(format = "0x{:x}")]
    pub msg_type: u8,
    #[debug(format = "0b{:08b}")]
    pub msg_id: u16,
}

impl UnsubAck {
    fn constraint_len(_val: &u8) -> bool {
        //dbg!(_val);
        true
    }
    fn constraint_msg_type(_val: &u8) -> bool {
        //dbg!(_val);
        true
    }
    fn constraint_msg_id(_val: &u16) -> bool {
        //dbg!(_val);
        true
    }

    pub fn rx(
        buf: &[u8],
        size: usize,
        client: &MqttSnClient,
    ) -> Result<u16, ExoError> {
        let (unsub_ack, read_len) = UnsubAck::try_read(&buf, size).unwrap();
        dbg!(unsub_ack.clone());

        if read_len == MSG_LEN_UNSUBACK as usize {
            // XXX Cancel the retransmision scheduled.
            //     No topic_id passing to send for now.
            //     because the subscribe message might not contain it.
            //     The retransmision was scheduled with 0.
            client.cancel_tx.send((
                client.remote_addr,
                unsub_ack.msg_type,
                0,
                unsub_ack.msg_id,
            ));
            // TODO check QoS in flags
            // TODO check flags
            Ok(unsub_ack.msg_id)
        } else {
            Err(ExoError::LenError(read_len, MSG_LEN_UNSUBACK as usize))
        }
    }

    // TODO error checking and return
    pub fn tx(client: &MqttSnClient, msg_id: u16, return_code: u8) {
        let unsub_ack = UnsubAck {
            len: MSG_LEN_UNSUBACK,
            msg_type: MSG_TYPE_UNSUBACK,
            msg_id,
        };
        let mut bytes_buf = BytesMut::with_capacity(MSG_LEN_UNSUBACK as usize);
        dbg!(unsub_ack.clone());
        unsub_ack.try_write(&mut bytes_buf);
        dbg!(bytes_buf.clone());
        dbg!(client.remote_addr);
        // transmit to network
        client
            .transmit_tx
            .send((client.remote_addr, bytes_buf.to_owned()));
    }
}
