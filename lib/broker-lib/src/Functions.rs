use std::sync::atomic::{AtomicU8, Ordering};
use std::{io, net::SocketAddr, net::SocketAddrV4, sync::Arc, sync::Mutex};

use crate::ConnAck::ConnAck;
use crate::Connect::Connect;
use crate::PubAck::PubAck;
use crate::Publish::Publish;
use crate::SubAck::SubAck;
use crate::Subscribe::Subscribe;

use crate::flags::{
    flag_qos_level, flags_set, CLEAN_SESSION_TRUE, DUP_FALSE, WILL_FALSE,
};
use crate::MainMachineClient::MainMachine;
use crate::MsgType::MsgType;
use crate::TimingWheel2::TimingWheel2;
use crate::Transfer::Transfer;
use crate::MTU;

use bytes::BytesMut;
use log::*;
use num_traits::FromPrimitive;
use rust_fsm::*;
use std::mem;

// use tokio::net::UdpSocket;
// use tokio::sync::mpsc::Sender;
// use std::sync::mpsc::{Sender, Receiver};
use crossbeam::channel::{bounded, unbounded, Receiver, Sender};
use std::net::UdpSocket;

type ReturnCodeConst = u8;
const RETURN_CODE_ACCEPTED: ReturnCodeConst = 0;
const RETURN_CODE_CONGESTION: ReturnCodeConst = 1;
const RETURN_CODE_INVALID_TOPIC_ID: ReturnCodeConst = 2;
const RETURN_CODE_NOT_SUPPORTED: ReturnCodeConst = 3;

/// Client Error
#[derive(Debug, thiserror::Error)]
pub enum ClientError {
    #[error("Len Error: {0} (expeted {1})")]
    LenError(usize, usize),
    #[error("Wrong Message Type: {0} (expect {1}")]
    WrongMessageType(u8, u8),

    // return code
    #[error("Congestion: {0}")]
    Congestion(u8),
    #[error("Invalid Topic Id: {0}")]
    InvalidTopicId(u8),
    #[error("Not Supported: {0}")]
    NotSupported(u8),
    #[error("Return Code Reserved: {0}")]
    Reserved(u8),
}

/// RetransmitHeader is stored in:
/// (1) HashMap for cancellation from an ACK
/// (2) timing wheel slots for timeouts.
/// When the wheel reads a slot, it iterates all entries in the vector.
/// Using the RetransmitHeader to get/remove the RetransmitData
/// in the HashMap.
/// If the new duration is greater that the maximum timeout period
/// the HashMap entry will be removed.
/// To cancel a scheduled event, remove the HashMap entry with the
/// RetransmitHeader. Don't need to remove the entry in the slot
/// because the slot entry lookup ignores missing HashMap entries.
#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
struct RetransmitHeader {
    addr: SocketAddr,
    msg_type: u8,
    topic_id: u16, // for pub and sub, default 0
    msg_id: u16,   // for pub and sub, default 0
}

#[derive(Debug, Clone)]
struct RetransmitData {
    bytes: BytesMut,
}

#[derive(Debug)]
struct ClientStruct {
    // for performance, use lockfree structure
    state: Arc<AtomicU8>,
    addr: SocketAddr,
    broker: UdpSocket,
    timing_wheel: &'static TimingWheel2<RetransmitHeader, RetransmitData>,
}

pub fn process_input(
    buf: &[u8],
    size: usize,
    transfer: &mut Transfer,
) -> Option<u8> {
    let mut offset = 0;
    let len: u8 = buf[offset];
    // if len != size, ignore the packet
    if size != len as usize {
        error!("size({}) != len({}).", size, len);
        return None;
    }
    dbg_buf!(buf, size);
    offset += mem::size_of::<u8>();
    let msg_type_u8 = buf[offset];
    let msg_type = FromPrimitive::from_u8(msg_type_u8);
    match transfer.connection_db.read(transfer.peer) {
        Some(old_machine) => {
            dbg!(old_machine.clone());
            let mut new_machine = old_machine.clone();
            let _ = new_machine.machine.consume(
                &msg_type.unwrap(),
                transfer,
                &buf,
                size,
            );
            // TODO check for return value
            // if return error, clear the egress_buffer
            transfer.connection_db.update(
                transfer.peer,
                &old_machine,
                &new_machine,
            );
            dbg!(old_machine.clone());
            dbg!(new_machine.machine.state());

            let state = new_machine.machine.state();
            // Some(1)
            // ()
        }
        None => {
            // packet without state machine
            dbg!(buf[1]);
            match FromPrimitive::from_u8(buf[1]) {
                Some(MsgType::CONNACK) => {
                    let mut new_machine = MainMachine {
                        machine: StateMachine::new(),
                    };
                    let _ = new_machine.machine.consume(
                        &msg_type.unwrap(),
                        transfer,
                        &buf,
                        size,
                    );
                    // TODO check for return value
                    transfer.connection_db.create(transfer.peer, &new_machine);
                    dbg!(new_machine);
                }
                Some(MsgType::CONNECT) => {
                    let mut new_machine = MainMachine {
                        machine: StateMachine::new(),
                    };
                    let _ = new_machine.machine.consume(
                        &msg_type.unwrap(),
                        transfer,
                        &buf,
                        size,
                    );
                    // TODO check for return value
                    transfer.connection_db.create(transfer.peer, &new_machine);
                    dbg!(new_machine);
                }
                _ => (),
            }
        }
    }
    None
    // Some(MsgType::MsgType::ACTIVE)
}

static PUBACK_LEN: u8 = 7;
static PUBREC_LEN: u8 = 4;

