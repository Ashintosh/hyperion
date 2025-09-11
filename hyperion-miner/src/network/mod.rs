pub mod node_client;
pub mod rpc;

pub use node_client::NodeClient;
pub use rpc::{BlockTemplate, MiningInfo, SubmitBlockRequest};