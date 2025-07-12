//! File watching system for detecting template changes

use anyhow::{Context, Result};
use notify::RecursiveMode;
use notify_debouncer_mini::{new_debouncer, DebounceEventResult};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};
use std::collections::HashMap;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};
use walkdir::WalkDir;

use crate::{
    parser::TemplateParser,
    protocol::TemplateUpdate,
    template_cache::TemplateCache,
    diff_integration::{DiffAwareTemplateCache, analysis_to_messages, EnhancedHotReloadMessage},
};

/// File change event
#[derive(Debug, Clone)]
pub struct FileChange {
    /// Path to the changed file
    pub path: PathBuf,
    /// Type of change
    pub kind: ChangeKind,
}

/// Type of file change
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeKind {
    /// File was created
    Created,
    /// File was modified
    Modified,
    /// File was removed
    Removed,
}

/// File watcher for detecting template changes
pub struct FileWatcher {
    /// Paths to watch
    watch_paths: Vec<PathBuf>,
    /// File extensions to watch
    extensions: Vec<String>,
    /// Template cache
    cache: Arc<TemplateCache>,
    /// Channel for sending updates
    tx: mpsc::Sender<Vec<TemplateUpdate>>,
    /// Last processing time per file for additional debouncing
    last_processed: Arc<tokio::sync::Mutex<HashMap<PathBuf, Instant>>>,
}

impl FileWatcher {
    /// Create a new file watcher
    pub fn new(
        watch_paths: Vec<PathBuf>,
        cache: Arc<TemplateCache>,
        tx: mpsc::Sender<Vec<TemplateUpdate>>,
    ) -> Self {
        Self {
            watch_paths,
            extensions: vec!["rs".to_string()],
            cache,
            tx,
            last_processed: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
        }
    }

    /// Start watching for file changes
    pub async fn start(self) -> Result<()> {
        let (notify_tx, mut notify_rx) = mpsc::channel(100);
        
        // Create debouncer with increased duration
        let mut debouncer = new_debouncer(
            Duration::from_millis(500),
            move |result: DebounceEventResult| {
                match result {
                    Ok(events) => {
                        for event in events {
                            let _ = notify_tx.blocking_send(event);
                        }
                    }
                    Err(error) => {
                        error!("Watch error: {:?}", error);
                    }
                }
            },
        )?;

        // Add watch paths
        for path in &self.watch_paths {
            info!("Watching path: {}", path.display());
            debouncer
                .watcher()
                .watch(path, RecursiveMode::Recursive)
                .context("Failed to watch path")?;
        }

        // Initial scan
        self.initial_scan().await?;

        // Process events
        while let Some(event) = notify_rx.recv().await {
            if let Err(e) = self.handle_event(event).await {
                error!("Error handling file event: {}", e);
            }
        }

        Ok(())
    }

    /// Perform initial scan of watched directories
    async fn initial_scan(&self) -> Result<()> {
        info!("Performing initial scan of watched directories");
        
        let mut all_updates = Vec::new();
        
        for watch_path in &self.watch_paths {
            for entry in WalkDir::new(watch_path)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                let path = entry.path();
                if self.should_process_file(path) {
                    match self.process_file(path).await {
                        Ok(updates) => {
                            for update in updates {
                                let (_, content_changed) = self.cache.insert(update.clone());
                                if content_changed {
                                    all_updates.push(update);
                                }
                            }
                        }
                        Err(e) => {
                            warn!("Failed to process file {}: {}", path.display(), e);
                        }
                    }
                }
            }
        }
        
        if !all_updates.is_empty() {
            info!("Found {} templates in initial scan - caching but not broadcasting", all_updates.len());
            // Don't broadcast initial scan results - they're not changes
            // Just cache them for when clients connect
        }
        
