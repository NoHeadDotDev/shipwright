use anyhow::Result;
use std::env;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod app;
mod controllers;
mod middleware;
mod routes;
mod state;

use app::create_app;
use state::AppState;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "{{crate_name}}_web=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let config = {{crate_name}}_config::load_config()?;
    
    // Initialize database connection
    let db_pool = {{crate_name}}_db::establish_connection(&config.database_url).await?;
    
    // Run database migrations
    {{crate_name}}_db::run_migrations(&db_pool).await?;

    // Create application state
    let state = AppState::new(config.clone(), db_pool);

    // Start hot reload server in development
    #[cfg(debug_assertions)]
    {
        tokio::spawn(async {
            let addr = "127.0.0.1:3001".parse().unwrap();
            let watch_paths = vec![std::path::PathBuf::from("src"), std::path::PathBuf::from("assets")];
            let hot_reload_server =
                shipwright_liveview_hotreload::HotReloadServer::new(addr, watch_paths);
            if let Err(e) = hot_reload_server.start().await {
                eprintln!("Hot reload server failed to start: {}", e);
            }
        });
        info!("🔥 Hot reload server started on ws://localhost:3001");
    }

    // Create the application
    let app = create_app(state);

    // Determine the bind address
    let port = env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse::<u16>()
        .unwrap_or(3000);
    
    let bind_addr = format!("0.0.0.0:{}", port);
    
    // Start the server
    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    info!("🚀 Server listening on {}", listener.local_addr()?);
    
    axum::serve(listener, app).await?;

    Ok(())
}