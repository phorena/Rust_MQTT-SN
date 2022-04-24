use crate::ConnAck::ConnAck;
use crate::Connect::Connect;
use crate::Disconnect::Disconnect;
use crate::MessageDb::{MessageDb, MessageDbKey, MessageDbValue};
use crate::MsgType::MsgType;
use crate::PingReq::PingReq;
use crate::PingResp::PingResp;
use crate::PubAck::PubAck;
use crate::PubRel::PubRel;
use crate::Publish::Publish;
use crate::RegAck::RegAck;
use crate::Register::Register;
use crate::StateEnum::StateEnum;
use crate::SubAck::SubAck;
use crate::Subscribe::Subscribe;
use crate::Transfer::Transfer;
use crate::UnsubAck::UnsubAck;
use crate::Unsubscribe::Unsubscribe;
use crate::WillMsg::WillMsg;
use crate::WillMsgReq::WillMsgReq;
use crate::WillMsgResp::WillMsgResp;
use crate::WillMsgUpd::WillMsgUpd;
use crate::WillTopic::WillTopic;
use crate::WillTopicReq::WillTopicReq;
use crate::WillTopicResp::WillTopicResp;
use crate::WillTopicUpd::WillTopicUpd;
use bytes::BytesMut;
use log::*;
use mqtt_sn_lib::MTU;
use rust_fsm::*;
use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MainMachine {
    pub machine: rust_fsm::StateMachine<MQTT_SN>,
    // event_vec: Vec<Event>, // event history
    // state_vec: Vec<StateEnum>, // state history
}

// It's possible to have different input_enum and output_enum
state_machine! {
    input_enum(MsgType)
    state_enum(StateEnum)
    output_enum(MsgType)
    transfer_struct(Transfer)
    derive(Serialize, Deserialize, Clone, Debug, PartialEq)
    pub MQTT_SN(DISCONNECTED)

    // state: ACTIVE, input SUBSCRIBE
    // function: verify_subscribe() is called to verify
    // if true the state is transition to ACTIVE and send SUBACK
    // else the state is transition to ACTIVE and log error

    DISCONNECTED => {
        CONNACK => verify_connack ? ACTIVE[REGISTER] : DISCONNECTED[MSG_TYPE_ERR],
        WILLTOPICREQ => verify_will_topic_req ? WILLSETUP[WILLTOPIC] : DISCONNECTED[MSG_TYPE_ERR],
    },

    //Used when the client sends a Connect message with the "Will" flag on.
    WILLSETUP => {
        WILLMSGREQ => verify_will_msg_req ? ACTIVE[WILLMSG] : DISCONNECTED[MSG_TYPE_ERR],
    },

    ACTIVE => {
        CONNACK => verify_connack ? ACTIVE[REGISTER] : DISCONNECTED[MSG_TYPE_ERR],
        REGACK => verify_regack ? ACTIVE[PUBLISH] : ACTIVE[MSG_TYPE_ERR],
        PUBACK => verify_puback ? ACTIVE[PUBLISH] : ACTIVE[MSG_TYPE_ERR],

        //Used when client reconnects without CleanSession set or has subscribed to a wildcard topic
        //If an unknown topic id has been sent, the client responds with a RegAck message
        //containing a rejection return code
        REGISTER => verify_register ? ACTIVE[REGACK] : ACTIVE[REGACK],

        //Client responds differently according to the QoS level set in the Publish message
        //and they can send a PubAck message with a rejection return code if an unknown topic id
        //has been used
        PUBLISH => verify_publish ? ACTIVE[PUBACK] : ACTIVE[PUBACK], //TODO, fix outputs for QoS 0, will not expect PubAck

        //Used after receiving PubRel message, client has to send out a PubComp message
        PUBREL => verify_pub_rel ? ACTIVE[PUBCOMP] : ACTIVE[MSG_TYPE_ERR],
        SUBACK => verify_suback ? ACTIVE[PUBLISH] : ACTIVE[MSG_TYPE_ERR], //TODO fix outputs
        UNSUBACK => verify_unsuback ? ACTIVE[MSG_TYPE_ERR] : ACTIVE[MSG_TYPE_ERR], //TODO fix outputs
        PINGREQ => verify_ping_req ? ACTIVE[PINGRESP] : ACTIVE[MSG_TYPE_ERR],

        //Q_NOTE client only sends disconnect if it is a sleeping client, otherwise it just
        //resumes normal operation.
        PINGRESP => verify_ping_resp ? ACTIVE[DISCONNECT] : ACTIVE[MSG_TYPE_ERR], //TODO fix outputs

        //The client wants to disconnect, go to sleep, or there is a connection error
        //TODO, ask Lawrence how to deal with three possibilities
        DISCONNECT => verify_disconnect ? ASLEEP[PINGREQ] : LOST[CONNECT], //TODO fix outputs

        WILLTOPICRESP => verify_will_topic_resp ? ACTIVE[PUBLISH] : ACTIVE[MSG_TYPE_ERR], //TODO fix outputs
        WILLMSGRESP => verify_will_msg_resp ? ACTIVE[PUBLISH] : ACTIVE[MSG_TYPE_ERR], //TODO fix outputs
    },

    LOST => {
        CONNACK => verify_connack? ACTIVE[REGISTER] : LOST[CONNECT],
    }
}

