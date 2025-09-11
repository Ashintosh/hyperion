use super::types::*;

use crate::mempool::Mempool;
use crate::utils;

use hyperion_core::block::{Block, Serializable};
use hyperion_core::chain::blockchain::Blockchain;
use hyperion_core::consensus::adjust_difficulty;
use hyperion_core::crypto::Hashable;

use std::ops::Deref;
use std::sync::Arc;
use axum::extract::State;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

#[derive(Clone)]
pub struct NodeState {
    pub chain: Arc<RwLock<Blockchain>>,
    pub mempool: Arc<RwLock<Mempool>>,
}

pub async fn get_block_template(
    State(state): State<NodeState>,
    _params: Option<serde_json::Value>,
) -> Result<BlockTemplate, RpcError> {
    let chain = state.chain.read().await;
    let mut mempool = state.mempool.write().await;

    // Get pending transactions
    let transactions = mempool.get_next_transaction(100)
        .unwrap_or_default();

    let latest_block = chain.latest_block();
    let difficulty = adjust_difficulty(&chain);
    let height = chain.len() as u64;
    let merkle_root = hyperion_core::block::block::compute_merkle_root(&transactions);

    info!("Providing block template for height {}", height);

    Ok(BlockTemplate {
        version: 1,
        previous_block_hash: hex::encode(latest_block.double_sha256()),
        transactions,
        difficulty_compact: difficulty,
        timestamp: utils::current_timestamp(),
        height,
        merkle_root: hex::encode(merkle_root),
    })
}

pub async fn submit_block(
    State(state): State<NodeState>,
    params: Option<SubmitBlockParams>,
) -> Result<SubmitBlockResult, RpcError> {
    let params = params.ok_or_else(|| RpcError::invalid_params("Missing block data"))?;

    // Decode block from hex
    let block_bytes = hex::decode(&params.block_hex)
        .map_err(|e| RpcError::invalid_params(&format!("Invalid hex: {}", e)))?;

    let block = Block::from_bytes(&block_bytes)
        .map_err(|e| RpcError::invalid_params(&format!("Invalid block: {}", e)))?;

    let block_hash = hex::encode(block.double_sha256());

    // Add block to chain
    let mut chain = state.chain.write().await;
    match chain.add_block(block.clone(), false) {
        Ok(()) => {
            info!("Block {} accepted and added to chain!", block_hash);

            // Remove transactions from mempool
            let mut mempool = state.mempool.write().await;
            for tx in &block.transactions {
                mempool.remove_tx(tx); 
            }

            // Save chain
            if let Err(e) = crate::storage::save_chain(&*chain) {
                warn!("Failed to save chain: {}", e);
            }

            Ok(SubmitBlockResult {
                accepted: true,
                message: None,
            })
        },
        Err(e) => {
            warn!("Block {} rejected: {:?}", block_hash, e);
            Ok(SubmitBlockResult {
                accepted: false,
                message: Some(format!("{:?}", e)),
            })
        }
    }
}

pub async fn get_mining_info(
    State(state): State<NodeState>,
    _params: Option<serde_json::Value>,
) -> Result<MiningInfo, RpcError> {
    let chain = state.chain.read().await;
    let mempool = state.mempool.read().await;

    let difficulty = adjust_difficulty(&chain);
    let difficulty_f64 = difficulty as f64;  // Convert compact to readable

    Ok(MiningInfo {
        blocks: chain.len() as u64,
        current_block_size: 0,  // TODO: Calculate
        current_block_tx: 0,  // TODO: Calculate
        difficulty: difficulty_f64,
        network_hashps: 0.0,  // TODO: Estimate
        pooled_tx: mempool.len() as u64,
        chain: "hyperion".to_string(),
    })
}

pub async fn get_blockchain_info(
    State(state): State<NodeState>,
    _params: Option<serde_json::Value>,
) -> Result<ChainInfo, RpcError> {
    let chain = state.chain.read().await;
    let latest_block = chain.latest_block();
    let difficulty = adjust_difficulty(&chain);

    Ok(ChainInfo {
        chain: "hyperion".to_string(),
        blocks: chain.len() as u64,
        headers: chain.len() as u64,
        best_blockhash: hex::encode(latest_block.double_sha256()),
        difficulty: difficulty as f64,
        median_time: latest_block.header.time,
    })
}

pub async fn get_block_count(
    State(state): State<NodeState>,
    _params: Option<serde_json::Value>,
) -> Result<u64, RpcError> {
    let chain = state.chain.read().await;
    Ok(chain.len() as u64 - 1)  // Bitcoin returns height, not count
}