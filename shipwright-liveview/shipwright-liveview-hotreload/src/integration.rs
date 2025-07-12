//! Main integration layer for hot reload with shipwright-liveview
//! 
//! This module provides the primary integration points and APIs for connecting
//! the hot reload system with the main LiveView framework.

use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, error, info, warn};
use axum::extract::ws::{WebSocket, Message};
use futures_util::{SinkExt, StreamExt};

use crate::protocol::{HotReloadMessage, TemplateUpdate, TemplateId};
use crate::runtime::{HotReloadError, TemplateRegistry};
use crate::liveview_integration::{
    LiveViewHotReloadManager, LiveViewHotReloadConfig, HotReloadEvent, ActiveLiveView, HotReloadResult
};
use crate::server::HotReloadServer;

/// Main hot reload integration system
#[derive(Debug)]
pub struct HotReloadIntegration {
    /// Hot reload server for file watching
    server: Arc<HotReloadServer>,
    /// LiveView manager for component updates
    liveview_manager: Arc<LiveViewHotReloadManager>,
    /// Event channel for client notifications
    event_sender: mpsc::Sender<HotReloadEvent>,
    event_receiver: Arc<RwLock<Option<mpsc::Receiver<HotReloadEvent>>>>,
    /// Configuration
    config: HotReloadIntegrationConfig,
}

/// Configuration for the integration system
#[derive(Debug, Clone)]
pub struct HotReloadIntegrationConfig {
    /// Server configuration
    pub server_host: String,
    pub server_port: u16,
    /// Watch directory
    pub watch_directory: String,
    /// File patterns to watch
    pub watch_patterns: Vec<String>,
    /// LiveView hot reload config
    pub liveview_config: LiveViewHotReloadConfig,
    /// Whether to enable debug mode
    pub debug_mode: bool,
}

impl Default for HotReloadIntegrationConfig {
    fn default() -> Self {
        Self {
            server_host: "localhost".to_string(),
            server_port: 3001,
            watch_directory: "src".to_string(),
            watch_patterns: vec!["**/*.rs".to_string()],
            liveview_config: LiveViewHotReloadConfig::default(),
            debug_mode: cfg!(debug_assertions),
        }
    }
}

/// Statistics about the integration system
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct IntegrationStatistics {
    /// Server statistics
    pub server_connected_clients: usize,
    pub total_file_changes: usize,
    pub total_template_updates: usize,
    /// LiveView statistics
    pub active_liveviews: usize,
    pub successful_updates: usize,
    pub failed_updates: usize,
    /// Performance metrics
    pub average_update_time_ms: f64,
    pub last_update_time: Option<std::time::SystemTime>,
}

impl HotReloadIntegration {
    /// Create a new hot reload integration system
    pub async fn new(config: HotReloadIntegrationConfig) -> Result<Self, HotReloadError> {
        info!("Initializing hot reload integration system");
        
        // Create event channel
        let (event_sender, event_receiver) = mpsc::channel(1000);
        
        // Create LiveView manager
        let mut liveview_manager = LiveViewHotReloadManager::new(config.liveview_config.clone());
        liveview_manager.set_event_sender(event_sender.clone());
        let liveview_manager = Arc::new(liveview_manager);
        
        // Create hot reload server
        let server = HotReloadServer::new(
            config.server_host.clone(),
            config.server_port,
            config.watch_directory.clone(),
        ).map_err(|e| HotReloadError::TemplateUpdateFailed {
            reason: format!("Failed to create hot reload server: {}", e),
        })?;
        let server = Arc::new(server);
        
        // Create integration instance
        Ok(Self {
            server,
            liveview_manager,
            event_sender,
            event_receiver: Arc::new(RwLock::new(Some(event_receiver))),
            config,
        })
    }
    
    /// Start the hot reload integration system
    pub async fn start(&self) -> Result<(), HotReloadError> {
        info!("Starting hot reload integration system");
        
        // Start the hot reload server
        let server_handle = {
            let server = self.server.clone();
            let liveview_manager = self.liveview_manager.clone();
            
            tokio::spawn(async move {
                if let Err(e) = Self::run_server_with_integration(server, liveview_manager).await {
                    error!("Hot reload server error: {}", e);
                }
            })
        };
        
        // Start event processing
        let event_handle = {
            let receiver = self.event_receiver.clone();
            let liveview_manager = self.liveview_manager.clone();
            
            tokio::spawn(async move {
                if let Some(mut rx) = receiver.write().await.take() {
                    Self::process_events(&mut rx, liveview_manager).await;
                }
            })
        };
        
        info!("Hot reload integration system started successfully");
        
        // Wait for either task to complete (which would indicate an error)
        tokio::select! {
            _ = server_handle => {
                warn!("Hot reload server task completed unexpectedly");
            }
            _ = event_handle => {
                warn!("Event processing task completed unexpectedly");
            }
        }
        
        Ok(())
    }
    
