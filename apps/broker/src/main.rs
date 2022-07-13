#![allow(unused_imports)]
// #[macro_use]
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
use std::sync::Arc;
use util::conn::*;
use webrtc_dtls::config::ExtendedMasterSecretType;
use webrtc_dtls::Error;
use webrtc_dtls::{config::Config, crypto::Certificate, listener::listen};
use env_logger::*;
use std::io::Write;
use clap::{App, AppSettings, Arg};
// use mongodb::{bson::doc, sync::Client};


// use DTLS::dtls_client::DtlsClient;
use broker_lib::{
    broker_lib::MqttSnClient,
    hub::Hub,
};
// use BrokerLib::MqttSnClient;

/*
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
    rx_thread2.join().expect("The sender thread has panicked");
    rx_thread.join().expect("The sender thread has panicked");
    tx_thread.join().expect("The sender thread has panicked");
}
    */
#[tokio::main]
async fn main() -> Result<(), Error> {

    // Get a handle to the cluster
    /*
    let client = Client::with_uri_str(
        "mongodb+srv://mongo-1001:JKLsWUuUnjdYbvem@cluster0.elom9.mongodb.net/?retryWrites=true&w=majority",
    ).unwrap();
    // Ping the server to see if you can connect to the cluster
    client
        .database("admin")
        .run_command(doc! {"ping": 1}, None);
    println!("Connected successfully.");
    // List the names of the databases in that cluster
    for db_name in client.list_database_names(None, None).unwrap() {
        println!("{}", db_name);
    }
    */

    env_logger::Builder::new()
        .format(|buf, record| {
            writeln!(
                buf,
                "{}:{} [{}] {} - {}",
                record.file().unwrap_or("unknown"),
                record.line().unwrap_or(0),
                record.level(),
                chrono::Local::now().format("%H:%M:%S.%6f"),
                record.args()
            )
        })
        .filter(None, log::LevelFilter::Trace)
        .init();


    let mut app = App::new("DTLS Server")
        .version("0.1.0")
        .author("Rain Liu <yliu@webrtc.rs>")
        .about("An example of DTLS Server")
        .setting(AppSettings::DeriveDisplayOrder)
        .setting(AppSettings::SubcommandsNegateReqs)
        .arg(
            Arg::with_name("FULLHELP")
                .help("Prints more detailed help information")
                .long("fullhelp"),
        )
        .arg(
            Arg::with_name("host")
                .required_unless("FULLHELP")
                .takes_value(true)
                .default_value("127.0.0.1:61003")
                .long("host")
                .help("DTLS host name."),
        );

    let matches = app.clone().get_matches();

    if matches.is_present("FULLHELP") {
        app.print_long_help().unwrap();
        std::process::exit(0);
    }

    let host = matches.value_of("host").unwrap().to_owned();

    // Generate a certificate and private key to secure the connection
    let certificate = Certificate::generate_self_signed(vec!["localhost".to_owned()])?;

    let cfg = Config {
        certificates: vec![certificate],
        extended_master_secret: ExtendedMasterSecretType::Require,
        ..Default::default()
    };

    println!("listening {}...\ntype 'exit' to shutdown gracefully", host);

    let remote_addr = "127.0.0.1:0".parse::<SocketAddr>().unwrap();
    let socket = UdpSocket::bind("0.0.0.0:60000").unwrap();

    let client = MqttSnClient::new();



    let listener = Arc::new(listen(host, cfg).await?);
    let listener2 = Arc::clone(&listener);
    let hub = Arc::clone(&client.hub);

    tokio::spawn(async move {
        while let Ok((dtls_conn, _remote_addr)) = listener2.accept().await {
            // Register the connection with the chat hub
            hub.register(dtls_conn).await;
        }
    });

    // init_logging();
    let client_loop = client.clone();
    let client_sub = client.clone();
    let client_ingress = client.clone();
    let client_egress = client.clone();
    client_loop.broker_rx_loop(socket);

    // This thread reads the channel for all subscribed topics.
    // The struct Publish is recv.
    // TODO return error for subscribe and publish function calls.
        let _result = client_ingress.handle_ingress();
        let _result = client_egress.handle_egress();

    let rx_thread2 = thread::spawn(move || loop {
        let _result = client_sub.subscribe_rx.recv();
    });

    let publish_thread = thread::spawn(move || loop {
        /*
        let msg = format!("hi {:?}", i);
        let msg2 = format!("hi {:?}", i + 1000);
        client_main.publish(1, i, QOS_LEVEL_0, RETAIN_TRUE, msg.to_string());
        client_main.publish(2, i, QOS_LEVEL_0, RETAIN_FALSE, msg2.to_string());
        i += 1;
        */
        thread::sleep(Duration::from_secs(2));
    });
    rx_thread2.join().expect("The sender thread has panicked");
    publish_thread
        .join()
        .expect("The sender thread has panicked");
    // handle_ingress_thread.join().expect("The handle_ingress thread has panicked");
    Ok(())
}

fn init_logging() {
    /*
    TermLogger::init(
        LevelFilter::Info,
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )
    .unwrap();
    */
}
