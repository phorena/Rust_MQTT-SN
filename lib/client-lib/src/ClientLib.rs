use std::net::UdpSocket;
use std::{hint, thread};
use std::{net::SocketAddr, sync::Arc, sync::Mutex};

use crate::TimingWheel2::RetransTimeWheel;
use bytes::{BytesMut};
use core::fmt::Debug;
use crossbeam::channel::{unbounded, Receiver, Sender};

use log::*;

use crate::{
    flags,
    Channels::Channels,
    ConnAck::ConnAck,
    Connect::Connect,
    PubAck::PubAck,
    Publish::Publish,
    StateMachine::{StateMachine, STATE_DISCONNECT},
    SubAck::SubAck,
    Subscribe::Subscribe,
    TimingWheel2::{RetransmitData, RetransmitHeader},
    MSG_TYPE_CONNACK, MSG_TYPE_CONNECT, MSG_TYPE_PUBACK, MSG_TYPE_PUBLISH,
    MSG_TYPE_PUBREC, MSG_TYPE_SUBACK, MSG_TYPE_SUBSCRIBE,
};
use trace_var::trace_var;

macro_rules! function {
    () => {{
        fn f() {}
        fn type_name_of<T>(_: T) -> &'static str {
            std::any::type_name::<T>()
        }
        let name = type_name_of(f);

        // Find and cut the rest of the path
        match &name[..name.len() - 3].rfind(':') {
            Some(pos) => &name[pos + 1..name.len() - 3],
            None => &name[..name.len() - 3],
        }
    }};
}

// dbg macro that prints function name instead of file name.
// https://stackoverflow.com/questions/65946195/understanding-the-dbg-macro-in-rust
macro_rules! dbg_fn {
    () => {
        $crate::eprintln!("[{}:{}]", function!(), line!());
    };
    ($val:expr $(,)?) => {
        // Use of `match` here is intentional because it affects the lifetimes
        // of temporaries - https://stackoverflow.com/a/48732525/1063961
        match $val {
            tmp => {

                // Added the timestamp
                let now = chrono::Local::now();
                eprint!(
                    "{:02}-{:02} {:02}:{:02}:{:02}.{:09}",
                    now.month(),
                    now.day(),
                    now.hour(),
                    now.minute(),
                    now.second(),
                    now.nanosecond(),
                    );
                // replace file!() with function!()
                eprintln!("<{}:{}> {} = {:#?}",
                          function!(), line!(), stringify!($val), &tmp);
                tmp
            }
        }
    };
    ($($val:expr),+ $(,)?) => {
        ($($dbg_fn!($val)),+,)
    };
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

#[derive(Debug, Clone)]
pub struct MqttSnClient {
    // socket: UdpSocket, // clone not implemented
    // state: AtomicU8, // clone not implemented
    pub remote_addr: SocketAddr,

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
}

impl MqttSnClient {
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
        let (subscribe_tx, subscribe_rx): (
            Sender<Publish>,
            Receiver<Publish>,
        ) = unbounded();
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
            retrans_time_wheel,
            state: Arc::new(Mutex::new(STATE_DISCONNECT)),
            state_machine: StateMachine::new(),
            schedule_tx,
            cancel_tx,
            transmit_tx,
            transmit_rx,
            subscribe_tx,
            subscribe_rx,
        }
    }

    fn rx_loop(mut self, socket: UdpSocket) {
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
                        // TODO process 3 bytes length
                        let msg_type = buf[1] as u8;
                        if msg_type == MSG_TYPE_PUBLISH {
                            Publish::rx(&buf, size, &self);
                            continue;
                        };
                        if msg_type == MSG_TYPE_PUBACK {
                            PubAck::rx(&buf, size, &self);
                            continue;
                        };
                        if msg_type == MSG_TYPE_SUBACK {
                            SubAck::rx(&buf, size, &self);
                            continue;
                        };
                        if msg_type == MSG_TYPE_SUBSCRIBE {
                            Subscribe::rx(&buf, size, &self);
                            continue;
                        };
                        if msg_type == MSG_TYPE_CONNACK {
                            match ConnAck::rx(&buf, size, &self) {
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
                            continue;
                        };
                    }
                    Err(why) => {
                        error!("{}", why);
                    }
                }
            }
        });
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
                        // TODO process 3 bytes length
                        let msg_type = buf[1] as u8;
                        if msg_type == MSG_TYPE_PUBLISH {
                            Publish::rx(&buf, size, &self);
                            continue;
                        };
                        if msg_type == MSG_TYPE_PUBACK {
                            PubAck::rx(&buf, size, &self);
                            continue;
                        };
                        if msg_type == MSG_TYPE_SUBACK {
                            SubAck::rx(&buf, size, &self);
                            continue;
                        };
                        if msg_type == MSG_TYPE_CONNECT {
                            Connect::rx(&buf, size, &self);
                            continue;
                        };
                        if msg_type == MSG_TYPE_CONNACK {
                            match ConnAck::rx(&buf, size, &self) {
                                Ok(_) => {
                                    // Broker shouldn't receive ConnAck
                                    // because it doesn't send Connect for now.
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
                    socket_tx.send_to(&bytes[..], addr);
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
    pub fn connect(mut self, client_id: String, socket: UdpSocket) {
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
                    socket_tx.send_to(&bytes[..], addr);
                }
                Err(why) => {
                    println!("channel_rx_thread: {}", why);
                }
            }
        });
        dbg!(&client_id);
        let conn_duration = 5;
        Connect::tx(client_id, conn_duration, &self);
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
                        match ConnAck::rx(&buf, size, &self) {
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
        self.rx_loop(socket);
    }

    pub fn subscribe(
        &self,
        topic: String,
        msg_id: u16,
        qos: u8,
        retain: u8,
    ) -> &Receiver<Publish> {
        let sub = Subscribe::tx(topic, msg_id, qos, retain, &self);
        &self.subscribe_rx
    }
    /// Publish a message
    /// 1. Format a message with Publish struct.
    /// 2. Serialize into a byte stream.
    /// 3. Send it to the channel.
    /// 4. Schedule retransmit for QoS Level 1 & 2.
    pub fn publish(
        &self,
        topic_id: u16,
        msg_id: u16,
        qos: u8,
        retain: u8,
        data: String,
    ) {
        Publish::tx(topic_id, msg_id, qos, retain, data, &self);
    }
}
