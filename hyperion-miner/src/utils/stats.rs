use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

#[derive(Clone)]
pub struct MiningStats {
    pub start_time: Instant,
    pub blocks_found: Arc<AtomicU64>,
    pub last_hash_count: Arc<AtomicU64>,
    pub last_stats_time: Arc<Mutex<Instant>>,
}

impl MiningStats {
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            blocks_found: Arc::new(AtomicU64::new(0)),
            last_hash_count: Arc::new(AtomicU64::new(0)),
            last_stats_time: Arc::new(Mutex::new(Instant::now())),
        }
    }

    pub fn calculate_hashrate(&self, total_hashes: u64) -> f64 {
        let mut last_time = self.last_stats_time.lock().unwrap();
        let now = Instant::now();
        let duration = now.duration_since(*last_time).as_secs_f64();
        
        let last_hashes = self.last_hash_count.swap(total_hashes, Ordering::SeqCst);
        let hash_diff = total_hashes.saturating_sub(last_hashes);
        
        *last_time = now;
        
        if duration > 0.0 {
            hash_diff as f64 / duration
        } else {
            0.0
        }
    }
}