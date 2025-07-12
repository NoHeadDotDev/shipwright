//! Integration layer between hot reload and LiveView components
//! 
//! This module provides the bridge between the hot reload system and shipwright-liveview,
//! enabling seamless component updates while preserving component state.

use std::collections::HashMap;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use tokio::sync::{RwLock, mpsc};
use tracing::{debug, error, info, warn};

use crate::protocol::{TemplateId, TemplateUpdate};
use crate::runtime::{HotReloadError, ComponentInstance, ComponentState};
use crate::dom_diff::{HotReloadDomDiffer, OptimizedPatch, ComponentBoundary};

/// Manager for LiveView component hot reload integration
#[derive(Debug)]
pub struct LiveViewHotReloadManager {
    /// Active LiveView instances
    active_views: Arc<RwLock<HashMap<String, ActiveLiveView>>>,
    /// DOM differ for generating patches
    dom_differ: Arc<RwLock<HotReloadDomDiffer>>,
    /// Event sender for notifying clients
    event_sender: Option<mpsc::Sender<HotReloadEvent>>,
    /// Configuration options
    config: LiveViewHotReloadConfig,
}

/// Configuration for LiveView hot reload
#[derive(Debug, Clone)]
pub struct LiveViewHotReloadConfig {
    /// Whether to preserve component state during updates
    pub preserve_state: bool,
    /// Whether to preserve form input values
    pub preserve_form_state: bool,
    /// Whether to preserve scroll positions
    pub preserve_scroll: bool,
    /// Whether to preserve focus
    pub preserve_focus: bool,
    /// Maximum time to wait for state preservation (ms)
    pub state_preservation_timeout: u64,
    /// Whether to enable debug logging
    pub debug_mode: bool,
}

impl Default for LiveViewHotReloadConfig {
    fn default() -> Self {
        Self {
            preserve_state: true,
            preserve_form_state: true,
            preserve_scroll: true,
            preserve_focus: true,
            state_preservation_timeout: 1000,
            debug_mode: cfg!(debug_assertions),
        }
    }
}

/// An active LiveView instance tracked for hot reload
#[derive(Debug, Clone)]
pub struct ActiveLiveView {
    /// Unique instance identifier
    pub instance_id: String,
    /// Template ID this view is based on
    pub template_id: TemplateId,
    /// Current HTML content
    pub current_html: String,
    /// Preserved component state
    pub preserved_state: Option<LiveViewState>,
    /// WebSocket connection ID for sending updates
    pub connection_id: Option<String>,
    /// Component type information
    pub component_type: String,
    /// Last update timestamp
    pub last_updated: std::time::SystemTime,
    /// Whether this view supports hot reload
    pub hot_reload_enabled: bool,
}

/// Preserved LiveView state during hot reload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveViewState {
    /// Serialized component state data
    pub component_data: serde_json::Value,
    /// Form field values
    pub form_values: HashMap<String, FormFieldValue>,
    /// Scroll positions
    pub scroll_positions: HashMap<String, ScrollPosition>,
    /// Currently focused element
    pub focused_element: Option<String>,
    /// Text selections
    pub text_selections: HashMap<String, TextSelection>,
    /// Custom user state
    pub user_state: Option<serde_json::Value>,
}

/// Form field value types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FormFieldValue {
    Text(String),
    Boolean(bool),
    Number(f64),
    Array(Vec<String>),
}

/// Scroll position data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrollPosition {
    pub x: f64,
    pub y: f64,
}

/// Text selection range
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextSelection {
    pub start: u32,
    pub end: u32,
    pub direction: String,
}

/// Events sent to clients for hot reload updates
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum HotReloadEvent {
    /// A LiveView component needs to be updated
    ComponentUpdate {
        instance_id: String,
        patch: OptimizedPatch,
        preserve_state: bool,
    },
    /// State preservation request
    StatePreservationRequest {
        instance_id: String,
        timeout_ms: u64,
    },
    /// State preservation response
    StatePreservationResponse {
        instance_id: String,
        state: LiveViewState,
        success: bool,
    },
    /// Full page refresh required
    FullRefresh {
        reason: String,
    },
    /// Hot reload error occurred
    Error {
        message: String,
        instance_id: Option<String>,
    },
}

