use crate::state::SerializedState;
use std::net::SocketAddr;

// key and value data store for connections
// Key: SocketAddr, Value: State Machine of the connection.
// will expand the value to store more info.
#[derive(Debug, Clone)]
pub struct ConnectionDb {
    db: sled::Db,
    name: String,
}

impl ConnectionDb {
    pub fn new(name: String) -> sled::Result<ConnectionDb> {
        let db: sled::Db = sled::open(name.clone())?;
        let new_db = ConnectionDb { db, name };
        // TODO check for error
        Ok(new_db)
    }
    // Create a new entry if existing one doesn't exist.
    // Return error it does.
    pub fn create(
        &self,
        key: SocketAddr,
        input_value: &SerializedState,
    ) -> sled::Result<Result<(), sled::CompareAndSwapError>> {
        // dbg!(key);
        let serialized_key = bincode::serialize(&key).unwrap();
        let value = bincode::serialize(&input_value).unwrap();
        // dbg!(value.clone());
        let result = self.db.compare_and_swap(
            serialized_key,
            None as Option<&[u8]>, // old value, None for not present
            Some(value),           // new value, None for delete
        );
        result
    }

    pub fn read(&self, key: SocketAddr) -> Option<SerializedState> {
        dbg!(key);
        let serialized_key = bincode::serialize(&key).unwrap();
        match self.db.get(serialized_key).unwrap() {
            Some(bytes) => {
                dbg!(bytes.clone());
                let machine: SerializedState = bincode::deserialize(&bytes).unwrap();
                Some(machine)
            }
            None => None,
        }
    }


    //pub fn update(
     //   &self,
        //key: SocketAddr,
        //old_input: &SerializedState,
       // new_input: &SerializedState,
    //) {
        
        //self.db.remove(key.to_string());
        //assert_eq!(self.db.get(key.to_string()), Ok(None));
        //let old_value = bincode::serialize(&old_input).unwrap();
        //let new_value = bincode::serialize(&new_input).unwrap();
        //let result = /*self.db.compare_and_swap(
            //key.to_string(), // TODO: Key should be serialized
            //None,
            //Some(new_value), // new value, None for delete
        //);self.create(key, new_input);
        //result.unwrap();
   // }


    pub fn remove_db(&self,key: SocketAddr){
        let serialized_key = bincode::serialize(&key).unwrap();
        self.db.remove(serialized_key); 
    }

    pub fn update(
        &self,
        key: SocketAddr,
        new_input: &SerializedState,
    ){
        self.remove_db(key);
        //assert_eq!(self.db.get(key.to_string()), Ok(None));
        
        let result = self.create(key, new_input);
        result.unwrap();
        
    }
    

    pub fn to_iter(&self) -> sled::Iter {
        self.db.iter()
    }
}
