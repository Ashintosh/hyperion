mod utils;
mod network;
mod storage;
mod mempool;

use mempool::Mempool;

use hyperion_core::chain::blockchain::Blockchain;
use hyperion_core::block::{Block, Transaction};
use hyperion_core::crypto::Hashable;
use hyperion_core::miner;

use std::time::Duration;
use tokio::time::sleep;
use hex;

#[tokio::main]
async fn main() {
    println!("Starting Hyperion Node...");

    // Load blockchain from disk or create genesis
    let mut chain = storage::load_chain().unwrap_or_else(|_| Blockchain::new_with_genesis());

    println!("Genesis Block Hash: {}", hex::encode(
        chain.get_block_by_height(0).unwrap().double_sha256()
    ));

    // Load mempool
    let mut mempool = Mempool::load();

    // Add dummy transactions for testing
    for i in 0..75 {
        let tx = Transaction::new(
            vec![format!("in{}", i).into_bytes()],
            vec![format!("out{}", i).into_bytes()]
        ).unwrap();
        mempool.add_tx(tx);
    }

    // Start network listener asynchronously
    tokio::spawn(async move {
        network::start_network_listener("127.0.0.1:6000").await;
    });

    // Main mining loop
    let mut iteration = 0;
    loop {
        if mempool.is_empty() {
            println!("Mempool empty. Waiting for transactions...");
            sleep(Duration::from_millis(100)).await;
            continue;
        }

        println!("Iteration: {}", iteration);

        if let Some(tx_batch) = mempool.get_next_transaction(5) {
            // Mine a new block with difficulty adjustment
            let block = miner::mine_new_block(&chain, tx_batch, utils::current_timestamp());

            if chain.add_block(block.clone(), false).is_ok() {
                println!("Mined new block: {}", hex::encode(block.double_sha256()));
                print_block_details(&block);

                // Save blockchain
                storage::save_chain(&chain).unwrap();
            }
        }

        iteration += 1;
        sleep(Duration::from_millis(50)).await;
    }
}

fn print_block_details(block: &Block) {
    println!("===== Block Details =====");
    println!("Block Hash: {}", hex::encode(block.double_sha256()));

    let header = &block.header;
    println!("Version: {}", header.version);
    println!("Timestamp: {}", header.time);
    println!("Difficulty (compact): 0x{:08x}", header.difficulty_compact);
    println!("Nonce: {}", header.nonce);
    println!("Previous Hash: {}", hex::encode(header.prev_hash));
    println!("Merkle Root: {}", hex::encode(header.merkle_root));

    println!("Transactions: ({})", block.transactions.len());
    for (i, tx) in block.transactions.iter().enumerate() {
        println!("  Tx {}: inputs = {:?}, outputs = {:?}", i, tx.inputs, tx.outputs);
    }

    println!("=========================");
}
