use crate::{
    broker_lib::MqttSnClient,
    client_id::ClientId,
    eformat,
    filter::{
        delete_topic_id, delete_topic_ids_with_socket_addr,
        get_subscribers_with_topic_id, remove_qos, subscribe_with_topic_id,
        try_insert_topic_name,
    },
    flags::{flag_is_clean_session, flag_is_will, RETAIN_FALSE},
    function,
    publish::Publish,
    TopicIdType,
};
// use log::*;
// use rand::Rng;
use bisetmap::BisetMap;
use bytes::{BufMut, Bytes, BytesMut};
use hashbrown::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::time::{SystemTime, UNIX_EPOCH};
use std::{sync::Arc, sync::Mutex};
use trace_caller::trace;
use uuid::v1::{Context, Timestamp};
use uuid::Uuid;

pub type ConnId = Uuid;

#[derive(Debug, Clone)]
pub enum StateEnum2 {
    ACTIVE,
    ASLEEP,
    AWAKE,
    DISCONNECTED,
    LOST,
}

/// Generate a new UUID
/// Use timestamp with nanoseconds precision
/// Use socket_addr, 6 bytes
/// Use Context for the namespace, etc.
pub fn generate_conn_id(
    socket_addr: SocketAddr,
    context_num: u16,
) -> Result<ConnId, String> {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    let time_stamp_nanos = since_the_epoch.as_nanos() as u32;
    let time_stamp_secs = since_the_epoch.as_secs();
    let context = Context::new(context_num);
    let time_stamp =
        Timestamp::from_unix(&context, time_stamp_secs, time_stamp_nanos);
    let ip4_bytes: [u8; 4];
    let port_bytes: [u8; 2] = socket_addr.port().to_be_bytes();

    match socket_addr.ip() {
        IpAddr::V4(ip4) => ip4_bytes = ip4.octets(),
        IpAddr::V6(ip6) => {
            let msg = format!(
                "ipv6: {}, segments: {:?} not supported",
                ip6,
                ip6.segments()
            );
            return Err(msg);
        }
    }
    let socket_addr_bytes: [u8; 6] = [
        ip4_bytes[0],
        ip4_bytes[1],
        ip4_bytes[2],
        ip4_bytes[3],
        port_bytes[0],
        port_bytes[1],
    ];

    // TODO error handling
    let uuid = match Uuid::new_v1(time_stamp, &socket_addr_bytes) {
        Ok(uuid) => uuid,
        Err(e) => return Err(format!("{}", e)),
    };
    // dbg!((&context, time_stamp, uuid));
    Ok(uuid)
}

lazy_static! {
    // TODO add comments!!!
    static ref CONN_HASHMAP: Mutex<HashMap<SocketAddr, Connection>> =
        Mutex::new(HashMap::new());

    // TODO: for connection migration, when the client has a new socket_addr,
    //       use the ConnId to locate the connection.
    static ref CONN_ID_BISET_MAP: Mutex<BisetMap<ConnId, SocketAddr>> =
        Mutex::new(BisetMap::new());
}

/// A connection is CURRENT network connection a client connects to the server.
/// The filter is used to delete its global filters when the client disconnects
/// or unsubscribes.
/// In the future, the client might be able to connect to multiple servers and
/// move to a different network connection.

// TODO: remove later
// #[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Connection {
    pub socket_addr: SocketAddr,
    pub flags: u8,
    pub protocol_id: u8,
    pub duration: u16,
    pub client_id: Bytes,
    state: Arc<Mutex<StateEnum2>>,
    pub will_topic_id: Option<TopicIdType>,
    pub will_topic: Bytes, // *NOTE: this is a Bytes, not a BytesMut.
    pub will_message: Bytes,
    // TODO pub sleep_msg_vec: Vec<Bytes>,
}

