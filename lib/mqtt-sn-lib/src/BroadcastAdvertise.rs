use tokio::net::UdpSocket;
use std::net::SocketAddr;
use std::*;
use tokio::time;
use bytes::BytesMut;

use crate::MTU;
use log::*;
use crate::ConnectionDb::ConnectionDb;
use crate::SubscriberDb::SubscriberDb;
use crate::Transfer::Transfer;
use crate::TopicDb::TopicDb;
use crate::Advertise::Advertise;
use crate::MsgType::MsgType;

static BROADCAST_INTERVAL: u8 = 8;
// Boardcast every n minutes
pub struct BroadcastAdvertise {
    pub socket: UdpSocket,
    pub addr: SocketAddr,
    // buf: Vec<u8>,
    pub buf: [u8; MTU],
    pub to_send: Option<(usize, SocketAddr)>,
    pub connection_db: ConnectionDb,
    pub subscriber_db: SubscriberDb,
    pub topic_db: TopicDb,
}

impl BroadcastAdvertise {
    pub async fn run(self) -> Result<(), io::Error> {
        let BroadcastAdvertise {
            socket,
            addr,
            mut buf,
            mut to_send,
            connection_db,
            subscriber_db,
            topic_db,
        } = self;

        let peer: SocketAddr = "127.0.0.1:80"
            .parse()
            .expect("Unable to parse socket address");
        let mut transfer = Transfer {
            peer,
            egress_buffers: Vec::new(),
            subscriber_db: subscriber_db.clone(),
            connection_db: connection_db.clone(),
            topic_db: topic_db.clone(),
            topic_id_counter: 1,
            input_bytes: Vec::new(),
            size: 0,
        };

        let advertise = Advertise {
            len: 5,
            msg_type: MsgType::ADVERTISE as u8,
            gw_id: 9,
            duration: BROADCAST_INTERVAL as u16,
        };
        dbg!(advertise.clone());
        let advertise_len = advertise.len;

        let mut bytes_buf = BytesMut::with_capacity(MTU);
        advertise.try_write(&mut bytes_buf);
        let mut interval = time::interval(time::Duration::from_secs(BROADCAST_INTERVAL as u64));
        loop {
            interval.tick().await;
            let amt = socket.send_to(&bytes_buf[..], &addr).await?;
            if amt == advertise_len as usize {
                info!("broadcast advertise message to {}", addr);
            } else {
                error!("broadcast advertise message length not match: amt: {}", amt);
            }
        }
    }
}