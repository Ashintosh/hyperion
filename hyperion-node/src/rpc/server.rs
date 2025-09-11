use super::handlers::*;
use super::types::*;
use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::post,
    Router,
};
use serde_json::Value;
use tower_http::cors::CorsLayer;
use tracing::debug;

pub fn create_router(state: NodeState) -> Router {
    Router::new()
        .route("/", post(handle_rpc))
        .route("/rpc", post(handle_rpc))
        .layer(CorsLayer::permissive())
        .with_state(state)
}

pub async fn handle_rpc(
    state: State<NodeState>,
    Json(request): Json<Value>,
) -> Result<Json<RpcResponse<Value>>, StatusCode> {
    debug!("RPC request: {}", request);

    // Parse the request
    let rpc_req: RpcRequest<Value> = match serde_json::from_value(request) {
        Ok(req) => req,
        Err(e) => {
            return Ok(Json(RpcResponse {
                jsonrpc: "2.0".to_string(),
                id: Value::Null,
                result: None,
                error: Some(RpcError::invalid_params(&e.to_string())),
            }));
        }
    };

    let response = match rpc_req.method.as_str() {
        "get_block_template" => {
            match get_block_template(state, rpc_req.params).await {
                Ok(result) => RpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: rpc_req.id,
                    result: Some(serde_json::to_value(result).unwrap()),
                    error: None,
                },
                Err(error) => RpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: rpc_req.id,
                    result: None,
                    error: Some(error),
                },
            }
        }
        "submit_block" => {
            let params: Option<SubmitBlockParams> = rpc_req.params
                .map(|p| serde_json::from_value(p))
                .transpose()
                .map_err(|e| RpcError::invalid_params(&e.to_string()))
                .unwrap_or(None);

            match submit_block(state, params).await {
                Ok(result) => RpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: rpc_req.id,
                    result: Some(serde_json::to_value(result).unwrap()),
                    error: None,
                },
                Err(error) => RpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: rpc_req.id,
                    result: None,
                    error: Some(error),
                },
            }
        }
        "get_mining_info" => {
            match get_mining_info(state, rpc_req.params).await {
                Ok(result) => RpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: rpc_req.id,
                    result: Some(serde_json::to_value(result).unwrap()),
                    error: None,
                },
                Err(error) => RpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: rpc_req.id,
                    result: None,
                    error: Some(error),
                },
            }
        }
        "get_blockchain_info" => {
            match get_blockchain_info(state, rpc_req.params).await {
                Ok(result) => RpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: rpc_req.id,
                    result: Some(serde_json::to_value(result).unwrap()),
                    error: None,
                },
                Err(error) => RpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: rpc_req.id,
                    result: None,
                    error: Some(error),
                },
            }
        }
        "get_block_count" => {
            match get_block_count(state, rpc_req.params).await {
                Ok(result) => RpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: rpc_req.id,
                    result: Some(serde_json::to_value(result).unwrap()),
                    error: None,
                },
                Err(error) => RpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: rpc_req.id,
                    result: None,
                    error: Some(error),
                },
            }
        }
        _ => RpcResponse {
            jsonrpc: "2.0".to_string(),
            id: rpc_req.id,
            result: None,
            error: Some(RpcError::method_not_found()),
        },
    };

    Ok(Json(response))
}

pub async fn start_server(state: NodeState, port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let app = create_router(state);

    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", port)).await?;
    println!("RPC server listening on http://127.0.0.1:{}", port);

    axum::serve(listener, app).await?;
    Ok(())
}