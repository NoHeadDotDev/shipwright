//! Template caching system for hot reload with LRU eviction and dependency tracking

use dashmap::DashMap;
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::protocol::{TemplateId, TemplateUpdate};

/// Cache entry for a template
#[derive(Debug, Clone)]
struct CacheEntry {
    /// The template update data
    update: TemplateUpdate,
    /// When this entry was last accessed
    last_accessed: Instant,
    /// When this entry was created
    created_at: Instant,
    /// Version number for this template
    version: u64,
    /// Size in bytes (approximate)
    size_bytes: usize,
    /// Compilation time in microseconds
    compile_time_us: u64,
    /// Number of cache hits
    hit_count: u64,
    /// Templates that depend on this one
    dependents: HashSet<String>,
    /// Templates this one depends on
    dependencies: HashSet<String>,
}

/// LRU node for tracking access order
#[derive(Debug, Clone)]
struct LruNode {
    key: String,
    prev: Option<String>,
    next: Option<String>,
}

/// Template cache for hot reload system with LRU eviction
#[derive(Debug, Clone)]
pub struct TemplateCache {
    /// Internal cache storage
    cache: Arc<DashMap<String, CacheEntry>>,
    /// LRU tracking
    lru: Arc<Mutex<LruTracker>>,
    /// Maximum age for cache entries
    max_age: Duration,
    /// Maximum cache size in bytes
    max_size_bytes: usize,
    /// Maximum number of entries
    max_entries: usize,
    /// Current total size in bytes
    total_size: Arc<Mutex<usize>>,
    /// Dependency graph
    dependency_graph: Arc<Mutex<DependencyGraph>>,
    /// Cache statistics
    stats: Arc<Mutex<CacheStatistics>>,
}

/// LRU tracker for cache eviction
#[derive(Debug)]
struct LruTracker {
    nodes: HashMap<String, LruNode>,
    head: Option<String>,
    tail: Option<String>,
}

impl LruTracker {
    fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            head: None,
            tail: None,
        }
    }

    fn access(&mut self, key: &str) {
        if let Some(node) = self.nodes.get(key).cloned() {
            self.remove_node(&node.key);
            self.push_front(node);
        } else {
            let node = LruNode {
                key: key.to_string(),
                prev: None,
                next: None,
            };
            self.push_front(node);
        }
    }

    fn push_front(&mut self, mut node: LruNode) {
        node.prev = None;
        node.next = self.head.clone();

        if let Some(ref head_key) = self.head {
            if let Some(head_node) = self.nodes.get_mut(head_key) {
                head_node.prev = Some(node.key.clone());
            }
        }

        self.head = Some(node.key.clone());

        if self.tail.is_none() {
            self.tail = Some(node.key.clone());
        }

        self.nodes.insert(node.key.clone(), node);
    }

    fn remove_node(&mut self, key: &str) {
        if let Some(node) = self.nodes.remove(key) {
            if let Some(ref prev_key) = node.prev {
                if let Some(prev_node) = self.nodes.get_mut(prev_key) {
                    prev_node.next = node.next.clone();
                }
            } else {
                self.head = node.next.clone();
            }

            if let Some(ref next_key) = node.next {
                if let Some(next_node) = self.nodes.get_mut(next_key) {
                    next_node.prev = node.prev.clone();
                }
            } else {
                self.tail = node.prev.clone();
            }
        }
    }

    fn pop_lru(&mut self) -> Option<String> {
        if let Some(tail_key) = self.tail.clone() {
            self.remove_node(&tail_key);
            Some(tail_key)
        } else {
            None
        }
    }
}

/// Dependency graph for template relationships
#[derive(Debug, Default)]
struct DependencyGraph {
    /// Map from template to its dependencies
    dependencies: HashMap<String, HashSet<String>>,
    /// Map from template to its dependents
    dependents: HashMap<String, HashSet<String>>,
}

impl DependencyGraph {
    fn add_dependency(&mut self, template: &str, dependency: &str) {
        self.dependencies
            .entry(template.to_string())
            .or_default()
            .insert(dependency.to_string());
        
        self.dependents
            .entry(dependency.to_string())
            .or_default()
            .insert(template.to_string());
    }

