//! Hot reload server implementation

use anyhow::{Context, Result};
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
    routing::get,
    Router,
};
use futures_util::{SinkExt, StreamExt};
use std::{
    net::SocketAddr,
    sync::Arc,
    time::Duration,
};
use tokio::{
    sync::{broadcast, mpsc, RwLock},
    time::interval,
};
use tower_http::cors::CorsLayer;
use tracing::{debug, error, info};

use crate::{
    protocol::{HotReloadMessage, ServerStats, ServerStatusType},
    template_cache::TemplateCache,
    watcher::FileWatcher,
};

/// Hot reload server state
#[derive(Clone)]
struct ServerState {
    /// Template cache
    cache: Arc<TemplateCache>,
    /// Broadcast channel for sending updates to all clients
    broadcast_tx: broadcast::Sender<HotReloadMessage>,
    /// Number of connected clients
    client_count: Arc<RwLock<usize>>,
    /// Number of updates sent
    updates_sent: Arc<RwLock<u64>>,
    /// Server start time
    start_time: std::time::Instant,
}

/// Hot reload server
pub struct HotReloadServer {
    /// Server address
    addr: SocketAddr,
    /// Paths to watch
    watch_paths: Vec<std::path::PathBuf>,
    /// Template cache
    cache: Arc<TemplateCache>,
}

impl HotReloadServer {
    /// Create a new hot reload server
    pub fn new(addr: SocketAddr, watch_paths: Vec<std::path::PathBuf>) -> Self {
        Self {
            addr,
            watch_paths,
            cache: Arc::new(TemplateCache::new()),
        }
    }

    /// Start the hot reload server
    pub async fn start(self) -> Result<()> {
        info!("Starting hot reload server on {}", self.addr);

        // Create broadcast channel for updates
        let (broadcast_tx, _) = broadcast::channel(100);

        // Create server state
        let state = ServerState {
            cache: self.cache.clone(),
            broadcast_tx: broadcast_tx.clone(),
            client_count: Arc::new(RwLock::new(0)),
            updates_sent: Arc::new(RwLock::new(0)),
            start_time: std::time::Instant::now(),
        };

        // Create channel for file watcher updates
        let (update_tx, mut update_rx) = mpsc::channel(100);

        // Start file watcher
        let watcher = FileWatcher::new(
            self.watch_paths.clone(),
            self.cache.clone(),
            update_tx,
        );
        
        tokio::spawn(async move {
            if let Err(e) = watcher.start().await {
                error!("File watcher error: {}", e);
            }
        });

        // Process updates from file watcher
        let broadcast_tx_clone = broadcast_tx.clone();
        let updates_sent_clone = state.updates_sent.clone();
        let client_count_clone = state.client_count.clone();
        tokio::spawn(async move {
            while let Some(updates) = update_rx.recv().await {
                let client_count = *client_count_clone.read().await;
                info!("🔥 Broadcasting {} template updates to {} connected clients", updates.len(), client_count);
                
                for update in updates {
                    info!("📤 Sending template update: {:?} -> {}", update.id, update.hash);
                    let message = HotReloadMessage::TemplateUpdated(update);
                    
                    match broadcast_tx_clone.send(message) {
                        Ok(receiver_count) => {
                            info!("✅ Template update broadcasted to {} receivers", receiver_count);
                            let mut sent_count = updates_sent_clone.write().await;
                            *sent_count += 1;
                        }
                        Err(e) => {
                            error!("❌ Failed to broadcast template update: {}", e);
                        }
                    }
                }
            }
        });

        // Start cache cleanup task
        let cache_clone = self.cache.clone();
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(300)); // 5 minutes
            loop {
                interval.tick().await;
                cache_clone.cleanup();
                debug!("Cache cleanup completed");
            }
        });

        // Build router
        let app = Router::new()
            .route("/ws", get(websocket_handler))
            .route("/health", get(health_check))
            .route("/stats", get(stats_handler))
            .layer(CorsLayer::permissive())
            .with_state(state);

        // Start server
        let listener = tokio::net::TcpListener::bind(&self.addr)
            .await
            .context("Failed to bind to address")?;
        
        info!("Hot reload server listening on {}", self.addr);
        
        axum::serve(listener, app)
            .await
            .context("Server error")?;

        Ok(())
    }
}

/// WebSocket handler
async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<ServerState>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

