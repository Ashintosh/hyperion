use hyperion_core::block::{Block, Serializable};

use std::sync::atomic::{AtomicU32, Ordering};
use super::rpc::{
    BlockTemplate, MiningInfo, RpcRequest, RpcResponse, SubmitBlockRequest, SubmitBlockResponse
};
use anyhow::{anyhow, Result};
use reqwest::Client;
use tracing::{debug, error, info};

pub struct NodeClient {
    client: Client,
    base_url: String,
    request_id: AtomicU32,
}

impl NodeClient {
    pub fn new(base_url: String) -> Self {
        Self {
            client: Client::new(),
            base_url,
            request_id: AtomicU32::new(1),
        }
    }

    pub async fn get_block_template(&self) -> Result<BlockTemplate> {
        debug!("Requesting block template from node");

        let request = RpcRequest {
            jsonrpc: "2.0".to_string(),
            id: self.request_id.fetch_add(1, Ordering::SeqCst),
            method: "get_block_template".to_string(),
            params: serde_json::Value::Null,
        };

        let response = self
            .client
            .post(&format!("{}/rpc", self.base_url))
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!("HTTP error: {}", response.status()));
        }

        let rpc_response: RpcResponse<BlockTemplate> = response.json().await?;

        if let Some(error) = rpc_response.error {
            return Err(anyhow!("RPC error: {}", error.message));
        }

        rpc_response
            .result
            .ok_or_else(|| anyhow!("Missing result in RPC response"))
    }

    pub async fn submit_block(&self, block: Block) -> Result<bool> {
        debug!("Submitting mined block to node");

        // Serialize block to hex
        let block_bytes = block.serialize().unwrap();  // TODO: Remove unwrap
        let block_hex = hex::encode(block_bytes);

        let submit_request = SubmitBlockRequest { block_hex };

        let request = RpcRequest {
            jsonrpc: "2.0".to_string(),
            id: self.request_id.fetch_add(1, Ordering::SeqCst),
            method: "submit_block".to_string(),
            params: serde_json::to_value(submit_request)?,
        };

        let response = self
            .client
            .post(&format!("{}/rpc", self.base_url))
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!("HTTP error: {}", response.status()));
        }

        let rpc_response: RpcResponse<SubmitBlockResponse> = response.json().await?;

        if let Some(error) = rpc_response.error {
            error!("Block submission failed: {}", error.message);
            return Ok(false);
        }

        if let Some(result) = rpc_response.result {
            if result.accepted {
                debug!("Block accepted by node!");
            } else {
                error!("Block rejected: {}", result.message.unwrap());
            }
            Ok(result.accepted)
        } else {
            Err(anyhow!("Missing result in RPC response"))
        }
    }

    pub async fn get_mining_info(&self) -> Result<MiningInfo> {
        let request = RpcRequest {
            jsonrpc: "2.0".to_string(),
            id: self.request_id.fetch_add(1, Ordering::SeqCst),
            method: "get_mining_info".to_string(),
            params: serde_json::Value::Null,
        };

        let response = self
            .client
            .post(&format!("{}/rpc", self.base_url))
            .json(&request)
            .send()
            .await?;

        let rpc_response: RpcResponse<MiningInfo> = response.json().await?;
        if let Some(error) = rpc_response.error {
            return Err(anyhow!("RPC error: {}", error.message));
        }

        rpc_response
            .result
            .ok_or_else(|| anyhow!("Missing result in RPC response"))
    }

    pub async fn test_connection(&self) -> Result<()> {
        debug!("Testing connection to node");
        self.get_mining_info().await?;
        //info!("Successfully connected to node");
        Ok(())
    }
}

impl Clone for NodeClient {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            base_url: self.base_url.clone(),
            request_id: AtomicU32::new(self.request_id.load(Ordering::SeqCst)),
        }
    }
}
