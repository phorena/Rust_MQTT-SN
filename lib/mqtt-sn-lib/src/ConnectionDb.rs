use crate::MainMachine::MainMachine;
use std::net::SocketAddr;

// key and value data store for connections
// Key: SocketAddr, Value: State Machine of the connection.
// will expand the value to store more info.
#[derive(Debug, Clone)]
pub struct ConnectionDb {
    db: sled::Db,
    name: String,
}

// TODO move to lib dir
// not fully functional
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
        input_value: &MainMachine,
    ) -> sled::Result<Result<(), sled::CompareAndSwapError>> {
        dbg!(key);
        let value = bincode::serialize(&input_value).unwrap();
        dbg!(value.clone());
        let result = self.db.compare_and_swap(
            key.to_string(),
            None as Option<&[u8]>, // old value, None for not present
            Some(value),           // new value, None for delete
        );
        result
    }
    pub fn read(&self, key: SocketAddr) -> Option<MainMachine> {
        dbg!(key);
        match self.db.get(key.to_string()).unwrap() {
            Some(bytes) => {
                dbg!(bytes.clone());
                let machine: MainMachine = bincode::deserialize(&bytes).unwrap();
                Some(machine)
            }
            None => None,
        }
    }
    pub fn update(
        &self,
        key: SocketAddr,
        old_input: &MainMachine,
        new_input: &MainMachine,
    ) -> Result<(), sled::CompareAndSwapError> {
        let old_value = bincode::serialize(&old_input).unwrap();
        let new_value = bincode::serialize(&new_input).unwrap();
        let result = self.db.compare_and_swap(
            key.to_string(),
            Some(old_value),
            Some(new_value), // new value, None for delete
        );
        result.unwrap()
    }

    // TODO delete() & contains_key()

    pub fn create3<T>(
        &self,
        key: SocketAddr,
        input_value: &T,
    ) -> sled::Result<Result<(), sled::CompareAndSwapError>> {
        dbg!(key);
        let value: &[u8] = unsafe { any_as_u8_slice(input_value) };
        let result = self.db.compare_and_swap(
            key.to_string(),
            None as Option<&[u8]>, // old value, None for not present
            Some(value),           // new value, None for delete
        );
        result
    }

    pub fn read3(&self, key: SocketAddr) -> Option<sled::IVec> {
        dbg!(key);
        self.db.get(key.to_string()).unwrap()
    }

    // Update existing entry.
    // Old value is different, other process has updated it.
    // Read old value, then update again.
    // TODO change key to SocketAddr
    pub fn update3<T, U, V>(
        &self,
        input_key: T,
        input_old: U,
        input_new: V,
    ) -> Result<(), sled::CompareAndSwapError> {
        let key: &[u8] = unsafe { any_as_u8_slice(&input_key) };
        let old_value: &[u8] = unsafe { any_as_u8_slice(&input_old) };
        let new_value: &[u8] = unsafe { any_as_u8_slice(&input_new) };
        let result = self.db.compare_and_swap(
            key,
            Some(old_value),
            Some(new_value), // new value, None for delete
        );
        result.unwrap()
    }

    // Delete an existing entry.
    // TODO change key to SocketAddr
    pub fn delete3<T>(&self, input_key: T) -> Option<sled::IVec> {
        let key: &[u8] = unsafe { any_as_u8_slice(&input_key) };
        self.db.remove(key).unwrap()
    }

    // TODO change key to SocketAddr
    pub fn contains_key3<T>(&self, input_key: T) -> bool {
        let key: &[u8] = unsafe { any_as_u8_slice(&input_key) };
        self.db.contains_key(key).unwrap()
    }
}

unsafe fn any_as_u8_slice<T: Sized>(p: &T) -> &[u8] {
    ::std::slice::from_raw_parts((p as *const T) as *const u8, ::std::mem::size_of::<T>())
}
