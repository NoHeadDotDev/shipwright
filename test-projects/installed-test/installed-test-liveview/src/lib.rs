
//! LiveView components and pages for installed-test
//!
//! This crate contains all LiveView components, pages, and related functionality
//! for the installed-test application. LiveView enables real-time, interactive
//! web applications with server-side rendering and minimal JavaScript.
//!
//! ## Overview
//!
//! LiveView allows you to build rich, interactive web applications using only
//! server-side Rust code. Changes to the application state are automatically
//! reflected in the UI without requiring manual DOM manipulation or complex
//! client-side state management.
//!
//! ## Features
//!
//! - Real-time UI updates with WebSocket communication
//! - Server-side rendering with automatic diffing
//! - Component-based architecture
//! - Type-safe event handling
//! - Hot reload support for development
//!
//! ## Basic Usage
//!
//! ```rust
//! use shipwright_liveview::{LiveView, Html, html};
//! use shipwright_liveview_macros::LiveView;
//! use serde::{Serialize, Deserialize};
//!
//! #[derive(LiveView, Clone, Default)]
//! struct Counter {
//!     count: i32,
//! }
//!
//! #[derive(Serialize, Deserialize)]
//! enum CounterMessage {
//!     Increment,
//!     Decrement,
//! }
//!
//! impl LiveView for Counter {
//!     type Message = CounterMessage;
//!
//!     fn update(mut self, msg: Self::Message, _data: Option<EventData>) -> Updated<Self> {
//!         match msg {
//!             CounterMessage::Increment => self.count += 1,
//!             CounterMessage::Decrement => self.count -= 1,
//!         }
//!         Updated::new(self)
//!     }
//!
//!     fn render(&self) -> Html<Self::Message> {
//!         html! {
//!             <div class="counter">
//!                 <h1>"Counter: " { self.count }</h1>
//!                 <button axm-click={ CounterMessage::Increment }>"+"</button>
//!                 <button axm-click={ CounterMessage::Decrement }>"-"</button>
//!             </div>
//!         }
//!     }
//! }
//! ```

pub mod components;
pub mod pages;
pub mod router;

// Re-export commonly used types and macros
pub use shipwright_liveview::{
    LiveView, Html, Updated, EventData,
    live_view::LiveViewUpgrade,
};
pub use shipwright_liveview_macros::{html, LiveView as LiveViewDerive};

use installed_test_shared::Config;

/// Configuration for LiveView features
#[derive(Debug, Clone)]
pub struct LiveViewConfig {
    /// Whether hot reload is enabled
    pub hot_reload: bool,
    /// WebSocket endpoint for LiveView connections
    pub websocket_path: String,
    /// Static asset path for LiveView JavaScript
    pub assets_path: String,
}

impl Default for LiveViewConfig {
    fn default() -> Self {
        Self {
            hot_reload: cfg!(debug_assertions),
            websocket_path: "/live/websocket".to_string(),
            assets_path: "/assets".to_string(),
        }
    }
}

impl From<&Config> for LiveViewConfig {
    fn from(config: &Config) -> Self {
        Self {
            hot_reload: config.is_development(),
            ..Default::default()
        }
    }
}

/// Initialize LiveView with the given configuration
pub fn init_liveview(config: LiveViewConfig) -> router::LiveViewRouter {
    router::LiveViewRouter::new(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_liveview_config_default() {
        let config = LiveViewConfig::default();
        assert_eq!(config.websocket_path, "/live/websocket");
        assert_eq!(config.assets_path, "/assets");
    }
}