/*
6.3 Clean session
With MQTT, when a client disconnects, its subscriptions are not deleted. They are persistent and valid for new
connections, until either they are explicitly un-subscribed by the client, or the client establishes a new connection
with the “clean session” flag set.
In MQTT-SN the meaning of a “clean session” is extended to the Will feature, i.e. not only the subscriptions
are persistent, but also the Will topic and the Will message. The two flags “CleanSession” and “Will” in the
CONNECT have then the following meanings:
• CleanSession=true, Will=true: The GW will delete all subscriptions and Will data related to the client,
and starts prompting for new Will topic and Will message.
• CleanSession=true, Will=false: The GW will delete all subscriptions and Will data related to the client,
and returns CONNACK (no prompting for Will topic and Will message).
• CleanSession=false, Will=true: The GW keeps all stored client’s data, but prompts for new Will topic and
Will message. The newly received Will data will overwrite the stored Will data.
• CleanSession=false, Will=false: The GW keeps all stored client’s data and returns CONNACK (no prompting for Will topic and Will message).
Note that if a client wants to delete only its Will data at connection setup, it could send a CONNECT message
with “CleanSession=false” and “Will=true”, and sends an empty WILLTOPIC message to the GW when prompted
to do so. It could also send a CONNECT message with “CleanSession=false” and “Will=false”, and use the
procedure of Section 6.4 to delete or modify the Will data.
*/

