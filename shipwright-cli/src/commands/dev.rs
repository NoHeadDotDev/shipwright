use clap::Args;
use std::path::PathBuf;
use tracing::{info, warn, error};
use tokio::signal;
use std::process::Stdio;
use tokio::io::AsyncBufReadExt;

use crate::{config::Config, error::ShipwrightError};
use super::CommandContext;

/// Start development server with hot reload
/// 
/// This command starts a development server that automatically reloads
/// when source files change. It combines both the application server
/// and the hot reload server into a single process.
/// 
/// Examples:
///   shipwright dev                    # Start on default port (8080)
///   shipwright dev --port 3000        # Start on custom port
///   shipwright dev --host 0.0.0.0     # Listen on all interfaces
///   shipwright dev --no-hot-reload    # Disable hot reload
///   shipwright dev --features "ssr"   # Enable specific features
#[derive(Args, Debug)]
pub struct DevCommand {
    /// Host to bind the server to
    #[arg(long, short = 'H')]
    pub host: Option<String>,

    /// Port to bind the server to
    #[arg(long, short = 'p')]
    pub port: Option<u16>,

    /// Disable hot reload functionality
    #[arg(long)]
    pub no_hot_reload: bool,

    /// Enable release mode optimizations
    #[arg(long)]
    pub release: bool,

    /// Features to enable during compilation
    #[arg(long, value_delimiter = ',')]
    pub features: Vec<String>,

    /// Additional arguments to pass to cargo
    #[arg(last = true)]
    pub cargo_args: Vec<String>,

    /// Open browser automatically
    #[arg(long)]
    pub open: bool,

    /// Target package to build (for workspaces)
    #[arg(long)]
    pub package: Option<String>,
}

impl DevCommand {
    pub async fn run(&self, config: Config) -> Result<(), ShipwrightError> {
        let ctx = CommandContext::new(config)?;
        
        info!("Starting Shipwright development server...");
        
        // Determine server configuration
        let serve_config = ctx.config.serve();
        let host = self.host.as_deref()
            .or(serve_config.host.as_deref())
            .unwrap_or("localhost");
        
        let port = self.port
            .or(serve_config.port)
            .unwrap_or(8080);
        
        let server_url = format!("http://{}:{}", host, port);
        
        info!("Server will start at: {}", server_url);
        
        // Check if hot reload is enabled
        let hot_reload_enabled = !self.no_hot_reload && 
            ctx.config.hot_reload().enabled.unwrap_or(true);
        
        if hot_reload_enabled {
            info!("Hot reload enabled");
        } else {
            warn!("Hot reload disabled");
        }
        
        // Build the project initially
        self.build_project(&ctx).await?;
        
        // Start the application server (not a static server)
        let server_handle = self.start_application(&ctx).await?;
        
        // Start hot reload server if enabled
        let hotreload_handle = if hot_reload_enabled {
            Some(self.start_hot_reload_server(&ctx).await?)
        } else {
            None
        };
        
        // Start hot reload watcher if enabled
        let _watcher_handle = if hot_reload_enabled {
            Some(self.start_hot_reload_watcher(&ctx).await?)
        } else {
            None
        };
        
        // Open browser if requested
        if self.open {
            if let Err(e) = open::that(&server_url) {
                warn!("Failed to open browser: {}", e);
            }
        }
        
        info!("Development server running at {}", server_url);
        info!("Press Ctrl+C to stop the server");
        
        // Wait for shutdown signal
        match signal::ctrl_c().await {
            Ok(()) => {
                info!("Received shutdown signal, stopping server...");
            }
            Err(e) => {
                error!("Failed to listen for shutdown signal: {}", e);
            }
        }
        
        // Cleanup
        server_handle.abort();
        if let Some(handle) = hotreload_handle {
            handle.abort();
        }
        info!("Development server stopped");
        
        Ok(())
    }
    
