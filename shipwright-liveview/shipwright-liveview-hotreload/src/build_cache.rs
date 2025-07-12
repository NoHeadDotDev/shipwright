//! Build cache for selective template recompilation

use dashmap::DashMap;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::protocol::TemplateId;

/// Build cache entry containing compilation metadata
#[derive(Debug, Clone)]
pub struct BuildEntry {
    /// Template identifier
    pub template_id: TemplateId,
    /// Last successful build time
    pub last_build: Instant,
    /// Build duration
    pub build_duration: Duration,
    /// Source file hash at build time
    pub source_hash: String,
    /// Dependencies at build time
    pub dependencies: HashSet<PathBuf>,
    /// Whether this template has errors
    pub has_errors: bool,
    /// Build error if any
    pub error: Option<String>,
}

/// Build cache for tracking compilation state
#[derive(Debug, Clone)]
pub struct BuildCache {
    /// Cache of build entries by template hash
    entries: Arc<DashMap<String, BuildEntry>>,
    /// Reverse dependency map (file -> templates that depend on it)
    file_dependents: Arc<Mutex<HashMap<PathBuf, HashSet<String>>>>,
    /// Build statistics
    stats: Arc<Mutex<BuildStats>>,
}

/// Build statistics
#[derive(Debug, Default)]
struct BuildStats {
    /// Total number of builds
    total_builds: u64,
    /// Number of cache hits (avoided rebuilds)
    cache_hits: u64,
    /// Number of incremental rebuilds
    incremental_rebuilds: u64,
    /// Number of full rebuilds
    full_rebuilds: u64,
    /// Total build time
    total_build_time: Duration,
}

impl BuildCache {
    /// Create a new build cache
    pub fn new() -> Self {
        Self {
            entries: Arc::new(DashMap::new()),
            file_dependents: Arc::new(Mutex::new(HashMap::new())),
            stats: Arc::new(Mutex::new(BuildStats::default())),
        }
    }

    /// Record a successful build
    pub fn record_build(
        &self,
        template_id: TemplateId,
        source_hash: String,
        dependencies: HashSet<PathBuf>,
        build_duration: Duration,
    ) {
        let hash = template_id.hash();
        
        // Update file dependents map
        {
            let mut file_deps = self.file_dependents.lock().unwrap();
            
            // Remove old dependencies
            if let Some(entry) = self.entries.get(&hash) {
                for dep in &entry.dependencies {
                    if let Some(dependents) = file_deps.get_mut(dep) {
                        dependents.remove(&hash);
                        if dependents.is_empty() {
                            file_deps.remove(dep);
                        }
                    }
                }
            }
            
            // Add new dependencies
            for dep in &dependencies {
                file_deps
                    .entry(dep.clone())
                    .or_default()
                    .insert(hash.clone());
            }
        }
        
        // Create build entry
        let entry = BuildEntry {
            template_id,
            last_build: Instant::now(),
            build_duration,
            source_hash,
            dependencies,
            has_errors: false,
            error: None,
        };
        
        self.entries.insert(hash, entry);
        
        // Update stats
        let mut stats = self.stats.lock().unwrap();
        stats.total_builds += 1;
        stats.total_build_time += build_duration;
    }

    /// Record a failed build
    pub fn record_error(
        &self,
        template_id: TemplateId,
        error: String,
        build_duration: Duration,
    ) {
        let hash = template_id.hash();
        
        if let Some(mut entry) = self.entries.get_mut(&hash) {
            entry.has_errors = true;
            entry.error = Some(error);
            entry.build_duration = build_duration;
            entry.last_build = Instant::now();
        } else {
            let entry = BuildEntry {
                template_id,
                last_build: Instant::now(),
                build_duration,
                source_hash: String::new(),
                dependencies: HashSet::new(),
                has_errors: true,
                error: Some(error),
            };
            self.entries.insert(hash, entry);
        }
        
        let mut stats = self.stats.lock().unwrap();
        stats.total_builds += 1;
        stats.total_build_time += build_duration;
    }

    /// Check if a template needs rebuilding
    pub fn needs_rebuild(&self, template_id: &TemplateId, current_hash: &str) -> RebuildDecision {
        let hash = template_id.hash();
        
        if let Some(entry) = self.entries.get(&hash) {
            // Always rebuild if there were errors
            if entry.has_errors {
                return RebuildDecision::Required(RebuildReason::PreviousError);
            }
            
            // Check if source changed
            if entry.source_hash != current_hash {
                return RebuildDecision::Required(RebuildReason::SourceChanged);
            }
            
            // Check if any dependencies changed
            for dep in &entry.dependencies {
                if self.is_file_modified(dep, entry.last_build) {
                    return RebuildDecision::Required(RebuildReason::DependencyChanged(dep.clone()));
                }
            }
            
            // No rebuild needed
            let mut stats = self.stats.lock().unwrap();
            stats.cache_hits += 1;
            
            RebuildDecision::NotRequired
        } else {
            // No cache entry, rebuild required
            RebuildDecision::Required(RebuildReason::NoCacheEntry)
        }
    }