/// Result of a hot reload update operation
#[derive(Debug)]
pub enum HotReloadResult {
    /// Update was applied successfully
    Updated {
        instance_id: String,
        patch_size: usize,
        state_preserved: bool,
    },
    /// Full refresh is required
    FullRefreshRequired {
        reason: String,
    },
    /// Update failed
    Failed {
        error: HotReloadError,
        fallback_to_refresh: bool,
    },
}

impl LiveViewHotReloadManager {
    /// Create a new hot reload manager
    pub fn new(config: LiveViewHotReloadConfig) -> Self {
        Self {
            active_views: Arc::new(RwLock::new(HashMap::new())),
            dom_differ: Arc::new(RwLock::new(HotReloadDomDiffer::new())),
            event_sender: None,
            config,
        }
    }
    
    /// Set the event sender for client notifications
    pub fn set_event_sender(&mut self, sender: mpsc::Sender<HotReloadEvent>) {
        self.event_sender = Some(sender);
    }
    
    /// Register a new LiveView instance for hot reload tracking
    pub async fn register_liveview_instance(&self, view: ActiveLiveView) -> Result<(), HotReloadError> {
        debug!("Registering LiveView instance: {}", view.instance_id);
        
        // Register component boundary for targeted updates
        let boundary = ComponentBoundary {
            component_id: view.instance_id.clone(),
            root_selector: format!("[data-live-view-id=\"{}\"]", view.instance_id),
            isolated: true,
            children: vec![],
            state_data: view.preserved_state.as_ref().map(|s| s.component_data.clone()),
        };
        
        {
            let mut differ = self.dom_differ.write().await;
            differ.register_component_boundary(boundary);
        }
        
        // Store the view
        let mut views = self.active_views.write().await;
        views.insert(view.instance_id.clone(), view);
        
        Ok(())
    }
    
    /// Unregister a LiveView instance
    pub async fn unregister_liveview_instance(&self, instance_id: &str) {
        debug!("Unregistering LiveView instance: {}", instance_id);
        
        let mut views = self.active_views.write().await;
        views.remove(instance_id);
    }
    
    /// Update a LiveView component with a template change
    pub async fn update_liveview_component(
        &self,
        template_update: &TemplateUpdate,
    ) -> Result<Vec<HotReloadResult>, HotReloadError> {
        info!("Updating LiveView components for template: {:?}", template_update.id);
        
        let mut results = Vec::new();
        
        // Find all active views using this template
        let views = self.active_views.read().await;
        let affected_views: Vec<_> = views
            .values()
            .filter(|view| view.template_id == template_update.id && view.hot_reload_enabled)
            .cloned()
            .collect();
        drop(views);
        
        if affected_views.is_empty() {
            debug!("No active views found for template: {:?}", template_update.id);
            return Ok(results);
        }
        
        for view in affected_views {
            match self.update_single_view(&view, template_update).await {
                Ok(result) => results.push(result),
                Err(e) => {
                    error!("Failed to update view {}: {}", view.instance_id, e);
                    results.push(HotReloadResult::Failed {
                        error: e,
                        fallback_to_refresh: true,
                    });
                }
            }
        }
        
        Ok(results)
    }
    
    /// Update a single LiveView instance
    async fn update_single_view(
        &self,
        view: &ActiveLiveView,
        template_update: &TemplateUpdate,
    ) -> Result<HotReloadResult, HotReloadError> {
        debug!("Updating single view: {}", view.instance_id);
        
        // Preserve state if enabled
        let preserved_state = if self.config.preserve_state {
            self.preserve_view_state(&view.instance_id).await?
        } else {
            None
        };
        
        // Generate DOM patch
        let patch = {
            let mut differ = self.dom_differ.write().await;
            differ.generate_hot_reload_patch(
                &view.current_html,
                &template_update.html,
                Some(&view.instance_id),
            )?
        };
        
        // Check if we need a full refresh
        if patch.requires_full_refresh {
            return Ok(HotReloadResult::FullRefreshRequired {
                reason: "Complex changes detected".to_string(),
            });
        }
        
        // Send update event to client
        if let Some(ref sender) = self.event_sender {
            let event = HotReloadEvent::ComponentUpdate {
                instance_id: view.instance_id.clone(),
                patch: patch.clone(),
                preserve_state: preserved_state.is_some(),
            };
            
            if let Err(e) = sender.try_send(event) {
                warn!("Failed to send hot reload event: {}", e);
            }
        }
        
        // Update stored view
        {
            let mut views = self.active_views.write().await;
            if let Some(stored_view) = views.get_mut(&view.instance_id) {
                stored_view.current_html = template_update.html.clone();
                stored_view.last_updated = std::time::SystemTime::now();
                if let Some(state) = preserved_state {
                    stored_view.preserved_state = Some(state);
                }
            }
        }
        
        Ok(HotReloadResult::Updated {
            instance_id: view.instance_id.clone(),
            patch_size: patch.patch.operations.len(),
            state_preserved: preserved_state.is_some(),
        })
    }
    