pub fn pub_ack(topic_id: u16, msg_id: u16) -> BytesMut {
    let pub_ack = PubAck {
        len: PUBACK_LEN,
        msg_type: MsgType::PUBACK as u8,
        topic_id,
        msg_id,
        return_code: 0,
    };
    let mut bytes_buf = BytesMut::with_capacity(PUBACK_LEN as usize);
    // serialize the con_ack struct into byte(u8) array for the network.
    dbg!(pub_ack.clone());
    pub_ack.try_write(&mut bytes_buf);
    bytes_buf
}


pub fn subscribe(topic_name: String, msg_id: u16) -> BytesMut {
    let len = topic_name.len() + 5;
    let subscribe = Subscribe {
        len: len as u8,
        msg_type: MsgType::SUBSCRIBE as u8,
        flags: 0b00100100,
        msg_id,
        topic_name, // TODO use enum for topic_name or topic_id
    };
    let mut bytes_buf = BytesMut::with_capacity(len);
    // serialize the con_ack struct into byte(u8) array for the network.
    dbg!(subscribe.clone());
    subscribe.try_write(&mut bytes_buf);
    bytes_buf
}

pub fn connect2(client_id: String, duration: u16) -> BytesMut {
    let len = client_id.len() + 6;
    let connect = Connect {
        len: len as u8,
        msg_type: MsgType::CONNECT as u8,
        flags: 0b00000100,
        protocol_id: 1,
        duration: 30,
        client_id,
    };
    let mut bytes_buf = BytesMut::with_capacity(len);
    // serialize the con_ack struct into byte(u8) array for the network.
    // serialize the con_ack struct into byte(u8) array for the network.
    dbg!(connect.clone());
    connect.try_write(&mut bytes_buf);
    dbg!(bytes_buf.clone());
    // return false of error, and set egree_buffers to empty.
    bytes_buf
}

pub fn connect(socket: &UdpSocket, client_id: String) -> BytesMut {
    let len = client_id.len() + 6;
    let connect = Connect {
        len: len as u8,
        msg_type: MsgType::CONNECT as u8,
        flags: 0b00000100,
        protocol_id: 1,
        duration: 30,
        client_id,
    };
    let mut bytes_buf = BytesMut::with_capacity(len);
    // serialize the con_ack struct into byte(u8) array for the network.
    dbg!(connect.clone());
    connect.try_write(&mut bytes_buf);
    dbg!(bytes_buf.clone());
    // return false of error, and set egree_buffers to empty.
    // let amt = socket.send(&bytes_buf[..]);
    bytes_buf
}

fn return_code(val: u8) -> Result<(), ClientError> {
    match val {
        0 => Ok(()),
        1 => Err(ClientError::Congestion(val)),
        2 => Err(ClientError::InvalidTopicId(val)),
        3 => Err(ClientError::NotSupported(val)),
        _ => Err(ClientError::Reserved(val)),
    }
}

// CONACK: len(0) MsgType(1) ReturnCode(2)
// expect:     3        0x5             0
pub fn verify_connack2(buf: &[u8], size: usize) -> Result<(), ClientError> {
    // fast path
    if buf[0] == 3 && buf[2] == 0 {
        return Ok(());
    }
    let msg_type = MsgType::CONNACK;
    // deserialize from u8 array to the Conn structure.
    // TODO replace unwrap
    let (conn_ack, read_len) = ConnAck::try_read(buf, size).unwrap();
    dbg!(conn_ack.clone());

    // TODO check length. Broker bug
    /*
    if (read_len != 3) {
        return Err(ClientError::LenError(read_len, 3));
    }
    */
    // TODO check return code
    Ok(())
}
pub fn verify_puback2(
    buf: &[u8],
    size: usize,
    // ) -> bool {
) -> Result<(u16, u16), ClientError> {
    let msg_type = MsgType::PUBACK;
    let (pub_ack, read_len) = PubAck::try_read(&buf, size).unwrap();
    dbg!(pub_ack.clone());

    // check length
    if read_len != 8 {
        return Err(ClientError::LenError(read_len, 8));
    }

    // check message type
    match pub_ack.msg_type {
        // TODO check flags
        //
        // TODO match msg_id & topic_id
        //
        // check return code
        msg_type => match return_code(pub_ack.return_code) {
            Ok(_) => Ok((pub_ack.topic_id, pub_ack.msg_id)),
            Err(why) => Err(why),
        },
        _ => Err(ClientError::WrongMessageType(
            pub_ack.return_code,
            msg_type.into(),
        )),
    }
}

pub fn verify_suback2(
    buf: &[u8],
    size: usize,
    // ) -> bool {
) -> Result<u16, ClientError> {
    let msg_type = MsgType::CONNACK;
    let (sub_ack, read_len) = SubAck::try_read(&buf, size).unwrap();
    dbg!(sub_ack.clone());

    // check length
    if read_len != 8 {
        return Err(ClientError::LenError(read_len, 8));
    }

    // check message type
    match sub_ack.msg_type {
        // TODO check flags
        //
        // TODO match msg_id & topic_id
        //
        // check return code
        msg_type => match return_code(sub_ack.return_code) {
            Ok(_) => Ok(sub_ack.msg_id),
            Err(why) => Err(why),
        },
        _ => Err(ClientError::WrongMessageType(
            sub_ack.return_code,
            msg_type.into(),
        )),
    }
}


pub fn get_qos_level(qos_bits: u8) -> i8 {
    match qos_bits {
        0 => 0,
        0b00100000 => 1,
        0b01000000 => 2,
        0b01100000 => -1,
        _ => 0,
    }
}

pub fn set_qos_bits(qos_level: i8) -> u8 {
    match qos_level {
        0 => 0,
        1 => 0b00100000,
        2 => 0b01000000,
        -1 => 0b01100000,
        _ => 0,
    }
}