/// Handle WebSocket connection
async fn handle_socket(socket: WebSocket, state: ServerState) {
    // Increment client count
    {
        let mut count = state.client_count.write().await;
        *count += 1;
        info!("Client connected. Total clients: {}", *count);
    }

    // Split socket
    let (mut sender, mut receiver) = socket.split();

    // Send initial connection message
    let connect_msg = HotReloadMessage::Connected {
        version: "1.0.0".to_string(),
    };
    
    if let Ok(json) = connect_msg.to_json() {
        let _ = sender.send(Message::Text(json)).await;
    }

    // Don't send cached templates to new clients to avoid triggering reloads
    // Cached templates are not "updates" - they're existing state
    // Only send actual changes that happen after connection

    // Subscribe to broadcasts
    let mut broadcast_rx = state.broadcast_tx.subscribe();

    // Start ping task
    let mut ping_interval = interval(Duration::from_secs(30));

    // Handle messages
    loop {
        tokio::select! {
            // Handle incoming messages
            msg = receiver.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        if let Ok(msg) = HotReloadMessage::from_json(&text) {
                            handle_client_message(msg, &state, &mut sender).await;
                        }
                    }
                    Some(Ok(Message::Pong(_))) => {
                        debug!("Received pong");
                    }
                    Some(Ok(Message::Close(_))) => {
                        info!("Client disconnected");
                        break;
                    }
                    _ => {}
                }
            }
            
            // Handle broadcast messages
            msg = broadcast_rx.recv() => {
                match msg {
                    Ok(message) => {
                        if let Ok(json) = message.to_json() {
                            if sender.send(Message::Text(json)).await.is_err() {
                                break;
                            }
                        }
                    }
                    Err(_) => {
                        error!("Broadcast channel closed");
                        break;
                    }
                }
            }
            
            // Send periodic pings
            _ = ping_interval.tick() => {
                if sender.send(Message::Ping(vec![])).await.is_err() {
                    break;
                }
            }
        }
    }

    // Decrement client count
    {
        let mut count = state.client_count.write().await;
        *count = count.saturating_sub(1);
        info!("Client disconnected. Total clients: {}", *count);
    }
}

/// Handle client messages
async fn handle_client_message(
    msg: HotReloadMessage,
    state: &ServerState,
    sender: &mut futures_util::stream::SplitSink<WebSocket, Message>,
) {
    match msg {
        HotReloadMessage::ReloadRequest { template_id } => {
            debug!("Client requested reload for template: {:?}", template_id);
            
            if let Some(update) = state.cache.get(&template_id) {
                let response = HotReloadMessage::TemplateUpdated(update);
                if let Ok(json) = response.to_json() {
                    let _ = sender.send(Message::Text(json)).await;
                }
            } else {
                let error = HotReloadMessage::Error {
                    message: "Template not found in cache".to_string(),
                    code: Some("TEMPLATE_NOT_FOUND".to_string()),
                    suggestions: Some(vec!["Try saving the file again".to_string()]),
                };
                if let Ok(json) = error.to_json() {
                    let _ = sender.send(Message::Text(json)).await;
                }
            }
        }
        HotReloadMessage::Ping => {
            let pong = HotReloadMessage::Pong;
            if let Ok(json) = pong.to_json() {
                let _ = sender.send(Message::Text(json)).await;
            }
        }
        _ => {
            debug!("Received unexpected message from client: {:?}", msg);
        }
    }
}

/// Health check endpoint
async fn health_check() -> impl IntoResponse {
    "OK"
}

/// Stats endpoint
async fn stats_handler(State(state): State<ServerState>) -> impl IntoResponse {
    let client_count = *state.client_count.read().await;
    let updates_sent = *state.updates_sent.read().await;
    let cache_stats = state.cache.stats();
    let uptime = state.start_time.elapsed().as_secs();
    
    let stats = ServerStats {
        connected_clients: client_count,
        cached_templates: cache_stats.total_entries,
        watched_files: 0, // TODO: Get from file watcher
        uptime_seconds: uptime,
        updates_sent,
    };
    
    serde_json::json!({
        "connected_clients": stats.connected_clients,
        "cached_templates": stats.cached_templates,
        "watched_files": stats.watched_files,
        "uptime_seconds": stats.uptime_seconds,
        "updates_sent": stats.updates_sent,
        "cache_oldest": cache_stats.oldest_entry.map(|t| t.elapsed().as_secs()),
        "cache_newest": cache_stats.newest_entry.map(|t| t.elapsed().as_secs()),
    })
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_server_creation() {
        let addr = "127.0.0.1:0".parse().unwrap();
        let server = HotReloadServer::new(addr, vec![]);
        assert_eq!(server.addr, addr);
        assert!(server.watch_paths.is_empty());
    }
}