    /// Preserve the current state of a LiveView
    async fn preserve_view_state(&self, instance_id: &str) -> Result<Option<LiveViewState>, HotReloadError> {
        debug!("Preserving state for view: {}", instance_id);
        
        // Request state from client
        if let Some(ref sender) = self.event_sender {
            let event = HotReloadEvent::StatePreservationRequest {
                instance_id: instance_id.to_string(),
                timeout_ms: self.config.state_preservation_timeout,
            };
            
            if let Err(e) = sender.try_send(event) {
                warn!("Failed to send state preservation request: {}", e);
                return Ok(None);
            }
        }
        
        // For now, return empty state - in a real implementation,
        // we would wait for the client response
        Ok(Some(LiveViewState {
            component_data: serde_json::Value::Null,
            form_values: HashMap::new(),
            scroll_positions: HashMap::new(),
            focused_element: None,
            text_selections: HashMap::new(),
            user_state: None,
        }))
    }
    
    /// Handle state preservation response from client
    pub async fn handle_state_preservation_response(
        &self,
        instance_id: &str,
        state: LiveViewState,
    ) -> Result<(), HotReloadError> {
        debug!("Received state preservation response for: {}", instance_id);
        
        let mut views = self.active_views.write().await;
        if let Some(view) = views.get_mut(instance_id) {
            view.preserved_state = Some(state);
            Ok(())
        } else {
            Err(HotReloadError::ComponentInstanceNotFound {
                instance_id: instance_id.to_string(),
            })
        }
    }
    
    /// Get all active LiveView instances
    pub async fn get_active_views(&self) -> HashMap<String, ActiveLiveView> {
        self.active_views.read().await.clone()
    }
    
    /// Get a specific LiveView instance
    pub async fn get_view(&self, instance_id: &str) -> Option<ActiveLiveView> {
        self.active_views.read().await.get(instance_id).cloned()
    }
    
    /// Clear all state and reset
    pub async fn reset(&self) {
        info!("Resetting LiveView hot reload manager");
        
        let mut views = self.active_views.write().await;
        views.clear();
        
        let mut differ = self.dom_differ.write().await;
        differ.clear_cache();
    }
    
    /// Get statistics about the hot reload system
    pub async fn get_statistics(&self) -> HotReloadStatistics {
        let views = self.active_views.read().await;
        
        HotReloadStatistics {
            active_views: views.len(),
            hot_reload_enabled_views: views.values().filter(|v| v.hot_reload_enabled).count(),
            total_updates: 0, // Would track this in a real implementation
            successful_updates: 0,
            failed_updates: 0,
            average_patch_size: 0.0,
        }
    }
}

/// Statistics about hot reload operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotReloadStatistics {
    pub active_views: usize,
    pub hot_reload_enabled_views: usize,
    pub total_updates: usize,
    pub successful_updates: usize,
    pub failed_updates: usize,
    pub average_patch_size: f64,
}

/// Helper functions for LiveView integration
impl ActiveLiveView {
    /// Create a new active LiveView instance
    pub fn new(
        instance_id: String,
        template_id: TemplateId,
        current_html: String,
        component_type: String,
    ) -> Self {
        Self {
            instance_id,
            template_id,
            current_html,
            preserved_state: None,
            connection_id: None,
            component_type,
            last_updated: std::time::SystemTime::now(),
            hot_reload_enabled: true,
        }
    }
    
