mod config;
mod mining;
mod network;
mod utils;

use anyhow::Result;
use clap::{Arg, Command};
use config::MiningConfig;
use mining::solo::SoloMiner;
use tracing::{info, warn, error, debug};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry};
use tracing_appender::non_blocking;
use tracing_rolling_file::RollingFileAppender;

#[tokio::main]
async fn main() -> Result<()> {
    let _log_guard = init_logging().unwrap_or_else(|e| {
        eprintln!("Failed to initialize logging: {}", e);
        std::process::exit(1);
    });
    
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

    info!("Starting Hyperion Miner...");
    info!("Node URL: {}", config.node_url);
    info!("Mining threads: {}", config.threads);

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
            info!("Received shutdown signal, stopping miner...");
            miner.stop().await?;
        }
    }

    info!("Miner stopped.");
    Ok(())
}

fn init_logging() -> Result<()> {
    // let file_appender = RollingFileAppender::builder()
    //     .filename("logs/hyperion-node.log".to_string())
    //     .max_filecount(9)
    //     .condition_max_file_size(10 * 1024 * 1024)
    //     .build()
    //     .map_err(|e| anyhow::anyhow!("Failed to create file appender: {}", e))?;

    // let (file_writer, guard) = non_blocking(file_appender);

    let console_layer = tracing_subscriber::fmt::layer()
        .with_target(false)
        .with_thread_ids(false)
        .with_level(false)
        .with_ansi(true)
        .compact();

    // let file_layer = tracing_subscriber::fmt::layer()
    //     .with_writer(file_writer)
    //     .json()
    //     .with_target(true)
    //     .with_thread_ids(true)
    //     .with_current_span(false);

    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("hyperion_miner=info,hyperion_core=info"));

    Registry::default()
        .with(env_filter)
        .with(console_layer)
        //.with(file_layer)
        .init();

    Ok(())
}