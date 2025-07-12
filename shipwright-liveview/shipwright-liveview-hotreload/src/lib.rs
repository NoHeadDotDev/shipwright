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

pub use protocol::{HotReloadMessage, TemplateUpdate, TemplateId};
pub use server::HotReloadServer;
pub use watcher::FileWatcher;
pub use template_cache::TemplateCache;