//! Benchmarking tools for hot reload performance testing

use crate::{
    enhanced_cache::EnhancedCache,
    protocol::{TemplateId, TemplateUpdate},
    template_cache::TemplateCache,
};
use std::path::PathBuf;
use std::time::{Duration, Instant};

/// Benchmark suite for cache performance
pub struct CacheBenchmarks {
    cache: EnhancedCache,
}

impl CacheBenchmarks {
    /// Create a new benchmark suite
    pub fn new() -> Self {
        Self {
            cache: EnhancedCache::new(),
        }
    }

    /// Run all benchmarks
    pub fn run_all_benchmarks(&mut self) -> BenchmarkResults {
        let mut results = BenchmarkResults::default();
        
        results.store_performance = self.benchmark_store_operations();
        results.get_performance = self.benchmark_get_operations();
        results.dependency_tracking = self.benchmark_dependency_tracking();
        results.cache_invalidation = self.benchmark_cache_invalidation();
        results.memory_usage = self.benchmark_memory_usage();
        results.concurrency = self.benchmark_concurrency();
        
        results
    }

    /// Benchmark template store operations
    fn benchmark_store_operations(&mut self) -> BenchmarkResult {
        let iterations = 1000;
        let start = Instant::now();
        
        for i in 0..iterations {
            let template_id = TemplateId::new(
                PathBuf::from(format!("test_{}.rs", i)),
                10,
                5,
            );
            let html = format!("<div>Template {}</div>", i);
            let update = TemplateUpdate {
                id: template_id.clone(),
                hash: template_id.hash(),
                content_hash: TemplateUpdate::compute_content_hash(&html, &[]),
                html,
                dynamic_parts: vec![],
            };
            
            self.cache.store_template(
                update,
                Duration::from_micros(50), // Simulated compile time
                format!("hash_{}", i),
                vec![],
            );
        }
        
        let total_time = start.elapsed();
        
        BenchmarkResult {
            name: "Store Operations".to_string(),
            iterations,
            total_time,
            avg_time: total_time / iterations,
            throughput: (iterations as f64 / total_time.as_secs_f64()) as u64,
        }
    }

    /// Benchmark template get operations
    fn benchmark_get_operations(&mut self) -> BenchmarkResult {
        // First populate cache
        let num_templates = 100;
        let mut template_ids = Vec::new();
        
        for i in 0..num_templates {
            let template_id = TemplateId::new(
                PathBuf::from(format!("get_test_{}.rs", i)),
                10,
                5,
            );
            let html = format!("<div>Get Test {}</div>", i);
            let update = TemplateUpdate {
                id: template_id.clone(),
                hash: template_id.hash(),
                content_hash: TemplateUpdate::compute_content_hash(&html, &[]),
                html,
                dynamic_parts: vec![],
            };
            
            self.cache.store_template(
                update,
                Duration::from_micros(50),
                format!("get_hash_{}", i),
                vec![],
            );
            template_ids.push(template_id);
        }
        
        // Benchmark get operations
        let iterations = 10000;
        let start = Instant::now();
        
        for i in 0..iterations {
            let template_id = &template_ids[i % num_templates];
            let _ = self.cache.get_template(template_id);
        }
        
        let total_time = start.elapsed();
        
        BenchmarkResult {
            name: "Get Operations".to_string(),
            iterations,
            total_time,
            avg_time: total_time / iterations,
            throughput: (iterations as f64 / total_time.as_secs_f64()) as u64,
        }
    }

    /// Benchmark dependency tracking
    fn benchmark_dependency_tracking(&mut self) -> BenchmarkResult {
        let num_templates = 100;
        let dependencies_per_template = 5;
        
        // Create template hierarchy
        for i in 0..num_templates {
            let template_id = TemplateId::new(
                PathBuf::from(format!("dep_test_{}.rs", i)),
                10,
                5,
            );
            let hash = template_id.hash();
            
            // Add dependencies
            for j in 0..dependencies_per_template {
                let dep_id = TemplateId::new(
                    PathBuf::from(format!("dep_{}.rs", j)),
                    10,
                    5,
                );
                let dep_hash = dep_id.hash();
                self.cache.add_dependency(&hash, &dep_hash);
            }
        }
        
        // Benchmark dependency resolution
        let iterations = 1000;
        let start = Instant::now();
        
        for i in 0..iterations {
            let file = PathBuf::from(format!("dep_{}.rs", i % dependencies_per_template));
            let _ = self.cache.get_affected_templates(&[file]);
        }
        
        let total_time = start.elapsed();
        
        BenchmarkResult {
            name: "Dependency Tracking".to_string(),
            iterations,
            total_time,
            avg_time: total_time / iterations,
            throughput: (iterations as f64 / total_time.as_secs_f64()) as u64,
        }
    }