    fn remove_template(&mut self, template: &str) {
        // Remove from dependencies
        if let Some(deps) = self.dependencies.remove(template) {
            for dep in deps {
                if let Some(dependents) = self.dependents.get_mut(&dep) {
                    dependents.remove(template);
                }
            }
        }

        // Remove from dependents
        if let Some(dependents) = self.dependents.remove(template) {
            for dependent in dependents {
                if let Some(deps) = self.dependencies.get_mut(&dependent) {
                    deps.remove(template);
                }
            }
        }
    }

    fn get_dependents(&self, template: &str) -> HashSet<String> {
        self.dependents
            .get(template)
            .cloned()
            .unwrap_or_default()
    }

    fn get_dependencies(&self, template: &str) -> HashSet<String> {
        self.dependencies
            .get(template)
            .cloned()
            .unwrap_or_default()
    }

    fn get_transitive_dependents(&self, template: &str) -> HashSet<String> {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        queue.push_back(template.to_string());

        while let Some(current) = queue.pop_front() {
            if visited.insert(current.clone()) {
                if let Some(deps) = self.dependents.get(&current) {
                    for dep in deps {
                        queue.push_back(dep.clone());
                    }
                }
            }
        }

        visited.remove(template);
        visited
    }
}

/// Cache statistics for monitoring
#[derive(Debug, Default, Clone)]
pub struct CacheStatistics {
    /// Total number of cache hits
    pub total_hits: u64,
    /// Total number of cache misses
    pub total_misses: u64,
    /// Total number of evictions
    pub total_evictions: u64,
    /// Average compilation time in microseconds
    pub avg_compile_time_us: f64,
    /// Total compilation time in microseconds
    pub total_compile_time_us: u64,
    /// Number of compilations
    pub compilation_count: u64,
    /// Current cache size in bytes
    pub current_size_bytes: usize,
    /// Peak cache size in bytes
    pub peak_size_bytes: usize,
    /// Total number of entries in the cache
    pub total_entries: usize,
    /// Timestamp of the oldest entry
    pub oldest_entry: Option<Instant>,
    /// Timestamp of the newest entry
    pub newest_entry: Option<Instant>,
}

impl TemplateCache {
    /// Create a new template cache
    pub fn new() -> Self {
        Self::with_config(
            Duration::from_secs(3600), // 1 hour default
            100 * 1024 * 1024,         // 100MB default
            10000,                      // 10k entries default
        )
    }

    /// Create a new template cache with custom configuration
    pub fn with_config(max_age: Duration, max_size_bytes: usize, max_entries: usize) -> Self {
        Self {
            cache: Arc::new(DashMap::new()),
            lru: Arc::new(Mutex::new(LruTracker::new())),
            max_age,
            max_size_bytes,
            max_entries,
            total_size: Arc::new(Mutex::new(0)),
            dependency_graph: Arc::new(Mutex::new(DependencyGraph::default())),
            stats: Arc::new(Mutex::new(CacheStatistics::default())),
        }
    }

    /// Estimate the size of a template update in bytes
    fn estimate_size(update: &TemplateUpdate) -> usize {
        update.html.len() + 
        update.hash.len() + 
        update.content_hash.len() +
        update.dynamic_parts.len() * 32 + // Approximate size per dynamic part
        200 // Overhead
    }

    /// Evict entries if necessary to maintain size and count limits
    fn evict_if_needed(&self, new_size: usize) {
        let mut lru = self.lru.lock().unwrap();
        let mut total_size = self.total_size.lock().unwrap();
        let mut stats = self.stats.lock().unwrap();
        
        // Evict based on entry count
        while self.cache.len() >= self.max_entries {
            if let Some(key) = lru.pop_lru() {
                if let Some((_, entry)) = self.cache.remove(&key) {
                    *total_size = total_size.saturating_sub(entry.size_bytes);
                    stats.total_evictions += 1;
                    
                    // Clean up dependency graph
                    let mut dep_graph = self.dependency_graph.lock().unwrap();
                    dep_graph.remove_template(&key);
                }
            } else {
                break;
            }
        }
        
        // Evict based on size
        while *total_size + new_size > self.max_size_bytes && self.cache.len() > 0 {
            if let Some(key) = lru.pop_lru() {
                if let Some((_, entry)) = self.cache.remove(&key) {
                    *total_size = total_size.saturating_sub(entry.size_bytes);
                    stats.total_evictions += 1;
                    
                    // Clean up dependency graph
                    let mut dep_graph = self.dependency_graph.lock().unwrap();
                    dep_graph.remove_template(&key);
                }
            } else {
                break;
            }
        }
    }