    async fn build_project(&self, ctx: &CommandContext) -> Result<(), ShipwrightError> {
        info!("Building project...");
        
        let mut cmd = tokio::process::Command::new("cargo");
        cmd.arg("build");
        
        if self.release {
            cmd.arg("--release");
        }
        
        if let Some(package) = &self.package {
            cmd.args(["--package", package]);
        } else if let Some(main_package) = ctx.workspace.get_main_package() {
            cmd.args(["--package", &main_package.name]);
        }
        
        if !self.features.is_empty() {
            cmd.args(["--features", &self.features.join(",")]);
        }
        
        cmd.args(&self.cargo_args);
        
        // Set environment variables
        for (key, value) in ctx.workspace.get_build_env(&ctx.config) {
            cmd.env(key, value);
        }
        
        cmd.stdout(Stdio::piped())
           .stderr(Stdio::piped());
        
        let output = cmd.output().await?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(ShipwrightError::BuildError(format!(
                "Build failed:\n{}", stderr
            )));
        }
        
        info!("Build completed successfully");
        Ok(())
    }
    
    async fn start_application(&self, ctx: &CommandContext) -> Result<tokio::task::JoinHandle<()>, ShipwrightError> {
        // Instead of creating our own server, run the actual application
        let mut cmd = tokio::process::Command::new("cargo");
        cmd.arg("run")
           .current_dir(&ctx.workspace.root)
           .env("RUST_LOG", "info")
           .env("SHIPWRIGHT_DEV_MODE", "1")
           .kill_on_drop(true);
        
        // If we have a target package, specify it
        if let Some(ref package) = ctx.workspace.target_package {
            cmd.arg("--package").arg(&package.name);
        }
        
        let mut cargo_run = cmd.spawn()
            .map_err(|e| ShipwrightError::BuildError(format!("Failed to start application: {}", e)))?;
        
        let server_handle = tokio::spawn(async move {
            match cargo_run.wait().await {
                Ok(status) => {
                    if !status.success() {
                        error!("Application exited with error code: {}", status.code().unwrap_or(-1));
                    }
                }
                Err(e) => {
                    error!("Failed to run application: {}", e);
                }
            }
        });
        
        // Give the application time to start
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        
        Ok(server_handle)
    }
    
    async fn start_hot_reload_watcher(&self, ctx: &CommandContext) -> Result<tokio::task::JoinHandle<()>, ShipwrightError> {
        use notify::{Watcher, RecursiveMode};
        use std::sync::mpsc;
        use std::time::Duration;
        
        let (tx, rx) = mpsc::channel();
        let mut watcher = notify::recommended_watcher(tx)?;
        
        // Add watch paths
        for path in ctx.workspace.get_watch_paths(&ctx.config) {
            if path.exists() {
                watcher.watch(&path, RecursiveMode::Recursive)?;
                info!("Watching: {}", path.display());
            }
        }
        
        let ignore_paths = ctx.workspace.get_ignore_paths(&ctx.config);
        let ctx_clone = ctx.clone();
        let self_clone = self.clone();
        
        let watcher_handle = tokio::spawn(async move {
            let mut last_reload = std::time::Instant::now();
            let debounce_duration = Duration::from_millis(
                ctx_clone.config.hot_reload().debounce_ms.unwrap_or(300)
            );
            
            while let Ok(event) = rx.recv() {
                if let Ok(event) = event {
                    if should_trigger_reload(&event, &ignore_paths) {
                        let now = std::time::Instant::now();
                        if now.duration_since(last_reload) >= debounce_duration {
                            info!("File change detected, rebuilding...");
                            
                            if let Err(e) = self_clone.build_project(&ctx_clone).await {
                                error!("Rebuild failed: {}", e);
                            } else {
                                info!("Rebuild completed");
                                // TODO: Trigger browser reload via WebSocket
                            }
                            
                            last_reload = now;
                        }
                    }
                }
            }
        });
        
        Ok(watcher_handle)
    }
    
    async fn start_hot_reload_server(&self, ctx: &CommandContext) -> Result<tokio::task::JoinHandle<()>, ShipwrightError> {
        info!("Starting hot reload server...");
        
        // Determine the path to the hot reload server binary
        // First try the relative path from workspace root
        let mut hotreload_dir = ctx.workspace.root
            .parent()
            .ok_or_else(|| ShipwrightError::BuildError("Could not find parent directory".to_string()))?
            .join("shipwright-liveview/shipwright-liveview-hotreload");
        
        // If that doesn't exist, try looking for it in the current workspace root 
        if !hotreload_dir.exists() {
            hotreload_dir = ctx.workspace.root.join("shipwright-liveview-hotreload");
        }
        
        // If still not found, try going up multiple levels to find shipwright-liveview
        if !hotreload_dir.exists() {
            let mut current = ctx.workspace.root.clone();
            while let Some(parent) = current.parent() {
                let candidate = parent.join("shipwright-liveview/shipwright-liveview-hotreload");
                if candidate.exists() {
                    hotreload_dir = candidate;
                    break;
                }
                current = parent.to_path_buf();
            }
        }
        
        if !hotreload_dir.exists() {
            return Err(ShipwrightError::BuildError(
                format!("Hot reload server directory not found: {}", hotreload_dir.display())
            ));
        }
        
        // Start the hot reload server
        info!("Hot reload server directory: {}", hotreload_dir.display());
        info!("Watch path: {}", ctx.workspace.root.display());
        
        let mut cmd = tokio::process::Command::new("cargo");
        cmd.arg("run")
           .arg("--bin")
           .arg("shipwright-hotreload")
           .arg("--")
           .arg("--port")
           .arg("3001")
           .arg("--host")
           .arg("127.0.0.1")
           .arg("--watch")
           .arg(&ctx.workspace.root)
           .current_dir(&hotreload_dir)
           .env("RUST_LOG", "debug")
           .stdout(Stdio::piped())
           .stderr(Stdio::piped())
           .kill_on_drop(true);
        
        // Log the exact command being executed
        info!("Executing: cargo run --bin shipwright-hotreload -- --port 3001 --host 127.0.0.1 --watch {}", ctx.workspace.root.display());
        info!("Working directory: {}", hotreload_dir.display());
        
        let mut hotreload_process = cmd.spawn()
            .map_err(|e| ShipwrightError::BuildError(format!("Failed to start hot reload server: {}", e)))?;
        
        // Capture stdout and stderr for monitoring
        let stdout = hotreload_process.stdout.take().expect("Failed to capture stdout");
        let stderr = hotreload_process.stderr.take().expect("Failed to capture stderr");
        
        // Monitor stdout
        let stdout_handle = tokio::spawn(async move {
            let mut reader = tokio::io::BufReader::new(stdout);
            let mut line = String::new();
            loop {
                line.clear();
                match tokio::io::AsyncBufReadExt::read_line(&mut reader, &mut line).await {
                    Ok(0) => break, // EOF
                    Ok(_) => {
                        let trimmed = line.trim();
                        if !trimmed.is_empty() {
                            info!("[HotReload] {}", trimmed);
                        }
                    }
                    Err(e) => {
                        error!("Error reading hot reload stdout: {}", e);
                        break;
                    }
                }
            }
        });
        
        // Monitor stderr
        let stderr_handle = tokio::spawn(async move {
            let mut reader = tokio::io::BufReader::new(stderr);
            let mut line = String::new();
            loop {
                line.clear();
                match tokio::io::AsyncBufReadExt::read_line(&mut reader, &mut line).await {
                    Ok(0) => break, // EOF
                    Ok(_) => {
                        let trimmed = line.trim();
                        if !trimmed.is_empty() {
                            error!("[HotReload Error] {}", trimmed);
                        }
                    }
                    Err(e) => {
                        error!("Error reading hot reload stderr: {}", e);
                        break;
                    }
                }
            }
        });
        
        let hotreload_handle = tokio::spawn(async move {
            match hotreload_process.wait().await {
                Ok(status) => {
                    if !status.success() {
                        error!("🔥 Hot reload server process exited with code: {}", status.code().unwrap_or(-1));
                        error!("🔥 This indicates the hot reload server failed to start properly");
                    } else {
                        info!("🔥 Hot reload server process completed successfully");
                    }
                }
                Err(e) => {
                    error!("🔥 Failed to wait for hot reload server process: {}", e);
                }
            }
            
            // Wait for output handlers to complete
            let _ = tokio::join!(stdout_handle, stderr_handle);
        });
        
        // Give the hot reload server time to start
        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
        
        // Health check: verify the server is actually listening
        let health_check_result = tokio::time::timeout(
            tokio::time::Duration::from_secs(2),
            async {
                for attempt in 1..=5 {
                    match tokio::net::TcpStream::connect("127.0.0.1:3001").await {
                        Ok(_) => {
                            info!("✅ Hot reload server health check passed (attempt {})", attempt);
                            return Ok(());
                        }
                        Err(e) => {
                            warn!("❌ Hot reload server health check failed (attempt {}): {}", attempt, e);
                            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                        }
                    }
                }
                Err("Hot reload server is not responding on port 3001")
            }
        ).await;
        
        match health_check_result {
            Ok(Ok(())) => {
                info!("🔥 Hot reload server confirmed listening on http://127.0.0.1:3001");
            }
            Ok(Err(_)) | Err(_) => {
                error!("🔥 Hot reload server health check failed");
                error!("🔥 The hot reload server process started but is not accepting connections");
                error!("🔥 Check the hot reload server logs above for errors");
            }
        }
        
        Ok(hotreload_handle)
    }
}

