use std::net::SocketAddr;
use std::net::UdpSocket;
use std::thread;

use bytes::BytesMut;
use core::fmt::Debug;
use crossbeam::channel::{unbounded, Receiver, Sender};
// use util::conn::*;
use log::*;
use std::sync::{Arc, Mutex};
use util::{replay_detector::*, Conn};

use crate::{
    advertise::*,
    conn_ack::ConnAck,
    connect::Connect,
    connection::Connection,
    connection::StateEnum2,
    dbg_buf,
    disconnect::Disconnect,
    eformat,
    function,
    gw_info::GwInfo,
    keep_alive::KeepAliveTimeWheel,
    msg_hdr::MsgHeader,
    ping_req::PingReq,
    // Connection::ConnHashMap,
    pub_ack::PubAck,
    pub_rel::PubRel,
    publish::Publish,
    retransmit::RetransTimeWheel,
    // search_gw::SearchGw,
    sub_ack::SubAck,
    subscribe::Subscribe,
    will_msg::WillMsg,
    will_topic::WillTopic,
    MSG_TYPE_CONNACK,
    MSG_TYPE_CONNECT,
    MSG_TYPE_DISCONNECT,
    MSG_TYPE_PINGREQ,
    MSG_TYPE_PUBACK,
    MSG_TYPE_PUBLISH,
    MSG_TYPE_PUBREL,
    MSG_TYPE_SUBACK,
    MSG_TYPE_SUBSCRIBE,
    MSG_TYPE_WILL_MSG,
    MSG_TYPE_WILL_TOPIC,
};
// use trace_var::trace_var;

#[derive(Debug, Clone)]
pub enum StateEnum3 {
    ACTIVE,
    ASLEEP,
    AWAKE,
    DISCONNECTED,
    LOST,
}
// #[derive(Debug, Clone)]
#[derive(Clone)]
pub struct MqttSn {
    pub conn: Arc<dyn Conn + Send + Sync>,
    pub remote_addr: SocketAddr,
    pub state: Arc<Mutex<StateEnum3>>,
}

impl MqttSn {
    // TODO change Client to Broker
    // TODO change remote_addr to local_addr
    pub fn new(
        remote_addr: SocketAddr,
        conn: Arc<dyn Conn + Send + Sync>,
    ) -> Self {
        MqttSn {
            conn,
            remote_addr,
            // state: Mutex::new(StateEnum2::DISCONNECTED)),
            state: Arc::new(Mutex::new(StateEnum3::DISCONNECTED)),
        }
    }

    pub async fn process_incoming(
        &mut self,
        buf: &[u8],
        size: usize,
        sock_addr: SocketAddr,
    ) -> Result<(), String> {
        let _result = KeepAliveTimeWheel::reschedule(sock_addr);
        let msg_header = match MsgHeader::try_read(&buf, size) {
            Ok(header) => header,
            Err(e) => return Err(e),
        };
        let msg_type = msg_header.msg_type;
        dbg!(&msg_header);
        dbg_buf!(buf, size);
        if let Ok(state) = self.state.lock() {
            match state {
                StateEnum3::ACTIVE => {
                    if msg_type == MSG_TYPE_CONNACK {
                        let conn_ack = ConnAck::try_read(&buf, size)?;
                        dbg!(conn_ack);
                        if conn_ack.return_code == 0 {
                            self.state.lock().unwrap().replace(StateEnum3::AWAKE);
        if let Ok(s) = self.state.lock(){
        match s {
            StateEnum3::DISCONNECTED => {
                dbg!("DISCONNECTED");
            }
            StateEnum3::ACTIVE => {
                dbg!("ACTIVE");
            }
            _ => {
                dbg!("UNKNOWN");
            }
        }
    }
        if msg_type == MSG_TYPE_CONNECT {
            if let Err(err) = Connect::recv2(&buf, size, msg_header, &mut self)
            {
                error!("{}", err);
            }
            return Ok(());
        }
        return Ok(());
    }
}
