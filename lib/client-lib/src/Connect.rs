use bytes::{BufMut, BytesMut};
use custom_debug::Debug;
use getset::{CopyGetters, Getters, MutGetters, Setters};
use std::mem;
use std::str;

use crate::{
    flags::{
        flag_is_clean_session, flag_is_will, flag_qos_level, flags_set,
        CleanSessionConst, DupConst, QoSConst, RetainConst, TopicIdTypeConst,
        WillConst, CLEAN_SESSION_FALSE, CLEAN_SESSION_TRUE, DUP_FALSE,
        DUP_TRUE, QOS_LEVEL_0, QOS_LEVEL_1, QOS_LEVEL_2, QOS_LEVEL_3,
        RETAIN_FALSE, RETAIN_TRUE, TOPIC_ID_TYPE_NORNAL,
        TOPIC_ID_TYPE_PRE_DEFINED, TOPIC_ID_TYPE_RESERVED, TOPIC_ID_TYPE_SHORT,
        WILL_FALSE, WILL_TRUE,
    },
    ClientLib::MqttSnClient,
    ConnAck::ConnAck,
    // flags::{flags_set, flag_qos_level, },
    Errors::ExoError,
    MSG_LEN_PUBACK,
    MSG_LEN_PUBREC,

    MSG_TYPE_CONNACK,
    MSG_TYPE_CONNECT,
    MSG_TYPE_PUBACK,
    MSG_TYPE_PUBCOMP,
    MSG_TYPE_PUBLISH,
    MSG_TYPE_PUBREC,
    MSG_TYPE_PUBREL,
    MSG_TYPE_SUBACK,

    MSG_TYPE_SUBSCRIBE,
    RETURN_CODE_ACCEPTED,
};

#[derive(Debug, Clone, Getters, Setters, MutGetters, CopyGetters, Default)]
#[getset(get, set)]
pub struct Connect {
    pub len: u8,
    #[debug(format = "0x{:x}")]
    pub msg_type: u8,
    #[debug(format = "0b{:08b}")]
    pub flags: u8,
    pub protocol_id: u8,
    pub duration: u16,
    pub client_id: String,
}

// TODO
impl Connect {
    pub fn constraint_len(_val: &u8) -> bool {
        //dbg!(_val);
        true
    }
    pub fn constraint_msg_type(_val: &u8) -> bool {
        //dbg!(_val);
        true
    }
    pub fn constraint_flags(_val: &u8) -> bool {
        //dbg!(_val);
        true
    }
    pub fn constraint_protocol_id(_val: &u8) -> bool {
        //dbg!(_val);
        true
    }
    pub fn constraint_duration(_val: &u16) -> bool {
        //dbg!(_val);
        true
    }
    pub fn constraint_client_id(_val: &String) -> bool {
        // dbg!(_val);
        true
    }

    // TODO error checking and return
    pub fn tx(client_id: String, duration: u16, client: &MqttSnClient) {
        let len = client_id.len() + 6;
        if len < 250 {
            let connect = Connect {
                len: len as u8,
                msg_type: MSG_TYPE_CONNECT,
                flags: 0b00000100,
                protocol_id: 1,
                duration,
                client_id,
            };
            let mut bytes_buf = BytesMut::with_capacity(len);
            // serialize the con_ack struct into byte(u8) array for the network.
            // serialize the con_ack struct into byte(u8) array for the network.
            dbg!(connect.clone());
            dbg!((bytes_buf.clone(), &connect));
            connect.try_write(&mut bytes_buf);
            dbg!(bytes_buf.clone());
            // transmit to network
            client
                .transmit_tx
                .send((client.remote_addr, bytes_buf.to_owned()));
            // schedule retransmit
            client.schedule_tx.send((
                client.remote_addr,
                MSG_TYPE_CONNACK,
                0,
                0,
                bytes_buf,
            ));
        } else {
            // TODO long message modify try_write
            ()
        }
    }

    #[inline(always)]
    pub fn rx(
        buf: &[u8],
        size: usize,
        client: &MqttSnClient,
    ) -> Result<(), ExoError> {
        // TODO replace unwrap
        let (connect, read_fixed_len) = Connect::try_read(&buf, size).unwrap();
        dbg!(connect.clone());
        let read_len = read_fixed_len + connect.client_id.len();

        if read_len == size {
            if flag_is_will(connect.flags) {
                // set will
            }
            if flag_is_clean_session(connect.flags) {
                // clean session
            }
            // protocol_id
            // duration
            // client_id
            // send connack

            ConnAck::tx(client, 0);

            Ok(())
        } else {
            return Err(ExoError::LenError(read_len, size));
        }
    }
}