    /// Store a template update in the cache with compilation time
    /// Returns (version, content_changed) where content_changed indicates if the template content actually changed
    pub fn insert_with_timing(&self, update: TemplateUpdate, compile_time_us: u64) -> (u64, bool) {
        let hash = update.hash.clone();
        let size_bytes = Self::estimate_size(&update);
        
        // Check for content changes
        let content_changed = self
            .cache
            .get(&hash)
            .map(|entry| entry.update.content_hash != update.content_hash)
            .unwrap_or(true);
        
        let version = self
            .cache
            .get(&hash)
            .map(|entry| entry.version + 1)
            .unwrap_or(1);

        // Evict if needed before inserting
        self.evict_if_needed(size_bytes);

        let now = Instant::now();
        let entry = CacheEntry {
            update,
            last_accessed: now,
            created_at: now,
            version,
            size_bytes,
            compile_time_us,
            hit_count: 0,
            dependents: HashSet::new(),
            dependencies: HashSet::new(),
        };

        self.cache.insert(hash.clone(), entry);
        
        // Update LRU tracking
        {
            let mut lru = self.lru.lock().unwrap();
            lru.access(&hash);
        }
        
        // Update statistics
        {
            let mut total_size = self.total_size.lock().unwrap();
            let mut stats = self.stats.lock().unwrap();
            
            *total_size += size_bytes;
            stats.current_size_bytes = *total_size;
            stats.peak_size_bytes = stats.peak_size_bytes.max(*total_size);
            stats.total_compile_time_us += compile_time_us;
            stats.compilation_count += 1;
            stats.avg_compile_time_us = stats.total_compile_time_us as f64 / stats.compilation_count as f64;
        }

        (version, content_changed)
    }

    /// Store a template update in the cache
    /// Returns (version, content_changed) where content_changed indicates if the template content actually changed
    pub fn insert(&self, update: TemplateUpdate) -> (u64, bool) {
        self.insert_with_timing(update, 0)
    }

    /// Get a template from the cache
    pub fn get(&self, id: &TemplateId) -> Option<TemplateUpdate> {
        let hash = id.hash();
        self.get_by_hash(&hash)
    }

    /// Get a template by its hash
    pub fn get_by_hash(&self, hash: &str) -> Option<TemplateUpdate> {
        let result = self.cache.get_mut(hash).map(|mut entry| {
            entry.last_accessed = Instant::now();
            entry.hit_count += 1;
            entry.update.clone()
        });
        
        // Update statistics and LRU
        if result.is_some() {
            let mut stats = self.stats.lock().unwrap();
            stats.total_hits += 1;
            
            let mut lru = self.lru.lock().unwrap();
            lru.access(hash);
        } else {
            let mut stats = self.stats.lock().unwrap();
            stats.total_misses += 1;
        }
        
        result
    }

    /// Add a dependency relationship between templates
    pub fn add_dependency(&self, template_hash: &str, dependency_hash: &str) {
        // Update the cache entry
        if let Some(mut entry) = self.cache.get_mut(template_hash) {
            entry.dependencies.insert(dependency_hash.to_string());
        }
        
        if let Some(mut entry) = self.cache.get_mut(dependency_hash) {
            entry.dependents.insert(template_hash.to_string());
        }
        
        // Update the dependency graph
        let mut dep_graph = self.dependency_graph.lock().unwrap();
        dep_graph.add_dependency(template_hash, dependency_hash);
    }

    /// Get all templates that depend on the given template
    pub fn get_dependents(&self, id: &TemplateId) -> Vec<TemplateUpdate> {
        let hash = id.hash();
        let dep_graph = self.dependency_graph.lock().unwrap();
        let dependents = dep_graph.get_dependents(&hash);
        
        dependents
            .iter()
            .filter_map(|dep_hash| self.get_by_hash(dep_hash))
            .collect()
    }

    /// Get all templates that the given template depends on
    pub fn get_dependencies(&self, id: &TemplateId) -> Vec<TemplateUpdate> {
        let hash = id.hash();
        let dep_graph = self.dependency_graph.lock().unwrap();
        let dependencies = dep_graph.get_dependencies(&hash);
        
        dependencies
            .iter()
            .filter_map(|dep_hash| self.get_by_hash(dep_hash))
            .collect()
    }

