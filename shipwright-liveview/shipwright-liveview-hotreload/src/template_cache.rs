//! Template caching system for hot reload

use dashmap::DashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::protocol::{TemplateId, TemplateUpdate};

/// Cache entry for a template
#[derive(Debug, Clone)]
struct CacheEntry {
    /// The template update data
    update: TemplateUpdate,
    /// When this entry was last accessed
    last_accessed: Instant,
    /// Version number for this template
    version: u64,
}

/// Template cache for hot reload system
#[derive(Debug, Clone)]
pub struct TemplateCache {
    /// Internal cache storage
    cache: Arc<DashMap<String, CacheEntry>>,
    /// Maximum age for cache entries
    max_age: Duration,
}

impl TemplateCache {
    /// Create a new template cache
    pub fn new() -> Self {
        Self::with_max_age(Duration::from_secs(3600)) // 1 hour default
    }

    /// Create a new template cache with custom max age
    pub fn with_max_age(max_age: Duration) -> Self {
        Self {
            cache: Arc::new(DashMap::new()),
            max_age,
        }
    }

    /// Store a template update in the cache
    /// Returns (version, content_changed) where content_changed indicates if the template content actually changed
    pub fn insert(&self, update: TemplateUpdate) -> (u64, bool) {
        let hash = update.hash.clone();
        let content_changed = self
            .cache
            .get(&hash)
            .map(|entry| entry.update.content_hash != update.content_hash)
            .unwrap_or(true); // No existing entry means it's new content
        
        let version = self
            .cache
            .get(&hash)
            .map(|entry| entry.version + 1)
            .unwrap_or(1);

        let entry = CacheEntry {
            update,
            last_accessed: Instant::now(),
            version,
        };

        self.cache.insert(hash, entry);
        (version, content_changed)
    }

    /// Get a template from the cache
    pub fn get(&self, id: &TemplateId) -> Option<TemplateUpdate> {
        let hash = id.hash();
        self.cache.get_mut(&hash).map(|mut entry| {
            entry.last_accessed = Instant::now();
            entry.update.clone()
        })
    }

    /// Get a template by its hash
    pub fn get_by_hash(&self, hash: &str) -> Option<TemplateUpdate> {
        self.cache.get_mut(hash).map(|mut entry| {
            entry.last_accessed = Instant::now();
            entry.update.clone()
        })
    }

    /// Check if a template exists in the cache
    pub fn contains(&self, id: &TemplateId) -> bool {
        self.cache.contains_key(&id.hash())
    }

    /// Remove a template from the cache
    pub fn remove(&self, id: &TemplateId) -> Option<TemplateUpdate> {
        self.cache
            .remove(&id.hash())
            .map(|(_, entry)| entry.update)
    }

    /// Clear all entries from the cache
    pub fn clear(&self) {
        self.cache.clear();
    }

    /// Get the number of cached templates
    pub fn len(&self) -> usize {
        self.cache.len()
    }

    /// Check if the cache is empty
    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }

    /// Clean up old entries
    pub fn cleanup(&self) {
        let now = Instant::now();
        let max_age = self.max_age;

        self.cache.retain(|_, entry| {
            now.duration_since(entry.last_accessed) < max_age
        });
    }

    /// Get all cached template IDs
    pub fn get_all_ids(&self) -> Vec<TemplateId> {
        self.cache
            .iter()
            .map(|entry| entry.update.id.clone())
            .collect()
    }

    /// Get all cached updates
    pub fn get_all_updates(&self) -> Vec<TemplateUpdate> {
        self.cache
            .iter()
            .map(|entry| entry.update.clone())
            .collect()
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        let mut total_entries = 0;
        let mut oldest_entry = None;
        let mut newest_entry = None;

        for entry in self.cache.iter() {
            total_entries += 1;
            
            if oldest_entry.is_none() || entry.last_accessed < oldest_entry.unwrap() {
                oldest_entry = Some(entry.last_accessed);
            }
            
            if newest_entry.is_none() || entry.last_accessed > newest_entry.unwrap() {
                newest_entry = Some(entry.last_accessed);
            }
        }

        CacheStats {
            total_entries,
            oldest_entry,
            newest_entry,
        }
    }
}

impl Default for TemplateCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about the cache
#[derive(Debug)]
pub struct CacheStats {
    /// Total number of entries
    pub total_entries: usize,
    /// Timestamp of oldest entry
    pub oldest_entry: Option<Instant>,
    /// Timestamp of newest entry
    pub newest_entry: Option<Instant>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_cache_operations() {
        let cache = TemplateCache::new();
        let id = TemplateId::new(PathBuf::from("test.rs"), 10, 5);
        let html = "<div>Test</div>".to_string();
        let dynamic_parts = vec![];
        let update = TemplateUpdate {
            id: id.clone(),
            hash: id.hash(),
            content_hash: TemplateUpdate::compute_content_hash(&html, &dynamic_parts),
            html,
            dynamic_parts,
        };

        // Test insert
        let (version, content_changed) = cache.insert(update.clone());
        assert_eq!(version, 1);
        assert!(content_changed); // First insert should indicate content changed

        // Test get
        let retrieved = cache.get(&id);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().html, "<div>Test</div>");

        // Test contains
        assert!(cache.contains(&id));

        // Test remove
        let removed = cache.remove(&id);
        assert!(removed.is_some());
        assert!(!cache.contains(&id));
    }

    #[test]
    fn test_cache_versioning() {
        let cache = TemplateCache::new();
        let id = TemplateId::new(PathBuf::from("test.rs"), 10, 5);
        
        let mut html = "<div>v1</div>".to_string();
        let dynamic_parts = vec![];
        let mut update = TemplateUpdate {
            id: id.clone(),
            hash: id.hash(),
            content_hash: TemplateUpdate::compute_content_hash(&html, &dynamic_parts),
            html,
            dynamic_parts,
        };

        let (v1, changed1) = cache.insert(update.clone());
        assert_eq!(v1, 1);
        assert!(changed1);

        html = "<div>v2</div>".to_string();
        update.html = html.clone();
        update.content_hash = TemplateUpdate::compute_content_hash(&html, &update.dynamic_parts);
        let (v2, changed2) = cache.insert(update);
        assert_eq!(v2, 2);
        assert!(changed2); // Content actually changed
    }

    #[test]
    fn test_content_hash_change_detection() {
        let cache = TemplateCache::new();
        let id = TemplateId::new(PathBuf::from("test.rs"), 10, 5);
        
        let html = "<div>Same content</div>".to_string();
        let dynamic_parts = vec![];
        let update = TemplateUpdate {
            id: id.clone(),
            hash: id.hash(),
            content_hash: TemplateUpdate::compute_content_hash(&html, &dynamic_parts),
            html,
            dynamic_parts,
        };

        // First insert should indicate content changed
        let (v1, changed1) = cache.insert(update.clone());
        assert_eq!(v1, 1);
        assert!(changed1);

        // Second insert with identical content should not indicate change
        let (v2, changed2) = cache.insert(update);
        assert_eq!(v2, 2);
        assert!(!changed2); // Content is identical
    }
}