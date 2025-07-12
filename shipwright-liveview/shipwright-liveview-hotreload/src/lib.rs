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
pub mod enhanced_watcher;

#[cfg(test)]
mod template_diff_tests;

// Enhanced runtime integration modules
pub mod dom_diff;
pub mod liveview_integration;
pub mod integration;
pub mod error_recovery;
pub mod logging;
pub mod state_serialization;

// Performance and caching modules
pub mod build_cache;
pub mod performance_monitor;
pub mod enhanced_cache;

// Benchmarking and testing utilities
#[cfg(feature = "benchmarks")]
pub mod benchmarks;

pub use protocol::{HotReloadMessage, TemplateUpdate, TemplateId};
pub use server::HotReloadServer;
pub use watcher::FileWatcher;
pub use template_cache::TemplateCache;
// Temporarily commented out until modules are fully implemented
// pub use template_diff::{TemplateDiffer, DiffResult, TemplateNode, CompatibilityChecker};
// pub use diff_integration::{HotReloadDecisionMaker, DiffAwareTemplateCache, HotReloadAnalysis, EnhancedHotReloadMessage};
// pub use enhanced_watcher::{EnhancedFileWatcher, WatcherStats};
// pub use build_cache::{BuildCache, RebuildDecision, RebuildSet};
// pub use performance_monitor::{PerformanceMonitor, PerformanceReport};
// pub use enhanced_cache::{EnhancedCache, ComprehensiveStats};