impl Connection {
    pub fn new(
        socket_addr: SocketAddr,
        flags: u8,
        protocol_id: u8,
        duration: u16,
        client_id: Bytes,
    ) -> Self {
        Connection {
            socket_addr,
            flags,
            protocol_id,
            duration,
            client_id: client_id.clone(),
            state: Arc::new(Mutex::new(StateEnum2::ACTIVE)),
            will_topic_id: None,
            will_topic: Bytes::new(),
            will_message: Bytes::new(),
        }
    }
    // Insert a new connection to the HashMap
    pub fn try_insert(
        socket_addr: SocketAddr,
        flags: u8,
        protocol_id: u8,
        duration: u16,
        client_id: Bytes,
    ) -> Result<(), String> {
        if ClientId::contains(&client_id, &socket_addr) {
            // An existing client with same the socket_addr reconnects
            Connection::update_state(&socket_addr, StateEnum2::ACTIVE)?;
            if flag_is_clean_session(flags) {
                // Delete all subscriptions
                let topic_id_vec =
                    delete_topic_ids_with_socket_addr(&socket_addr);
                for topic_id in topic_id_vec {
                    let _qos = remove_qos(&topic_id, &socket_addr);
                }
            }
            if flag_is_will(flags) {
                // Delete will data, will_topic_id from the connection struct
                // and subscription map.
                let will_topic_id =
                    Connection::delete_will_topic_id(&socket_addr)?;
                delete_topic_id(&will_topic_id);
            }
            return Ok(());
        }
        // For existing client_id with different socket_addr or new client_id.
        // Default values for a new will
        let mut will_topic_id = None;
        let mut will_topic = Bytes::new();
        let mut will_message = Bytes::new();
        // ClientId::get() should return one old_socket_addr, but the get() returns
        // vec. Use for loop to traverse.
        for old_socket_addr in ClientId::get(&client_id) {
            // Existing client id with different socket_addr
            // Possible client migration or restart.
            dbg!(old_socket_addr);
            // Move existing subscriptions for non-clean session
            if !flag_is_clean_session(flags) {
                // remove all the topic ids link to the old socket_addr
                let topic_id_vec =
                    delete_topic_ids_with_socket_addr(&old_socket_addr);
                for topic_id in topic_id_vec {
                    // remove each QoS entries
                    let qos = remove_qos(&topic_id, &old_socket_addr).unwrap();
                    // subscribe with new socket_addr
                    let _result =
                        subscribe_with_topic_id(socket_addr, topic_id, qos);
                }
            }
            // copy will data for will flag == false
            if !flag_is_will(flags) {
                match CONN_HASHMAP.lock().unwrap().get(&old_socket_addr) {
                    Some(conn) => {
                        will_topic_id = conn.will_topic_id;
                        will_topic = conn.will_topic.clone();
                        will_message = conn.will_message.clone();
                    }
                    None => {
                        return Err(eformat!(socket_addr, client_id));
                    }
                }
            }
        }
        // Initialize the connection with new socket_addr with
        // existing or new client_id.
        let conn = Connection {
            socket_addr,
            flags,
            protocol_id,
            duration,
            client_id: client_id.clone(),
            state: Arc::new(Mutex::new(StateEnum2::ACTIVE)),
            will_topic_id,
            will_topic,
            will_message,
            // TODO  sleep_msg_vec: Vec::new(),
        };
        let mut conn_hashmap = CONN_HASHMAP.lock().unwrap();
        ClientId::insert(client_id, socket_addr);
        if let Err(why) = conn_hashmap.try_insert(socket_addr, conn) {
            return Err(eformat!(
                socket_addr,
                why.entry.key(),
                "already exists."
            ));
        }
        Ok(())
    }
    pub fn get_state(socket_addr: &SocketAddr) -> Result<StateEnum2, String> {
        let conn_hashmap = CONN_HASHMAP.lock().unwrap();
        match conn_hashmap.get(socket_addr) {
            Some(conn) => {
                let state = conn.state.lock().unwrap().clone();
                Ok(state)
            }
            None => Err(eformat!(socket_addr, "state not found.")),
        }
    }
    pub fn update_state(
        socket_addr: &SocketAddr,
        new_state: StateEnum2,
    ) -> Result<(), String> {
        let mut conn_hashmap = CONN_HASHMAP.lock().unwrap();
        match conn_hashmap.get_mut(socket_addr) {
            Some(conn) => {
                *conn.state.lock().unwrap() = new_state;
                Ok(())
            }
            None => Err(eformat!(socket_addr, "state not found.")),
        }
    }
    pub fn contains_key(socket_addr: SocketAddr) -> bool {
        CONN_HASHMAP.lock().unwrap().contains_key(&socket_addr)
    }
    #[trace]
    pub fn remove(socket_addr: SocketAddr) -> Result<Connection, String> {
        let mut conn_hashmap = CONN_HASHMAP.lock().unwrap();
        match conn_hashmap.remove(&socket_addr) {
            Some(val) => Ok(val),
            None => Err(eformat!(socket_addr, "not found.")),
        }
    }
    // Update will topic to an existing connection
    pub fn update_will_topic(
        socket_addr: SocketAddr,
        topic: String,
    ) -> Result<(), String> {
        let mut conn_hashmap = CONN_HASHMAP.lock().unwrap();
        match conn_hashmap.get_mut(&socket_addr) {
            Some(conn) => {
                conn.will_topic = Bytes::from(topic.clone());
                let topic_id = try_insert_topic_name(topic)?;
                conn.will_topic_id = Some(topic_id);
                Ok(())
            }
            None => Err(eformat!(socket_addr, "not found.")),
        }
    }
    pub fn update_will_msg(
        socket_addr: SocketAddr,
        message: String,
    ) -> Result<(), String> {
        let mut conn_hashmap = CONN_HASHMAP.lock().unwrap();
        match conn_hashmap.get_mut(&socket_addr) {
            Some(conn) => {
                conn.will_message = Bytes::from(message);
                Ok(())
            }
            None => Err(eformat!(socket_addr, "not found.")),
        }
    }
    pub fn delete_will_topic_id(
        socket_addr: &SocketAddr,
    ) -> Result<TopicIdType, String> {
        let mut conn_hashmap = CONN_HASHMAP.lock().unwrap();
        match conn_hashmap.get_mut(socket_addr) {
            Some(conn) => {
                let topic_id = conn.will_topic_id;
                conn.will_topic_id = None;
                Ok(topic_id.unwrap())
            }
            None => Err(eformat!(socket_addr, "not found.")),
        }
    }
    pub fn publish_will(self, client: &MqttSnClient) -> Result<(), String> {
        if let Some(topic_id) = self.will_topic_id {
            let subscriber_vec = get_subscribers_with_topic_id(topic_id);
            for subscriber in subscriber_vec {
                // Can't return error, because not all subscribers will have error.
                // TODO error for every subscriber/message
                // TODO use Bytes not BytesMut to eliminate clone/copy.
                // TODO new tx method to reduce have try_write() run once for every subscriber.
                let mut msg = BytesMut::new();
                msg.put(self.will_message.clone()); // TODO replace BytesMut with Bytes because clone doesn't copy data in Bytes
                let _result = Publish::send(
                    topic_id,
                    0, // TODO what is the msg_id?
                    subscriber.qos,
                    RETAIN_FALSE,
                    msg,
                    client,
                    subscriber.socket_addr,
                );
            }
        }
        Ok(())
    }