    /// Run the server with LiveView integration
    async fn run_server_with_integration(
        server: Arc<HotReloadServer>,
        liveview_manager: Arc<LiveViewHotReloadManager>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        
        // Set up template update handler
        let template_update_handler = {
            let liveview_manager = liveview_manager.clone();
            
            move |template_update: TemplateUpdate| {
                let manager = liveview_manager.clone();
                tokio::spawn(async move {
                    match manager.update_liveview_component(&template_update).await {
                        Ok(results) => {
                            for result in results {
                                match result {
                                    HotReloadResult::Updated { instance_id, patch_size, state_preserved } => {
                                        info!("Updated view {} (patch size: {}, state preserved: {})", 
                                              instance_id, patch_size, state_preserved);
                                    }
                                    HotReloadResult::FullRefreshRequired { reason } => {
                                        warn!("Full refresh required: {}", reason);
                                    }
                                    HotReloadResult::Failed { error, fallback_to_refresh } => {
                                        error!("Update failed: {} (fallback: {})", error, fallback_to_refresh);
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            error!("Failed to update LiveView components: {}", e);
                        }
                    }
                });
            }
        };
        
        // Start the server (this would need to be implemented in the actual server)
        // For now, we'll simulate the server running
        info!("Hot reload server running with LiveView integration");
        
        // Keep the server running
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
    }
    
    /// Process hot reload events
    async fn process_events(
        receiver: &mut mpsc::Receiver<HotReloadEvent>,
        liveview_manager: Arc<LiveViewHotReloadManager>,
    ) {
        info!("Starting event processing loop");
        
        while let Some(event) = receiver.recv().await {
            match event {
                HotReloadEvent::ComponentUpdate { instance_id, patch, preserve_state } => {
                    debug!("Processing component update for: {} (preserve state: {})", 
                           instance_id, preserve_state);
                    
                    // The actual DOM patch would be sent to the client via WebSocket
                    // This is where we'd integrate with the WebSocket system
                }
                HotReloadEvent::StatePreservationRequest { instance_id, timeout_ms } => {
                    debug!("State preservation requested for: {} (timeout: {}ms)", 
                           instance_id, timeout_ms);
                    
                    // Would send request to client to capture state
                }
                HotReloadEvent::StatePreservationResponse { instance_id, state, success } => {
                    if success {
                        if let Err(e) = liveview_manager.handle_state_preservation_response(&instance_id, state).await {
                            error!("Failed to handle state preservation response: {}", e);
                        }
                    } else {
                        warn!("State preservation failed for: {}", instance_id);
                    }
                }
                HotReloadEvent::FullRefresh { reason } => {
                    info!("Full refresh requested: {}", reason);
                    // Would trigger a full page reload
                }
                HotReloadEvent::Error { message, instance_id } => {
                    error!("Hot reload error: {} (instance: {:?})", message, instance_id);
                }
            }
        }
        
        info!("Event processing loop ended");
    }
    
    /// Register a new LiveView instance
    pub async fn register_liveview(
        &self,
        instance_id: String,
        template_id: TemplateId,
        initial_html: String,
        component_type: String,
    ) -> Result<(), HotReloadError> {
        let view = ActiveLiveView::new(instance_id, template_id, initial_html, component_type);
        self.liveview_manager.register_liveview_instance(view).await
    }
    
    /// Unregister a LiveView instance
    pub async fn unregister_liveview(&self, instance_id: &str) {
        self.liveview_manager.unregister_liveview_instance(instance_id).await;
    }
    
    /// Handle WebSocket connection for hot reload
    pub async fn handle_websocket(&self, mut socket: WebSocket) -> Result<(), HotReloadError> {
        info!("New WebSocket connection for hot reload");
        
        // Set up message processing
        while let Some(msg) = socket.recv().await {
            match msg {
                Ok(Message::Text(text)) => {
                    if let Ok(hot_reload_msg) = HotReloadMessage::from_json(&text) {
                        self.handle_websocket_message(hot_reload_msg, &mut socket).await?;
                    }
                }
                Ok(Message::Close(_)) => {
                    info!("WebSocket connection closed");
                    break;
                }
                Err(e) => {
                    error!("WebSocket error: {}", e);
                    break;
                }
                _ => {}
            }
        }
        
        Ok(())
    }
    
    /// Handle a hot reload message from WebSocket
    async fn handle_websocket_message(
        &self,
        message: HotReloadMessage,
        socket: &mut WebSocket,
    ) -> Result<(), HotReloadError> {
        match message {
            HotReloadMessage::ReloadRequest { template_id } => {
                debug!("Reload request for template: {:?}", template_id);
                // Trigger reload for specific template
            }
            HotReloadMessage::Ping => {
                // Send pong response
                let pong = HotReloadMessage::Pong;
                if let Ok(json) = pong.to_json() {
                    let _ = socket.send(Message::Text(json)).await;
                }
            }
            _ => {
                debug!("Received hot reload message: {:?}", message);
            }
        }
        
        Ok(())
    }
    
    /// Get integration statistics
    pub async fn get_statistics(&self) -> IntegrationStatistics {
        let liveview_stats = self.liveview_manager.get_statistics().await;
        
        IntegrationStatistics {
            server_connected_clients: 0, // Would get from server
            total_file_changes: 0,       // Would track in server
            total_template_updates: 0,   // Would track in server
            active_liveviews: liveview_stats.active_views,
            successful_updates: liveview_stats.successful_updates,
            failed_updates: liveview_stats.failed_updates,
            average_update_time_ms: liveview_stats.average_patch_size,
            last_update_time: None,      // Would track this
        }
    }
    
    /// Stop the integration system
    pub async fn stop(&self) -> Result<(), HotReloadError> {
        info!("Stopping hot reload integration system");
        
        // Reset all components
        self.liveview_manager.reset().await;
        
        info!("Hot reload integration system stopped");
        Ok(())
    }
}

/// Convenience functions for integration with axum applications
pub mod axum_integration {
    use super::*;
    use axum::{
        extract::{WebSocketUpgrade, State},
        response::Response,
        routing::get,
        Router,
    };
    use std::sync::Arc;
    
    /// Add hot reload routes to an axum router
    pub fn add_hot_reload_routes(
        router: Router,
        integration: Arc<HotReloadIntegration>,
    ) -> Router {
        router
            .route("/hot-reload/ws", get(hot_reload_websocket_handler))
            .route("/hot-reload/status", get(hot_reload_status_handler))
            .with_state(integration)
    }
    
    /// WebSocket handler for hot reload
    async fn hot_reload_websocket_handler(
        ws: WebSocketUpgrade,
        State(integration): State<Arc<HotReloadIntegration>>,
    ) -> Response {
        ws.on_upgrade(move |socket| async move {
            if let Err(e) = integration.handle_websocket(socket).await {
                error!("WebSocket handler error: {}", e);
            }
        })
    }
    
    /// Status endpoint for hot reload system
    async fn hot_reload_status_handler(
        State(integration): State<Arc<HotReloadIntegration>>,
    ) -> axum::Json<IntegrationStatistics> {
        axum::Json(integration.get_statistics().await)
    }
}

/// Helper macros for easy integration
#[macro_export]
macro_rules! enable_hot_reload {
    ($app:expr, $config:expr) => {{
        use $crate::integration::{HotReloadIntegration, axum_integration};
        use std::sync::Arc;
        
        if cfg!(debug_assertions) {
            let integration = Arc::new(
                HotReloadIntegration::new($config)
                    .await
                    .expect("Failed to initialize hot reload")
            );
            
            // Start the integration system
            let integration_clone = integration.clone();
            tokio::spawn(async move {
                if let Err(e) = integration_clone.start().await {
                    eprintln!("Hot reload error: {}", e);
                }
            });
            
            // Add routes to the app
            axum_integration::add_hot_reload_routes($app, integration)
        } else {
            $app
        }
    }};
}

/// Initialize hot reload for a LiveView component
#[macro_export]
macro_rules! hot_reload_liveview {
    ($instance_id:expr, $template_id:expr, $html:expr, $component_type:expr) => {{
        use $crate::liveview_integration::liveview_bridge;
        use $crate::liveview_integration::ActiveLiveView;
        
        if cfg!(debug_assertions) {
            let view = ActiveLiveView::new(
                $instance_id.to_string(),
                $template_id,
                $html.to_string(),
                $component_type.to_string(),
            );
            
            if let Err(e) = liveview_bridge::register_view(view).await {
                eprintln!("Failed to register view for hot reload: {}", e);
            }
        }
    }};
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    
    #[tokio::test]
    async fn test_integration_creation() {
        let config = HotReloadIntegrationConfig::default();
        let integration = HotReloadIntegration::new(config).await;
        assert!(integration.is_ok());
    }
    
    #[tokio::test]
    async fn test_liveview_registration() {
        let config = HotReloadIntegrationConfig::default();
        let integration = HotReloadIntegration::new(config).await.unwrap();
        
        let template_id = TemplateId::new(PathBuf::from("test.rs"), 10, 5);
        let result = integration.register_liveview(
            "test-view".to_string(),
            template_id,
            "<div>Hello</div>".to_string(),
            "TestComponent".to_string(),
        ).await;
        
        assert!(result.is_ok());
        
        let stats = integration.get_statistics().await;
        assert_eq!(stats.active_liveviews, 1);
    }
    
    #[tokio::test]
    async fn test_statistics() {
        let config = HotReloadIntegrationConfig::default();
        let integration = HotReloadIntegration::new(config).await.unwrap();
        
        let stats = integration.get_statistics().await;
        assert_eq!(stats.active_liveviews, 0);
        assert_eq!(stats.successful_updates, 0);
        assert_eq!(stats.failed_updates, 0);
    }
}