//! Runtime integration for hot reload system

use once_cell::sync::OnceCell;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::protocol::TemplateId;

/// Global template registry
static TEMPLATE_REGISTRY: OnceCell<Arc<RwLock<TemplateRegistry>>> = OnceCell::new();

/// Registry for runtime template registration
#[derive(Debug, Default)]
pub struct TemplateRegistry {
    /// Registered templates with their metadata
    templates: Vec<RegisteredTemplate>,
}

/// A template registered at runtime
#[derive(Debug, Clone)]
pub struct RegisteredTemplate {
    /// Template identifier
    pub id: TemplateId,
    /// Type name of the component
    pub component_type: String,
    /// Function name where template is defined
    pub function_name: String,
}

impl TemplateRegistry {
    /// Get the global template registry
    pub fn global() -> Arc<RwLock<TemplateRegistry>> {
        TEMPLATE_REGISTRY
            .get_or_init(|| Arc::new(RwLock::new(TemplateRegistry::default())))
            .clone()
    }

    /// Register a template
    pub fn register(&mut self, template: RegisteredTemplate) {
        // Check if template already exists
        if !self.templates.iter().any(|t| t.id == template.id) {
            self.templates.push(template);
        }
    }

    /// Get all registered templates
    pub fn get_all(&self) -> Vec<RegisteredTemplate> {
        self.templates.clone()
    }

    /// Find a template by ID
    pub fn find(&self, id: &TemplateId) -> Option<RegisteredTemplate> {
        self.templates.iter().find(|t| &t.id == id).cloned()
    }

    /// Clear all registered templates
    pub fn clear(&mut self) {
        self.templates.clear();
    }
}

/// Register a template at runtime
///
/// This macro should be called by the view! macro to register templates
#[macro_export]
macro_rules! register_template {
    ($file:expr, $line:expr, $column:expr, $component_type:expr, $function_name:expr) => {{
        use $crate::runtime::{RegisteredTemplate, TemplateRegistry};
        use $crate::protocol::TemplateId;
        use std::path::PathBuf;

        let template = RegisteredTemplate {
            id: TemplateId::new(PathBuf::from($file), $line, $column),
            component_type: $component_type.to_string(),
            function_name: $function_name.to_string(),
        };

        let registry = TemplateRegistry::global();
        let mut registry = registry.blocking_write();
        registry.register(template);
    }};
}

/// Hot reload client for runtime template updates
pub struct HotReloadClient {
    /// WebSocket URL for hot reload server
    server_url: String,
    /// Template update handler
    update_handler: Arc<dyn Fn(TemplateId, String) + Send + Sync>,
}

impl HotReloadClient {
    /// Create a new hot reload client
    pub fn new<F>(server_url: String, update_handler: F) -> Self
    where
        F: Fn(TemplateId, String) + Send + Sync + 'static,
    {
        Self {
            server_url,
            update_handler: Arc::new(update_handler),
        }
    }

    /// Connect to the hot reload server
    pub async fn connect(&self) -> Result<(), Box<dyn std::error::Error>> {
        use tokio_tungstenite::{connect_async, tungstenite::Message};
        use futures_util::StreamExt;
        
        let (ws_stream, _) = connect_async(&self.server_url).await?;
        let (_write, mut read) = ws_stream.split();

        // Handle incoming messages
        while let Some(msg) = read.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    if let Ok(msg) = crate::protocol::HotReloadMessage::from_json(&text) {
                        self.handle_message(msg).await;
                    }
                }
                Ok(Message::Close(_)) => break,
                Err(e) => {
                    eprintln!("WebSocket error: {}", e);
                    break;
                }
                _ => {}
            }
        }

        Ok(())
    }

    /// Handle a hot reload message
    async fn handle_message(&self, msg: crate::protocol::HotReloadMessage) {
        use crate::protocol::HotReloadMessage;
        
        match msg {
            HotReloadMessage::TemplateUpdated(update) => {
                (self.update_handler)(update.id, update.html);
            }
            HotReloadMessage::BatchUpdate { updates } => {
                for update in updates {
                    (self.update_handler)(update.id, update.html);
                }
            }
            _ => {}
        }
    }
}

/// Initialize hot reload in development mode
pub async fn init_hot_reload(server_url: String) -> Result<(), Box<dyn std::error::Error>> {
    if cfg!(debug_assertions) {
        let client = HotReloadClient::new(server_url, |id, html| {
            // This is where we would update the template in the runtime
            // For now, just log it
            eprintln!("Template updated: {:?} -> {}", id, html);
        });

        tokio::spawn(async move {
            if let Err(e) = client.connect().await {
                eprintln!("Failed to connect to hot reload server: {}", e);
            }
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_template_registry() {
        let registry = TemplateRegistry::global();
        
        let template = RegisteredTemplate {
            id: TemplateId::new(PathBuf::from("test.rs"), 10, 5),
            component_type: "TestComponent".to_string(),
            function_name: "render".to_string(),
        };
        
        {
            let mut reg = registry.write().await;
            reg.register(template.clone());
        }
        
        {
            let reg = registry.read().await;
            let found = reg.find(&template.id);
            assert!(found.is_some());
            assert_eq!(found.unwrap().component_type, "TestComponent");
        }
    }
}