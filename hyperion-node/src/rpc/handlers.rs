use super::types::*;

use crate::mempool::Mempool;
use crate::utils;

use hyperion_core::block::{Block, Serializable};
use hyperion_core::chain::blockchain::Blockchain;
use hyperion_core::consensus::adjust_difficulty;
use hyperion_core::crypto::Hashable;

use std::sync::Arc;
use axum::extract::State;
use tokio::sync::RwLock;
use tracing::{debug, info, warn, error, instrument};

#[derive(Clone)]
pub struct NodeState {
    pub chain: Arc<RwLock<Blockchain>>,
    pub mempool: Arc<RwLock<Mempool>>,
}

#[instrument(skip(state), fields(height))]
pub async fn get_block_template(
    State(state): State<NodeState>,
    _params: Option<serde_json::Value>,
) -> Result<BlockTemplate, RpcError> {
    let chain = state.chain.read().await;
    let mut mempool = state.mempool.write().await;

    let transactions = mempool.get_next_transaction(100).unwrap_or_default();
    let latest_block = chain.latest_block();
    let difficulty = adjust_difficulty(&chain);
    let merkle_root = hyperion_core::block::block::compute_merkle_root(&transactions);

    let height = chain.len() as u64;
    tracing::Span::current().record("height", &height);

    let template = BlockTemplate {
        version: 1,
        previous_block_hash: hex::encode(latest_block.double_sha256()),
        transactions,
        difficulty_compact: difficulty,
        timestamp: utils::current_timestamp(),
        height,
        merkle_root: hex::encode(merkle_root),
    };

    debug!(
        //height = %template.height,
        difficulty = %template.difficulty_compact,
        tx_count = %template.transactions.len(),
        "Providing block template"
    );
    
    //info!("Providing block template for height {}", height);

    Ok(template)
}

#[instrument(skip(state, params), fields(block_hash))]
pub async fn submit_block(
    State(state): State<NodeState>,
    params: Option<SubmitBlockParams>,
) -> Result<SubmitBlockResult, RpcError> {
    let params = params.ok_or_else(|| RpcError::invalid_params("Missing block data"))?;

    let block_bytes = hex::decode(&params.block_hex)
        .map_err(|e| RpcError::invalid_params(&format!("Invalid hex: {}", e)))?;

    let block = Block::from_bytes(&block_bytes)
        .map_err(|e| RpcError::invalid_params(&format!("Invalid block: {}", e)))?;

    let block_hash = hex::encode(block.double_sha256());
    tracing::Span::current().record("block_hash", &block_hash);

    // Add block to chain
    let mut chain = state.chain.write().await;
    match chain.add_block(block.clone(), false) {
        Ok(()) => {
            info!(
                //block_hash = %block_hash,
                height = %chain.len(),
                tx_count = %block.transactions.len(),
                "Block accepted"
            );

            let mut mempool = state.mempool.write().await;
            for tx in &block.transactions {
                mempool.remove_tx(tx); 
            }

            if let Err(e) = crate::storage::save_chain(&*chain) {
                error!("Failed to save blockchain to disk: {}", e);
            }

            Ok(SubmitBlockResult {
                accepted: true,
                message: None,
            })
        },
        Err(e) => {
            warn!(
                //block_hash = %block_hash,
                error = ?e,
                "Block rejected"
            );

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