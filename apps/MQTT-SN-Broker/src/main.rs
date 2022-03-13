#![warn(rust_2018_idioms)]
//
// NOTE:
// 0. This is prototyping/sample source code for you to get started.
//    Method inputs and returns are not verified.
//    Few logs are generated.
//
// 1. When broker receives a SUBSCRIBE message type, includes a Topic Name (string of bytes).
//    To save bandwidth, the broker responds with SUBACK with a Topic Id (u16)
//    respresents the Topic Name.
//    The TopicDb stores the TopicName(key):TopicId(value).
//    The SubscriberDb stores the TopicId(key): Subscribers(value, HashMap<SocketAddre, u8>).
//    If the Topic Name is not in the TopicDb, a new entry is created.
//    Subscriber (SocketAddr) is added to the TopicDb.
//
// 2. When the broker receives a PUBLISH message type, includes a message and the Topic Id.
//    A PUBLISH message is sent to all subscribers that subscribe to that Topic Id.
//
// 3. The out going messages are store in the egress_buffers Vec<(SocketAddr, BytesMut)>
//    because PUBLISH might send multiple messages to different peers. Other message types
//    reply to the sender with one message.
//
// 4. To process a message type:
//      1. Define the struct, similar to the MQTT-SN specification.
//
//         #[derive(Debug, Clone, Copy, Getters, Setters, MutGetters, CopyGetters, Default)]
//         #[getset(get, set)]
//         pub struct ConAck {
//             len: u8,
//             #[debug(format = "0x{:x}")]
//             msg_type: u8,
//             return_code: u8, // use enum for print
//         }
//
//      2. Define the state transition in the state_machine! macro.
//
//          ACTIVE => {
//              SUBSCRIBE => verify_subscribe ? ACTIVE[SUBACK] : ACTIVE[MSG_TYPE_ERR],
//              PUBLISH => verify_publish ? ACTIVE[PUBLISH] : ACTIVE[MSG_TYPE_ERR],
//                                           ^^^^             ^^^^^
//                         verify_publish -> true           : false
//         For some transition functions, the state will stay the same, but the action
//         will be different. Others will transition to different state.
//
//      3. Define the verify method for processing the state transition. See verifty_connect()
//         for details.
//
//          fn verify_connect(_state:StateEnum,
//                            input:MsgType,
//                            transfer:&mut Transfer,
//                            buf: &[u8],
//                            size: usize,
//                            ) -> bool {
//
// TODO
// 1. move databases to libraries
// 2. move each message type of fsm (state machine) to a separate file
//    For example, connect.rs for message type CONNECT

// use custom_debug::{Debug, hexbuf, hexbuf_str};
// use custom_debug::Debug;
// use getset::{CopyGetters, Getters, MutGetters, Setters};

use std::env;
use std::error::Error;
use tokio::net::UdpSocket;
use log::*;
use simplelog::*;
use std::str;
use tokio::time;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use tokio::task;

//const MTU: usize = 1500; // for Ethernet
const DEFAULT_MULTICAST_PORT: u16 = 60006;
const DEFAULT_MULTICAST: &str = "239.255.42.98";
const IP_ALL: [u8; 4] = [0, 0, 0, 0];

use std::io::{self};

//modules
use mqtt_sn_lib::{
    ConnectionDb::ConnectionDb, SubscriberDb::SubscriberDb, TopicDb::TopicDb,
    Transfer::Transfer, Functions::process_input, MTU,BroadcastAdvertise::BroadcastAdvertise
};

use DTLS::dtls_server::DtlsServer;

// TODO move to utility lib
macro_rules! function {
    () => {{
        fn f() {}
        fn type_name_of<T>(_: T) -> &'static str {
            std::any::type_name::<T>()
        }
        let name = type_name_of(f);
        &name[..name.len() - 3]
    }};
}

macro_rules! dbg_buf {
    ($buf:ident, $size:ident) => {
        let mut i: usize = 0;
        eprint!("[{}:{}] ", function!(), line!());
        while i < $size {
            eprint!("{:#04X?} ", $buf[i]);
            i += 1;
        }
        eprintln!("");
    };
}

// print function name instead of file name.
// https://stackoverflow.com/questions/65946195/understanding-the-dbg-macro-in-rust
macro_rules! dbg_fn {
    () => {
        $crate::eprintln!("[{}:{}]", function!(), line!());
    };
    ($val:expr $(,)?) => {
        // Use of `match` here is intentional because it affects the lifetimes
        // of temporaries - https://stackoverflow.com/a/48732525/1063961
        match $val {
            tmp => {
                // replace file!() with function!()
                eprintln!("[{}:{}] {} = {:#?}",
                    function!(), line!(), stringify!($val), &tmp);
                tmp
            }
        }
    };
    ($($val:expr),+ $(,)?) => {
        ($($dbg_fn!($val)),+,)
    };
}


struct Server {
    socket: UdpSocket,
    // buf: Vec<u8>,
    buf: [u8; MTU],
    to_send: Option<(usize, SocketAddr)>,
    connection_db: ConnectionDb,
    subscriber_db: SubscriberDb,
    topic_db: TopicDb,
}


