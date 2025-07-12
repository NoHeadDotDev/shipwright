//! CLI tool for running the hot reload server

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::net::SocketAddr;
use std::path::PathBuf;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

use shipwright_liveview_hotreload::HotReloadServer;

#[derive(Parser)]
#[command(name = "shipwright-hotreload")]
#[command(about = "Hot reload server for Shipwright LiveView", long_about = None)]
struct Cli {
    /// Port to listen on
    #[arg(short, long, default_value = "3001")]
    port: u16,

    /// Host to bind to
    #[arg(short = 'H', long, default_value = "127.0.0.1")]
    host: String,

    /// Paths to watch for changes
    #[arg(short, long, default_value = ".")]
    watch: Vec<PathBuf>,

    /// Log level (trace, debug, info, warn, error)
    #[arg(short, long, default_value = "info")]
    log_level: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    let level = match cli.log_level.to_lowercase().as_str() {
        "trace" => Level::TRACE,
        "debug" => Level::DEBUG,
        "info" => Level::INFO,
        "warn" => Level::WARN,
        "error" => Level::ERROR,
        _ => Level::INFO,
    };

    let subscriber = FmtSubscriber::builder()
        .with_max_level(level)
        .with_target(false)
        .with_thread_ids(false)
        .with_thread_names(false)
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .context("Failed to set tracing subscriber")?;

    // Parse address
    let addr: SocketAddr = format!("{}:{}", cli.host, cli.port)
        .parse()
        .context("Invalid address")?;

    // Resolve watch paths
    let watch_paths: Vec<PathBuf> = cli.watch
        .into_iter()
        .map(|p| {
            if p.is_absolute() {
                p
            } else {
                std::env::current_dir()
                    .unwrap_or_default()
                    .join(&p)
                    .canonicalize()
                    .unwrap_or(p)
            }
        })
        .collect();

    info!("Starting Shipwright LiveView hot reload server");
    info!("Listening on: {}", addr);
    info!("Watching paths:");
    for path in &watch_paths {
        info!("  - {}", path.display());
    }

    // Create and start server
    let server = HotReloadServer::new(addr, watch_paths);
    server.start().await?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn test_cli_parsing() {
        Cli::command().debug_assert();
    }
}