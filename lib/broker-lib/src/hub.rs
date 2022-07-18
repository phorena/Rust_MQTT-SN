use bytes::Bytes;
use crossbeam::channel::Sender;
use hashbrown::HashMap;
use std::io::{BufRead, BufReader};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
use util::Conn;

use webrtc_dtls::Error;
// use async_channel::*;

const BUF_SIZE: usize = 8192;

/// Hub sends messages from ingress to processing channels.
#[derive(Clone)]
pub struct Hub {
    channel_tx: Arc<Sender<(SocketAddr, Bytes, Arc<dyn Conn + Send + Sync>)>>,
    conns: Arc<Mutex<HashMap<String, Arc<dyn Conn + Send + Sync>>>>,
}

impl Hub {
    pub fn new(
        channel_tx: Arc<
            Sender<(SocketAddr, Bytes, Arc<dyn Conn + Send + Sync>)>,
        >,
    ) -> Self {
        // pub fn new() -> Self {
        Hub {
            conns: Arc::new(Mutex::new(HashMap::new())),
            channel_tx,
        }
    }

    /// register adds a new conn to the Hub
    pub async fn register(&self, conn: Arc<dyn Conn + Send + Sync>) {
        println!("Connected to {}", conn.remote_addr().await.unwrap());

        if let Some(remote_addr) = conn.remote_addr().await {
            let mut conns = self.conns.lock().await;
            conns.insert(remote_addr.to_string(), Arc::clone(&conn));
        }

        let conns = Arc::clone(&self.conns);
        let channel_tx = Arc::clone(&self.channel_tx);
        tokio::spawn(async move {
            let _ = Hub::read_loop(
                conn.remote_addr().await.unwrap(),
                channel_tx,
                conns,
                conn,
            )
            .await;
        });
    }

    /// register adds a new conn to the Hub
    pub async fn get_conn(
        &self,
        socket_addr: SocketAddr,
    ) -> Option<Arc<dyn Conn + Send + Sync>> {
        let conns = self.conns.lock().await;
        match conns.get(&socket_addr.to_string()) {
            Some(conn) => Some(Arc::clone(conn)),
            None => None,
        }
    }

    async fn read_loop(
        remote_addr: SocketAddr,
        channel_tx: Arc<
            Sender<(SocketAddr, Bytes, Arc<dyn Conn + Send + Sync>)>,
        >,
        conns: Arc<Mutex<HashMap<String, Arc<dyn Conn + Send + Sync>>>>,
        conn: Arc<dyn Conn + Send + Sync>,
    ) -> Result<(), Error> {
        let mut b = vec![0u8; BUF_SIZE];

        while let Ok(n) = conn.recv(&mut b).await {
            let msg = String::from_utf8(b[..n].to_vec())?;
            let bytes = Bytes::from(msg.to_owned());
            let conn2 = Arc::clone(&conn);
            // let result = channel_tx.send((remote_addr, bytes, conn2)).await;
            let result = channel_tx.send((remote_addr, bytes, conn2));
            dbg!(result);
            print!("Got message: {}", msg);
        }

        Hub::unregister(conns, conn).await
    }

    async fn unregister(
        conns: Arc<Mutex<HashMap<String, Arc<dyn Conn + Send + Sync>>>>,
        conn: Arc<dyn Conn + Send + Sync>,
    ) -> Result<(), Error> {
        if let Some(remote_addr) = conn.remote_addr().await {
            {
                let mut cs = conns.lock().await;
                cs.remove(&remote_addr.to_string());
            }

            if let Err(err) = conn.close().await {
                println!(
                    "Failed to disconnect: {} with err {}",
                    remote_addr, err
                );
            } else {
                println!("Disconnected: {} ", remote_addr);
            }
        }

        Ok(())
    }

    async fn broadcast(&self, msg: &[u8]) {
        let conns = self.conns.lock().await;
        for conn in conns.values() {
            if let Err(err) = conn.send(msg).await {
                println!(
                    "Failed to write message to {:?}: {}",
                    conn.remote_addr().await,
                    err
                );
            }
        }
    }

    /// Chat starts the stdin readloop to dispatch messages to the hub
    pub async fn chat(&self) {
        let input = std::io::stdin();
        let mut reader = BufReader::new(input.lock());
        loop {
            let mut msg = String::new();
            match reader.read_line(&mut msg) {
                Ok(0) => return,
                Err(err) => {
                    println!("stdin read err: {}", err);
                    return;
                }
                _ => {}
            };
            if msg.trim() == "exit" {
                return;
            }
            self.broadcast(msg.as_bytes()).await;
        }
    }
}
