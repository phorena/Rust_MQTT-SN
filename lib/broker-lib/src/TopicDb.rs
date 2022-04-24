// Store <Topic Name> -> <Topic Id> in hashmap
// No duplicates allowed
use crate::SubscriberDb;
use custom_debug::Debug;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TopicDb {
    hash_map: HashMap<String, u16>,
}

// TODO replace u16 with generic <T>
// TODO add comments
impl TopicDb {
    pub fn new() -> TopicDb {
        let hash_map: HashMap<String, u16> = HashMap::new();
        let new_db = TopicDb { hash_map };
        new_db
    }

    // Create a new entry, duplicates are not allowed.
    // if <topic name> exist, return found <topic id>,
    // else insert and return new <topic id>
    pub fn create(&mut self, topic_string: &String, new_topic_id: u16) -> u16 {
        match self.hash_map.get(topic_string) {
            Some(old_topic_id) => {
                dbg!(old_topic_id);
                *old_topic_id
                // None
            }
            None => {
                self.hash_map.insert(topic_string.clone(), new_topic_id);
                dbg!(self.clone());
                new_topic_id
            }
        }
    }

    pub fn get(&mut self, topic_string: &String) -> Option<u16> {
        match self.hash_map.get(topic_string) {
            Some(topic_id) => Some(*topic_id),
            None => None,
        }
    }

    pub fn delete(&mut self, _topic_string: &String) -> Option<u16> {
        unimplemented!();
    }
}

pub fn test_subs_db() {
    let server: SocketAddr = "10.1.1.1:80"
        .parse()
        .expect("Unable to parse socket address");
    let mut db = SubscriberDb::SubscriberDb::new();

    db.insert(1, server, 9);
    db.insert(2, server, 9);

    let server: SocketAddr = "11.1.1.1:88"
        .parse()
        .expect("Unable to parse socket address");

    db.insert(1, server, 8);
    db.insert(2, server, 8);
    let subs = db.get(1);
    dbg!(subs.clone());

    let bytes = bincode::serialize(&db).unwrap();
    println!("{:?}", bytes);
    db = bincode::deserialize(&bytes).unwrap();
    dbg!(db.clone());

    db.delete(1, server);

    let server: SocketAddr = "10.1.1.1:80"
        .parse()
        .expect("Unable to parse socket address");

    db.delete(1, server);
}
