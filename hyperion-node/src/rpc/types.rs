use hyperion_core::block::Transaction;
use serde::{Deserialize, Serialize};


// JSON-RPC 2.0 standard types
#[derive(Debug, Deserialize)]
pub struct RpcRequest<T> {
    pub jsonrpc: String,
    pub id: serde_json::Value,
    pub method: String,
    pub params: Option<T>,
}

#[derive(Debug, Serialize)]
pub struct RpcResponse<T> {
    pub jsonrpc: String,
    pub id: serde_json::Value,
    pub result: Option<T>,
    pub error: Option<RpcError>,
}

#[derive(Debug, Serialize)]
pub struct RpcError {
    pub code: i32,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

// Mining specific types
#[derive(Debug, Serialize)]
pub struct BlockTemplate {
    pub version: u32,
    pub previous_block_hash: String,
    pub transactions: Vec<Transaction>,
    pub difficulty_compact: u32,
    pub timestamp: u32,
    pub height: u64,
    pub merkle_root: String,
}

#[derive(Debug, Deserialize)]
pub struct SubmitBlockParams {
    pub block_hex: String,
}

#[derive(Debug, Serialize)]
pub struct SubmitBlockResult {
    pub accepted: bool,
    pub message: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct MiningInfo {
    pub blocks: u64,
    pub current_block_size: u64,
    pub current_block_tx: u64,
    pub difficulty: f64,
    pub network_hashps: f64,
    pub pooled_tx: u64,
    pub chain: String,
}

#[derive(Debug, Serialize)]
pub struct ChainInfo {
    pub chain: String,
    pub blocks: u64,
    pub headers: u64,
    pub best_blockhash: String,
    pub difficulty: f64,
    pub median_time: u32,
}

// Error codes (Bitcoin-compatible)
impl RpcError {
    pub fn method_not_found() -> Self {
        Self {
            code: -32601,
            message: "Method not found".to_string(),
            data: None,
        }
    }

    pub fn invalid_params(msg: &str) -> Self {
        Self {
            code: -32602,
            message: format!("Invalid params: {}", msg),
            data: None,
        }
    }

    pub fn internal_error(msg: &str) -> Self {
        Self {
            code: -32603,
            message: format!("Internal error: {}", msg),
            data: None,
        }
    }

    pub fn custom(code: i32, msg: &str) -> Self {
        Self {
            code,
            message: msg.to_string(),
            data: None,
        }
    }
}