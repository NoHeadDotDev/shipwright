//! Enhanced caching system integrating template cache, build cache, and performance monitoring

use crate::{
    build_cache::{BuildCache, RebuildDecision, RebuildSet},
    performance_monitor::{PerformanceMonitor, PerformanceReport},
    protocol::{TemplateId, TemplateUpdate},
    template_cache::{TemplateCache, CacheStatistics},
};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Enhanced caching system that coordinates template cache, build cache, and performance monitoring
#[derive(Debug, Clone)]
pub struct EnhancedCache {
    /// Template cache for compiled templates
    template_cache: Arc<TemplateCache>,
    /// Build cache for tracking compilation state
    build_cache: Arc<BuildCache>,
    /// Performance monitor for metrics
    performance_monitor: Arc<PerformanceMonitor>,
}

impl EnhancedCache {
    /// Create a new enhanced cache system
    pub fn new() -> Self {
        Self {
            template_cache: Arc::new(TemplateCache::new()),
            build_cache: Arc::new(BuildCache::new()),
            performance_monitor: Arc::new(PerformanceMonitor::new()),
        }
    }

    /// Create with custom configuration
    pub fn with_config(
        max_age: Duration,
        max_size_bytes: usize,
        max_entries: usize,
    ) -> Self {
        Self {
            template_cache: Arc::new(TemplateCache::with_config(
                max_age,
                max_size_bytes,
                max_entries,
            )),
            build_cache: Arc::new(BuildCache::new()),
            performance_monitor: Arc::new(PerformanceMonitor::new()),
        }
    }

    /// Store a template with full tracking
    pub fn store_template(
        &self,
        update: TemplateUpdate,
        compile_time: Duration,
        source_hash: String,
        dependencies: Vec<PathBuf>,
    ) -> CacheResult {
        let _timer = self.performance_monitor.scoped_timer("store_template");
        
        // Insert into template cache with timing
        let (version, content_changed) = self
            .template_cache
            .insert_with_timing(update.clone(), compile_time.as_micros() as u64);

        // Record build in build cache
        let deps = dependencies.into_iter().collect();
        self.build_cache.record_build(
            update.id.clone(),
            source_hash,
            deps,
            compile_time,
        );

        // Update performance counters
        self.performance_monitor.increment_counter("templates_stored");
        self.performance_monitor.record_timing("template_compilation", compile_time);
        
        if content_changed {
            self.performance_monitor.increment_counter("content_changes");
        } else {
            self.performance_monitor.increment_counter("no_change_updates");
        }

        CacheResult {
            version,
            content_changed,
            cache_hit: false, // This was a store operation
            dependencies_invalidated: 0,
        }
    }

    /// Get a template with performance tracking
    pub fn get_template(&self, id: &TemplateId) -> Option<TemplateUpdate> {
        let _timer = self.performance_monitor.scoped_timer("get_template");
        
        let result = self.template_cache.get(id);
        
        if result.is_some() {
            self.performance_monitor.increment_counter("template_cache_hits");
        } else {
            self.performance_monitor.increment_counter("template_cache_misses");
        }
        
        result
    }

    /// Check if rebuild is needed using build cache
    pub fn needs_rebuild(&self, template_id: &TemplateId, current_hash: &str) -> RebuildDecision {
        let _timer = self.performance_monitor.scoped_timer("needs_rebuild_check");
        
        let decision = self.build_cache.needs_rebuild(template_id, current_hash);
        
        match &decision {
            RebuildDecision::Required(_) => {
                self.performance_monitor.increment_counter("rebuild_required");
            }
            RebuildDecision::NotRequired => {
                self.performance_monitor.increment_counter("rebuild_avoided");
            }
        }
        
        decision
    }