    /// Get all transitive dependents (templates that directly or indirectly depend on this one)
    pub fn get_transitive_dependents(&self, id: &TemplateId) -> Vec<TemplateUpdate> {
        let hash = id.hash();
        let dep_graph = self.dependency_graph.lock().unwrap();
        let dependents = dep_graph.get_transitive_dependents(&hash);
        
        dependents
            .iter()
            .filter_map(|dep_hash| self.get_by_hash(dep_hash))
            .collect()
    }

    /// Check if a template exists in the cache
    pub fn contains(&self, id: &TemplateId) -> bool {
        self.cache.contains_key(&id.hash())
    }

    /// Remove a template from the cache
    pub fn remove(&self, id: &TemplateId) -> Option<TemplateUpdate> {
        let hash = id.hash();
        
        // Remove from LRU
        {
            let mut lru = self.lru.lock().unwrap();
            lru.remove_node(&hash);
        }
        
        // Remove from dependency graph
        {
            let mut dep_graph = self.dependency_graph.lock().unwrap();
            dep_graph.remove_template(&hash);
        }
        
        // Remove from cache and update size
        self.cache.remove(&hash).map(|(_, entry)| {
            let mut total_size = self.total_size.lock().unwrap();
            *total_size = total_size.saturating_sub(entry.size_bytes);
            
            let mut stats = self.stats.lock().unwrap();
            stats.current_size_bytes = *total_size;
            
            entry.update
        })
    }