// Helper function to determine if a file change should trigger a reload
fn should_trigger_reload(event: &notify::Event, ignore_paths: &[PathBuf]) -> bool {
    match event.kind {
        notify::EventKind::Create(_) | notify::EventKind::Modify(_) | notify::EventKind::Remove(_) => {
            for path in &event.paths {
                // Check if path should be ignored
                for ignore_path in ignore_paths {
                    if path.starts_with(ignore_path) {
                        return false;
                    }
                }
                
                // Check file extension
                if let Some(ext) = path.extension() {
                    match ext.to_str() {
                        Some("rs") | Some("toml") | Some("html") | Some("css") | Some("js") => {
                            return true;
                        }
                        _ => {}
                    }
                }
            }
            false
        }
        _ => false,
    }
}

// HTTP handlers
async fn serve_index() -> axum::response::Html<String> {
    // TODO: Serve the actual index.html with hot reload script injected
    axum::response::Html(r#"
<!DOCTYPE html>
<html>
<head>
    <title>Shipwright Dev Server</title>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
</head>
<body>
    <h1>Shipwright Development Server</h1>
    <p>Your application is running in development mode.</p>
    <script>
        // Hot reload WebSocket connection will be injected here
        console.log('Shipwright dev server ready');
    </script>
</body>
</html>
    "#.to_string())
}

async fn health_check() -> &'static str {
    "OK"
}