/*
Client does not need to verify a Connect message
fn verify_connect(_state:StateEnum,
                  input:MsgType,
                  transfer:&mut Transfer,
                  buf: &[u8],
                  size: usize,
                  ) -> bool {
    // deserialize from u8 array to the Conn structure.
    let (conn, _read_len) = Connect::try_read(buf, size).unwrap();
    // TODO verify the content of conn, return false for errors
    dbg!(conn.clone());
    // TODO convert ConnAck into a function,
    // from the return of the transition function?
    let con_ack = ConnAck {
        len: 3,
        msg_type: MsgType::CONNACK as u8,
        return_code: 0,
    };
    let mut bytes_buf = BytesMut::with_capacity(MTU);
    // serialize the con_ack struct into byte(u8) array for the network.
    con_ack.try_write(&mut bytes_buf);
    // dbg!(bytes_buf.clone());
    transfer.egress_buffers.push((transfer.peer, bytes_buf));
    dbg!(con_ack.clone());
    // return false of error, and set egree_buffers to empty.
    true
}
*/

//Ask Lawrence how to deal with disconnect/brainstorm some ideas

fn verify_disconnect(
    _state: StateEnum,
    input: MsgType,
    transfer: &mut Transfer,
    buf: &[u8],
    size: usize,
) -> bool {
    let (disconnect, _read_len) = Disconnect::try_read(&buf, size).unwrap();
    dbg!(disconnect.clone());
    true
}
fn verify_will_topic(
    _state: StateEnum,
    input: MsgType,
    transfer: &mut Transfer,
    buf: &[u8],
    size: usize,
) -> bool {
    let (will_topic, _read_len) = WillTopic::try_read(&buf, size).unwrap();
    dbg!(will_topic.clone());
    // TODO Check return values
    // TODO Update will db
    true
}
fn verify_will_msg(
    _state: StateEnum,
    input: MsgType,
    transfer: &mut Transfer,
    buf: &[u8],
    size: usize,
) -> bool {
    let (will_msg, _read_len) = WillMsg::try_read(&buf, size).unwrap();
    dbg!(will_msg.clone());
    // TODO Check return values
    // TODO Update will db
    true
}
fn verify_will_topic_update(
    _state: StateEnum,
    input: MsgType,
    transfer: &mut Transfer,
    buf: &[u8],
    size: usize,
) -> bool {
    let (will_topic_update, _read_len) =
        WillTopicUpd::try_read(&buf, size).unwrap();
    dbg!(will_topic_update.clone());
    // TODO Check return values
    // TODO Update will db
    let will_topic_resp = WillTopicResp {
        len: 3,
        msg_type: MsgType::WILLTOPICRESP as u8,
        return_code: 0, // verify return code
    };
    let mut bytes_buf = BytesMut::with_capacity(MTU);
    will_topic_resp.try_write(&mut bytes_buf);
    // dbg!(bytes_buf.clone());
    transfer.egress_buffers.push((transfer.peer, bytes_buf));
    true
}