    /// Get all templates affected by file changes
    pub fn get_affected_templates(&self, changed_files: &[PathBuf]) -> RebuildSet {
        let _timer = self.performance_monitor.scoped_timer("get_affected_templates");
        
        let rebuild_set = self.build_cache.get_rebuild_set(changed_files);
        
        self.performance_monitor.add_to_counter(
            "affected_templates",
            rebuild_set.total_affected as u64,
        );
        
        rebuild_set
    }

    /// Invalidate templates and their dependents
    pub fn invalidate_templates(&self, template_ids: &[TemplateId]) -> InvalidationResult {
        let _timer = self.performance_monitor.scoped_timer("invalidate_templates");
        let start = Instant::now();
        
        let mut removed_count = 0;
        let mut dependent_count = 0;
        
        for template_id in template_ids {
            // Get dependents before removal
            let dependents = self.template_cache.get_transitive_dependents(template_id);
            dependent_count += dependents.len();
            
            // Remove from template cache
            if self.template_cache.remove(template_id).is_some() {
                removed_count += 1;
            }
            
            // Remove dependents as well
            for dependent in dependents {
                if self.template_cache.remove(&dependent.id).is_some() {
                    removed_count += 1;
                }
            }
        }
        
        self.performance_monitor.add_to_counter("templates_invalidated", removed_count as u64);
        
        InvalidationResult {
            templates_removed: removed_count,
            dependents_removed: dependent_count,
            duration: start.elapsed(),
        }
    }

    /// Warm the cache with commonly used templates
    pub fn warm_cache(&self, templates: Vec<TemplateUpdate>) -> WarmingResult {
        let _timer = self.performance_monitor.scoped_timer("warm_cache");
        
        let warming_result = self.template_cache.warm_cache(templates);
        
        self.performance_monitor.add_to_counter(
            "templates_warmed",
            warming_result.templates_loaded as u64,
        );
        
        WarmingResult {
            loaded: warming_result.templates_loaded,
            failed: warming_result.templates_failed,
            total_size_bytes: warming_result.total_size_bytes,
            duration: warming_result.duration,
        }
    }

    /// Optimize all caches
    pub fn optimize(&self) -> OptimizationSummary {
        let _timer = self.performance_monitor.scoped_timer("optimize_caches");
        let start = Instant::now();
        
        // Optimize template cache
        let template_result = self.template_cache.optimize();
        
        // Prune build cache
        self.build_cache.prune_old_entries(Duration::from_secs(3600)); // 1 hour
        
        OptimizationSummary {
            templates_removed: template_result.entries_removed,
            space_freed_bytes: template_result.space_freed_bytes,
            total_duration: start.elapsed(),
        }
    }

    /// Get comprehensive cache statistics
    pub fn get_statistics(&self) -> ComprehensiveStats {
        let template_stats = self.template_cache.stats();
        let build_stats = self.build_cache.stats();
        let performance_report = self.performance_monitor.generate_report();
        
        ComprehensiveStats {
            template_cache: template_stats,
            build_cache: build_stats,
            performance: performance_report,
        }
    }

    /// Get detailed performance metrics
    pub fn get_detailed_stats(&self) -> DetailedMetrics {
        let template_details = self.template_cache.detailed_stats();
        let all_timings = self.performance_monitor.all_timing_stats();
        
        DetailedMetrics {
            template_details,
            timing_breakdown: all_timings,
            current_memory_mb: self.performance_monitor.current_memory() / 1_048_576,
            peak_memory_mb: self.performance_monitor.peak_memory() / 1_048_576,
        }
    }

    /// Reset all caches and metrics
    pub fn reset(&self) {
        self.template_cache.clear();
        self.build_cache.clear();
        self.performance_monitor.reset();
        
        self.performance_monitor.increment_counter("cache_resets");
    }

    /// Add dependency tracking between templates
    pub fn add_dependency(&self, template_hash: &str, dependency_hash: &str) {
        self.template_cache.add_dependency(template_hash, dependency_hash);
    }

