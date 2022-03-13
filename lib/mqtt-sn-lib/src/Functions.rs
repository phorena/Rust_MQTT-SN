use crate::Connect::Connect;
use crate::MainMachine::MainMachine;
use crate::MsgType::MsgType;
use crate::Transfer::Transfer;
use crate::MTU;

use bytes::BytesMut;
use log::*;
use num_traits::FromPrimitive;
use rust_fsm::*;
use std::mem;

use tokio::net::UdpSocket;


// TODO move to utility lib
macro_rules! function {
    () => {{
        fn f() {}
        fn type_name_of<T>(_: T) -> &'static str {
            std::any::type_name::<T>()
        }
        let name = type_name_of(f);
        &name[..name.len() - 3]
    }};
}


macro_rules! dbg_buf {
    ($buf:ident, $size:ident) => {
        let mut i: usize = 0;
        eprint!("[{}:{}] ", function!(), line!());
        while i < $size {
            eprint!("{:#04X?} ", $buf[i]);
            i += 1;
        }
        eprintln!("");
    };
}


// TODO return Result instead of Option 
pub fn process_input(buf: &[u8], size: usize, transfer: &mut Transfer) -> Option<BytesMut> {
    let mut offset = 0;
    let len: u8 = buf[offset];
    // if len != size, ignore the packet
    if size != len as usize {
        error!("datagram size:({}) != msg len({}).", size, len);
        dbg_buf!(buf, size);
        return None;
    }
    //dbg_buf!(buf, size);
    offset += mem::size_of::<u8>();
    let msg_type_u8 = buf[offset];
    let msg_type = FromPrimitive::from_u8(msg_type_u8);
    match transfer.connection_db.read(transfer.peer) {
        Some(old_machine) => {
            let mut new_machine = old_machine.clone();
            let _ = new_machine
                .machine
                .consume(&msg_type.unwrap(), transfer, &buf, size);
            // TODO check for return value
            // if return error, clear the egress_buffer
            transfer
                .connection_db
                .update(transfer.peer, &old_machine, &new_machine);
        }
        None => {
            // packet without state machine
            dbg!(buf[2]);
            match FromPrimitive::from_u8(buf[2]) {
                Some(MsgType::CONNECT) => {
                    let mut new_machine = MainMachine {
                        machine: StateMachine::new(),
                    };
                    let _ = new_machine
                        .machine
                        .consume(&msg_type.unwrap(), transfer, &buf, size);
                    // TODO check for return value
                    transfer.connection_db.create(transfer.peer, &new_machine);
                    dbg!(new_machine);
                }
                _ => (),
            }
        }
    }
    None
}

pub fn connect(socket: &UdpSocket) -> BytesMut {
    let connect = Connect {
        len: 10,
        msg_type: MsgType::CONNECT as u8,
        flags: 0b00000100,
        protocol_id: 1,
        duration: 30,
        client_id: "linh".to_string(),
    };
    let mut bytes_buf = BytesMut::with_capacity(MTU);
    // serialize the con_ack struct into byte(u8) array for the network.
    dbg!(connect.clone());
    connect.try_write(&mut bytes_buf);
    // dbg!(bytes_buf.clone());
    // return false of error, and set egree_buffers to empty.
    // let amt = socket.send(&bytes_buf[..]);
    bytes_buf
}
