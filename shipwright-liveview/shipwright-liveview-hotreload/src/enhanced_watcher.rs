//! Enhanced file watcher with AST-level diffing support
//!
//! This module provides an enhanced file watcher that integrates with the template
//! diffing engine to make intelligent hot reload decisions.

use anyhow::{Context, Result};
use notify::RecursiveMode;
use notify_debouncer_mini::{new_debouncer, DebounceEventResult};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};
use std::collections::HashMap;
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, error, info, warn};
use walkdir::WalkDir;

use crate::{
    parser::TemplateParser,
    protocol::{TemplateUpdate, HotReloadMessage},
    diff_integration::{DiffAwareTemplateCache, analysis_to_messages, EnhancedHotReloadMessage},
    watcher::{FileChange, ChangeKind},
};

/// Enhanced file watcher with diff-aware hot reload
pub struct EnhancedFileWatcher {
    /// Paths to watch
    watch_paths: Vec<PathBuf>,
    /// File extensions to watch
    extensions: Vec<String>,
    /// Diff-aware template cache
    cache: Arc<RwLock<DiffAwareTemplateCache>>,
    /// Channel for sending hot reload messages
    tx: mpsc::Sender<Vec<EnhancedHotReloadMessage>>,
    /// Last processing time per file for additional debouncing
    last_processed: Arc<tokio::sync::Mutex<HashMap<PathBuf, Instant>>>,
    /// Statistics
    stats: Arc<RwLock<WatcherStats>>,
}

/// Statistics for the enhanced watcher
#[derive(Debug, Default)]
pub struct WatcherStats {
    /// Total files processed
    pub files_processed: usize,
    /// Hot reloadable changes
    pub hot_reloadable_changes: usize,
    /// Changes requiring rebuild
    pub rebuild_required_changes: usize,
    /// Delta updates sent
    pub delta_updates_sent: usize,
}

impl EnhancedFileWatcher {
    /// Create a new enhanced file watcher
    pub fn new(
        watch_paths: Vec<PathBuf>,
        extensions: Vec<String>,
        cache_size: usize,
    ) -> (Self, mpsc::Receiver<Vec<EnhancedHotReloadMessage>>) {
        let (tx, rx) = mpsc::channel(100);
        
        let watcher = Self {
            watch_paths,
            extensions,
            cache: Arc::new(RwLock::new(DiffAwareTemplateCache::new(cache_size))),
            tx,
            last_processed: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
            stats: Arc::new(RwLock::new(WatcherStats::default())),
        };
        
        (watcher, rx)
    }

    /// Start watching for file changes
    pub async fn watch(self) -> Result<()> {
        info!("Starting enhanced file watcher with AST-level diffing");
        
        // Perform initial scan
        self.initial_scan().await?;
        
        // Set up file watcher
        let (tx, rx) = std::sync::mpsc::channel();
        
        let mut debouncer = new_debouncer(Duration::from_millis(300), tx)?;
        
        // Add watch paths
        for path in &self.watch_paths {
            debouncer.watcher()
                .watch(path, RecursiveMode::Recursive)
                .context("Failed to watch directory")?;
            info!("Watching directory: {}", path.display());
        }
        
        // Spawn task to handle file change events
        let watcher = Arc::new(self);
        tokio::spawn(async move {
            watcher.handle_events(rx).await;
        });
        
        // Keep the watcher alive
        tokio::signal::ctrl_c().await?;
        info!("Shutting down enhanced file watcher");
        
        Ok(())
    }

    /// Perform initial scan of watched directories
    async fn initial_scan(&self) -> Result<()> {
        info!("Performing initial template scan...");
        let start_time = Instant::now();
        let mut total_templates = 0;
        
        for watch_path in &self.watch_paths {
            for entry in WalkDir::new(watch_path)
                .follow_links(true)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                let path = entry.path();
                if self.should_process_file(path) {
                    match self.process_file(path).await {
                        Ok(updates) if !updates.is_empty() => {
                            total_templates += updates.len();
                            
                            // Analyze and cache templates without broadcasting
                            let mut cache = self.cache.write().await;
                            match cache.insert_with_analysis(updates) {
                                Ok(analysis) => {
                                    debug!(
                                        "Initial scan: {} hot-reloadable, {} require rebuild",
                                        analysis.hot_reloadable.len(),
                                        analysis.require_rebuild.len()
                                    );
                                }
                                Err(e) => {
                                    warn!("Failed to analyze templates during initial scan: {}", e);
                                }
                            }
                        }
                        Ok(_) => {}
                        Err(e) => {
                            warn!("Failed to process {} during initial scan: {}", path.display(), e);
                        }
                    }
                }
            }
        }
        
