use super::worker::{MiningWorker, WorkItem, MiningResult};
use crate::config::MiningConfig;
use crate::network::NodeClient;
use crate::utils::stats::MiningStats;
use anyhow::Result;
use hyperion_core::block::Header;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::cell::RefCell;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, watch};
use tokio::time::sleep;
use tracing::{debug, error, info, warn};

pub struct SoloMiner {
    config: MiningConfig,
    node_client: NodeClient,
    workers: Vec<MiningWorker>,
    stats: MiningStats,
    running: Arc<std::sync::atomic::AtomicBool>,
    work_counter: Arc<std::sync::atomic::AtomicU64>,
    cancel_tx: RefCell<Option<watch::Sender<bool>>>,
    solution_found: Arc<AtomicBool>,
}

impl SoloMiner {
    pub async fn new(config: MiningConfig) -> Result<Self> {
        let node_client = NodeClient::new(config.node_url.clone());
        
        // Test connection to node
        node_client.test_connection().await?;

        // Create workers
        let mut workers = Vec::new();
        for i in 0..config.threads {
            workers.push(MiningWorker::new(i));
        }

        Ok(Self {
            config,
            node_client,
            workers,
            stats: MiningStats::new(),
            running: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            work_counter: Arc::new(std::sync::atomic::AtomicU64::new(0)),
            cancel_tx: RefCell::new(None),
            solution_found: Arc::new(AtomicBool::new(false)),
        })
    }

    pub async fn start_mining(&mut self) -> Result<()> {
        self.running.store(true, std::sync::atomic::Ordering::SeqCst);
        info!("Starting solo mining with {} threads", self.config.threads);

        // Create channels for work distribution
        let (result_tx, mut result_rx) = mpsc::channel(10);
        
        // Create work channels for each worker
        let mut work_senders = Vec::new();
        let mut worker_handles = Vec::new();

        // Start worker tasks
        for worker in &self.workers {
            let (work_tx, work_rx) = mpsc::channel::<WorkItem>(10);
            work_senders.push(work_tx);
            
            let worker_clone = worker.clone();
            let result_tx_clone = result_tx.clone();
            
            let handle = tokio::spawn(async move {
                worker_clone.start(work_rx, result_tx_clone).await;
            });
            worker_handles.push(handle);
        }

        drop(result_tx);

        // Start stats reporting task
        let stats_handle = {
            let stats = self.stats.clone();
            let workers = self.workers.clone();
            let interval = self.config.stats_interval;
            
            tokio::spawn(async move {
                let mut stats_timer = tokio::time::interval(Duration::from_secs(interval));
                loop {
                    stats_timer.tick().await;
                    Self::report_stats(&stats, &workers).await;
                }
            })
        };

        // Get initial work
        self.get_and_distribute_work(&work_senders).await?;
        let mut last_template_time = Instant::now();

        while self.running.load(std::sync::atomic::Ordering::SeqCst) {
            tokio::select! {
                // Check for mining results
                result = result_rx.recv() => {
                    if let Some(mining_result) = result {
                        println!("Block found by worker {}!", mining_result.worker_id);
                        
                        // Set solution found flag to prevent other workers from submitting
                        // TODO: Figure out why this is not working on new chain and causing rejected blocks
                        self.solution_found.store(true, std::sync::atomic::Ordering::SeqCst);

                        // Cancel other workers
                        if let Some(ref cancel_tx) = *self.cancel_tx.borrow() {
                            let _ = cancel_tx.send(true);
                            debug!("Cancelled all current work");
                        }

                        if let Err(e) = self.node_client.submit_block(mining_result.block).await {
                            println!("Failed to submit block: {}", e);
                        } else {
                            self.stats.blocks_found.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                            println!("Block submitted successfully!");
                        }
                        
                        // Small delay to ensure other workers stop
                        tokio::time::sleep(Duration::from_millis(10)).await;
                        
                        // Get fresh work
                        info!("Restarting mining with fresh work...");
                        match self.get_and_distribute_work(&work_senders).await {
                            Ok(()) => {
                                info!("All workers restarted with new work");
                                last_template_time = Instant::now();
                            }
                            Err(e) => {
                                warn!("Failed to restart workers with new work: {}", e);
                            }
                        }
                    }
                }
                
                _ = sleep(Duration::from_secs(30)) => { 
                    if last_template_time.elapsed() > Duration::from_secs(60) { 
                        info!("Work is very stale, getting fresh template...");
                        match self.get_and_distribute_work(&work_senders).await {
                            Ok(()) => {
                                last_template_time = Instant::now();
                                debug!("Updated stale work");
                            }
                            Err(e) => {
                                warn!("Failed to get fresh work: {}", e);
                                sleep(Duration::from_secs(self.config.reconnect_delay)).await;
                            }
                        }
                    }
                }
                
                // Just a periodic check without doing anything
                _ = sleep(Duration::from_millis(100)) => {
                    // This keeps the select loop responsive but doesn't do any work updates
                }
            }
        }

        // Clean shutdown
        info!("Stopping workers...");
        for worker in &self.workers {
            worker.stop();
        }

        drop(work_senders);
        for handle in worker_handles {
            let _ = handle.await;
        }

        stats_handle.abort();
        info!("Solo miner stopped");
        Ok(())
    }