    /// Benchmark cache invalidation
    fn benchmark_cache_invalidation(&mut self) -> BenchmarkResult {
        // Populate cache
        let num_templates = 500;
        let mut template_ids = Vec::new();
        
        for i in 0..num_templates {
            let template_id = TemplateId::new(
                PathBuf::from(format!("inv_test_{}.rs", i)),
                10,
                5,
            );
            let html = format!("<div>Invalidation Test {}</div>", i);
            let update = TemplateUpdate {
                id: template_id.clone(),
                hash: template_id.hash(),
                content_hash: TemplateUpdate::compute_content_hash(&html, &[]),
                html,
                dynamic_parts: vec![],
            };
            
            self.cache.store_template(
                update,
                Duration::from_micros(50),
                format!("inv_hash_{}", i),
                vec![],
            );
            template_ids.push(template_id);
        }
        
        // Benchmark invalidation
        let iterations = 100;
        let batch_size = 10;
        let start = Instant::now();
        
        for i in 0..iterations {
            let start_idx = (i * batch_size) % num_templates;
            let end_idx = (start_idx + batch_size).min(num_templates);
            let batch = &template_ids[start_idx..end_idx];
            
            let _ = self.cache.invalidate_templates(batch);
        }
        
        let total_time = start.elapsed();
        
        BenchmarkResult {
            name: "Cache Invalidation".to_string(),
            iterations,
            total_time,
            avg_time: total_time / iterations,
            throughput: (iterations as f64 / total_time.as_secs_f64()) as u64,
        }
    }

    /// Benchmark memory usage patterns
    fn benchmark_memory_usage(&mut self) -> BenchmarkResult {
        let iterations = 1000;
        let start = Instant::now();
        let mut memory_samples = Vec::new();
        
        for i in 0..iterations {
            // Simulate memory usage recording
            let simulated_memory = 1024 * 1024 * (i + 1); // 1MB per iteration
            self.cache.record_memory_usage(simulated_memory, Some(format!("iteration_{}", i)));
            
            if i % 100 == 0 {
                memory_samples.push(simulated_memory);
            }
        }
        
        let total_time = start.elapsed();
        
        BenchmarkResult {
            name: "Memory Usage Tracking".to_string(),
            iterations,
            total_time,
            avg_time: total_time / iterations,
            throughput: (iterations as f64 / total_time.as_secs_f64()) as u64,
        }
    }

    /// Benchmark concurrent operations
    fn benchmark_concurrency(&mut self) -> BenchmarkResult {
        use std::thread;
        use std::sync::Arc;
        
        let cache = Arc::new(EnhancedCache::new());
        let iterations_per_thread = 100;
        let num_threads = 4;
        let total_iterations = iterations_per_thread * num_threads;
        
        let start = Instant::now();
        
        let handles: Vec<_> = (0..num_threads)
            .map(|thread_id| {
                let cache_clone = Arc::clone(&cache);
                thread::spawn(move || {
                    for i in 0..iterations_per_thread {
                        let template_id = TemplateId::new(
                            PathBuf::from(format!("concurrent_{}_{}.rs", thread_id, i)),
                            10,
                            5,
                        );
                        let html = format!("<div>Concurrent {} {}</div>", thread_id, i);
                        let update = TemplateUpdate {
                            id: template_id.clone(),
                            hash: template_id.hash(),
                            content_hash: TemplateUpdate::compute_content_hash(&html, &[]),
                            html,
                            dynamic_parts: vec![],
                        };
                        
                        cache_clone.store_template(
                            update,
                            Duration::from_micros(50),
                            format!("concurrent_hash_{}_{}", thread_id, i),
                            vec![],
                        );
                        
                        // Also do some gets
                        let _ = cache_clone.get_template(&template_id);
                    }
                })
            })
            .collect();
        
        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }
        
        let total_time = start.elapsed();
        
        BenchmarkResult {
            name: "Concurrent Operations".to_string(),
            iterations: total_iterations,
            total_time,
            avg_time: total_time / total_iterations as u32,
            throughput: (total_iterations as f64 / total_time.as_secs_f64()) as u64,
        }
    }
}

/// Result of a single benchmark
#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    pub name: String,
    pub iterations: usize,
    pub total_time: Duration,
    pub avg_time: Duration,
    pub throughput: u64, // operations per second
}

impl std::fmt::Display for BenchmarkResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}: {} iterations in {:?} (avg: {:?}, {} ops/sec)",
            self.name, self.iterations, self.total_time, self.avg_time, self.throughput
        )
    }
}

/// Complete benchmark results
#[derive(Debug, Default)]
pub struct BenchmarkResults {
    pub store_performance: BenchmarkResult,
    pub get_performance: BenchmarkResult,
    pub dependency_tracking: BenchmarkResult,
    pub cache_invalidation: BenchmarkResult,
    pub memory_usage: BenchmarkResult,
    pub concurrency: BenchmarkResult,
}

