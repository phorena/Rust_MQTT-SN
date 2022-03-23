use crate::{
    Publish::Publish,
    TimingWheel2::{RetransmitHeader},
};
use bytes::{BytesMut};
use crossbeam::channel::{unbounded, Receiver, Sender};
use std::{net::SocketAddr, };

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
    // For transmiting data to the remote address.
    pub transmit_tx: Sender<TransChannelData>,
    pub transmit_rx: Receiver<TransChannelData>,
    // For cancel a scheduled retransmission on the timing wheel.
    pub cancel_tx: Sender<RetransmitHeader>,
    pub cancel_rx: Receiver<RetransmitHeader>,
    // For schedule a retransmission.
    pub schedule_tx: Sender<ScheduleChannelData>,
    pub schedule_rx: Receiver<ScheduleChannelData>,
    // For receiving publish messages from subscribed messages.
    // Mostly for the client.
    pub subscribe_tx: Sender<Publish>,
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
