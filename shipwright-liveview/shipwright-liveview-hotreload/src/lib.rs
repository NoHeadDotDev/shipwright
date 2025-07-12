//! Hot reload infrastructure for Shipwright LiveView

pub mod parser;
pub mod protocol;
pub mod server;
pub mod watcher;
pub mod template_cache;
pub mod runtime;
pub mod diff_integration;
pub mod template_diff;

pub use protocol::{HotReloadMessage, TemplateUpdate, TemplateId};
pub use server::HotReloadServer;
pub use watcher::FileWatcher;
pub use template_cache::TemplateCache;

use std::path::PathBuf;
use std::net::SocketAddr;

/// Initialize hot reload with default settings
pub fn init_hot_reload() {
    init_hot_reload_with_config(HotReloadConfig::default());
}

/// Initialize hot reload with custom configuration
pub fn init_hot_reload_with_config(config: HotReloadConfig) {
    let addr = config.addr;
    let watch_paths = config.watch_paths.clone();
    
    tokio::spawn(async move {
        if let Err(e) = start_hot_reload_server(config).await {
            eprintln!("🔥 Hot reload server error: {}", e);
        }
    });
    
    println!("🔥 Hot reload server starting on {}...", addr);
    println!("📝 Watching for changes in: {:?}", watch_paths);
}

/// Start the hot reload server
async fn start_hot_reload_server(config: HotReloadConfig) -> anyhow::Result<()> {
    let server = HotReloadServer::new(config.addr, config.watch_paths);
    server.start().await
}

/// Configuration for hot reload server
#[derive(Debug, Clone)]
pub struct HotReloadConfig {
    /// Server address to bind to
    pub addr: SocketAddr,
    /// Paths to watch for changes
    pub watch_paths: Vec<PathBuf>,
}

impl Default for HotReloadConfig {
    fn default() -> Self {
        Self {
            addr: SocketAddr::from(([127, 0, 0, 1], 3001)),
            watch_paths: vec![
                PathBuf::from("src"),
                PathBuf::from("templates"),
                PathBuf::from("assets"),
            ],
        }
    }
}

impl HotReloadConfig {
    /// Create a new config with custom address
    pub fn with_addr(mut self, addr: SocketAddr) -> Self {
        self.addr = addr;
        self
    }
    
    /// Add a watch path
    pub fn add_watch_path<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.watch_paths.push(path.into());
        self
    }
    
    /// Set watch paths
    pub fn with_watch_paths(mut self, paths: Vec<PathBuf>) -> Self {
        self.watch_paths = paths;
        self
    }
}
