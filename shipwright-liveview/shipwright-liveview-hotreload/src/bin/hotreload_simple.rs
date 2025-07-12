//! Simple hot reload server for testing

use std::net::SocketAddr;
use tokio::net::TcpListener;
use axum::{routing::get, Router};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔥 Starting simple hot reload server on port 3001");
    
    let app = Router::new()
        .route("/", get(|| async { "Hot reload server running!" }))
        .route("/ws", get(|| async { "WebSocket endpoint ready" }));
    
    let addr = SocketAddr::from(([127, 0, 0, 1], 3001));
    let listener = TcpListener::bind(addr).await?;
    
    println!("✅ Hot reload server listening on {}", addr);
    axum::serve(listener, app).await?;
    
    Ok(())
}
