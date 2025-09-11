use anyhow::Result;
use serde::{Serialize, Deserialize};
use std::fs;
use std::path::Path;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiningConfig {
    pub node_url: String,
    pub threads: usize,
    pub reconnect_delay: u64,
    pub work_update_interval: u64,
    pub stats_interval: u64,
    pub log_level: String,
}

impl MiningConfig {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        if path.as_ref().exists() {
            let content = fs::read_to_string(path)?;
            let config: Self = toml::from_str(&content)?;
            Ok(config)
        } else {
            // Create default config file
            let default = Self::default();
            let content = toml::to_string_pretty(&default)?;
            fs::write(path, content)?;
            Ok(default)
        }
    }
}

impl Default for MiningConfig {
    fn default() -> Self {
        Self {
            node_url: "http://127.0.0.1:45154".to_string(),
            threads: num_cpus::get(),
            reconnect_delay: 5,
            work_update_interval: 1000,  // ms
            stats_interval: 30,  // seconds
            log_level: "info".to_string(),
        }
    }
}
