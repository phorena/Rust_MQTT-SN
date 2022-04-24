use crate::MTU;
use bytes::{BufMut, BytesMut};
use custom_debug::Debug;
use getset::{CopyGetters, Getters, MutGetters /*Setters*/};
use std::mem;
use std::str;
// TODO rewrite with generics
#[derive(Debug, Clone)]
pub struct MessageDb {
    pub db: sled::Db,
    pub name: String,
    pub old_value: String,
}

#[derive(
    Debug,
    Clone,
    Copy,
    Getters,
    /*Setters,*/ MutGetters,
    CopyGetters,
    Default,
)]
#[getset(get, set)]
pub struct MessageDbKey {
    pub topic_id: u16,
}

// TODO
impl MessageDbKey {
    /*fn constraint_topic_id(_val: &u16) -> bool {
        //dbg!(_val);
        true
    }*/
}

#[derive(
    Debug, Clone, Getters, /*Setters,*/ MutGetters, CopyGetters, Default,
)]
#[getset(get, set)]
pub struct MessageDbValue {
    pub message: String,
}

// TODO
impl MessageDbValue {
    /*fn constraint_message(_val: &String) -> bool {
        //dbg!(_val);
        true
    }*/
}

impl MessageDb {
    pub fn new(name: String) -> sled::Result<MessageDb> {
        let db: sled::Db = sled::open(name.clone())?;
        let new_db = MessageDb {
            db,
            name,
            old_value: String::new(),
        };
        // TODO check for error
        Ok(new_db)
    }

    // Insert() creates a new entry if existing one doesn't exist.
    // Returns error it does.
    pub fn insert(
        &self,
        key: MessageDbKey,
        value: MessageDbValue,
    ) -> sled::Result<Result<(), sled::CompareAndSwapError>> {
        let mut key_buf = BytesMut::with_capacity(MTU);
        let mut value_buf = BytesMut::with_capacity(MTU);
        // serialize the con_ack struct into byte(u8) array for the network.
        // TODO check for return values, might have errors.
        key.try_write(&mut key_buf);
        value.try_write(&mut value_buf);

        self.db.compare_and_swap(
            &key_buf[..],
            None as Option<&[u8]>, // old value, None for not present
            Some(&value_buf[..]),  // new value, None for delete
        )
    }

    // Update or insert, if the value doesn't exist then insert,
    // else update the value.
    // TODO return Result()
    pub fn upsert(&self, key: MessageDbKey, value: MessageDbValue) {
        let mut key_buf = BytesMut::with_capacity(MTU);
        let mut value_buf = BytesMut::with_capacity(MTU);
        // serialize the con_ack struct into byte(u8) array for the network.
        key.try_write(&mut key_buf);
        value.try_write(&mut value_buf);
        let get_result = self.db.get(&key_buf[..]).unwrap();
        match get_result {
            Some(old_value) => {
                // TODO check return value of results
                let _result = self.db.compare_and_swap(
                    &key_buf[..],
                    Some(old_value), // old value, None for not present
                    Some(&value_buf[..]), // new value, None for delete
                );
            }
            None => {
                let _result = self.db.compare_and_swap(
                    &key_buf[..],
                    None as Option<&[u8]>, // old value, None for not present
                    Some(&value_buf[..]),  // new value, None for delete
                );
            }
        }
    }

    pub fn get(&self, key: MessageDbKey) -> Option<sled::IVec> {
        let mut key_buf = BytesMut::with_capacity(MTU);
        key.try_write(&mut key_buf);
        match self.db.get(&key_buf[..]).unwrap() {
            Some(bytes) => Some(bytes),
            None => None,
        }
    }

    pub fn delete(
        &self,
        key: MessageDbKey,
    ) -> sled::Result<Option<sled::IVec>> {
        let mut key_buf = BytesMut::with_capacity(MTU);
        key.try_write(&mut key_buf);
        match self.db.remove(&key_buf[..]).unwrap() {
            Some(bytes) => Ok(Some(bytes)),
            None => Ok(None),
        }
    }
}
