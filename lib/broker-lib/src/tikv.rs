// Copyright 2018 TiKV Project Authors. Licensed under Apache-2.0.

// mod common;

// use crate::common::parse_args;
use serde::{Deserialize, Serialize};
use tikv_client::{
    BoundRange, Config, Key, KvPair, TransactionClient as Client, Value,
};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct Entity {
    x: f32,
    y: f32,
}

struct TiKV {
    client: Client,
    config: Config,
}
impl TiKV {
    async fn new() -> Self {
        let config = Config::default();

        let client =
            Client::new_with_config(vec!["localhost:2379"], config.clone())
                .await
                .expect("Could not connect to tikv");
        Self { client, config }
    }
    async fn put(&self, key: Key, value: Value) {
        let mut txn = self
            .client
            .begin_optimistic()
            .await
            .expect("Could not begin a transaction");
        let req = txn.put(key, value).await.expect("couldn't set");
        dbg!(req);
        txn.commit().await.expect("Could not commit transaction");
    }

    async fn puts(&self, pairs: impl IntoIterator<Item = impl Into<KvPair>>) {
        let mut txn = self
            .client
            .begin_optimistic()
            .await
            .expect("Could not begin a transaction");
        for pair in pairs {
            let (key, value) = pair.into().into();
            dbg!(value.clone());
            txn.put(key, value).await.expect("Could not set key value");
        }
        txn.commit().await.expect("Could not commit transaction");
    }

    async fn get(&self, key: Key) -> Option<Value> {
        let mut txn = self
            .client
            .begin_optimistic()
            .await
            .expect("Could not begin a transaction");
        let res = txn.get(key).await.expect("Could not get value");
        txn.commit()
            .await
            .expect("Committing read-only transaction should not fail");
        res
    }

    async fn key_exists(&self, key: Key) -> bool {
        let mut txn = self
            .client
            .begin_optimistic()
            .await
            .expect("Could not begin a transaction");
        let res = txn
            .key_exists(key)
            .await
            .expect("Could not check key exists");
        txn.commit()
            .await
            .expect("Committing read-only transaction should not fail");
        res
    }

    async fn scan(&self, range: impl Into<BoundRange>, limit: u32) {
        let mut txn = self
            .client
            .begin_optimistic()
            .await
            .expect("Could not begin a transaction");
        txn.scan(range, limit)
            .await
            .expect("Could not scan key-value pairs in range")
            .for_each(|pair| println!("{:?}", pair));
        txn.commit().await.expect("Could not commit transaction");
    }

    async fn dels(&self, keys: impl IntoIterator<Item = Key>) {
        let mut txn = self
            .client
            .begin_optimistic()
            .await
            .expect("Could not begin a transaction");
        for key in keys {
            txn.delete(key).await.expect("Could not delete the key");
        }
        txn.commit().await.expect("Could not commit transaction");
    }
}

/*
#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct Test {
    u: u32,
    s: String,
}

#[tokio::main]
async fn main() {
    // You can try running this example by passing your pd endpoints
    // (and SSL options if necessary) through command line arguments.
    let args = parse_args("txn");

    // Create a configuration to use for the example.
    // Optionally encrypt the traffic.
    let config = if let (Some(ca), Some(cert), Some(key)) = (args.ca, args.cert, args.key) {
        Config::default().with_security(ca, cert, key)
    } else {
        Config::default()
    };

    let txn = Client::new_with_config(args.pd, config)
        .await
        .expect("Could not connect to tikv");

    let test = Test { u: 12, s: "hello".to_string() };
    // set
    let key1: Key = b"key1".to_vec().into();
    let value1: Value = b"value1".to_vec();
    let key2: Key = b"key2".to_vec().into();
    let value2: Value = b"value2".to_vec();
    let key3:Key = b"key3".to_vec().into();
    let value3:Value = b"value3".to_vec();

    let key4: Key = b"key4".to_vec().into();
    let bytes = bincode::serialize(&test).unwrap();
    dbg!(bytes.clone());
    let value4: Value = bytes;
    puts(&txn, vec![(key1, value1), (key2, value2)]).await;
    puts(&txn, vec![(key4.clone(), value4)]).await;
    puts2(&txn, key3.clone(), value3).await;
    let return_value3 = get(&txn, key3.clone()).await;
    dbg!(return_value3);

    // get
    let key1: Key = b"key1".to_vec().into();
    let value1 = get(&txn, key1.clone()).await;
    println!("{:?}", (key1, value1.unwrap()));
    let key1: Key = b"key3".to_vec().into();
    let value1 = get(&txn, key1.clone()).await;
    let value4 = get(&txn, key4.clone()).await;
    dbg!(value4.clone());
    let test: Test = bincode::deserialize(&value4.unwrap()).unwrap();
    dbg!(test);
    println!("{:?}", (key1, value1));

    // check key exists
    let key1: Key = b"key1".to_vec().into();
    let key1_exists = key_exists(&txn, key1.clone()).await;
    let key2: Key = b"key_not_exist".to_vec().into();
    let key2_exists = key_exists(&txn, key2.clone()).await;
    println!(
        "check exists {:?}",
        vec![(key1, key1_exists), (key2, key2_exists)]
    );

    // scan
    let key1: Key = b"".to_vec().into();
    scan(&txn, key1.., 10).await;

    // delete
    let key1: Key = b"key1".to_vec().into();
    let key2: Key = b"key2".to_vec().into();
    dels(&txn, vec![key1, key2]).await;
}

*/
