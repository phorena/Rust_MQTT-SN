use bytes::*;
use core::fmt::Debug;
use crossbeam::channel::*;
use log::*;
use std::net::SocketAddr;
use std::net::UdpSocket;
use std::sync::Arc;
use std::thread;
use util::conn::*;

use crate::{
    advertise::*,
    // Channels::Channels,
    conn_ack::ConnAck,
    connect::Connect,
    connection::Connection,
    dbg_buf,
    disconnect::Disconnect,
    eformat,
    function,
    gw_info::GwInfo,
    hub::Hub,
    keep_alive::KeepAliveTimeWheel,
    msg_hdr::MsgHeader,
    ping_req::PingReq,
    ping_resp::PingResp,
    // Connection::ConnHashMap,
    pub_ack::PubAck,
    pub_comp::PubComp,
    pub_rec::PubRec,
    pub_rel::PubRel,
    publish::Publish,
    reg_ack::RegAck,
    register::Register,
    retransmit::RetransTimeWheel,
    search_gw::SearchGw,
    sub_ack::SubAck,
    subscribe::Subscribe,
    unsub_ack::UnsubAck,
    unsubscribe::Unsubscribe,
    will_msg::WillMsg,
    will_msg_req::WillMsgReq,
    will_msg_resp::WillMsgResp,
    will_msg_upd::WillMsgUpd,
    will_topic::WillTopic,
    will_topic_req::WillTopicReq,
    will_topic_resp::WillTopicResp,
    will_topic_upd::WillTopicUpd,
    MSG_TYPE_CONNECT,
};
// use trace_var::trace_var;

fn reserved(
    _buf: &[u8],
    _size: usize,
    _client: &MqttSnClient,
    msg_header: MsgHeader,
) -> Result<(), String> {
    Err(eformat!(
        msg_header.remote_socket_addr,
        "reserved",
        msg_header.msg_type
    ))
}

#[derive(Debug, Clone, PartialEq)]
pub enum MessageTypeEnum {
    Connect(Connect),
    ConnAct(ConnAck),
    Subscribe(Subscribe),
    SubAck(SubAck),
    Publish(Publish),
    PubAck2(PubAck),
}
pub type IngressChannelType = (SocketAddr, Bytes, Arc<dyn Conn + Send + Sync>);
pub type EgressChannelType = (SocketAddr, BytesMut);

#[derive(Clone)]
pub struct MqttSnClient {
    // pub remote_addr: SocketAddr,
    pub transmit_tx: Sender<(SocketAddr, BytesMut)>,
    pub subscribe_tx: Sender<Publish>,
    pub transmit_rx: Receiver<(SocketAddr, BytesMut)>,
    pub subscribe_rx: Receiver<Publish>,
    pub ingress_tx: Sender<IngressChannelType>,
    pub ingress_rx: Receiver<IngressChannelType>,
    pub egress_tx: Sender<EgressChannelType>,
    pub egress_rx: Receiver<EgressChannelType>,
    pub hub: Arc<Hub>,
}

impl MqttSnClient {
    // TODO change Client to Broker
    // TODO change remote_addr to local_addr
    pub fn new() -> Self {
        let (transmit_tx, transmit_rx): (
            Sender<(SocketAddr, BytesMut)>,
            Receiver<(SocketAddr, BytesMut)>,
        ) = unbounded();
        let (subscribe_tx, subscribe_rx): (Sender<Publish>, Receiver<Publish>) =
            unbounded();
        // Channel for ingress messages.
        // Incoming messages from the socket are sent from this channel for processing.
        // Multiple consumer threads can receive from this channel.
        let (ingress_tx, ingress_rx): (
            Sender<IngressChannelType>,
            Receiver<IngressChannelType>,
        ) = unbounded();
        // Channel for egress messages.
        // Outgoing messages to the socket are sent to this channel for sending.
        let (egress_tx, egress_rx): (
            Sender<EgressChannelType>,
            Receiver<EgressChannelType>,
        ) = unbounded();
        let hub = Arc::new(Hub::new(Arc::new(ingress_tx.clone())));
        MqttSnClient {
            // remote_addr,
            transmit_tx,
            transmit_rx,
            subscribe_tx,
            subscribe_rx,
            ingress_tx,
            ingress_rx,
            egress_tx,
            egress_rx,
            hub,
        }
    }

