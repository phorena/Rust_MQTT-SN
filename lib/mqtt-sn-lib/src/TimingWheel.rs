// clients use 100 milli seconds
// brokers use 10 milli seconds
static TIME_WHEEL_SLEEP_DURATION:usize = 100; // in milli seconds
// For 64 seconds at 100 ms sleeping interval, we need 10 * 64 slots, (1000/100 = 10)
// For 64 seconds at 10 ms sleeping interval, we need 100 * 64 slots, (1000/10 = 100)

// The maximum timeout duration is 64 seconds
// The last one might be over 64, rounding up.
static TIME_WHEEL_MAX_SLOTS:usize = (1000 / TIME_WHEEL_SLEEP_DURATION) * 64 * 2;
// Initial timeout duration is 300 ms
static TIME_WHEEL_INIT_DURATION:usize = 300 / TIME_WHEEL_SLEEP_DURATION;


#[derive(Debug, Clone)]
pub struct TimingWheel <KEY: Debug + Clone, VAL: Debug + Clone> {
    max_slot: usize,
    slot_index: usize,
    slot_vec: Vec<Slot<KEY>>,
    hash: Arc<Mutex<HashMap<KEY, VAL>>>,
}

impl<KEY: Eq + Hash + Debug + Clone, VAL: Debug + Clone> TimingWheel<KEY, VAL> {
    pub fn new(max_slot: usize) -> Self {
        let mut slot_vec:Vec<Slot<KEY>> = Vec::with_capacity(max_slot);
        // Must use for loop to initialize
        // let slot_vec:Vec<Slot<KEY>> = vec![Vec<Slot<KEY>>, max_slot] didn'KEY work.
        // It allocates one Vec<Slot<KEY>> and all entries point to it.
        for _ in 0..max_slot {
            slot_vec.push(Slot::new());
        }
        TimingWheel {
            max_slot,
            slot_vec,
            slot_index: 0,
            hash: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    // The initial duration is set to TIME_WHEEL_INIT_DURATION, but can be
    // changed to reflect the network the client is on, (LAN or WAN),
    // or the latency pattern.
    #[inline(always)]
    fn schedule(&mut self,
              key: KEY,
              val: VAL,
              duration: usize) -> Result<(), (KEY, usize)>
    {
        // store the key in a slot of the timing wheel
        let slot_index = (self.slot_index + duration) % self.max_slot;
        let slot = &self.slot_vec[slot_index];
        let mut hash = self.hash.lock().unwrap();
        let mut vec = slot.entries.lock().unwrap();
        // store the data the hash
        hash.insert(key.clone(), val);
        // use tuple to add duration and avoid define new struct
        dbg_fn!((key.clone(), duration, SystemTime::now()));
        vec.push((key, duration));
        return Ok(());
    }

    #[inline(always)]
    fn cancel(&mut self,
              key: KEY) -> Result<(), KEY>
    {
        let mut hash = self.hash.lock().unwrap();
        // remove the data from the hash only
        // the expire() won't return data from the slot if it has been deleted
        match hash.remove(&key) {
            Some(result) => {
                dbg!((key, result));
                Ok(())
            }
            None => Err(key)
        }
    }

    /// expire() is called every TIME_WHEEL_SLEEP_DURATION to iterate all the entries
    /// in the slot.
    /// If the new duration is greater than the maximun duration (TIME_WHEEL_MAX_SLOTS)
    /// remove it from the hashmap, else reschedule to the new duration.
    #[inline(always)]
    fn expire(&mut self) -> Vec<(KEY, VAL)>
    {
        // select and lock the current time slot in the Vec
        let cur_slot = &self.slot_vec[self.slot_index];
        // returning data store in this vector
        let mut result_vec = Vec::new();
        // TODO replace unwrap()
        let mut cur_slot_lock = cur_slot.entries.lock().unwrap();
        // iterate for each entry in the selected Vec
        while let Some(top) = cur_slot_lock.pop() {
            dbg_fn!((top.clone(), SystemTime::now()));
            let hash_hdr = top.0;
            // exponetial backup is inside the expire (lib)
            // the caller doesn't have to do it
            let duration = top.1 * 2;
            { // Locking the hashmap as short as possible
                let mut hash = self.hash.lock().unwrap();
                if duration < TIME_WHEEL_MAX_SLOTS {
                    // reschedule, don't remove hash entry
                    match hash.get(&hash_hdr) {
                        Some(result) => {
                            // dbg!((hash_hdr.clone(), result.clone()));

                            // insert entry into next slot
                            let next_slot_index = (self.slot_index +
                                                   duration) % self.max_slot;
                            // gather results
                            result_vec.push((hash_hdr.clone(), result.to_owned()));
                            let next_slot = &self.slot_vec[next_slot_index];

                            let mut next_slot_lock = next_slot.
                                entries.lock().unwrap();
                            dbg_fn!(hash_hdr.clone());
                            next_slot_lock.push((hash_hdr, duration));
                            // TODO send it back inside the loop, eliminate return loop
                            // need the to use RetransmitHeader to access the address
                            // can't access it from generic KEY
                        }
                        // TODO clean up
                        None => println!("+++++++++++++++++++empty.")
                    }
                } else {
                    // timeout duration is over the limit, remove hash entry
                    match hash.remove(&hash_hdr) {
                        Some(result) => {
                            // dbg!((hash_hdr.clone(), result.clone()));
                            // gather results
                            result_vec.push((hash_hdr.clone(), result.to_owned()));
                        }
                        // TODO clean up
                        None => println!("+++++++++++++++++++empty.")
                    }
                }
            }
        }
        // next slot with overflow back to 0
        self.slot_index = (self.slot_index + 1) % self.max_slot;
        // TODO test preload this slot in CPU cache?
        result_vec
    }

    fn print(&mut self)
    {
        let slot = &self.slot_vec[0];
        println!("+++++++++++++++++++{:?}", slot);
        let slot = &self.slot_vec[1];
        println!("+++++++++++++++++++{:?}", slot);
        let slot = &self.slot_vec[2];
        println!("+++++++++++++++++++{:?}", slot);
        let slot = &self.slot_vec[3];
        println!("+++++++++++++++++++{:?}", slot);
    }
}