impl Server {
    async fn run(self) -> Result<(), io::Error> {
        let Server {
            socket,
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

        // buffer for sending
        // butes_buff:BytesMut, for flexibility, functions can write multiple times,
        // clear it etc...
        loop {
            while let Some(buf) = transfer.egress_buffers.pop() {
                dbg!(buf.clone());
                let (peer, bytes_buf) = buf;
                let amt = socket.send_to(&bytes_buf[..], &peer).await?;
                info!("send_to {} to {}", amt, peer);
            }

            // If we're here then `to_send` is `None`, so we take a look for the
            // next message we're going to echo back.
            to_send = Some(socket.recv_from(&mut buf).await?);
            if let Some((size, peer)) = to_send.clone() {
                info!("recv_from: {:?}", peer);
                transfer.input_bytes = buf.to_vec();
                transfer.peer = peer;
                transfer.size = size;
                dbg_buf!(buf, size); // Moved this macro call from process_input() to here
                // TODO check for return value, including error
                // need to identify the client IP/port number with error
                process_input(&buf, size, &mut transfer);
            }
        }
    }
}

async fn task_that_takes_a_second() {
    // println!("task_that_takes_a_second");
    time::sleep(time::Duration::from_secs(1)).await
}

async fn task_that_takes_a_second2() {
    // println!("task_that_takes_a_second2");
    time::sleep(time::Duration::from_secs(1)).await
}

async fn timing_wheel() {
    let mut interval = time::interval(time::Duration::from_secs(2));
    for _i in 0..10000 {
        interval.tick().await;
        task_that_takes_a_second().await;
    }
}

async fn timing_wheel2() {
    let mut interval = time::interval(time::Duration::from_secs(2));
    for _i in 0..10000 {
        interval.tick().await;
        task_that_takes_a_second2().await;
    }
}

/// Bind socket to multicast address with IP_MULTICAST_LOOP and SO_REUSEADDR Enabled
fn bind_multicast(
    addr: &SocketAddrV4,
    multi_addr: &SocketAddrV4,
) -> Result<std::net::UdpSocket, Box<dyn Error>> {
    use socket2::{Domain, Protocol, Socket, Type};

    assert!(multi_addr.ip().is_multicast(), "Must be multcast address");

    let socket = Socket::new(Domain::ipv4(), Type::dgram(), Some(Protocol::udp()))?;

    socket.set_reuse_address(true)?;
    socket.bind(&socket2::SockAddr::from(*addr))?;
    socket.set_multicast_loop_v4(true)?;
    socket.join_multicast_v4(multi_addr.ip(), addr.ip())?;

    Ok(socket.into_udp_socket())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // test_db();
    // test_subs_db();
    //
    init_logging();
    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "0.0.0.0:60000".to_string());

    let socket = UdpSocket::bind(&addr).await?;
    dbg!(&socket);
    println!("Listening on: {}", socket.local_addr()?);

    let server = Server {
        socket,
        buf: [0u8; MTU],
        to_send: None,
        connection_db: ConnectionDb::new("/tmp/exo-sn-db".to_string()).unwrap(),
        subscriber_db: SubscriberDb::new(),
        topic_db: TopicDb::new(),
    };

    let server_address: SocketAddr = "0.0.0.0:61000".parse().unwrap();
    let dtls_server = DtlsServer {
        server_address,
        buf: [0u8; MTU],
        to_send: None,
        connection_db: ConnectionDb::new("/tmp/exo-sn-db3".to_string()).unwrap(),
        subscriber_db: SubscriberDb::new(),
        topic_db: TopicDb::new(),
    };

    let addr = SocketAddrV4::new(IP_ALL.into(), DEFAULT_MULTICAST_PORT);

    let multi_addr = SocketAddrV4::new(
        // matches.value_of("ip")
        DEFAULT_MULTICAST
            //    .unwrap()
            .parse::<Ipv4Addr>()
            .expect("Invalid IP"),
        DEFAULT_MULTICAST_PORT,
    );

    println!("Starting server on: {}", addr);
    println!("Multicast address: {}\n", multi_addr);

    let std_socket = bind_multicast(&addr, &multi_addr).expect("Failed to bind multicast socket");

    let socket = UdpSocket::from_std(std_socket).unwrap();

    let ss: SocketAddr = multi_addr.into();
    let broadcast_advertise = BroadcastAdvertise {
        socket,
        addr: ss,
        buf: [0u8; MTU], // Ethernet MTU
        to_send: None,
        connection_db: ConnectionDb::new("/tmp/exo-sn-db2".to_string()).unwrap(),
        subscriber_db: SubscriberDb::new(),
        topic_db: TopicDb::new(),
    };

    let resp1 = task::spawn(timing_wheel());
    let resp2 = task::spawn(timing_wheel2());
    // let broadcast_advertise_thread = task::spawn(broadcast_advertise.run());
    // This starts the server task.
    let broker_thread = task::spawn(server.run());
    let broker_thread2 = task::spawn(dtls_server.run());

    let _ = broker_thread2.await?;
    let _ = broker_thread.await?;
    // let _ = broadcast_advertise_thread.await?;
    let _ = resp1.await?;
    let _ = resp2.await?;

    Ok(())
}

fn init_logging() {
    TermLogger::init(
        LevelFilter::Debug,
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )
    .unwrap();
}
