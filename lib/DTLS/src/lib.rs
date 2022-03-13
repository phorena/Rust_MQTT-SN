pub mod dtls_client;
pub mod dtls_server;

#[cfg(test)]
mod tests {
    use super::*;
    use dtls_client::DtlsClient;
    use dtls_server::DtlsServer;
    use mqtt_sn_lib::{
        ConnectionDb::ConnectionDb, SubscriberDb::SubscriberDb, TopicDb::TopicDb, MTU,
    };
    use tokio::task;

    #[tokio::test]
    async fn test_dtls_connection() {
        let dtls_server = DtlsServer {
            server_address: "127.0.0.1:61000".parse().unwrap(),
            buf: [0u8; MTU],
            to_send: None,
            connection_db: ConnectionDb::new("mqtt-sn-db".to_string()).unwrap(),
            subscriber_db: SubscriberDb::new(),
            topic_db: TopicDb::new(),
        };

        let server_thread = task::spawn(dtls_server.run());

        let dtls_client = DtlsClient {
            server_address: "127.0.0.1:61000".parse().unwrap(),
            buf: [0u8; MTU],
            to_send: None,
            connection_db: mqtt_sn_lib::ConnectionDb::ConnectionDb::new(
                "connection-db".to_string(),
            )
            .unwrap(),
            subscriber_db: mqtt_sn_lib::SubscriberDb::SubscriberDb::new(),
            topic_db: mqtt_sn_lib::TopicDb::TopicDb::new(),
            //message_db: MessageDb::new("message-db".to_string()).unwrap(),
        };

        let client_thread = task::spawn(dtls_client.run());

        server_thread.await.unwrap();
        client_thread.await.unwrap();
    }
}