    pub fn handle_egress(self) {
        let hub2 = Arc::clone(&self.hub);
        // *NOTE: thread and tokio spawn are not compatible.
        // use thread instead of tokio spawn to read from channel.
        tokio::spawn(async move {
            loop {
                match self.egress_rx.recv() {
                    Ok((addr, data)) => {
                        let dtls_conn = hub2.get_conn(addr).await.unwrap();
                        let _result = dtls_conn.send(&data[..]).await;
                    }
                    Err(why) => {
                        error!("{}", eformat!(why));
                        break;
                    }
                }
            }
        });
    }
    pub fn handle_ingress(self) {
        // *NOTE: thread and tokio spawn are not compatible.
        // use thread instead of tokio spawn to read from channel.

        let functions: Vec<
            fn(
                buf: &[u8],
                size: usize,
                client: &MqttSnClient,
                msg_header: MsgHeader,
            ) -> Result<(), String>,
        > = vec![
            Advertise::recv,     // 0x00
            GwInfo::recv,        // 0x01
            GwInfo::recv,        // 0x02
            reserved,            // 0x03
            Connect::recv,       // 0x04
            ConnAck::recv,       // 0x05
            WillTopicReq::recv,  // 0x06
            WillTopic::recv,     // 0x07
            WillMsgReq::recv,    // 0x08
            WillMsg::recv,       // 0x09
            Register::recv,      // 0x0A
            RegAck::recv,        // 0x0B
            Publish::recv,       // 0x0C
            PubAck::recv,        // 0x0D
            reserved,            // 0x0E
            PubRec::recv,        // 0x0F
            PubRel::recv,        // 0x10
            reserved,            // 0x11
            Subscribe::recv,     // 0x12
            SubAck::recv,        // 0x13
            Unsubscribe::recv,   // 0x14
            UnsubAck::recv,      // 0x15
            PingReq::recv,       // 0x16
            PingResp::recv,      // 0x17
            Disconnect::recv,    // 0x18
            reserved,            // 0x19
            WillTopicUpd::recv,  // 0x1A
            WillTopicResp::recv, // 0x1B
            WillMsgUpd::recv,    // 0x1C
            WillMsgResp::recv,   // 0x1D
        ];

        tokio::spawn(async move {
            loop {
                match self.ingress_rx.recv() {
                    Ok((addr, bytes, conn)) => {
                        let buf = &bytes[..];
                        let size = bytes.len();
                        // Update the last seen time of the client.
                        let _result = KeepAliveTimeWheel::reschedule(addr);
                        // Parse the message header: length, and message type.
                        let msg_header =
                            match MsgHeader::try_read(&buf, size, addr, conn) {
                                Ok(header) => header,
                                Err(e) => {
                                    error!("{}", e);
                                    continue;
                                }
                            };
                        let msg_type = msg_header.msg_type;
                        let fn_index = msg_header.msg_type as usize;
                        // Existing MQTT-SN connection or new connection.
                        // DTLS connection is created at lower layer.
                        if Connection::contains_key(addr) {
                            // New connection.
                            // TODO: the broadcast messages doesn't have connection.
                            // TODO: broadcast messages are not encrypted.
                            if msg_type == MSG_TYPE_CONNECT {
                                error!("{}", "Connect message received twice.");
                                continue;
                            }
                        } else {
                            // Existing connection shouldn't receive CONNECT message.
                            if msg_type != MSG_TYPE_CONNECT {
                                error!("{}", "No connection found");
                                continue;
                            }
                        }
                        if fn_index >= functions.len() {
                            error!(
                                "{}",
                                eformat!(
                                    msg_header.remote_socket_addr,
                                    "Invalid message type",
                                    fn_index
                                )
                            );
                            continue;
                        }
                        let result = functions[fn_index](
                            &buf,
                            size,
                            &self,
                            msg_header.clone(),
                        );
                        if result.is_err() {
                            error!("{}", result.unwrap_err());
                        }
                        continue;
                    }
                    Err(why) => {
                        error!("{:?}", why);
                        continue;
                    }
                }
            }
        });
    }