    /// Clear all entries from the cache
    pub fn clear(&self) {
        self.cache.clear();
        
        let mut lru = self.lru.lock().unwrap();
        *lru = LruTracker::new();
        
        let mut dep_graph = self.dependency_graph.lock().unwrap();
        *dep_graph = DependencyGraph::default();
        
        let mut total_size = self.total_size.lock().unwrap();
        *total_size = 0;
        
        let mut stats = self.stats.lock().unwrap();
        stats.current_size_bytes = 0;
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
    pub fn stats(&self) -> CacheStatistics {
        let mut stats = self.stats.lock().unwrap().clone();
        
        // Update dynamic fields
        stats.total_entries = self.cache.len();
        
        // Find oldest and newest entries
        let mut oldest: Option<Instant> = None;
        let mut newest: Option<Instant> = None;
        
        for entry in self.cache.iter() {
            let created_at = entry.created_at;
            if oldest.is_none() || created_at < oldest.unwrap() {
                oldest = Some(created_at);
            }
            if newest.is_none() || created_at > newest.unwrap() {
                newest = Some(created_at);
            }
        }
        
        stats.oldest_entry = oldest;
        stats.newest_entry = newest;
        
        stats
    }

    /// Get detailed cache metrics
    pub fn detailed_stats(&self) -> DetailedCacheStats {
        let stats = self.stats.lock().unwrap().clone();
        let mut hot_templates = Vec::new();
        let mut cold_templates = Vec::new();
        let mut template_stats = Vec::new();
        
        let now = Instant::now();
        
        for entry in self.cache.iter() {
            let age = now.duration_since(entry.created_at);
            let last_access = now.duration_since(entry.last_accessed);
            
            let template_stat = TemplateStats {
                hash: entry.key().clone(),
                size_bytes: entry.size_bytes,
                hit_count: entry.hit_count,
                compile_time_us: entry.compile_time_us,
                age_seconds: age.as_secs(),
                last_access_seconds: last_access.as_secs(),
                version: entry.version,
                dependency_count: entry.dependencies.len(),
                dependent_count: entry.dependents.len(),
            };
            
            template_stats.push(template_stat);
            
            // Categorize hot/cold based on access patterns
            if entry.hit_count > 10 && last_access.as_secs() < 60 {
                hot_templates.push(entry.key().clone());
            } else if last_access.as_secs() > 300 {
                cold_templates.push(entry.key().clone());
            }
        }
        
        // Sort by various metrics
        let mut by_hits = template_stats.clone();
        by_hits.sort_by_key(|s| std::cmp::Reverse(s.hit_count));
        
        let mut by_size = template_stats.clone();
        by_size.sort_by_key(|s| std::cmp::Reverse(s.size_bytes));
        
        DetailedCacheStats {
            basic_stats: stats,
            hot_templates,
            cold_templates,
            top_by_hits: by_hits.into_iter().take(10).collect(),
            top_by_size: by_size.into_iter().take(10).collect(),
            total_entries: self.cache.len(),
        }
    }

    /// Warm the cache by preloading templates
    pub fn warm_cache(&self, templates: Vec<TemplateUpdate>) -> CacheWarmingResult {
        let start = Instant::now();
        let mut loaded = 0;
        let mut failed = 0;
        let mut total_size = 0;
        
        for update in templates {
            let size = Self::estimate_size(&update);
            if total_size + size <= self.max_size_bytes {
                self.insert(update);
                loaded += 1;
                total_size += size;
            } else {
                failed += 1;
            }
        }
        
        CacheWarmingResult {
            templates_loaded: loaded,
            templates_failed: failed,
            total_size_bytes: total_size,
            duration: start.elapsed(),
        }
    }

    /// Preemptively load related templates based on access patterns
    pub fn preemptive_load(&self, accessed_id: &TemplateId) -> Vec<String> {
        let hash = accessed_id.hash();
        let mut loaded = Vec::new();
        
        // Get dependencies that aren't already loaded
        let dep_graph = self.dependency_graph.lock().unwrap();
        let dependencies = dep_graph.get_dependencies(&hash);
        
        for dep_hash in dependencies {
            if !self.cache.contains_key(&dep_hash) {
                // In a real implementation, this would trigger loading from disk
                // For now, we just track what would be loaded
                loaded.push(dep_hash);
            }
        }
        
        loaded
    }

    /// Optimize cache by removing cold entries and compacting
    pub fn optimize(&self) -> OptimizationResult {
        let start = Instant::now();
        let initial_count = self.cache.len();
        let initial_size = self.total_size.lock().unwrap().clone();
        
        let now = Instant::now();
        let mut removed = 0;
        
        // Remove very old entries
        self.cache.retain(|_, entry| {
            let age = now.duration_since(entry.last_accessed);
            if age > self.max_age {
                removed += 1;
                false
            } else {
                true
            }
        });
        
        // Remove entries with very low hit rates
        let avg_hits = if initial_count > 0 {
            self.cache.iter()
                .map(|entry| entry.hit_count)
                .sum::<u64>() / initial_count as u64
        } else {
            0
        };
        
        self.cache.retain(|_, entry| {
            let age = now.duration_since(entry.created_at);
            // Remove if old and rarely accessed
            if age.as_secs() > 300 && entry.hit_count < avg_hits / 4 {
                removed += 1;
                false
            } else {
                true
            }
        });
        
        let final_count = self.cache.len();
        let final_size = self.total_size.lock().unwrap().clone();
        
        OptimizationResult {
            entries_removed: removed,
            space_freed_bytes: initial_size.saturating_sub(final_size),
            duration: start.elapsed(),
            initial_entries: initial_count,
            final_entries: final_count,
        }
    }
}

/// Detailed statistics for individual templates
#[derive(Debug, Clone)]
pub struct TemplateStats {
    pub hash: String,
    pub size_bytes: usize,
    pub hit_count: u64,
    pub compile_time_us: u64,
    pub age_seconds: u64,
    pub last_access_seconds: u64,
    pub version: u64,
    pub dependency_count: usize,
    pub dependent_count: usize,
}

/// Detailed cache statistics
#[derive(Debug)]
pub struct DetailedCacheStats {
    pub basic_stats: CacheStatistics,
    pub hot_templates: Vec<String>,
    pub cold_templates: Vec<String>,
    pub top_by_hits: Vec<TemplateStats>,
    pub top_by_size: Vec<TemplateStats>,
    pub total_entries: usize,
}

/// Result of cache warming operation
#[derive(Debug)]
pub struct CacheWarmingResult {
    pub templates_loaded: usize,
    pub templates_failed: usize,
    pub total_size_bytes: usize,
    pub duration: Duration,
}

/// Result of cache optimization
#[derive(Debug)]
pub struct OptimizationResult {
    pub entries_removed: usize,
    pub space_freed_bytes: usize,
    pub duration: Duration,
    pub initial_entries: usize,
    pub final_entries: usize,
}

impl Default for TemplateCache {
    fn default() -> Self {
        Self::new()
    }
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