    /// Get all templates affected by a file change
    pub fn get_affected_templates(&self, file_path: &Path) -> Vec<TemplateId> {
        let file_deps = self.file_dependents.lock().unwrap();
        
        if let Some(template_hashes) = file_deps.get(file_path) {
            template_hashes
                .iter()
                .filter_map(|hash| {
                    self.entries
                        .get(hash)
                        .map(|entry| entry.template_id.clone())
                })
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get templates that need rebuilding after file changes
    pub fn get_rebuild_set(&self, changed_files: &[PathBuf]) -> RebuildSet {
        let mut direct_changes = HashSet::new();
        let mut dependency_changes = HashSet::new();
        
        for file in changed_files {
            // Check if this is a direct template file
            for entry in self.entries.iter() {
                if entry.template_id.file == *file {
                    direct_changes.insert(entry.template_id.clone());
                }
            }
            
            // Check for dependency changes
            let affected = self.get_affected_templates(file);
            dependency_changes.extend(affected);
        }
        
        // Remove direct changes from dependency changes to avoid duplicates
        for template in &direct_changes {
            dependency_changes.remove(template);
        }
        
        let total = direct_changes.len() + dependency_changes.len();
        
        if total > 0 {
            let mut stats = self.stats.lock().unwrap();
            stats.incremental_rebuilds += 1;
        }
        
        RebuildSet {
            direct_changes: direct_changes.into_iter().collect(),
            dependency_changes: dependency_changes.into_iter().collect(),
            total_affected: total,
        }
    }

    /// Clear the build cache
    pub fn clear(&self) {
        self.entries.clear();
        self.file_dependents.lock().unwrap().clear();
    }

    /// Get build statistics
    pub fn stats(&self) -> BuildStatistics {
        let stats = self.stats.lock().unwrap();
        let total_entries = self.entries.len();
        let error_count = self.entries.iter().filter(|e| e.has_errors).count();
        
        BuildStatistics {
            total_builds: stats.total_builds,
            cache_hits: stats.cache_hits,
            incremental_rebuilds: stats.incremental_rebuilds,
            full_rebuilds: stats.full_rebuilds,
            average_build_time: if stats.total_builds > 0 {
                stats.total_build_time / stats.total_builds as u32
            } else {
                Duration::from_secs(0)
            },
            total_entries,
            error_entries: error_count,
            cache_hit_rate: if stats.total_builds > 0 {
                (stats.cache_hits as f64 / stats.total_builds as f64) * 100.0
            } else {
                0.0
            },
        }
    }

    /// Prune old entries from the cache
    pub fn prune_old_entries(&self, max_age: Duration) {
        let now = Instant::now();
        self.entries.retain(|_, entry| {
            now.duration_since(entry.last_build) < max_age
        });
    }

    /// Check if a file has been modified since a given time
    fn is_file_modified(&self, _path: &Path, _since: Instant) -> bool {
        // In a real implementation, this would check file system metadata
        // For now, we'll return false (can be implemented with std::fs::metadata)
        false
    }
}

/// Decision on whether a template needs rebuilding
#[derive(Debug, Clone, PartialEq)]
pub enum RebuildDecision {
    Required(RebuildReason),
    NotRequired,
}

/// Reason for rebuilding a template
#[derive(Debug, Clone, PartialEq)]
pub enum RebuildReason {
    /// No cache entry exists
    NoCacheEntry,
    /// Source file changed
    SourceChanged,
    /// A dependency changed
    DependencyChanged(PathBuf),
    /// Previous build had errors
    PreviousError,
}

/// Set of templates that need rebuilding
#[derive(Debug)]
pub struct RebuildSet {
    /// Templates that changed directly
    pub direct_changes: Vec<TemplateId>,
    /// Templates affected by dependency changes
    pub dependency_changes: Vec<TemplateId>,
    /// Total number of affected templates
    pub total_affected: usize,
}

/// Build cache statistics
#[derive(Debug, Clone)]
pub struct BuildStatistics {
    pub total_builds: u64,
    pub cache_hits: u64,
    pub incremental_rebuilds: u64,
    pub full_rebuilds: u64,
    pub average_build_time: Duration,
    pub total_entries: usize,
    pub error_entries: usize,
    pub cache_hit_rate: f64,
}

impl Default for BuildCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_cache_basic() {
        let cache = BuildCache::new();
        let template_id = TemplateId::new(PathBuf::from("test.rs"), 10, 5);
        let source_hash = "hash123".to_string();
        let deps = HashSet::new();
        
        // Record a build
        cache.record_build(
            template_id.clone(),
            source_hash.clone(),
            deps,
            Duration::from_millis(100),
        );
        
        // Should not need rebuild with same hash
        assert_eq!(
            cache.needs_rebuild(&template_id, &source_hash),
            RebuildDecision::NotRequired
        );
        
        // Should need rebuild with different hash
        assert!(matches!(
            cache.needs_rebuild(&template_id, "different_hash"),
            RebuildDecision::Required(RebuildReason::SourceChanged)
        ));
    }

    #[test]
    fn test_dependency_tracking() {
        let cache = BuildCache::new();
        let template_id = TemplateId::new(PathBuf::from("test.rs"), 10, 5);
        let dep_path = PathBuf::from("dep.rs");
        
        let mut deps = HashSet::new();
        deps.insert(dep_path.clone());
        
        cache.record_build(
            template_id.clone(),
            "hash123".to_string(),
            deps,
            Duration::from_millis(100),
        );
        
        // Should find affected templates
        let affected = cache.get_affected_templates(&dep_path);
        assert_eq!(affected.len(), 1);
        assert_eq!(affected[0], template_id);
    }

    #[test]
    fn test_error_handling() {
        let cache = BuildCache::new();
        let template_id = TemplateId::new(PathBuf::from("test.rs"), 10, 5);
        
        // Record an error
        cache.record_error(
            template_id.clone(),
            "Compilation failed".to_string(),
            Duration::from_millis(50),
        );
        
        // Should always rebuild after error
        assert!(matches!(
            cache.needs_rebuild(&template_id, "any_hash"),
            RebuildDecision::Required(RebuildReason::PreviousError)
        ));
    }
}