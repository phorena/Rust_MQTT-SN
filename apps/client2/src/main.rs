#![warn(rust_2018_idioms)]
#![allow(unused_imports)]
#[macro_use]
// use std::sync::mpsc::{Sender, Receiver};
// use std::sync::mpsc;
use core::fmt::Debug;
use std::net::SocketAddr;
use std::net::UdpSocket;
use std::time::{Duration, SystemTime};
use std::{hint, thread};

use arr_macro::arr;
use log::*;
use nanoid::nanoid;
use simplelog::*;

use chrono::{Datelike, Local, Timelike};
use crossbeam::channel::{unbounded, Receiver, Sender};
use trace_var::trace_var;

use bytes::{BufMut, BytesMut};

// use DTLS::dtls_client::DtlsClient;
use client_lib::{
    //    ConnectionDb::ConnectionDb,
    //    SubscriberDb::SubscriberDb,
    //    Advertise::Advertise,
    //    Transfer::Transfer,MTU,
    //    TopicDb::TopicDb,
    //    MessageDb::MessageDb,
    flags::{
        CleanSessionConst, DupConst, QoSConst, RetainConst, TopicIdTypeConst,
        WillConst, CLEAN_SESSION_FALSE, CLEAN_SESSION_TRUE, DUP_FALSE,
        DUP_TRUE, QOS_LEVEL_0, QOS_LEVEL_1, QOS_LEVEL_2, QOS_LEVEL_3,
        RETAIN_FALSE, RETAIN_TRUE, TOPIC_ID_TYPE_NORNAL,
        TOPIC_ID_TYPE_PRE_DEFINED, TOPIC_ID_TYPE_RESERVED, TOPIC_ID_TYPE_SHORT,
        WILL_FALSE, WILL_TRUE,
    },
    ClientLib::MqttSnClient,
};

fn generate_client_id() -> String {
    format!("exofense/{}", nanoid!())
}

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
    let client_sub = client.clone();
    let client_id = generate_client_id();
    client_connect.connect(client_id, socket);
    client_main.subscribe("hello".to_string(), 1, QOS_LEVEL_0, RETAIN_FALSE);
    client_main.subscribe("hello2".to_string(), 2, QOS_LEVEL_0, RETAIN_FALSE);
    let mut i = 0;

    // This thread reads the channel for all subscribed topics.
    // The struct Publish is recv.
    // TODO return error for subscribe and publish function calls.
    let rx_thread2 = thread::spawn(move || loop {
        dbg!(client_sub.subscribe_rx.recv());
    });

    let publish_thread = thread::spawn(move || loop {
        let msg = format!("hi {:?}", i);
        let msg2 = format!("hi {:?}", i + 1000);
        client_main.publish(1, i, QOS_LEVEL_0, RETAIN_TRUE, msg.to_string());
        client_main.publish(2, i, QOS_LEVEL_0, RETAIN_FALSE, msg2.to_string());
        i += 1;
        thread::sleep(Duration::from_secs(2));
    });
    rx_thread2.join().expect("The sender thread has panicked");
    publish_thread
        .join()
        .expect("The sender thread has panicked");
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
