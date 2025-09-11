mod config;
mod mining;
mod network;
mod utils;

use anyhow::Result;
use clap::{Arg, Command};
use config::MiningConfig;
use mining::solo::SoloMiner;
use tracing::{info, error};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    //tracing_subscriber::init();
    

    // Parse command line arguments
    let matches = Command::new("hyperion-miner")
        .version("0.1.0")
        .about("Hyperion cryptocurrency miner")
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .value_name("FILE")
                .help("Configuration file path")
                .default_value("config.toml")
        )
        .arg(
            Arg::new("node-url")
                .short('n')
                .long("node-url")
                .value_name("URL")
                .help("Hyperion node URL")
                .default_value("http://127.0.0.1:6001")
        )
        .arg(
            Arg::new("threads")
                .short('t')
                .long("threads")
                .value_name("NUMBER")
                .help("Number of mining threads")
        )
        .get_matches();

    // Load configuration
    let config_path = matches.get_one::<String>("config").unwrap();
    let mut config = MiningConfig::load(config_path)?;

    // Override config with CLI arguments
    if let Some(node_url) = matches.get_one::<String>("node-url") {
        config.node_url = node_url.clone();
    }
    if let Some(threads_str) = matches.get_one::<String>("threads") {
        config.threads = threads_str.parse()?;
    }

    println!("Starting Hyperion Miner");
    println!("Node URL: {}", config.node_url);
    println!("Mining threads: {}", config.threads);

    // Start mining
    let mut miner = SoloMiner::new(config).await?;
    
    // Handle graceful shutdown
    let shutdown = tokio::signal::ctrl_c();
    
    tokio::select! {
        result = miner.start_mining() => {
            if let Err(e) = result {
                error!("Mining error: {}", e);
            }
        }
        _ = shutdown => {
            println!("Received shutdown signal, stopping miner...");
            miner.stop().await?;
        }
    }

    println!("Miner stopped");
    Ok(())
}