    async fn get_and_distribute_work(&self, work_senders: &[mpsc::Sender<WorkItem>]) -> Result<()> {
        // Reset the solution found flag for new work
        self.solution_found.store(false, std::sync::atomic::Ordering::SeqCst);
        
        // Cancel any existing work
        if let Some(ref cancel_tx) = *self.cancel_tx.borrow() {
            let _ = cancel_tx.send(true);
        }

        // Create new cancellation token for this work batch
        let (cancel_tx, cancel_rx) = watch::channel(false);
        *self.cancel_tx.borrow_mut() = Some(cancel_tx);

        let template = self.node_client.get_block_template().await?;
        let work_id = self.work_counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

        // Convert template to work item
        let prev_hash = hex::decode(&template.previous_block_hash)?
            .try_into()
            .map_err(|_| anyhow::anyhow!("Invalid previous block hash length"))?;

        let merkle_root = hex::decode(&template.merkle_root)?
            .try_into()
            .map_err(|_| anyhow::anyhow!("Invalid merkle root length"))?;

        let header = Header::new(
            template.version,
            template.timestamp,
            template.difficulty_compact,
            0, // nonce starts at 0
            prev_hash,
            merkle_root,
        );

        // Distribute work across workers
        let nonce_range_per_worker = u64::MAX / work_senders.len() as u64;
        
        for (i, sender) in work_senders.iter().enumerate() {
            let work_item = WorkItem {
                header: header.clone(),
                nonce_start: i as u64 * nonce_range_per_worker,
                nonce_range: nonce_range_per_worker,
                transactions: template.transactions.clone(),
                work_id,
                cancel_rx: cancel_rx.clone(),
                solution_found: self.solution_found.clone(),
            };

            if sender.send(work_item).await.is_err() {
                println!("Failed to send work to worker {}", i);
            }
        }

        println!("Distributed work ID {} to {} workers", work_id, work_senders.len());
        Ok(())
    }

    async fn report_stats(stats: &MiningStats, workers: &[MiningWorker]) {
        let total_hashes: u64 = workers
            .iter()
            .map(|w| w.hashes_computed.load(std::sync::atomic::Ordering::SeqCst))
            .sum();

        let hashrate = stats.calculate_hashrate(total_hashes);
        let blocks_found = stats.blocks_found.load(std::sync::atomic::Ordering::SeqCst);
        let uptime = stats.start_time.elapsed();

        println!(
            "Mining Stats - Hashrate: {:.2} H/s, Blocks: {}, Uptime: {:?}",
            hashrate, blocks_found, uptime
        );
    }

    pub async fn stop(&mut self) -> Result<()> {
        info!("Stopping miner...");
        self.running.store(false, std::sync::atomic::Ordering::SeqCst);
        Ok(())
    }

    fn stop_all_workers(&self) {
        for worker in &self.workers {
            worker.stop();
        }
    }
    
    fn restart_all_workers(&self) {
        for worker in &self.workers {
            worker.restart();
        }
    }
}