        let elapsed = start_time.elapsed();
        info!(
            "Initial scan complete: found {} templates in {:.2}s",
            total_templates,
            elapsed.as_secs_f64()
        );
        
        Ok(())
    }

    /// Handle file change events
    async fn handle_events(&self, rx: std::sync::mpsc::Receiver<DebounceEventResult>) {
        loop {
            match rx.recv() {
                Ok(Ok(events)) => {
                    for event in events {
                        let path = event.path;
                        let kind = match event.kind {
                            notify::EventKind::Create(_) => ChangeKind::Created,
                            notify::EventKind::Modify(_) => ChangeKind::Modified,
                            notify::EventKind::Remove(_) => ChangeKind::Removed,
                            _ => continue,
                        };
                        
                        let change = FileChange { path, kind };
                        if let Err(e) = self.handle_file_change(change).await {
                            error!("Error handling file change: {}", e);
                        }
                    }
                }
                Ok(Err(e)) => {
                    error!("Watch error: {:?}", e);
                }
                Err(_) => {
                    info!("File watcher channel closed");
                    break;
                }
            }
        }
    }

    /// Handle a single file change
    async fn handle_file_change(&self, change: FileChange) -> Result<()> {
        let path = &change.path;
        
        if !self.should_process_file(path) {
            return Ok(());
        }
        
        // Apply debouncing
        let now = Instant::now();
        let mut last_processed = self.last_processed.lock().await;
        if let Some(&last_time) = last_processed.get(path) {
            if now.duration_since(last_time) < Duration::from_millis(1000) {
                debug!("Debouncing file change for {}", path.display());
                return Ok(());
            }
        }
        
        // Update last processed time
        last_processed.insert(path.clone(), now);
        drop(last_processed);
        
        // Update stats
        let mut stats = self.stats.write().await;
        stats.files_processed += 1;
        drop(stats);
        
        match change.kind {
            ChangeKind::Created | ChangeKind::Modified => {
                info!("Processing file change: {}", path.display());
                
                if let Ok(updates) = self.process_file(path).await {
                    if !updates.is_empty() {
                        // Analyze updates with diffing
                        let mut cache = self.cache.write().await;
                        match cache.insert_with_analysis(updates) {
                            Ok(analysis) => {
                                // Update stats
                                let mut stats = self.stats.write().await;
                                stats.hot_reloadable_changes += analysis.hot_reloadable.len();
                                stats.rebuild_required_changes += analysis.require_rebuild.len();
                                
                                if !analysis.delta_operations.is_empty() {
                                    stats.delta_updates_sent += analysis.delta_operations.len();
                                }
                                drop(stats);
                                
                                // Convert to messages
                                let messages = analysis_to_messages(analysis);
                                
                                if !messages.is_empty() {
                                    info!("Broadcasting {} hot reload messages", messages.len());
                                    let _ = self.tx.send(messages).await;
                                }
                            }
                            Err(e) => {
                                error!("Failed to analyze template updates: {}", e);
                            }
                        }
                    }
                }
            }
            ChangeKind::Removed => {
                // Handle file removal
                let cache = self.cache.read().await;
                let ids = cache.cache.get_all_ids();
                drop(cache);
                
                let mut removed_count = 0;
                for id in ids {
                    if id.file == path {
                        let mut cache = self.cache.write().await;
                        if cache.cache.remove(&id).is_some() {
                            removed_count += 1;
                        }
                    }
                }
                
                if removed_count > 0 {
                    info!("Removed {} templates from {}", removed_count, path.display());
                }
            }
        }
        
        Ok(())
    }

    /// Process a file and extract templates
    async fn process_file(&self, path: &Path) -> Result<Vec<TemplateUpdate>> {
        let content = tokio::fs::read_to_string(path)
            .await
            .context("Failed to read file")?;
        
        let mut parser = TemplateParser::new(path);
        let updates = parser.parse_file(&content)?;
        
        debug!("Found {} templates in {}", updates.len(), path.display());
        
        Ok(updates)
    }

    /// Check if a file should be processed
    fn should_process_file(&self, path: &Path) -> bool {
        if !path.is_file() {
            return false;
        }
        
        // Filter out files in target/ directory
        if path.components().any(|c| c.as_os_str() == "target") {
            return false;
        }
        
        // Get file name for filtering
        let file_name = match path.file_name() {
            Some(name) => name.to_string_lossy(),
            None => return false,
        };
        
        // Filter out temporary files
        if file_name.ends_with('~') ||
           file_name.starts_with(".#") ||
           file_name.starts_with('#') && file_name.ends_with('#') ||
           file_name.starts_with(".DS_Store") {
            return false;
        }
        
        // Check if file has the correct extension
        if let Some(ext) = path.extension() {
            self.extensions
                .iter()
                .any(|e| e == &ext.to_string_lossy())
        } else {
            false
        }
    }

    /// Get watcher statistics
    pub async fn stats(&self) -> WatcherStats {
        self.stats.read().await.clone()
    }

    /// Get cache statistics
    pub async fn cache_stats(&self) -> (crate::template_cache::CacheStats, crate::diff_integration::DecisionMakerStats) {
        self.cache.read().await.stats()
    }
}

