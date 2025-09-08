use std::time::{SystemTime, UNIX_EPOCH};

pub fn current_timestamp() -> u32 {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    now.as_secs() as u32
}