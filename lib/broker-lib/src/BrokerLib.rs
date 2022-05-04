use std::net::UdpSocket;
use std::thread;
use std::{net::SocketAddr, sync::Arc, sync::Mutex};

use crate::TimingWheel2::RetransTimeWheel;
use bytes::{Bytes, BytesMut};
use core::fmt::Debug;
use crossbeam::channel::{unbounded, Receiver, Sender};

use log::*;

use crate::{
    dbg_buf,
    eformat,
    function,
    message::MsgHeader,
    // Channels::Channels,
    ConnAck::ConnAck,
    Connect::Connect,
    Connection::Connection,
    Disconnect::Disconnect,
    // Connection::ConnHashMap,
    PubAck::PubAck,
    PubRel::PubRel,
    Publish::Publish,
    StateMachine::{StateMachine, STATE_DISCONNECT},
    SubAck::SubAck,
    Subscribe::Subscribe,
    // TimingWheel2::{RetransmitData, RetransmitHeader},
    MSG_TYPE_CONNACK,
    MSG_TYPE_CONNECT,
    MSG_TYPE_DISCONNECT,
    MSG_TYPE_PUBACK,
    MSG_TYPE_PUBLISH,
    MSG_TYPE_PUBREL,
    MSG_TYPE_SUBACK,
    MSG_TYPE_SUBSCRIBE,
};
// use trace_var::trace_var;

#[derive(Debug, Clone, PartialEq)]
pub enum MessageTypeEnum {
    Connect(Connect),
    ConnAct(ConnAck),
    Subscribe(Subscribe),
    SubAck(SubAck),
    Publish(Publish),
    PubAck(PubAck),
}

#[derive(Debug, Clone)]
pub struct MqttSnClient {
    // socket: UdpSocket, // clone not implemented
    // state: AtomicU8, // clone not implemented
    pub remote_addr: SocketAddr,
    //    pub local_addr: SocketAddr,
    pub context: u16,

    pub transmit_tx: Sender<(SocketAddr, BytesMut)>,
    pub cancel_tx: Sender<(SocketAddr, u8, u16, u16)>,
    pub schedule_tx: Sender<(SocketAddr, u8, u16, u16, BytesMut)>,

    // Channel for subscriber to receive messages from the broker
    // publish messages.
    pub subscribe_tx: Sender<Publish>,

    transmit_rx: Receiver<(SocketAddr, BytesMut)>,
    // cancel_rx: Receiver<(SocketAddr, u8, u16, u16)>,
    // schedule_rx: Receiver<(SocketAddr, u8, u16, u16, BytesMut)>,
    retrans_time_wheel: RetransTimeWheel,

    pub subscribe_rx: Receiver<Publish>,
    state: Arc<Mutex<u8>>,
    state_machine: StateMachine,
    // pub conn_hashmap: ConnHashMap,
}

impl MqttSnClient {
    // TODO change Client to Broker
    // TODO change remote_addr to local_addr
    pub fn new(remote_addr: SocketAddr) -> Self {
        let (cancel_tx, cancel_rx): (
            Sender<(SocketAddr, u8, u16, u16)>,
            Receiver<(SocketAddr, u8, u16, u16)>,
        ) = unbounded();
        let (schedule_tx, schedule_rx): (
            Sender<(SocketAddr, u8, u16, u16, BytesMut)>,
            Receiver<(SocketAddr, u8, u16, u16, BytesMut)>,
        ) = unbounded();
        let (transmit_tx, transmit_rx): (
            Sender<(SocketAddr, BytesMut)>,
            Receiver<(SocketAddr, BytesMut)>,
        ) = unbounded();
        let (subscribe_tx, subscribe_rx): (Sender<Publish>, Receiver<Publish>) =
            unbounded();
        let retrans_time_wheel = RetransTimeWheel::new(
            100,
            300,
            schedule_tx.clone(),
            schedule_rx.clone(),
            cancel_tx.clone(),
            cancel_rx.clone(),
            transmit_tx.clone(),
            transmit_rx.clone(),
        );
        MqttSnClient {
            remote_addr,
            context: 0,
            retrans_time_wheel,
            state: Arc::new(Mutex::new(STATE_DISCONNECT)),
            state_machine: StateMachine::new(),
            schedule_tx,
            cancel_tx,
            transmit_tx,
            transmit_rx,
            subscribe_tx,
            subscribe_rx,
            // conn_hashmap: ConnHashMap::new(),
        }
    }