/// Convert enhanced messages to standard protocol messages
pub fn enhanced_to_standard_messages(messages: Vec<EnhancedHotReloadMessage>) -> Vec<HotReloadMessage> {
    messages.into_iter().map(|msg| {
        match msg {
            EnhancedHotReloadMessage::Standard(standard) => standard,
            EnhancedHotReloadMessage::DeltaUpdate { template_id, .. } => {
                // For now, convert delta updates to standard updates
                // In a full implementation, this would be handled by the client
                HotReloadMessage::ReloadRequest { template_id }
            }
            EnhancedHotReloadMessage::BatchDeltaUpdate { updates } => {
                // Convert to reload requests
                HotReloadMessage::BatchUpdate {
                    updates: updates.into_iter()
                        .map(|(id, hash, _)| TemplateUpdate {
                            id: id.clone(),
                            hash,
                            content_hash: String::new(),
                            html: String::new(),
                            dynamic_parts: vec![],
                        })
                        .collect(),
                }
            }
        }
    }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use tokio::fs;

    #[tokio::test]
    async fn test_enhanced_watcher_creation() {
        let temp_dir = TempDir::new().unwrap();
        let (watcher, _rx) = EnhancedFileWatcher::new(
            vec![temp_dir.path().to_path_buf()],
            vec!["rs".to_string()],
            100,
        );
        
        assert_eq!(watcher.extensions, vec!["rs"]);
        assert_eq!(watcher.watch_paths.len(), 1);
    }

    #[tokio::test]
    async fn test_should_process_file() {
        let temp_dir = TempDir::new().unwrap();
        let (watcher, _rx) = EnhancedFileWatcher::new(
            vec![temp_dir.path().to_path_buf()],
            vec!["rs".to_string()],
            100,
        );
        
        // Should process .rs files
        let rs_file = temp_dir.path().join("test.rs");
        fs::write(&rs_file, "// test").await.unwrap();
        assert!(watcher.should_process_file(&rs_file));
        
        // Should not process .txt files
        let txt_file = temp_dir.path().join("test.txt");
        fs::write(&txt_file, "test").await.unwrap();
        assert!(!watcher.should_process_file(&txt_file));
        
        // Should not process temporary files
        let temp_file = temp_dir.path().join("test.rs~");
        fs::write(&temp_file, "// test").await.unwrap();
        assert!(!watcher.should_process_file(&temp_file));
    }

    #[tokio::test]
    async fn test_stats_tracking() {
        let temp_dir = TempDir::new().unwrap();
        let (watcher, _rx) = EnhancedFileWatcher::new(
            vec![temp_dir.path().to_path_buf()],
            vec!["rs".to_string()],
            100,
        );
        
        let stats = watcher.stats().await;
        assert_eq!(stats.files_processed, 0);
        assert_eq!(stats.hot_reloadable_changes, 0);
        assert_eq!(stats.rebuild_required_changes, 0);
    }
}