        Ok(())
    }

    /// Handle a file system event
    async fn handle_event(&self, event: notify_debouncer_mini::DebouncedEvent) -> Result<()> {
        let path = &event.path;
        
        if !self.should_process_file(path) {
            return Ok(());
        }

        // Additional debouncing: check if we've processed this file recently
        let now = Instant::now();
        let mut last_processed = self.last_processed.lock().await;
        
        if let Some(&last_time) = last_processed.get(path) {
            if now.duration_since(last_time) < Duration::from_millis(1000) {
                debug!("Skipping rapid successive update for {}", path.display());
                return Ok(());
            }
        }
        
        // Update last processed time
        last_processed.insert(path.clone(), now);
        drop(last_processed);

        println!("🔥 Processing file change: {}", path.display());

        match event.kind {
            notify_debouncer_mini::DebouncedEventKind::Any => {
                // Process as modification
                self.handle_file_change(path, ChangeKind::Modified).await?;
            }
            _ => {
                // Handle other event kinds as modifications
                self.handle_file_change(path, ChangeKind::Modified).await?;
            }
        }

        Ok(())
    }

    /// Handle a specific file change
    async fn handle_file_change(&self, path: &Path, kind: ChangeKind) -> Result<()> {
        match kind {
            ChangeKind::Created | ChangeKind::Modified => {
                let updates = self.process_file(path).await?;
                if !updates.is_empty() {
                    let mut changed_updates = Vec::new();
                    
                    // Update cache and collect only actually changed templates
                    for update in updates {
                        let (_, content_changed) = self.cache.insert(update.clone());
                        if content_changed {
                            changed_updates.push(update);
                        }
                    }
                    
                    if !changed_updates.is_empty() {
                        println!("🔥 Found {} template content changes, broadcasting...", changed_updates.len());
                        let _ = self.tx.send(changed_updates).await;
                    } else {
                        println!("⚡ File changed but template content is identical, skipping broadcast");
                    }
                }
            }
            ChangeKind::Removed => {
                // Remove templates from this file
                let ids = self.cache.get_all_ids();
                let mut removed_updates = Vec::new();
                
                for id in ids {
                    if id.file == path {
                        if let Some(update) = self.cache.remove(&id) {
                            removed_updates.push(update);
                        }
                    }
                }
                
                if !removed_updates.is_empty() {
                    info!("Removed {} templates from {}", removed_updates.len(), path.display());
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

        // Filter out temporary files ending with ~
        if file_name.ends_with('~') {
            return false;
        }

        // Filter out numbered temporary files (files with only numeric names)
        if file_name.chars().all(|c| c.is_ascii_digit()) {
            return false;
        }

        // Filter out common temporary file extensions
        let temp_extensions = [".tmp", ".swp", ".bak", ".swo", ".orig", ".rej"];
        if temp_extensions.iter().any(|ext| file_name.ends_with(ext)) {
            return false;
        }

        // Filter out other common editor temporary files
        if file_name.starts_with(".#") || // Emacs lock files
           file_name.starts_with('#') && file_name.ends_with('#') || // Emacs auto-save files
           file_name.starts_with(".DS_Store") || // macOS files
           file_name.ends_with(".autosave") || // Qt Creator auto-save files
           file_name.ends_with("~") // Backup files
        {
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_should_process_file() {
        let cache = Arc::new(TemplateCache::new());
        let (tx, _rx) = mpsc::channel(10);
        let watcher = FileWatcher::new(vec![], cache, tx);

        // Should process valid .rs files
        assert!(watcher.should_process_file(Path::new("test.rs")));
        assert!(watcher.should_process_file(Path::new("src/main.rs")));
        
        // Should not process non-.rs files
        assert!(!watcher.should_process_file(Path::new("test.txt")));
        assert!(!watcher.should_process_file(Path::new("test")));
        
        // Should not process temporary files ending with ~
        assert!(!watcher.should_process_file(Path::new("main.rs~")));
        assert!(!watcher.should_process_file(Path::new("src/lib.rs~")));
        
        // Should not process numbered temporary files
        assert!(!watcher.should_process_file(Path::new("4913")));
        assert!(!watcher.should_process_file(Path::new("1234")));
        
        // Should not process files in target/ directory
        assert!(!watcher.should_process_file(Path::new("target/debug/main.rs")));
        assert!(!watcher.should_process_file(Path::new("target/release/lib.rs")));
        
        // Should not process common temporary file extensions
        assert!(!watcher.should_process_file(Path::new("test.tmp")));
        assert!(!watcher.should_process_file(Path::new("main.swp")));
        assert!(!watcher.should_process_file(Path::new("backup.bak")));
        assert!(!watcher.should_process_file(Path::new("file.swo")));
        assert!(!watcher.should_process_file(Path::new("patch.orig")));
        assert!(!watcher.should_process_file(Path::new("conflict.rej")));
        
        // Should not process editor temporary files
        assert!(!watcher.should_process_file(Path::new(".#main.rs")));
        assert!(!watcher.should_process_file(Path::new("#main.rs#")));
        assert!(!watcher.should_process_file(Path::new(".DS_Store")));
        assert!(!watcher.should_process_file(Path::new("main.rs.autosave")));
    }

    #[tokio::test]
    async fn test_process_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.rs");
        
        let content = r#"
            fn render() {
                html! {
                    <div>Test</div>
                }
            }
        "#;
        
        tokio::fs::write(&file_path, content).await.unwrap();

        let cache = Arc::new(TemplateCache::new());
        let (tx, _rx) = mpsc::channel(10);
        let watcher = FileWatcher::new(vec![], cache, tx);

        let updates = watcher.process_file(&file_path).await.unwrap();
        assert_eq!(updates.len(), 1);
    }
}