async fn reload_endpoint() -> &'static str {
    // TODO: Implement WebSocket endpoint for hot reload
    "Hot reload endpoint"
}

impl Clone for DevCommand {
    fn clone(&self) -> Self {
        Self {
            host: self.host.clone(),
            port: self.port,
            no_hot_reload: self.no_hot_reload,
            release: self.release,
            features: self.features.clone(),
            cargo_args: self.cargo_args.clone(),
            open: self.open,
            package: self.package.clone(),
        }
    }
}

impl Clone for CommandContext {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            workspace: self.workspace.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_should_trigger_reload() {
        use std::path::PathBuf;
        
        let ignore_paths = vec![PathBuf::from("target"), PathBuf::from("dist")];
        
        // Test Rust file change
        let event = notify::Event {
            kind: notify::EventKind::Modify(notify::event::ModifyKind::Data(
                notify::event::DataChange::Content
            )),
            paths: vec![PathBuf::from("src/main.rs")],
            attrs: Default::default(),
        };
        assert!(should_trigger_reload(&event, &ignore_paths));
        
        // Test ignored path
        let event = notify::Event {
            kind: notify::EventKind::Modify(notify::event::ModifyKind::Data(
                notify::event::DataChange::Content
            )),
            paths: vec![PathBuf::from("target/debug/main")],
            attrs: Default::default(),
        };
        assert!(!should_trigger_reload(&event, &ignore_paths));
    }
}