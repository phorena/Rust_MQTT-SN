use mongodb::{bson::doc, sync::Client, sync::Collection, sync::Database};
use serde::{Deserialize as Ser_Deserialize, Serialize as Ser_Serialize};
use serde_bytes::{ByteBuf, Bytes};

#[derive(Debug, Clone, Ser_Serialize, Ser_Deserialize)]
struct Retain {
    topic_name: String,
    topic_id: String,
    msg_id: String,
    #[serde(with = "serde_bytes")]
    msg: ByteBuf, // TODO: use Bytes? See https://docs.serde.rs/serde_bytes/
}

#[derive(Debug, Clone)]
struct RetainDb {
    pub client: Client,
    url: String,
    pub database: Database,
    collection: Collection<Retain>,
}

impl RetainDb {
    fn new(url: &str) -> RetainDb {
        let client = Client::with_uri_str(url).unwrap();
        let database = client.database("MQTT-SN");
        let collection = database.collection("retain");
        RetainDb {
            client,
            url: url.to_string(),
            database,
            collection,
        }
    }
    fn set(
        &self,
        topic_id: String,
        topic_name: String,
        msg_id: String,
        msg: &[u8],
    ) {
        let retain = Retain {
            topic_id,
            topic_name,
            msg_id,
            msg: ByteBuf::from(msg),
        };
        let result = self.collection.insert_one(retain, None).unwrap();
        dbg!(result);
    }
    fn update(&self, topic_id: String, msg: &str) -> Result<bool, String> {
        // Update the document:
        let update_result = self.collection.update_one(
            doc! {
               "topic_id": topic_id,
            },
            doc! {
               "$set": { "msg": msg },
            },
            None,
        );
        dbg!(&update_result);
        match update_result {
            Ok(result) => {
                if result.matched_count == 0 {
                    println!("No documents matched the filter");
                    return Ok(false);
                }
                return Ok(true);
            }
            Err(e) => {
                dbg!(e.clone());
                return Err(format!("{:?}", e));
            }
        }
    }
    fn get_topic_name(&self, topic_name: &String) -> Result<ByteBuf, String> {
        let filter = doc! { "topic_name": topic_name };
        match self.collection.find_one(filter, None) {
            Ok(Some(retain)) => {
                dbg!(&retain);
                Ok(retain.msg.clone())
            }
            Ok(None) => {
                Err(format!("No retain message for topic_id {}", topic_name))
            }
            Err(e) => Err(format!(
                "Error getting retain message for topic_name {}: {}",
                topic_name, e
            )),
        }
    }
    fn get_topic_id(&self, topic_id: &String) -> Result<ByteBuf, String> {
        let filter = doc! { "topic_id": topic_id };
        match self.collection.find_one(filter, None) {
            Ok(Some(retain)) => {
                dbg!(&retain);
                Ok(retain.msg.clone())
            }
            Ok(None) => {
                Err(format!("No retain message for topic_id {}", topic_id))
            }
            Err(e) => Err(format!(
                "Error getting retain message for topic_id {}: {}",
                topic_id, e
            )),
        }
    }
    fn get_msg_id(&self, msg_id: &String) -> Result<ByteBuf, String> {
        let filter = doc! { "msg_id": msg_id };
        match self.collection.find_one(filter, None) {
            Ok(Some(retain)) => {
                dbg!(&retain);
                Ok(retain.msg.clone())
            }
            Ok(None) => Err(format!("No retain message for msg_id {}", msg_id)),
            Err(e) => Err(format!(
                "Error getting retain message for msg_id {}: {}",
                msg_id, e
            )),
        }
    }
}

