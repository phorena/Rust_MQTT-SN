#![warn(rust_2018_idioms)]
#[macro_use]
// use std::sync::mpsc::{Sender, Receiver};
// use std::sync::mpsc;
use core::fmt::Debug;
use std::net::UdpSocket;
use std::time::{Duration, SystemTime};
use std::{hint, thread};
use std::{net::SocketAddr};

use arr_macro::arr;
use log::*;
use nanoid::nanoid;
use simplelog::*;

use chrono::{Datelike, Local, Timelike};
use crossbeam::channel::{unbounded, Receiver, Sender};
use trace_var::trace_var;

// use DTLS::dtls_client::DtlsClient;
use client_lib::{
    ClientLib::MqttSnClient,
    //    ConnectionDb::ConnectionDb,
    //    SubscriberDb::SubscriberDb,
    //    Advertise::Advertise,
    //    Transfer::Transfer,MTU,
    //    TopicDb::TopicDb,
    //    MessageDb::MessageDb,
    flags:: {
        DupConst,
        DUP_FALSE,
        DUP_TRUE,

        QoSConst,
        QOS_LEVEL_0,
        QOS_LEVEL_1,
        QOS_LEVEL_2,
        QOS_LEVEL_3,

        RetainConst,
        RETAIN_FALSE,
        RETAIN_TRUE,

        WillConst,
        WILL_FALSE,
        WILL_TRUE,

        CleanSessionConst,
        CLEAN_SESSION_FALSE,
        CLEAN_SESSION_TRUE,

        TopicIdTypeConst,
        TOPIC_ID_TYPE_NORNAL,
        TOPIC_ID_TYPE_PRE_DEFINED,
        TOPIC_ID_TYPE_SHORT,
        TOPIC_ID_TYPE_RESERVED,
    },
};

fn generate_client_id() -> String {
    format!("exofense/{}", nanoid!())
}

static NTHREADS: i32 = 3;

// TODO move to utility lib



fn foo(i: u8) -> u8 {
    i + 1
}
fn bar(i: u8) -> u8 {
    i + 2
}
const MESSAGES: usize = 10;
const THREADS: usize = 4;

fn mpmc() {
    let (tx, rx) = unbounded();
    let rx2 = rx.clone();

    let tx_thread = thread::spawn(move || {
        let mut i = 0;
        loop {
            tx.send(i).unwrap();
            i += 1;
            thread::sleep(Duration::from_millis(1000));
        }
    });
    let rx_thread = thread::spawn(move || loop {
        dbg!(rx.recv());
    });

    let rx_thread2 = thread::spawn(move || loop {
        dbg!(rx2.recv());
    });
    /*
    rx_thread2.join().expect("The sender thread has panicked");
    rx_thread.join().expect("The sender thread has panicked");
    tx_thread.join().expect("The sender thread has panicked");
    */
}

fn main() {

    init_logging();
    let remote_addr = "127.0.0.1:60000".parse::<SocketAddr>().unwrap();
    let socket = UdpSocket::bind("0.0.0.0:0").unwrap();

    let client = MqttSnClient::new(remote_addr);
    let client_connect = client.clone();
    let client_main = client.clone();
    client.rx_loop(socket);
    let client_id = generate_client_id();
    client_connect.connect(client_id);
    client_main.subscribe("hello".to_string(), 1, QOS_LEVEL_1, RETAIN_FALSE);
    let mut i = 0;
    loop {
        let msg = format!("hello {:?}", i);
        client_main.publish(1, i, QOS_LEVEL_0, RETAIN_FALSE, msg.to_string());
        i += 1;
        thread::sleep(Duration::from_secs(2));
    }

    /*
    loop {
        println!("main");
        thread::sleep(Duration::from_secs(60));
    }
    */
}

fn init_logging() {
    TermLogger::init(
        LevelFilter::Info,
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )
    .unwrap();
}