    /// Enable or disable hot reload for this view
    pub fn set_hot_reload_enabled(&mut self, enabled: bool) {
        self.hot_reload_enabled = enabled;
    }
    
    /// Set the WebSocket connection ID
    pub fn set_connection_id(&mut self, connection_id: String) {
        self.connection_id = Some(connection_id);
    }
    
    /// Check if this view has been recently updated
    pub fn is_recently_updated(&self, threshold_secs: u64) -> bool {
        self.last_updated
            .elapsed()
            .map(|d| d.as_secs() < threshold_secs)
            .unwrap_or(false)
    }
}

/// Integration with the shipwright-liveview system
pub mod liveview_bridge {
    use super::*;
    use std::sync::OnceLock;
    
    /// Global hot reload manager instance
    static GLOBAL_MANAGER: OnceLock<Arc<LiveViewHotReloadManager>> = OnceLock::new();
    
    /// Initialize the global hot reload manager
    pub fn init_hot_reload_manager(config: LiveViewHotReloadConfig) -> Arc<LiveViewHotReloadManager> {
        let manager = Arc::new(LiveViewHotReloadManager::new(config));
        GLOBAL_MANAGER.set(manager.clone()).unwrap_or_else(|_| {
            panic!("Hot reload manager already initialized");
        });
        manager
    }
    
    /// Get the global hot reload manager
    pub fn get_hot_reload_manager() -> Option<Arc<LiveViewHotReloadManager>> {
        GLOBAL_MANAGER.get().cloned()
    }
    
    /// Register a LiveView instance with the global manager
    pub async fn register_view(view: ActiveLiveView) -> Result<(), HotReloadError> {
        if let Some(manager) = get_hot_reload_manager() {
            manager.register_liveview_instance(view).await
        } else {
            Err(HotReloadError::TemplateUpdateFailed {
                reason: "Hot reload manager not initialized".to_string(),
            })
        }
    }
    
    /// Unregister a LiveView instance
    pub async fn unregister_view(instance_id: &str) {
        if let Some(manager) = get_hot_reload_manager() {
            manager.unregister_liveview_instance(instance_id).await;
        }
    }
    
    /// Update views for a template change
    pub async fn update_views_for_template(
        template_update: &TemplateUpdate,
    ) -> Result<Vec<HotReloadResult>, HotReloadError> {
        if let Some(manager) = get_hot_reload_manager() {
            manager.update_liveview_component(template_update).await
        } else {
            Err(HotReloadError::TemplateUpdateFailed {
                reason: "Hot reload manager not initialized".to_string(),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    
    #[tokio::test]
    async fn test_liveview_manager_creation() {
        let config = LiveViewHotReloadConfig::default();
        let manager = LiveViewHotReloadManager::new(config);
        
        let stats = manager.get_statistics().await;
        assert_eq!(stats.active_views, 0);
    }
    
    #[tokio::test]
    async fn test_view_registration() {
        let config = LiveViewHotReloadConfig::default();
        let manager = LiveViewHotReloadManager::new(config);
        
        let template_id = TemplateId::new(PathBuf::from("test.rs"), 10, 5);
        let view = ActiveLiveView::new(
            "test-view".to_string(),
            template_id,
            "<div>Hello</div>".to_string(),
            "TestComponent".to_string(),
        );
        
        manager.register_liveview_instance(view).await.unwrap();
        
        let stats = manager.get_statistics().await;
        assert_eq!(stats.active_views, 1);
        assert_eq!(stats.hot_reload_enabled_views, 1);
    }
    
    #[tokio::test]
    async fn test_view_unregistration() {
        let config = LiveViewHotReloadConfig::default();
        let manager = LiveViewHotReloadManager::new(config);
        
        let template_id = TemplateId::new(PathBuf::from("test.rs"), 10, 5);
        let view = ActiveLiveView::new(
            "test-view".to_string(),
            template_id,
            "<div>Hello</div>".to_string(),
            "TestComponent".to_string(),
        );
        
        manager.register_liveview_instance(view).await.unwrap();
        manager.unregister_liveview_instance("test-view").await;
        
        let stats = manager.get_statistics().await;
        assert_eq!(stats.active_views, 0);
    }
}