    /// Record memory usage
    pub fn record_memory_usage(&self, usage_bytes: usize, label: Option<String>) {
        self.performance_monitor.record_memory(usage_bytes, label);
    }

    /// Get access to individual components
    pub fn template_cache(&self) -> &Arc<TemplateCache> {
        &self.template_cache
    }

    pub fn build_cache(&self) -> &Arc<BuildCache> {
        &self.build_cache
    }

    pub fn performance_monitor(&self) -> &Arc<PerformanceMonitor> {
        &self.performance_monitor
    }
}

/// Result of a cache operation
#[derive(Debug)]
pub struct CacheResult {
    pub version: u64,
    pub content_changed: bool,
    pub cache_hit: bool,
    pub dependencies_invalidated: usize,
}

/// Result of template invalidation
#[derive(Debug)]
pub struct InvalidationResult {
    pub templates_removed: usize,
    pub dependents_removed: usize,
    pub duration: Duration,
}

/// Result of cache warming
#[derive(Debug)]
pub struct WarmingResult {
    pub loaded: usize,
    pub failed: usize,
    pub total_size_bytes: usize,
    pub duration: Duration,
}

/// Summary of cache optimization
#[derive(Debug)]
pub struct OptimizationSummary {
    pub templates_removed: usize,
    pub space_freed_bytes: usize,
    pub total_duration: Duration,
}

/// Comprehensive statistics from all cache components
#[derive(Debug)]
pub struct ComprehensiveStats {
    pub template_cache: CacheStatistics,
    pub build_cache: crate::build_cache::BuildStatistics,
    pub performance: PerformanceReport,
}

/// Detailed metrics for debugging and optimization
#[derive(Debug)]
pub struct DetailedMetrics {
    pub template_details: crate::template_cache::DetailedCacheStats,
    pub timing_breakdown: HashMap<String, crate::performance_monitor::TimingStats>,
    pub current_memory_mb: usize,
    pub peak_memory_mb: usize,
}

impl Default for EnhancedCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_enhanced_cache_operations() {
        let cache = EnhancedCache::new();
        let template_id = TemplateId::new(PathBuf::from("test.rs"), 10, 5);
        let html = "<div>Test</div>".to_string();
        let update = TemplateUpdate {
            id: template_id.clone(),
            hash: template_id.hash(),
            content_hash: TemplateUpdate::compute_content_hash(&html, &[]),
            html,
            dynamic_parts: vec![],
        };

        // Store template
        let result = cache.store_template(
            update.clone(),
            Duration::from_millis(100),
            "source_hash123".to_string(),
            vec![PathBuf::from("dep.rs")],
        );
        
        assert_eq!(result.version, 1);
        assert!(result.content_changed);

        // Get template
        let retrieved = cache.get_template(&template_id);
        assert!(retrieved.is_some());

        // Check rebuild not needed
        let rebuild = cache.needs_rebuild(&template_id, "source_hash123");
        assert_eq!(rebuild, RebuildDecision::NotRequired);

        // Get statistics
        let stats = cache.get_statistics();
        assert!(stats.template_cache.total_hits > 0);
    }

    #[test]
    fn test_invalidation() {
        let cache = EnhancedCache::new();
        let template_id = TemplateId::new(PathBuf::from("test.rs"), 10, 5);
        let html = "<div>Test</div>".to_string();
        let update = TemplateUpdate {
            id: template_id.clone(),
            hash: template_id.hash(),
            content_hash: TemplateUpdate::compute_content_hash(&html, &[]),
            html,
            dynamic_parts: vec![],
        };

        // Store template
        cache.store_template(
            update,
            Duration::from_millis(100),
            "source_hash123".to_string(),
            vec![],
        );

        // Invalidate
        let result = cache.invalidate_templates(&[template_id.clone()]);
        assert_eq!(result.templates_removed, 1);

        // Should not be found after invalidation
        assert!(cache.get_template(&template_id).is_none());
    }
}