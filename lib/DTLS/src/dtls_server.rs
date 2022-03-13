use anyhow::Result;
use std::io;
use std::sync::Arc;
// use tokio::signal;
use log::*;
use mqtt_sn_lib::{
    ConnectionDb::ConnectionDb, Functions::process_input, SubscriberDb, TopicDb,
    Transfer::Transfer, MTU,
};
use std::net::SocketAddr;
use webrtc_dtls::{
    config::Config, crypto::Certificate, extension::extension_use_srtp::SrtpProtectionProfile,
    listener::listen,
};
use webrtc_util::conn::*;

//const MTU: usize = 1500;

pub struct DtlsServer {
    pub server_address: SocketAddr,
    // buf: Vec<u8>,
    pub buf: [u8; MTU],
    pub to_send: Option<(usize, SocketAddr)>,
    pub connection_db: ConnectionDb,
    pub subscriber_db: SubscriberDb::SubscriberDb,
    pub topic_db: TopicDb::TopicDb,
}

impl DtlsServer {
    pub async fn run(self) -> Result<(), io::Error> {
        let cfg = Config {
            certificates: vec![
                Certificate::generate_self_signed(vec!["localhost".to_owned()]).unwrap(),
            ],
            srtp_protection_profiles: vec![SrtpProtectionProfile::Srtp_Aes128_Cm_Hmac_Sha1_80],
            ..Default::default()
        };

        let DtlsServer {
            server_address,
            mut buf,
            mut to_send,
            connection_db,
            subscriber_db,
            topic_db,
        } = self;

        let listener = Arc::new(listen(server_address, cfg).await.unwrap());

        info!("DTLS server running on {}", server_address);

        let peer: SocketAddr = "127.0.0.1:80"
            .parse()
            .expect("Unable to parse socket address");

        loop {
            tokio::select! {
            //    _= signal::ctrl_c() => {
            //         break;
            //     }

                result = listener.accept() => {
                    if let Ok(dtls_conn) = result {
                        if cfg!(test) {
                            return Ok(());
                        }
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

                        tokio::spawn(async move {
                            loop {
                                while let Some(buf) = transfer.egress_buffers.pop() {
                                    dbg!(buf.clone());
                                    let (peer, bytes_buf) = buf;
                                    let amt = dtls_conn.send(&bytes_buf[..]).await.unwrap();
                                    info!("send_to {} to {}", amt, peer);
                                }

                                to_send = Some(dtls_conn.recv_from(&mut buf).await.unwrap());
                                if let Some((size, peer)) = to_send.clone() {
                                    info!("recv_from: {:?}", peer);
                                    transfer.input_bytes = buf.to_vec();
                                    transfer.peer = peer;
                                    transfer.size = size;
                                    // dbg_buf!(buf, size);
                                    process_input(&buf, size, &mut transfer);
                                }
                            }
                        });
                    }
                }
            }
        }
        listener.close().await;
        Ok(())
    }
}
