use hyperion_core::block::{Block, Transaction};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockTemplate {
    pub version: u32,
    pub previous_block_hash: String,
    pub transactions: Vec<Transaction>,
    pub difficulty_compact: u32,
    pub timestamp: u32,
    pub height: u64,
    pub merkle_root: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiningInfo {
    pub blocks: u64,
    pub current_block_size: u64,
    pub current_block_tx: u64,
    pub difficulty: f64,
    pub network_hashps: f64,
    pub pooled_tx: u64,
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitBlockRequest {
    pub block_hex: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitBlockResponse {
    pub accepted: bool,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetWorkRequest {
    pub miner_address: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcRequest<T> {
    pub jsonrpc: String,
    pub id: u32,
    pub method: String,
    pub params: T,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcResponse<T> {
    pub jsonrpc: String,
    pub id: u32,
    pub result: Option<T>,
    pub error: Option<RpcError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcError {
    pub code: i32,
    pub message: String,
}