#[derive(Debug, Ser_Serialize, Ser_Deserialize)]
struct Subscription {
    subscriber: String,
    topic_name: String,
    topic_id: String,
    qos: u8,
}
struct SubDb {
    pub client: Client,
    url: String,
    pub database: Database,
    collection: Collection<Subscription>,
}
impl SubDb {
    fn new(url: &str) -> Self {
        let client = Client::with_uri_str(url).unwrap();
        let database = client.database("MQTT-SN");
        let collection = database.collection("subscriptions");
        SubDb {
            client,
            url: url.to_string(),
            database,
            collection,
        }
    }
    pub fn subscribe(
        &self,
        subscriber: String,
        topic_name: String,
        topic_id: String,
        qos: u8,
    ) -> Result<(), String> {
        let subscription = Subscription {
            subscriber,
            topic_name,
            topic_id,
            qos,
        };
        let result = self.collection.insert_one(subscription, None).unwrap();
        dbg!(result);
        Ok(())
    }
    pub fn get_subscribers_with_topic_id(
        &self,
        topic_id: &String,
    ) -> Result<Vec<(String, u8)>, String> {
        let filter = doc! { "topic_id": topic_id };
        let cursor = self.collection.find(filter, None).unwrap();
        let mut subscribers = Vec::new();
        for result in cursor {
            let subscription = result.unwrap();
            subscribers
                .push((subscription.subscriber.clone(), subscription.qos));
        }
        Ok(subscribers)
    }
    pub fn get_subscribers_with_topic_name(
        &self,
        topic_name: &String,
    ) -> Result<Vec<(String, u8)>, String> {
        let filter = doc! { "topic_name": topic_name };
        let cursor = self.collection.find(filter, None).unwrap();
        let mut subscribers = Vec::new();
        for result in cursor {
            let subscription = result.unwrap();
            subscribers
                .push((subscription.subscriber.clone(), subscription.qos));
        }
        Ok(subscribers)
    }
    pub fn get_topics(
        &self,
        subscriber: &String,
    ) -> Result<Vec<(String, u8)>, String> {
        let filter = doc! { "subscriber": subscriber };
        let cursor = self.collection.find(filter, None).unwrap();
        let mut topics = Vec::new();
        for result in cursor {
            let subscription = result.unwrap();
            topics.push((subscription.topic_name.clone(), subscription.qos));
        }
        Ok(topics)
    }
    pub fn unsubscribe(
        &self,
        subscriber: String,
        topic_name: String,
    ) -> Result<(), String> {
        let filter =
            doc! { "subscriber": subscriber, "topic_name": topic_name };
        let result = self.collection.delete_many(filter, None).unwrap();
        dbg!(result);
        Ok(())
    }
    pub fn unsub_subscriber_all(
        &self,
        subscriber: String,
    ) -> Result<(), String> {
        let filter = doc! { "subscriber": subscriber };
        let result = self.collection.delete_many(filter, None).unwrap();
        dbg!(result);
        Ok(())
    }
    pub fn unsub_topic_id_all(&self, topic_id: String) -> Result<(), String> {
        let filter = doc! { "topic_id": topic_id };
        let result = self.collection.delete_many(filter, None).unwrap();
        dbg!(result);
        Ok(())
    }
    pub fn unsub_topic_name_all(
        &self,
        topic_name: String,
    ) -> Result<(), String> {
        let filter = doc! { "topic_name": topic_name };
        let result = self.collection.delete_many(filter, None).unwrap();
        dbg!(result);
        Ok(())
    }
}
/*
fn main() -> mongodb::error::Result<()> {
    let sub_db = SubDb::new(
        "mongodb+srv://mongo-1001:JKLsWUuUnjdYbvem@cluster0.elom9.mongodb.net/?retryWrites=true&w=majority");
    let result = sub_db.subscribe(
        "1.2.3.4:5555".to_string(),
        "topic_1".to_string(),
        1.to_string(),
        1,
    );
    let result = sub_db.subscribe(
        "1.2.3.4:5556".to_string(),
        "topic_1".to_string(),
        1.to_string(),
        1,
    );
    let result = sub_db.subscribe(
        "1.2.3.4:5555".to_string(),
        "topic_2".to_string(),
        2.to_string(),
        1,
    );
    let result = sub_db.subscribe(
        "1.2.3.4:5555".to_string(),
        "topic_3".to_string(),
        3.to_string(),
        1,
    );
    dbg!(result);
    let result = sub_db.get_subscribers_with_topic_name(&"topic_1".to_string());
    dbg!(result);
    let result = sub_db.get_topics(&"1.2.3.4:5555".to_string());
    dbg!(result);
    let result = sub_db.unsubscribe("1.2.3.4:5555".to_string(), "topic_3".to_string());
    let result = sub_db.get_topics(&"1.2.3.4:5555".to_string());
    dbg!(result);
    let result = sub_db.unsub_subscriber_all("1.2.3.4:5555".to_string());
    let result = sub_db.get_topics(&"1.2.3.4:5555".to_string());
    dbg!(result);
    let result = sub_db.get_topics(&"1.2.3.4:5555".to_string());
    dbg!(result);
    let result = sub_db.get_topics(&"1.2.3.4:5556".to_string());
    dbg!(result);
    let result = sub_db.unsub_topic_id_all(1.to_string());
    let result = sub_db.get_topics(&"1.2.3.4:5555".to_string());
    dbg!(result);

    let retain_db = RetainDb::new(
        "mongodb+srv://mongo-1001:JKLsWUuUnjdYbvem@cluster0.elom9.mongodb.net/?retryWrites=true&w=majority");

    for db_name in retain_db.client.list_database_names(None, None)? {
        println!("{}", db_name);
    }

    retain_db.set(
        11.to_string(),
        "test2".to_string(),
        "test".to_string(),
        b"test",
    );
    let msg = retain_db.get_topic_id(&11.to_string());
    dbg!(msg);
    retain_db.update(11.to_string(), "test111");
    retain_db.update(12.to_string(), "test11");
    let msg = retain_db.get_topic_id(&11.to_string());
    dbg!(msg);
    /*
    let msg = retain_db.get_msg_id(&"test".to_string());
    dbg!(msg);
    let msg = retain_db.get_topic_name(&"test2".to_string());
    dbg!(msg);
    */
    Ok(())
}
*/