fn verify_will_msg_req(
    _state: StateEnum,
    input: MsgType,
    transfer: &mut Transfer,
    buf: &[u8],
    size: usize,
) -> bool {
    let (will_msg_req, _read_len) = WillMsgReq::try_read(&buf, size).unwrap();
    dbg!(will_msg_req.clone());
    match will_msg_req.msg_type {
        0x08 => true, //TODO Send a WillMsg
        _ => {
            error!(
                "Unexpected MsgType with WillMsgReq. MsgType: {}",
                will_msg_req.msg_type
            );
            false
        }
    }
}

fn verify_will_topic_req(
    _state: StateEnum,
    input: MsgType,
    transfer: &mut Transfer,
    buf: &[u8],
    size: usize,
) -> bool {
    let (will_topic_req, _read_len) =
        WillTopicReq::try_read(&buf, size).unwrap();
    dbg!(will_topic_req.clone());
    match will_topic_req.msg_type {
        0x06 => true, //TODO, send the WillTopic message
        _ => {
            error!(
                "Unexpected MsgType with WillTopicReq. MsgType: {}",
                will_topic_req.msg_type
            );
            false
        }
    }
}

fn verify_will_topic_resp(
    _state: StateEnum,
    input: MsgType,
    transfer: &mut Transfer,
    buf: &[u8],
    size: usize,
) -> bool {
    let (will_topic_resp, _read_len) =
        WillTopicResp::try_read(&buf, size).unwrap();
    dbg!(will_topic_resp);
    match will_topic_resp.return_code {
        0 => true,
        1..=3 => {
            println!(
                "WillTopicUpd rejection. ReturnCode: {}",
                will_topic_resp.return_code
            );
            false
        }
        _ => {
            error!(
                "WillTopicUpd rejection. Invalid Return Code: {}",
                will_topic_resp.return_code
            );
            false
        }
    }
}
fn verify_will_msg_update(
    _state: StateEnum,
    input: MsgType,
    transfer: &mut Transfer,
    buf: &[u8],
    size: usize,
) -> bool {
    let (will_msg_update, _read_len) =
        WillMsgUpd::try_read(&buf, size).unwrap();
    dbg!(will_msg_update.clone());
    // TODO Check return values
    // TODO Update will db
    let will_msg_resp = WillMsgResp {
        len: 3,
        msg_type: MsgType::WILLMSGRESP as u8,
        return_code: 0, // verify return code
    };
    let mut bytes_buf = BytesMut::with_capacity(MTU);
    will_msg_resp.try_write(&mut bytes_buf);
    // dbg!(bytes_buf.clone());
    transfer.egress_buffers.push((transfer.peer, bytes_buf));
    true
}
fn verify_will_msg_resp(
    _state: StateEnum,
    input: MsgType,
    transfer: &mut Transfer,
    buf: &[u8],
    size: usize,
) -> bool {
    let (will_msg_resp, _read_len) = WillMsgResp::try_read(&buf, size).unwrap();
    dbg!(will_msg_resp.clone());
    match will_msg_resp.msg_type {
        0x1D => match will_msg_resp.return_code {
            0x00 => true,
            0x01..=0x03 => {
                println!(
                    "WillMsgResp Rejection Return Code: {}",
                    will_msg_resp.return_code
                );
                false
            }
            _ => {
                error!(
                    "WillMsgResp unexpected Return Code: {}",
                    will_msg_resp.return_code
                );
                false
            }
        },
        _ => {
            error!(
                "Wrong MsgType with WillMsgResp. MsgType: {}",
                will_msg_resp.msg_type
            );
            false
        }
    }
}

