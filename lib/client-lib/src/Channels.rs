use crate::{
    Publish::Publish,
    StateMachine::{StateMachine, STATE_DISCONNECT},
    TimingWheel2::{RetransmitData, RetransmitHeader},
    MSG_TYPE_CONNACK, MSG_TYPE_CONNECT, MSG_TYPE_PUBACK, MSG_TYPE_PUBLISH,
    MSG_TYPE_PUBREC, MSG_TYPE_SUBACK, MSG_TYPE_SUBSCRIBE,
};
use bytes::{Bytes, BytesMut};
use crossbeam::channel::{unbounded, Receiver, Sender};
use std::{net::SocketAddr, sync::Arc, sync::Mutex};

pub struct TransChannelData {
    addr: SocketAddr,
    bytes: BytesMut,
}

pub struct ScheduleChannelData {
    header: RetransmitHeader,
    bytes: BytesMut,
}

#[derive(Debug, Clone)]
pub struct Channels {
    pub transmit_tx: Sender<TransChannelData>,
    pub cancel_tx: Sender<RetransmitHeader>,
    pub schedule_tx: Sender<ScheduleChannelData>,
    pub subscribe_tx: Sender<Publish>,
    pub transmit_rx: Receiver<TransChannelData>,
    pub cancel_rx: Receiver<RetransmitHeader>,
    pub schedule_rx: Receiver<ScheduleChannelData>,
    pub subscribe_rx: Receiver<Publish>,
}

impl Channels {
    pub fn new() -> Self {
        let (cancel_tx, cancel_rx): (
            Sender<RetransmitHeader>,
            Receiver<RetransmitHeader>,
        ) = unbounded();
        let (schedule_tx, schedule_rx): (
            Sender<ScheduleChannelData>,
            Receiver<ScheduleChannelData>,
        ) = unbounded();
        let (transmit_tx, transmit_rx): (
            Sender<TransChannelData>,
            Receiver<TransChannelData>,
        ) = unbounded();
        let (subscribe_tx, subscribe_rx): (Sender<Publish>, Receiver<Publish>) =
            unbounded();
        Channels {
            schedule_tx,
            schedule_rx,
            cancel_tx,
            cancel_rx,
            transmit_tx,
            transmit_rx,
            subscribe_tx,
            subscribe_rx,
        }
    }
}