    pub fn broker_rx_loop(self, socket: UdpSocket) {
        let self_transmit = self.clone();
        // name for easy debug
        let socket_tx = socket.try_clone().expect("couldn't clone the socket");
        let builder = thread::Builder::new().name("recv_thread".into());

        let broadcast_socket_addr =
            "224.0.0.123:61000".parse::<SocketAddr>().unwrap();
        let gateway_info_socket_addr =
            "224.0.0.123:62000".parse::<SocketAddr>().unwrap();

        KeepAliveTimeWheel::init();
        KeepAliveTimeWheel::run(self.clone());
        RetransTimeWheel::init();
        RetransTimeWheel::run(self.clone());
        Advertise::run(broadcast_socket_addr, 5, 2);
        GwInfo::run(gateway_info_socket_addr);

        // client runs this to search for gateway.
        // SearchGw::run(gateway_info_socket_addr, 2, 2);

        // process input datagram from network
        /*
        let new_self = self.clone();
        let _recv_thread = builder.spawn(move || {
            // TODO optimization
            // recv_from(&mut buf[2..], size -2 ) instead of recv_from(&mut buf size).
            // declare the struct with one:u8 and len:u16
            // if the message header is short, backup 2 bytes to try_read() and len += 2.
            // the message is mapped to the struct with one=0 and correct len.
            // The buf[0..1] must be init to 0.

            let mut buf = [0; 1500];
            loop {
                match socket.recv_from(&mut buf) {
                    Ok((size, addr)) => {
                        self.remote_addr = addr;
                        let _result = KeepAliveTimeWheel::reschedule(addr);
                        // Decode message header
                        let msg_header = match MsgHeader::try_read(&buf, size, addr, conn) {
                            Ok(header) => header,
                            Err(e) => {
                                error!("{}", e);
                                continue;
                            }
                        };
                        let msg_type = msg_header.msg_type;
                        // Existing connection?
                        if Connection::contains_key(addr) {
                            dbg!(&msg_header);
                            dbg_buf!(buf, size);
                            if msg_type == MSG_TYPE_PUBLISH {
                                if let Err(err) =
                                    Publish::recv(&buf, size, &self, msg_header)
                                {
                                    error!("{}", err);
                                }
                                continue;
                            };
                            if msg_type == MSG_TYPE_PUBREL {
                                if let Err(err) =
                                    PubRel::recv(&buf, size, &self)
                                {
                                    error!("{}", err);
                                }
                                continue;
                            };
                            if msg_type == MSG_TYPE_PUBACK {
                                let _result = PubAck::recv(&buf, size, &self);
                                continue;
                            };
                            if msg_type == MSG_TYPE_PINGREQ {
                                if let Err(err) =
                                    PingReq::recv(&buf, size, &self, msg_header)
                                {
                                    error!("{}", err);
                                }
                                continue;
                            }
                            if msg_type == MSG_TYPE_SUBACK {
                                let _result = SubAck::recv(&buf, size, &self);
                                continue;
                            };
                            if msg_type == MSG_TYPE_SUBSCRIBE {
                                let _result = Subscribe::recv(
                                    &buf, size, &self, msg_header,
                                );
                                continue;
                            };
                            if msg_type == MSG_TYPE_DISCONNECT {
                                let _result =
                                    Disconnect::recv(&buf, size, &mut self);
                                continue;
                            };
                            if msg_type == MSG_TYPE_WILL_TOPIC {
                                if let Err(why) = WillTopic::recv(&buf, size, &self) {
                                    error!("{}", why);
                                }
                                continue;
                            }
                            if msg_type == MSG_TYPE_WILL_MSG {
                                if let Err(why) = WillMsg::recv(&buf, size, &self) {
                                    error!("{}", why);
                                }
                                continue;
                            }
                            if msg_type == MSG_TYPE_CONNACK {
                                match ConnAck::recv(&buf, size, &self) {
                                    // Broker shouldn't receive ConnAck
                                    // because it doesn't send Connect for now.
                                    Ok(_) => {
                                        error!("Broker shouldn't receive ConnAck {:?}", addr);
                                    }
                                    Err(why) => error!("ConnAck {:?}", why),
                                }
                                continue;
                            };
                            error!( "{}", eformat!( addr, "message type not supported:", msg_type));
                        } else {
                            // New connection, not in the connection hashmap.
                            if msg_type == MSG_TYPE_CONNECT {
                                if let Err(err) = Connect::recv(
                                    &buf, size, &mut self, msg_header,
                                ) {
                                    error!("{}", err);
                                }
                                //let clone_socket = socket.try_clone().expect("couldn't clone the socket");
                                // clone_socket.connect(addr).unwrap();
                                continue;
                            }
                            if msg_type == MSG_TYPE_PUBLISH {
                                if let Err(err) = Publish::recv(
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
                    }
                    Err(why) => {
                        error!("{}", why);
                    }
                }
            }
        });
        */
        let builder = thread::Builder::new().name("transmit_rx_thread".into());
        // process input datagram from network
        let egress_tx = self.egress_tx.clone();
        let _transmit_rx_thread = builder.spawn(move || loop {
            match self_transmit.transmit_rx.recv() {
                Ok((addr, bytes)) => {
                    // TODO DTLS
                    dbg!((addr, &bytes));

                    let new_bytes = bytes.clone();
                    egress_tx.send((addr, new_bytes)).unwrap();

                    match socket_tx.send_to(&bytes[..], addr) {
                        Ok(size) if size == bytes.len() => (),
                        Ok(size) => {
                            error!(
                                "send_to: {} bytes sent, but {} bytes expected",
                                size,
                                bytes.len()
                            );
                        }
                        Err(why) => {
                            error!("{}", why);
                        }
                    }
                }
                Err(why) => {
                    println!("channel_rx_thread: {}", why);
                }
            }
        });
    }

    /*
    pub fn connect(mut self, flags: u8, client_id: String, socket: UdpSocket) {
        let self_time_wheel = self.clone();
        let self_transmit = self.clone();
        let socket_tx = socket.try_clone().expect("couldn't clone the socket");
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
        */

    /* XXX TODO client code.
    pub fn subscribe(
        &self,
        topic: String,
        msg_id: u16,
        qos: u8,
        retain: u8,
    ) -> &Receiver<Publish> {
        let _result = Subscribe::send(topic, msg_id, qos, retain, self);
        &self.subscribe_rx
    }
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
