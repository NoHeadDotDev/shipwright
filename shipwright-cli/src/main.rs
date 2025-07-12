use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tracing::info;
use tracing_subscriber::EnvFilter;

mod commands;
mod config;
mod error;
mod workspace;

use commands::{build::BuildCommand, dev::DevCommand, new::NewCommand, serve::ServeCommand};
use error::ShipwrightError;

/// Shipwright CLI - Enhanced hot reload system for Rust web applications
/// 
/// Examples:
///   shipwright dev                    # Start development server with hot reload
///   shipwright serve                  # Start production server
///   shipwright build                  # Build application for production
///   shipwright dev --port 3000        # Start dev server on custom port
///   shipwright serve --release        # Serve optimized build
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Set logging level (trace, debug, info, warn, error)
    #[arg(long, global = true, default_value = "info")]
    log_level: String,

    /// Path to Shipwright.toml config file
    #[arg(long, global = true)]
    config: Option<PathBuf>,

    /// Working directory (defaults to current directory)
    #[arg(long, global = true)]
    cwd: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Commands {
    /// Start development server with hot reload
    Dev(DevCommand),
    /// Start production server
    Serve(ServeCommand),
    /// Build application for production
    Build(BuildCommand),
    /// Create a new Shipwright project from a template
    New(NewCommand),
}

#[tokio::main]
async fn main() -> Result<(), ShipwrightError> {
    let cli = Cli::parse();

    // Initialize logging
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(&cli.log_level));

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .init();

    // Change working directory if specified
    if let Some(cwd) = &cli.cwd {
        std::env::set_current_dir(cwd).map_err(|e| {
            ShipwrightError::IoError(format!("Failed to change directory to {}: {}", cwd.display(), e))
        })?;
    }

    // Load configuration
    let config_path = cli.config.unwrap_or_else(|| PathBuf::from("Shipwright.toml"));
    let config = config::Config::load(&config_path)?;

    info!("Starting Shipwright CLI v{}", env!("CARGO_PKG_VERSION"));

    // Execute command
    match cli.command {
        Commands::Dev(cmd) => cmd.run(config).await,
        Commands::Serve(cmd) => cmd.run(config).await,
        Commands::Build(cmd) => cmd.run(config).await,
        Commands::New(cmd) => cmd.run().await,
    }
}