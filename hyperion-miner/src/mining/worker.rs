use hyperion_core::block::{Block, Header, Transaction};
use hyperion_core::consensus::mine_block;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::{mpsc, watch};
use tracing::{debug, info};


#[derive(Clone)]
pub struct WorkItem {
    pub header: Header,
    pub nonce_start: u64,
    pub nonce_range: u64,
    pub transactions: Vec<Transaction>,
    pub work_id: u64,
    pub cancel_rx: watch::Receiver<bool>,
    pub solution_found: Arc<AtomicBool>,
}

pub struct MiningResult {
    pub block: Block,
    pub nonce: u64,
    pub worker_id: usize,
}

#[derive(Clone)]
pub struct MiningWorker {
    pub id: usize,
    pub running: Arc<AtomicBool>,
    pub hashes_computed: Arc<AtomicU64>,
    pub current_work_id: Arc<AtomicU64>,
}

impl MiningWorker {
    pub fn new(id: usize) -> Self {
        Self {
            id,
            running: Arc::new(AtomicBool::new(false)),
            hashes_computed: Arc::new(AtomicU64::new(0)),
            current_work_id: Arc::new(AtomicU64::new(0)),
        }
    }

    pub async fn start(
        &self,
        mut work_rx: mpsc::Receiver<WorkItem>,
        result_tx: mpsc::Sender<MiningResult>,
    ) {
        self.running.store(true, Ordering::SeqCst);
        println!("Mining worker {} started", self.id);

        while self.running.load(Ordering::SeqCst) {
            tokio::select! {
                work_item = work_rx.recv() => {
                    match work_item {
                        Some(work) => {
                            self.current_work_id.store(work.work_id, Ordering::SeqCst);

                            if let Some(result) = self.mine_work(work).await {
                                if result_tx.send(result).await.is_err() {
                                    println!("Failed to send mining result");
                                    break;
                                }
                            }
                        },
                        None => break,  // Channel closed
                    }
                },
                _ = tokio::time::sleep(tokio::time::Duration::from_millis(100)) => {
                    // Periodic check to ensure we're responsive to shutdown
                }
            }
        }

        println!("Mining worker {} stopped", self.id);
    }

    pub async fn mine_work(&self, work: WorkItem) -> Option<MiningResult> {
        let mut header = work.header.clone();
        let start_nonce = work.nonce_start;
        let end_nonce = start_nonce + work.nonce_range;
        let work_id = work.work_id;
        let mut cancel_rx = work.cancel_rx;

        println!(
            "Worker {} mining nonce range {} to {}",
            self.id, start_nonce, end_nonce
        );

        const BATCH_SIZE: u64 = 10000;
        
        for batch_start in (start_nonce..end_nonce).step_by(BATCH_SIZE as usize) {
            // Check if we should continue with this work
            if !self.running.load(Ordering::SeqCst)
                || *cancel_rx.borrow()
                || work.solution_found.load(Ordering::SeqCst) {
                println!("Worker {} work cancelled or stopped", self.id);
                return None;
            }

            // Check if work is stale (new work arrived)
            if self.current_work_id.load(Ordering::SeqCst) != work_id {
                println!("Worker {} abandoning state work ID {}", self.id, work_id);
                return None;
            }

            let batch_end = (batch_start + BATCH_SIZE).min(end_nonce);
            
            for nonce in batch_start..batch_end {
                header.nonce = nonce;
                
                if header.validate_pow().is_ok() {
                    // Double-check cancellation before submitting result
                    if *cancel_rx.borrow() {
                        println!("Work cancelled just before solution submission");
                        return None;
                    }

                    println!("Worker {} found solution! Nonce: {}", self.id, nonce);
                    
                    // Create the complete block with transactions
                    let block = Block::new(header, work.transactions.clone());
                    
                    return Some(MiningResult {
                        block,
                        nonce,
                        worker_id: self.id,
                    });
                }
                
                self.hashes_computed.fetch_add(1, Ordering::SeqCst);
            }

            // Check for cancellation between batches
            if cancel_rx.has_changed().unwrap_or(false) {
                if *cancel_rx.borrow() {
                    println!("Worker {} work cancelled mid-batch", self.id);
                    return None;
                }
            }

            // Yield control periodically
            tokio::task::yield_now().await;
        }

        None
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }

    pub fn restart(&self) {
        // Stop current work
        self.running.store(false, Ordering::SeqCst);
        // Small delay to ensure worker stops
        std::thread::sleep(std::time::Duration::from_millis(10));
        // Restart
        self.running.store(true, Ordering::SeqCst);
    }

    pub fn get_hashrate(&self, duration_secs: f64) -> f64 {
        let hashes = self.hashes_computed.load(Ordering::SeqCst) as f64;
        hashes / duration_secs
    }
}