use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

static COUNTERS: OnceLock<Mutex<HashMap<(u32, u32), u32>>> = OnceLock::new();

pub fn get_next_int(min_val: u32, max_val: u32) -> u32 {
    let counters = COUNTERS.get_or_init(|| Mutex::new(HashMap::new()));
    let mut counters = counters.lock().expect("counter lock poisoned");
    let entry = counters.entry((min_val, max_val)).or_insert(min_val);
    *entry = entry.wrapping_add(1);
    (*entry % max_val) + min_val
}