fn verify_ping_req(
    _state: StateEnum,
    input: MsgType,
    transfer: &mut Transfer,
    buf: &[u8],
    size: usize,
) -> bool {
    let (ping_req, _read_len) = PingReq::try_read(&buf, size).unwrap();
    dbg!(ping_req.clone());
    // TODO Check return values
    // TODO Update client db
    let ping_resp = PingResp {
        len: 2,
        msg_type: MsgType::PINGRESP as u8,
    };
    let mut bytes_buf = BytesMut::with_capacity(MTU);
    ping_resp.try_write(&mut bytes_buf);
    // dbg!(bytes_buf.clone());
    transfer.egress_buffers.push((transfer.peer, bytes_buf));
    true
}
fn verify_ping_resp(
    _state: StateEnum,
    input: MsgType,
    transfer: &mut Transfer,
    buf: &[u8],
    size: usize,
) -> bool {
    let (ping_resp, _read_len) = PingResp::try_read(&buf, size).unwrap();
    dbg!(ping_resp.clone());
    match ping_resp.msg_type {
        0x17 => {
            true //TODO, if the client wants to go back to sleep, they would send a disconnect message
        }
        _ => {
            error!(
                "Wrong MsgType for PingResp. MsgType: {}",
                ping_resp.msg_type
            );
            false
        }
    }
}
fn verify_connack(
    _state: StateEnum,
    input: MsgType,
    transfer: &mut Transfer,
    buf: &[u8],
    size: usize,
) -> bool {
    // deserialize from u8 array to the Conn structure.
    dbg!(input);
    let input = MsgType::CONNACK;
    // dbg!(input.into());
    let (conn_ack, _read_len) = ConnAck::try_read(buf, size).unwrap();
    // TODO verify the content of conn_ack, return false for errors
    dbg!(conn_ack.clone());

    match conn_ack.msg_type {
        0x05 => match conn_ack.return_code {
            0x00 => {
                /*
                let reg = Register {
                    len: ("topic_sample".len() + 6) as u8,
                    msg_type: MsgType::REGISTER as u8,
                    topic_id: 0x0000,
                    msg_id: 0x0001,
                    topic_name: "topic_sample".to_string(),
                };
                let mut bytes_buf = BytesMut::with_capacity(MTU);
                reg.try_write(&mut bytes_buf);
                transfer.egress_buffers.push((transfer.peer, bytes_buf));
                */
                true
            }
            0x01..=0x03 => {
                error!("CONNACK Rejection: {}", conn_ack.return_code);
                false
            }
            _ => {
                error!("Unknown return code");
                false
            }
        },
        _ => {
            error!(
                "Unexpected message type for CONNACK. Received MsgType: {}",
                conn_ack.msg_type
            );
            false
        }
    }
}

