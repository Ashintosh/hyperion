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
use tokio::sync::RwLock;
use hex;
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;

#[tokio::main]
async fn main() {
    println!("Starting Hyperion Node...");
    
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    // Load blockchain and mempool (wrap in Arc<RwLock>)
    let chain = Arc::new(RwLock::new(
        storage::load_chain().unwrap_or_else(|_| Blockchain::new_with_genesis())
    ));
    let mempool = Arc::new(RwLock::new(Mempool::load()));

    println!("Genesis Block Hash: {}", hex::encode(
        chain.read().await.get_block_by_height(0).unwrap().double_sha256()
    ));

    // Add test transactions
    {
        let mut mempool_guard = mempool.write().await;
        for i in 0..215 {
            let tx = generate_random_tx(i);
            mempool_guard.add_tx(tx);
        }
    }

    // Start RPC server
    let rpc_state = NodeState {
        chain: chain.clone(),
        mempool: mempool.clone(),
    };
    
    tokio::spawn(async move {
        if let Err(e) = start_server(rpc_state, 6001).await {
            eprintln!("RPC server error: {}", e);
        }
    });

    // Start network listener asynchronously
    tokio::spawn(async move {
        network::start_network_listener("127.0.0.1:6000").await; // Changed port to 6000
    });

    // Keep the program running
    println!("Node is running. RPC server on port 6001, Network listener on port 6000");
    println!("Press Ctrl+C to stop");
    
    // Wait for Ctrl+C
    tokio::signal::ctrl_c().await.expect("Failed to listen for ctrl+c");
    println!("Shutting down...");

    storage::save_chain(&*chain.read().await)
        .expect("Failed to save blockchain to disk");
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
