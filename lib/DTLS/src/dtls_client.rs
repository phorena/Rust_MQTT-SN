use anyhow::Result;
//use std::io::Write;
use std::sync::Arc;
// use std::time::Duration;
use std::io;
use tokio::net::UdpSocket;
use webrtc_dtls::{
    config::Config, conn::DTLSConn, crypto::Certificate,
    extension::extension_use_srtp::SrtpProtectionProfile,
};
use webrtc_util::Conn;

use mqtt_sn_lib::{
    ConnectionDb::ConnectionDb,
    Functions::{connect, process_input},
    SubscriberDb::SubscriberDb,
    TopicDb::TopicDb,
    Transfer::Transfer,
    MTU,
};
//const MTU: usize = 1500;
use log::*;
use std::net::SocketAddr;

pub struct DtlsClient {
    pub server_address: SocketAddr,
    //socket: UdpSocket,
    // buf: Vec<u8>,
    pub buf: [u8; MTU],
    pub to_send: Option<(usize, SocketAddr)>,
    pub connection_db: ConnectionDb,
    pub subscriber_db: SubscriberDb,
    pub topic_db: TopicDb,
    //message_db: MessageDb,
}

impl DtlsClient {
    pub async fn run(self) -> Result<(), io::Error> {
        let DtlsClient {
            server_address,
            mut buf,
            mut to_send,
            connection_db,
            subscriber_db,
            topic_db,
            //message_db,
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
            //message_db: message_db.clone(),
            topic_id_counter: 1,
            input_bytes: Vec::new(),
            size: 0,
        };

        let conn = Arc::new(UdpSocket::bind("0.0.0.0:0").await?);
        //let arc_socket = Arc::new(&socket);
        //let clone_socket = Arc::clone(&arc_socket);

        let buf2 = connect(&conn);
        // dbg!(&socket);
        //println!("Client address: {}", conn.local_addr()?);
        conn.connect(server_address).await.unwrap();

        let cfg = Config {
            srtp_protection_profiles: vec![SrtpProtectionProfile::Srtp_Aes128_Cm_Hmac_Sha1_80],
            ..Default::default()
        };
        let dtls_conn = create_client(conn, cfg, true).await.unwrap();

        if cfg!(test) {
            return Ok(());
        }

        info!("DTLS connection successful");

        dtls_conn.send(&buf2).await.unwrap();

        //println!("Connected to server!");

        //let buf2 = subscribe("hello".to_string());
        //clone_socket.send(&buf2).await?;

        // buffer for sending
        // butes_buff:BytesMut, for flexibility, functions can write multiple times,
        // clear it etc...
        loop {
            while let Some(buf) = transfer.egress_buffers.pop() {
                dbg!(buf.clone());
                let (peer, bytes_buf) = buf;
                dtls_conn.send(&bytes_buf).await.unwrap();
                //let amt = socket.send_to(&bytes_buf[..], &peer).await?;
                //info!("send_to {} to {}", amt, peer);
                info!("Sent to {}", peer);
            }

            // If we're here then `to_send` is `None`, so we take a look for the
            // next message we're going to echo back.
            to_send = Some(dtls_conn.recv_from(&mut buf).await.unwrap());
            if let Some((size, peer)) = to_send.clone() {
                info!("recv_from: {:?}", peer);
                transfer.input_bytes = buf.to_vec();
                transfer.peer = peer;
                transfer.size = size;
                process_input(&buf, size, &mut transfer);
            }
        }
    }
}

async fn create_client(
    ca: Arc<dyn Conn + Send + Sync>,
    mut cfg: Config,
    generate_certificate: bool,
) -> Result<impl Conn> {
    if generate_certificate {
        let client_cert = Certificate::generate_self_signed(vec!["localhost".to_owned()])?;
        cfg.certificates = vec![client_cert];
    }

    cfg.insecure_skip_verify = true;
    DTLSConn::new(ca, cfg, true, None).await
}
