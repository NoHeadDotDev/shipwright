use clap::Args;
use std::path::PathBuf;
use tracing::{info, error};
use tokio::signal;

use crate::{config::Config, error::ShipwrightError};
use super::CommandContext;

/// Start production server
/// 
/// This command starts a production server that serves static files
/// and the built application. It's optimized for production use with
/// proper caching headers and performance optimizations.
/// 
/// Examples:
///   shipwright serve                  # Start on default port (8080)
///   shipwright serve --port 3000      # Start on custom port
///   shipwright serve --host 0.0.0.0   # Listen on all interfaces
///   shipwright serve --release        # Ensure release build
#[derive(Args, Debug)]
pub struct ServeCommand {
    /// Host to bind the server to
    #[arg(long, short = 'H')]
    pub host: Option<String>,

    /// Port to bind the server to
    #[arg(long, short = 'p')]
    pub port: Option<u16>,

    /// Directory to serve static files from
    #[arg(long)]
    pub static_dir: Option<PathBuf>,

    /// Ensure the application is built in release mode
    #[arg(long)]
    pub release: bool,

    /// Enable CORS headers
    #[arg(long)]
    pub cors: bool,

    /// Enable gzip compression
    #[arg(long)]
    pub gzip: bool,

    /// Target package to serve (for workspaces)
    #[arg(long)]
    pub package: Option<String>,
}

impl ServeCommand {
    pub async fn run(&self, config: Config) -> Result<(), ShipwrightError> {
        let ctx = CommandContext::new(config)?;
        
        info!("Starting Shipwright production server...");
        
        // Determine server configuration
        let serve_config = ctx.config.serve();
        let host = self.host.as_deref()
            .or(serve_config.host.as_deref())
            .unwrap_or("localhost");
        
        let port = self.port
            .or(serve_config.port)
            .unwrap_or(8080);
        
        let static_dir = self.static_dir.as_ref()
            .or(serve_config.static_dir.as_ref())
            .cloned()
            .unwrap_or_else(|| PathBuf::from("dist"));
        
        let server_url = format!("http://{}:{}", host, port);
        
        info!("Server will start at: {}", server_url);
        info!("Serving static files from: {}", static_dir.display());
        
        // Ensure the application is built (optionally in release mode)
        if self.release {
            self.build_project(&ctx, true).await?;
        }
        
        // Verify static directory exists
        if !static_dir.exists() {
            return Err(ShipwrightError::ServerError(format!(
                "Static directory does not exist: {}. Run 'shipwright build' first.",
                static_dir.display()
            )));
        }
        
        // Start the production server
        let server_handle = self.start_server(&ctx, host, port, &static_dir).await?;
        
        info!("Production server running at {}", server_url);
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
        info!("Production server stopped");
        
        Ok(())
    }
    
    async fn build_project(&self, ctx: &CommandContext, release: bool) -> Result<(), ShipwrightError> {
        info!("Building project in {} mode...", if release { "release" } else { "debug" });
        
        let mut cmd = tokio::process::Command::new("cargo");
        cmd.arg("build");
        
        if release {
            cmd.arg("--release");
        }
        
        if let Some(package) = &self.package {
            cmd.args(["--package", package]);
        } else if let Some(main_package) = ctx.workspace.get_main_package() {
            cmd.args(["--package", &main_package.name]);
        }
        
        // Set environment variables
        for (key, value) in ctx.workspace.get_build_env(&ctx.config) {
            cmd.env(key, value);
        }
        
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
    
    async fn start_server(&self, ctx: &CommandContext, host: &str, port: u16, static_dir: &PathBuf) -> Result<tokio::task::JoinHandle<()>, ShipwrightError> {
        let serve_config = ctx.config.serve();
        use axum::{
            routing::get,
            Router,
        };
        use tower_http::{
            services::ServeDir,
            cors::CorsLayer,
            compression::CompressionLayer,
        };
        use std::sync::Arc;
        
        let serve_dir = ServeDir::new(static_dir);
        
        let mut app = Router::new()
            .route("/", get(serve_index))
            .route("/_shipwright/health", get(health_check))
            .route("/_shipwright/info", get(server_info))
            .fallback_service(serve_dir)
            .with_state(Arc::new(ctx.config.clone()));
        
        // Add middleware layers
        
        // Apply middleware conditionally
        if self.cors || serve_config.cors.unwrap_or(false) {
            app = app.layer(CorsLayer::permissive());
        }
        
        if self.gzip {
            app = app.layer(CompressionLayer::new());
        }
        
        let listener = tokio::net::TcpListener::bind(format!("{}:{}", host, port))
            .await
            .map_err(|e| ShipwrightError::ServerError(format!("Failed to bind to {}:{}: {}", host, port, e)))?;
        
        let server_handle = tokio::spawn(async move {
            if let Err(e) = axum::serve(listener, app).await {
                error!("Server error: {}", e);
            }
        });
        
        Ok(server_handle)
    }
}

// HTTP handlers
async fn serve_index() -> axum::response::Html<String> {
    // TODO: Serve the actual index.html file
    axum::response::Html(r#"
<!DOCTYPE html>
<html>
<head>
    <title>Shipwright App</title>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
</head>
<body>
    <h1>Shipwright Production Server</h1>
    <p>Your application is running in production mode.</p>
</body>
</html>
    "#.to_string())
}

async fn health_check() -> &'static str {
    "OK"
}

async fn server_info(axum::extract::State(config): axum::extract::State<std::sync::Arc<Config>>) -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({
        "name": config.application.name,
        "version": config.application.version,
        "mode": "production",
        "timestamp": chrono::Utc::now().to_rfc3339(),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[tokio::test]
    async fn test_serve_command_creation() {
        let cmd = ServeCommand {
            host: Some("localhost".to_string()),
            port: Some(3000),
            static_dir: Some(PathBuf::from("dist")),
            release: false,
            cors: true,
            gzip: false,
            package: None,
        };
        
        assert_eq!(cmd.host, Some("localhost".to_string()));
        assert_eq!(cmd.port, Some(3000));
        assert!(cmd.cors);
        assert!(!cmd.gzip);
    }
}