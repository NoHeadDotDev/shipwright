//! Hot reload infrastructure for Shipwright LiveView
//!
//! This crate provides development-time hot reloading for LiveView templates.
//! It watches for changes in Rust files containing view! macros, parses the
//! changed templates, and serves updates via WebSocket.

pub mod parser;
pub mod protocol;
pub mod server;
pub mod watcher;
pub mod template_cache;
pub mod runtime;
pub mod template_diff;
pub mod diff_integration;

// Enhanced runtime integration modules
pub mod dom_diff;
pub mod liveview_integration;
pub mod integration;
pub mod error_recovery;
pub mod logging;
pub mod state_serialization;

pub use protocol::{HotReloadMessage, TemplateUpdate, TemplateId};
pub use server::HotReloadServer;
pub use watcher::FileWatcher;
pub use template_cache::TemplateCache;