impl std::fmt::Display for BenchmarkResults {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "=== Cache Performance Benchmarks ===")?;
        writeln!(f)?;
        writeln!(f, "{}", self.store_performance)?;
        writeln!(f, "{}", self.get_performance)?;
        writeln!(f, "{}", self.dependency_tracking)?;
        writeln!(f, "{}", self.cache_invalidation)?;
        writeln!(f, "{}", self.memory_usage)?;
        writeln!(f, "{}", self.concurrency)?;
        Ok(())
    }
}

/// Load testing utilities
pub struct LoadTester {
    cache: EnhancedCache,
}

impl LoadTester {
    /// Create a new load tester
    pub fn new() -> Self {
        Self {
            cache: EnhancedCache::new(),
        }
    }

    /// Run a sustained load test
    pub fn sustained_load_test(&mut self, duration: Duration, ops_per_second: u64) -> LoadTestResult {
        let start = Instant::now();
        let mut operations_completed = 0;
        let mut errors = 0;
        
        let interval = Duration::from_nanos(1_000_000_000 / ops_per_second);
        let mut next_operation = start;
        
        while start.elapsed() < duration {
            if Instant::now() >= next_operation {
                match self.perform_mixed_operation(operations_completed) {
                    Ok(_) => operations_completed += 1,
                    Err(_) => errors += 1,
                }
                next_operation += interval;
            }
            
            // Small sleep to prevent busy waiting
            std::thread::sleep(Duration::from_millis(1));
        }
        
        let actual_duration = start.elapsed();
        let actual_ops_per_second = operations_completed as f64 / actual_duration.as_secs_f64();
        
        LoadTestResult {
            duration: actual_duration,
            operations_completed,
            errors,
            actual_ops_per_second,
            target_ops_per_second: ops_per_second,
        }
    }

    /// Perform a mixed operation (store, get, or invalidate)
    fn perform_mixed_operation(&mut self, operation_id: usize) -> Result<(), String> {
        match operation_id % 10 {
            0..=6 => {
                // 70% store operations
                let template_id = TemplateId::new(
                    PathBuf::from(format!("load_test_{}.rs", operation_id)),
                    10,
                    5,
                );
                let html = format!("<div>Load Test {}</div>", operation_id);
                let update = TemplateUpdate {
                    id: template_id,
                    hash: format!("hash_{}", operation_id),
                    content_hash: TemplateUpdate::compute_content_hash(&html, &[]),
                    html,
                    dynamic_parts: vec![],
                };
                
                self.cache.store_template(
                    update,
                    Duration::from_micros(50),
                    format!("source_hash_{}", operation_id),
                    vec![],
                );
                Ok(())
            }
            7..=8 => {
                // 20% get operations
                let template_id = TemplateId::new(
                    PathBuf::from(format!("load_test_{}.rs", operation_id % 100)),
                    10,
                    5,
                );
                let _ = self.cache.get_template(&template_id);
                Ok(())
            }
            9 => {
                // 10% invalidation operations
                if operation_id > 10 {
                    let template_id = TemplateId::new(
                        PathBuf::from(format!("load_test_{}.rs", operation_id - 10)),
                        10,
                        5,
                    );
                    self.cache.invalidate_templates(&[template_id]);
                }
                Ok(())
            }
            _ => unreachable!(),
        }
    }
}

/// Result of a load test
#[derive(Debug)]
pub struct LoadTestResult {
    pub duration: Duration,
    pub operations_completed: usize,
    pub errors: usize,
    pub actual_ops_per_second: f64,
    pub target_ops_per_second: u64,
}

impl std::fmt::Display for LoadTestResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Load Test: {} ops in {:?} ({:.1} ops/sec, target: {} ops/sec, {} errors)",
            self.operations_completed,
            self.duration,
            self.actual_ops_per_second,
            self.target_ops_per_second,
            self.errors
        )
    }
}

impl Default for CacheBenchmarks {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for BenchmarkResult {
    fn default() -> Self {
        Self {
            name: String::new(),
            iterations: 0,
            total_time: Duration::from_secs(0),
            avg_time: Duration::from_secs(0),
            throughput: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_benchmark_runs() {
        let mut benchmarks = CacheBenchmarks::new();
        let results = benchmarks.run_all_benchmarks();
        
        // Basic sanity checks
        assert!(results.store_performance.iterations > 0);
        assert!(results.get_performance.iterations > 0);
        assert!(results.dependency_tracking.iterations > 0);
        assert!(results.cache_invalidation.iterations > 0);
        assert!(results.memory_usage.iterations > 0);
        assert!(results.concurrency.iterations > 0);
    }

    #[test]
    fn test_load_tester() {
        let mut tester = LoadTester::new();
        let result = tester.sustained_load_test(Duration::from_millis(100), 100);
        
        assert!(result.operations_completed > 0);
        assert!(result.actual_ops_per_second > 0.0);
    }
}