    pub fn debug() {
        let conn_hashmap = CONN_HASHMAP.lock().unwrap();
        dbg!(conn_hashmap);
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_conn_hashmap() {

        /*
        use std::net::SocketAddr;
        // insert first connection
        let socket = "127.0.0.1:1200".parse::<SocketAddr>().unwrap();
        let connection = super::Connection::new(socket, 0).unwrap();
        let result = super::connection_insert(connection);
        assert!(result.is_ok());

        // insert duplicate, should fail.
        let socket = "127.0.0.1:1200".parse::<SocketAddr>().unwrap();
        let connection = super::Connection::new(socket, 0).unwrap();
        let result = super::connection_insert(connection);
        assert!(!result.is_ok());
        dbg!(result);

        // insert different socket_addr, should succeed.
        let socket = "127.0.0.2:1200".parse::<SocketAddr>().unwrap();
        let connection = super::Connection::new(socket, 0).unwrap();
        let result = super::connection_insert(connection);
        assert!(result.is_ok());
        dbg!(super::CONN_HASHMAP.lock().unwrap());

        // insert concrete topic to existing socket_addr/connection, should succeed.
        let socket = "127.0.0.2:1200".parse::<SocketAddr>().unwrap();
        let result = super::connection_filter_insert("test", socket);
        assert!(result.is_ok());
        dbg!(super::CONN_HASHMAP.lock().unwrap());

        // insert filter to non-existing socket_addr, should fail.
        let socket_new = "127.0.0.99:1200".parse::<SocketAddr>().unwrap();
        let result = super::connection_filter_insert("test", socket_new);
        assert!(!result.is_ok());
        dbg!(result);

        // insert duplicate filter to existing socket_addr, should fail.
        let socket = "127.0.0.2:1200".parse::<SocketAddr>().unwrap();
        let result = super::connection_filter_insert("test", socket);
        assert!(!result.is_ok());
        dbg!(result);

        // insert wildcard filter to existing socket_addr/connection, should succeed.
        let socket = "127.0.0.2:1200".parse::<SocketAddr>().unwrap();
        let result = super::connection_filter_insert("test/#", socket);
        assert!(result.is_ok());
        dbg!(super::CONN_HASHMAP.lock().unwrap());
        */
    }
    #[test]
    fn test_generate_uuid() {
        use std::net::{IpAddr, Ipv6Addr, SocketAddr};

        let socket = SocketAddr::new(
            IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 65535, 0, 1)),
            8080,
        );
        let id = super::generate_conn_id(socket, 0);
        assert!(!id.is_ok());
        dbg!(id);

        let socket = "127.0.0.1:1200".parse::<SocketAddr>().unwrap();
        let id = super::generate_conn_id(socket, 0);
        assert!(id.is_ok());
        dbg!(id);

        let socket = "127.0.0.1:1200".parse::<SocketAddr>().unwrap();
        let id = super::generate_conn_id(socket, 1);
        assert!(id.is_ok());
        dbg!(id);

        let socket = "127.0.0.2:1200".parse::<SocketAddr>().unwrap();
        let id = super::generate_conn_id(socket, 0);
        assert!(id.is_ok());
        dbg!(id);
    }
}
