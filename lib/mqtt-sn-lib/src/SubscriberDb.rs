use std::collections::HashMap;
use custom_debug::Debug;
use std::net::SocketAddr;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Subscribers {
    pub peers: HashMap<SocketAddr, u8>,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
// Stores the subscribers who subscribe to topic ids (u16).
// For performance, subscribers are stored in a hashmap instead of a vec.
// The u8 value is not used.
pub struct SubscriberDb {
    hash_map: HashMap<u16, Subscribers>,
}

// TODO replace u16 with generic <T>
impl SubscriberDb {
    pub fn new() -> SubscriberDb {
        let hash_map:HashMap<u16, Subscribers> = HashMap::new();
        let new_db = SubscriberDb {
            hash_map
        };
        new_db
    }

    pub fn insert(&mut self, topic: u16, subscriber: SocketAddr, value: u8) -> Option<u16> {
        match self.hash_map.get(&topic) {
            // For existing topic id, clone the subscriber hashmap and insert new subscriber.
            // Insert the new subscriber hashmap into topic hashmap.
            Some(subscribers) => {
                // dbg!(subscribers.clone());
                // Can't use Rc or pointer because serialize won't work
                let mut subscribers = subscribers.clone();
                subscribers.peers.insert(subscriber, value);
                self.hash_map.insert(topic.clone(), subscribers);
            }
            // For new topic id, instantiate a new subscriber hashmap, and insert subscriber.
            // Insert the new subscriber hashmap into topic hashmap.
            None => {
                let mut peers:HashMap<SocketAddr, u8> = HashMap::new();
                peers.insert(subscriber, value);
                let subscriber = Subscribers {
                    peers,
                };
                self.hash_map.insert(topic.clone(), subscriber);
            }
        }
        // dbg!(self.clone());
        Some(topic.clone())
    }

    // Get the subscriber hashmap for a topic id.
    pub fn get(&self, topic: u16) -> Option<Subscribers> {
        match self.hash_map.get(&topic) {
            Some(subscribers) => {
                Some(subscribers.clone())
            },
            None => {
                None
            }
        }
    }


    // TODO implement for timeout connections.
    pub fn delete(&mut self, topic: u16, subscriber: SocketAddr) -> Option<u16> {
        match self.hash_map.get(&topic) {
            // if the last subscriber, delete the hash map too
            Some(subscribers) => {
                dbg!(subscribers.clone());
                let mut subscribers = subscribers.clone();
                subscribers.peers.remove(&subscriber);
                dbg!(subscribers.clone());
                match subscribers.peers.is_empty() {
                    false => {
                        self.hash_map.insert(topic.clone(), subscribers);
                    },
                    true => {
                        self.hash_map.remove(&topic);
                    },
                }
                dbg!(self.clone());
                Some(topic)
            }
            None => {
                None
            }
        }
    }
}