    pub fn broker_rx_loop(mut self, socket: UdpSocket) {
        let self_transmit = self.clone();
        // name for easy debug
        let socket_tx = socket.try_clone().expect("couldn't clone the socket");
        let builder = thread::Builder::new().name("recv_thread".into());
        // process input datagram from network
        let _recv_thread = builder.spawn(move || {
            let mut buf = [0; 1500];
            loop {
                match socket.recv_from(&mut buf) {
                    Ok((size, addr)) => {
                        self.remote_addr = addr;
                        // Decode message header
                        let msg_header: MsgHeader;
                        match MsgHeader::try_read(&buf, size) {
                            Ok(header) => {
                                msg_header = header;
                            }
                            Err(e) => {
                                error!("{}", e);
                                continue;
                            }
                        }
                        let msg_type = msg_header.msg_type;
                        if Connection::contains_key(addr) == false {
                            // New connection, not in the connection hashmap.
                            if msg_type == MSG_TYPE_CONNECT {
                                if let Err(err) = Connect::recv(
                                    &buf, size, &mut self, msg_header,
                                ) {
                                    error!("{}", err);
                                }
                                continue;
                            } else {
                                error!(
                                    "{}",
                                    eformat!(
                                        addr,
                                        "Not in connection map",
                                        msg_type
                                    )
                                );
                                continue;
                            }
                        }
                        // Existing connection
                        dbg!(&msg_header);
                        dbg_buf!(buf, size);
                        if msg_type == MSG_TYPE_PUBLISH {
                            if let Err(err) =
                                Publish::recv(&buf, size, &mut self, msg_header)
                            {
                                error!("{}", err);
                            }
                            continue;
                        };
                        if msg_type == MSG_TYPE_PUBREL {
                            if let Err(err) = PubRel::recv(&buf, size, &mut self)
                            {
                                error!("{}", err);
                            }
                            continue;
                        };
                        if msg_type == MSG_TYPE_PUBACK {
                            let _result = PubAck::recv(&buf, size, &self);
                            continue;
                        };
                        if msg_type == MSG_TYPE_SUBACK {
                            let _result = SubAck::recv(&buf, size, &self);
                            continue;
                        };
                        if msg_type == MSG_TYPE_SUBSCRIBE {
                            let _result = Subscribe::recv(&buf, size, &self);
                            continue;
                        };
                        if msg_type == MSG_TYPE_DISCONNECT {
                            let _result = Disconnect::recv(&buf, size, &mut self);
                            continue;
                        };
                        if msg_type == MSG_TYPE_CONNACK {
                            match ConnAck::recv(&buf, size, &self) {
                                // Broker shouldn't receive ConnAck
                                // because it doesn't send Connect for now.
                                Ok(_) => {
                                    error!("Broker ConnAck {:?}", addr);
                                }
                                Err(why) => error!("ConnAck {:?}", why),
                            }
                            continue;
                        };
                    }
                    Err(why) => {
                        error!("{}", why);
                    }
                }
            }
        });
        let builder = thread::Builder::new().name("transmit_rx_thread".into());
        // process input datagram from network
        let _transmit_rx_thread = builder.spawn(move || loop {
            match self_transmit.transmit_rx.recv() {
                Ok((addr, bytes)) => {
                    // TODO DTLS
                    dbg!((addr, &bytes));
                    let _result = socket_tx.send_to(&bytes[..], addr);
                }
                Err(why) => {
                    println!("channel_rx_thread: {}", why);
                }
            }
        });
    }

    /// Connect to a remote broker
    /// 1. send connect message
    /// 2. schedule retransmit
    /// 3. wait for CONNACK
    ///    3.1. receive CONNACK message
    ///    3.2. change state
    /// 4. run the rx_loop to process rx messages
    // TODO return errors
    pub fn connect(mut self, flags: u8, client_id: String, socket: UdpSocket) {
        let self_time_wheel = self.clone();
        let self_transmit = self.clone();
        let socket_tx = socket.try_clone().expect("couldn't clone the socket");
        self_time_wheel.retrans_time_wheel.run();
        let builder = thread::Builder::new().name("send_thread".into());
        // process input datagram from network
        let _send_thread = builder.spawn(move || loop {
            match self_transmit.transmit_rx.recv() {
                Ok((addr, bytes)) => {
                    // TODO DTLS
                    dbg!(("#####", addr, &bytes));
                    let _result = socket_tx.send_to(&bytes[..], addr);
                }
                Err(why) => {
                    println!("channel_rx_thread: {}", why);
                }
            }
        });
        dbg!(&client_id);
        let duration = 5;
        let client_id = Bytes::from(client_id);
        let _result = Connect::send(flags, 1, duration, client_id, &self);
        dbg!(*self.state.lock().unwrap());
        let cur_state = *self.state.lock().unwrap();
        *self.state.lock().unwrap() = self
            .state_machine
            .transition(cur_state, MSG_TYPE_CONNECT)
            .unwrap();
        dbg!(*self.state.lock().unwrap());
        'outer: loop {
            let mut buf = [0; 1500];
            match socket.recv_from(&mut buf) {
                Ok((size, addr)) => {
                    dbg!((size, addr, buf));
                    self.remote_addr = addr;
                    // TODO process 3 bytes length
                    let msg_type = buf[1] as u8;
                    if msg_type == MSG_TYPE_CONNACK {
                        match ConnAck::recv(&buf, size, &self) {
                            Ok(_) => {
                                dbg!(*self.state.lock().unwrap());
                                let cur_state = *self.state.lock().unwrap();
                                *self.state.lock().unwrap() = self
                                    .state_machine
                                    .transition(cur_state, MSG_TYPE_CONNACK)
                                    .unwrap();
                                dbg!(*self.state.lock().unwrap());
                            }
                            Err(why) => error!("ConnAck {:?}", why),
                        }
                        break 'outer;
                    };
                }
                Err(why) => {
                    error!("{}", why);
                }
            }
        }
    }

    pub fn subscribe(
        &self,
        topic: String,
        msg_id: u16,
        qos: u8,
        retain: u8,
    ) -> &Receiver<Publish> {
        let _result = Subscribe::send(topic, msg_id, qos, retain, &self);
        &self.subscribe_rx
    }
    /* XXX TODO client code.
    pub fn publish(
        &self,
        topic_id: u16,
        msg_id: u16,
        qos: u8,
        retain: u8,
        data: String,
    ) {
        let _result = Publish::send(topic_id, msg_id, qos, retain, data, &self);
    }
    */
}