fn verify_regack(
    _state: StateEnum,
    input: MsgType,
    transfer: &mut Transfer,
    buf: &[u8],
    size: usize,
) -> bool {
    let (reg_ack, _read_len) = RegAck::try_read(&buf, size).unwrap();
    dbg!(reg_ack.clone());
    match reg_ack.msg_type {
        0x0B => {
            match reg_ack.return_code {
                0x00 => {
                    //TODO: Have to store the new topic_id, for now will just publish and store
                    //it in the transfer_topic_id_counter
                    transfer.topic_id_counter = reg_ack.topic_id;
                    let publish = Publish {
                        len: ("hello".len() + 7) as u8,
                        msg_type: MsgType::REGACK as u8,
                        flags: 0b00000000,
                        topic_id: transfer.topic_id_counter,
                        msg_id: 23,
                        data: "hello".to_string(),
                    };
                    true
                }
                0x01..=0x03 => {
                    error!("RegAck Rejection: {}", reg_ack.return_code);
                    false
                }
                _ => {
                    error!("Unknown return code from REGACK message");
                    error!(
                        "RegAck Rejection ReturnCode: {}",
                        reg_ack.return_code
                    );
                    false
                }
            }
        }
        _ => {
            error!(
                "Unexpected message type for RegAck. Received MsgType: {}",
                reg_ack.return_code
            );
            false
        }
    }
}
fn verify_register(
    _state: StateEnum,
    input: MsgType,
    transfer: &mut Transfer,
    buf: &[u8],
    size: usize,
) -> bool {
    let (register, _read_len) = Register::try_read(&buf, size).unwrap();
    dbg!(register.clone());
    match transfer.topic_db.get(&register.topic_name) {
        Some(id) => {
            let reg_ack = RegAck {
                len: 7,
                msg_type: MsgType::REGACK as u8,
                topic_id: id,
                msg_id: register.msg_id,
                return_code: 0, // verify with specification
            };
            dbg!(reg_ack.clone());

            let mut bytes_buf = BytesMut::with_capacity(MTU);
            reg_ack.try_write(&mut bytes_buf);
            // dbg!(bytes_buf.clone());
            transfer.egress_buffers.push((transfer.peer, bytes_buf));
            true
        }
        None => {
            // topic_id = transfer.topic_db.get(&sub.topic_name).unwrap();
            error!(
                "Register: topic name doesn't exist, {:?}",
                register.topic_name
            );
            false
        }
    }
}
// TODO verify the content of the sub structure.
// if any of the content is invalid, log error and return false.
// else format SubAck struct.
fn verify_subscribe(
    _state: StateEnum,
    input: MsgType,
    transfer: &mut Transfer,
    buf: &[u8],
    size: usize,
) -> bool {
    let (sub, _read_len) = Subscribe::try_read(&buf, size).unwrap();
    dbg!(sub.clone());
    let topic_id = transfer
        .topic_db
        .create(&sub.topic_name, transfer.topic_id_counter);
    // new topic_name, topic_id is used, increase it by 1
    if topic_id == transfer.topic_id_counter {
        transfer.topic_id_counter += 1;
    }
    // example/prototype code, should get content from sub struct and other variables.
    // save subscriber to database for publish later
    // TODO sub to the same message, reply the same topic_id
    transfer.subscriber_db.insert(topic_id, transfer.peer, 1);
    let sub_ack = SubAck {
        len: 8,
        msg_type: MsgType::SUBACK as u8,
        flags: 0b101111, // SUBACK_FLAG: u8 = ...
        topic_id: topic_id,
        msg_id: sub.msg_id,
        return_code: 0, // verify with specification
    };
    dbg!(sub_ack.clone());

    let mut bytes_buf = BytesMut::with_capacity(MTU);
    sub_ack.try_write(&mut bytes_buf);
    // dbg!(bytes_buf.clone());
    transfer.egress_buffers.push((transfer.peer, bytes_buf));
    true
}
fn verify_unsubscribe(
    _state: StateEnum,
    input: MsgType,
    transfer: &mut Transfer,
    buf: &[u8],
    size: usize,
) -> bool {
    let (unsub, _read_len) = Unsubscribe::try_read(&buf, size).unwrap();
    dbg!(unsub.clone());
    // get topic_id from topic_name
    match transfer.topic_db.get(&unsub.topic_name) {
        Some(topic_id) => {
            // delete peer from db
            match transfer.subscriber_db.delete(topic_id, transfer.peer) {
                Some(_id) => {
                    let unsub_ack = UnsubAck {
                        len: 8,
                        msg_type: MsgType::UNSUBACK as u8,
                        msg_id: unsub.msg_id,
                    };
                    dbg!(unsub_ack.clone());
                    let mut bytes_buf = BytesMut::with_capacity(MTU);
                    unsub_ack.try_write(&mut bytes_buf);
                    transfer.egress_buffers.push((transfer.peer, bytes_buf));
                    true
                }
                None => {
                    error!(
                        "unsubscribe topic id and peer not found: {} {:?}",
                        topic_id, transfer.peer
                    );
                    false
                }
            }
        }
        None => {
            error!("unsubscribe topic name not found: {}", unsub.topic_name);
            false
        }
    }
}
fn verify_unsuback(
    _state: StateEnum,
    input: MsgType,
    transfer: &mut Transfer,
    buf: &[u8],
    size: usize,
) -> bool {
    let (unsuback, _read_len) = UnsubAck::try_read(&buf, size).unwrap();
    dbg!(unsuback.clone());
    match unsuback.msg_type {
        0x15 => {
            true //TODO, Check that the msg_id matches with the original Unsubscribe message
        }
        _ => {
            error!(
                "Unexpected MsgType for UnsubAck. MsgType {}",
                unsuback.msg_type
            );
            false
        }
    }
}
fn verify_suback(
    _state: StateEnum,
    input: MsgType,
    transfer: &mut Transfer,
    buf: &[u8],
    size: usize,
) -> bool {
    let (suback, _read_len) = SubAck::try_read(&buf, size).unwrap();
    dbg!(suback.clone());
    // TODO replace the broker code with client code
    /*
    let topic_id = transfer.topic_db.create(&sub.topic_name,
                                            transfer.topic_id_counter);
    // new topic_name, topic_id is used, increase it by 1
    if topic_id == transfer.topic_id_counter {
            transfer.topic_id_counter += 1;
    }
    // example/prototype code, should get content from sub struct and other variables.
    // save subscriber to database for publish later
    // TODO sub to the same message, reply the same topic_id
    transfer.subscriber_db.insert(topic_id, transfer.peer, 1);
    let sub_ack = SubAck {
        len: 8,
        msg_type: MsgType::SUBACK as u8,
        flags: 0b101111, // SUBACK_FLAG: u8 = ...
        topic_id: topic_id,
        msg_id: sub.msg_id,
        return_code: 0, // verify with specification
    };
    dbg!(sub_ack.clone());

    let mut bytes_buf = BytesMut::with_capacity(MTU);
    sub_ack.try_write(&mut bytes_buf);
    // dbg!(bytes_buf.clone());
    transfer.egress_buffers.push((transfer.peer, bytes_buf));
    */
    true
}

