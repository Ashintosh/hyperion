mod utils;
mod network;
mod storage;
mod mempool;
mod rpc;

use mempool::Mempool;
use rpc::{NodeState, start_server};

use hyperion_core::chain::blockchain::Blockchain;
use hyperion_core::block::Transaction;
use hyperion_core::crypto::Hashable;

use std::sync::Arc;
use hex;
use tokio::sync::RwLock;
use tracing::{info, warn, error, debug};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry};
use tracing_appender::non_blocking;
use tracing_rolling_file::RollingFileAppender;
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;

#[tokio::main]
async fn main() {
    let _log_guard = init_logging().unwrap_or_else(|e| {
        eprintln!("Failed to initialize logging: {}", e);
        std::process::exit(1);
    });

    info!("Staring Hyperion Node...");
    
    // Load blockchain and mempool
    let chain = Arc::new(RwLock::new(
        storage::load_chain().unwrap_or_else(|e| {
            warn!("Failed to load chain from disk: {}, creating new genesis", e);
            Blockchain::new_with_genesis()
        })
    ));

    let mempool = Arc::new(RwLock::new(Mempool::load()));

    info!("Genesis Block: {}", hex::encode(
        chain.read().await.get_block_by_height(0).unwrap().double_sha256()
    ));

    // Add test transactions
    {
        let tx_count = 215;
        let mut mempool_guard = mempool.write().await;
        for i in 0..tx_count {
            let tx = generate_random_tx(i);
            mempool_guard.add_tx(tx);
        }
        info!("Added {} test transactions to mempool", tx_count);
    }

    // Start RPC server
    let rpc_state = NodeState {
        chain: chain.clone(),
        mempool: mempool.clone(),
    };
    
    tokio::spawn(async move {
        if let Err(e) = start_server(rpc_state, 6001).await {
            error!("RPC server error: {}", e);
        }
    });

    // Start network listener asynchronously
    tokio::spawn(async move {
        network::start_network_listener("127.0.0.1:6000").await; // Changed port to 6000
    });

    info!("RPC server listening on 127.0.0.1:6001");
    info!("P2P listener on 127.0.0.1:6000");
    info!("Press Ctrl+C to stop");
    
    // Wait for Ctrl+C
    tokio::signal::ctrl_c().await.expect("Failed to listen for ctrl+c");
    info!("Shutting down Hyperion Node...");

    if let Err(e) = storage::save_chain(&*chain.read().await) {
        error!("Failed to save blockchain to disk: {}", e);
    }

    info!("Node stopped.");
}

fn generate_random_tx(seed: i32) -> Transaction {
    let mut rng = StdRng::seed_from_u64(seed as u64);
    
    let num_inputs = rng.random_range(1..=3);
    let num_outputs = rng.random_range(1..=3);

    let mut inputs = Vec::new();
    for i in 0..num_inputs {
        inputs.push(format!("in{}_{}", i, rng.random::<u32>()).into_bytes());
    }

    let mut outputs = Vec::new();
    for i in 0..num_outputs {
        outputs.push(format!("out{}_{}", i, rng.random::<u32>()).into_bytes());
    }

    Transaction::new(inputs, outputs).unwrap()
}

fn init_logging() -> anyhow::Result<tracing_appender::non_blocking::WorkerGuard> {
    let file_appender = RollingFileAppender::builder()
        .filename("logs/hyperion-node.log".to_string())
        .max_filecount(9)
        .condition_max_file_size(10 * 1024 * 1024)
        .build()
        .map_err(|e| anyhow::anyhow!("Failed to create file appender: {}", e))?;

    let (file_writer, guard) = non_blocking(file_appender);

    let console_layer = tracing_subscriber::fmt::layer()
        .with_target(false)
        .with_thread_ids(false)
        .with_level(false)
        .compact();

    let file_layer = tracing_subscriber::fmt::layer()
        .with_writer(file_writer)
        .json()
        .with_target(true)
        .with_thread_ids(true)
        .with_current_span(false);

    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("hyperion_node=info,hyperion_core=info"));

    Registry::default()
        .with(env_filter)
        .with(console_layer)
        .with(file_layer)
        .init();

    Ok(guard)
}

// fn print_block_details(block: &Block) {
//     println!("===== Block Details =====");
//     println!("Block Hash: {}", hex::encode(block.double_sha256()));

//     let header = &block.header;
//     println!("Version: {}", header.version);
//     println!("Timestamp: {}", header.time);
//     println!("Difficulty (compact): 0x{:08x}", header.difficulty_compact);
//     println!("Nonce: {}", header.nonce);
//     println!("Previous Hash: {}", hex::encode(header.prev_hash));
//     println!("Merkle Root: {}", hex::encode(header.merkle_root));

//     println!("Transactions: ({})", block.transactions.len());
//     for (i, tx) in block.transactions.iter().enumerate() {
//         println!("  Tx {}: inputs = {:?}, outputs = {:?}", i, tx.inputs, tx.outputs);
//     }

//     println!("=========================");
// }