fn verify_puback(
    _state: StateEnum,
    input: MsgType,
    transfer: &mut Transfer,
    buf: &[u8],
    size: usize,
) -> bool {
    let (pub_ack, _read_len) = PubAck::try_read(&buf, size).unwrap();
    dbg!(pub_ack.clone());
    match pub_ack.msg_type {
        0x0D => {
            match pub_ack.return_code {
                0x00 => {
                    true //TODO Need to check the msg_id to match it to the original publish message
                }
                0x01..=0x03 => {
                    println!("PubAck RejectionCode: {}", pub_ack.return_code);
                    false
                }
                _ => {
                    error!(
                        "Unexpected PubAck Return Code: {}",
                        pub_ack.return_code
                    );
                    false
                }
            }
        }
        _ => {
            error!(
                "Wrong MsgType for PubAck message. MsgType: {}",
                pub_ack.return_code
            );
            false
        }
    }
}
fn verify_pub_rel(
    _state: StateEnum,
    input: MsgType,
    transfer: &mut Transfer,
    buf: &[u8],
    size: usize,
) -> bool {
    let (pub_rel, _read_len) = PubRel::try_read(&buf, size).unwrap();
    dbg!(pub_rel.clone());
    match pub_rel.msg_type {
        0x10 => {
            true //TODO, need to make sure MsgId matches with the PubRec message and then send a PubComp
        }
        _ => {
            error!("Wrong MsgType for PubRel");
            false
        }
    }
}

fn verify_publish(
    _state: StateEnum,
    input: MsgType,
    transfer: &mut Transfer,
    buf: &[u8],
    size: usize,
) -> bool {
    let (publish, _read_len) = Publish::try_read(&buf, size).unwrap();
    // dbg_buf!(buf, size);
    dbg!(publish.clone());
    let msg_key = MessageDbKey {
        topic_id: publish.topic_id,
    };
    let msg_val = MessageDbValue {
        message: publish.data.clone(),
    };

    transfer.message_db.upsert(msg_key, msg_val);
    let subscribers = transfer.subscriber_db.get(publish.topic_id);
    dbg!(subscribers.clone());
    dbg!(transfer.message_db.get(msg_key));

    match subscribers {
        Some(subs) => {
            // write out the original message, might have the change data in the fields.
            let mut bytes_buf = BytesMut::with_capacity(MTU);
            publish.try_write(&mut bytes_buf);
            for (sub_socket_addr, _) in subs.peers {
                transfer
                    .egress_buffers
                    .push((sub_socket_addr, bytes_buf.clone()));
            }
            true
        }
        None => false,
    }
}
