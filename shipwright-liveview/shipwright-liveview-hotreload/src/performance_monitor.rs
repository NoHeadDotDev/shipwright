//! Performance monitoring and profiling for hot reload system

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::fmt;

/// Performance monitor for tracking hot reload metrics
#[derive(Debug, Clone)]
pub struct PerformanceMonitor {
    /// Timing measurements
    timings: Arc<Mutex<TimingTracker>>,
    /// Memory measurements
    memory: Arc<Mutex<MemoryTracker>>,
    /// Event counters
    counters: Arc<Mutex<HashMap<String, u64>>>,
    /// Active spans for tracing
    active_spans: Arc<Mutex<HashMap<String, Instant>>>,
}

/// Tracks timing information for different operations
#[derive(Debug, Default)]
struct TimingTracker {
    /// Timing samples by operation name
    samples: HashMap<String, Vec<Duration>>,
    /// Total time by operation
    totals: HashMap<String, Duration>,
}

/// Tracks memory usage information
#[derive(Debug, Default)]
struct MemoryTracker {
    /// Peak memory usage in bytes
    peak_usage: usize,
    /// Current memory usage in bytes
    current_usage: usize,
    /// Memory samples over time
    samples: Vec<MemorySample>,
}

/// A memory usage sample
#[derive(Debug, Clone)]
struct MemorySample {
    timestamp: Instant,
    usage_bytes: usize,
    /// Optional label for what was happening
    label: Option<String>,
}

impl PerformanceMonitor {
    /// Create a new performance monitor
    pub fn new() -> Self {
        Self {
            timings: Arc::new(Mutex::new(TimingTracker::default())),
            memory: Arc::new(Mutex::new(MemoryTracker::default())),
            counters: Arc::new(Mutex::new(HashMap::new())),
            active_spans: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Start timing an operation
    pub fn start_timing(&self, operation: &str) -> TimingGuard {
        let start = Instant::now();
        self.active_spans
            .lock()
            .unwrap()
            .insert(operation.to_string(), start);
        
        TimingGuard {
            monitor: self.clone(),
            operation: operation.to_string(),
            start,
        }
    }

    /// Record a timing measurement
    pub fn record_timing(&self, operation: &str, duration: Duration) {
        let mut timings = self.timings.lock().unwrap();
        
        timings
            .samples
            .entry(operation.to_string())
            .or_default()
            .push(duration);
        
        *timings
            .totals
            .entry(operation.to_string())
            .or_default() += duration;
    }

    /// End timing for an operation
    fn end_timing(&self, operation: &str, start: Instant) {
        let duration = start.elapsed();
        self.active_spans.lock().unwrap().remove(operation);
        self.record_timing(operation, duration);
    }

    /// Increment a counter
    pub fn increment_counter(&self, name: &str) {
        self.add_to_counter(name, 1);
    }

    /// Add to a counter
    pub fn add_to_counter(&self, name: &str, value: u64) {
        let mut counters = self.counters.lock().unwrap();
        *counters.entry(name.to_string()).or_default() += value;
    }

    /// Get counter value
    pub fn get_counter(&self, name: &str) -> u64 {
        self.counters
            .lock()
            .unwrap()
            .get(name)
            .copied()
            .unwrap_or(0)
    }

    /// Record current memory usage
    pub fn record_memory(&self, usage_bytes: usize, label: Option<String>) {
        let mut memory = self.memory.lock().unwrap();
        
        memory.current_usage = usage_bytes;
        memory.peak_usage = memory.peak_usage.max(usage_bytes);
        
        memory.samples.push(MemorySample {
            timestamp: Instant::now(),
            usage_bytes,
            label,
        });
        
        // Keep only last 1000 samples to prevent unbounded growth
        if memory.samples.len() > 1000 {
            memory.samples.remove(0);
        }
    }

    /// Get current memory usage
    pub fn current_memory(&self) -> usize {
        self.memory.lock().unwrap().current_usage
    }

    /// Get peak memory usage
    pub fn peak_memory(&self) -> usize {
        self.memory.lock().unwrap().peak_usage
    }

    /// Get timing statistics for an operation
    pub fn timing_stats(&self, operation: &str) -> Option<TimingStats> {
        let timings = self.timings.lock().unwrap();
        
        if let Some(samples) = timings.samples.get(operation) {
            if samples.is_empty() {
                return None;
            }
            
            let total = timings.totals.get(operation).copied().unwrap_or_default();
            let count = samples.len();
            let avg = total / count as u32;
            
            let mut sorted = samples.clone();
            sorted.sort();
            
            let min = sorted.first().copied().unwrap_or_default();
            let max = sorted.last().copied().unwrap_or_default();
            let median = sorted[count / 2];
            
            let p95_idx = (count as f64 * 0.95) as usize;
            let p95 = sorted.get(p95_idx).copied().unwrap_or(max);
            
            let p99_idx = (count as f64 * 0.99) as usize;
            let p99 = sorted.get(p99_idx).copied().unwrap_or(max);
            
            Some(TimingStats {
                count,
                total,
                average: avg,
                min,
                max,
                median,
                p95,
                p99,
            })
        } else {
            None
        }
    }

    /// Get all timing statistics
    pub fn all_timing_stats(&self) -> HashMap<String, TimingStats> {
        let timings = self.timings.lock().unwrap();
        let mut stats = HashMap::new();
        
        for operation in timings.samples.keys() {
            if let Some(stat) = self.timing_stats(operation) {
                stats.insert(operation.clone(), stat);
            }
        }
        
        stats
    }

    /// Generate a performance report
    pub fn generate_report(&self) -> PerformanceReport {
        let all_timings = self.all_timing_stats();
        let counters = self.counters.lock().unwrap().clone();
        let memory = self.memory.lock().unwrap();
        
        PerformanceReport {
            timings: all_timings,
            counters,
            peak_memory_bytes: memory.peak_usage,
            current_memory_bytes: memory.current_usage,
            timestamp: Instant::now(),
        }
    }

    /// Reset all metrics
    pub fn reset(&self) {
        self.timings.lock().unwrap().samples.clear();
        self.timings.lock().unwrap().totals.clear();
        self.memory.lock().unwrap().samples.clear();
        self.counters.lock().unwrap().clear();
        self.active_spans.lock().unwrap().clear();
    }

    /// Create a scoped timer
    pub fn scoped_timer(&self, operation: &str) -> ScopedTimer {
        ScopedTimer::new(self.clone(), operation)
    }
}

/// Guard for timing operations
pub struct TimingGuard {
    monitor: PerformanceMonitor,
    operation: String,
    start: Instant,
}

impl Drop for TimingGuard {
    fn drop(&mut self) {
        self.monitor.end_timing(&self.operation, self.start);
    }
}

/// Scoped timer for measuring operation duration
pub struct ScopedTimer {
    monitor: PerformanceMonitor,
    operation: String,
    start: Instant,
}

impl ScopedTimer {
    fn new(monitor: PerformanceMonitor, operation: &str) -> Self {
        Self {
            monitor,
            operation: operation.to_string(),
            start: Instant::now(),
        }
    }

    /// Get elapsed time without ending the timer
    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }
}

impl Drop for ScopedTimer {
    fn drop(&mut self) {
        let duration = self.start.elapsed();
        self.monitor.record_timing(&self.operation, duration);
    }
}

/// Statistics for a timed operation
#[derive(Debug, Clone)]
pub struct TimingStats {
    pub count: usize,
    pub total: Duration,
    pub average: Duration,
    pub min: Duration,
    pub max: Duration,
    pub median: Duration,
    pub p95: Duration,
    pub p99: Duration,
}

impl fmt::Display for TimingStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "count: {}, avg: {:?}, min: {:?}, max: {:?}, p50: {:?}, p95: {:?}, p99: {:?}",
            self.count, self.average, self.min, self.max, self.median, self.p95, self.p99
        )
    }
}

/// Complete performance report
#[derive(Debug)]
pub struct PerformanceReport {
    pub timings: HashMap<String, TimingStats>,
    pub counters: HashMap<String, u64>,
    pub peak_memory_bytes: usize,
    pub current_memory_bytes: usize,
    pub timestamp: Instant,
}

impl fmt::Display for PerformanceReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "=== Performance Report ===")?;
        writeln!(f)?;
        
        writeln!(f, "Timings:")?;
        let mut timings: Vec<_> = self.timings.iter().collect();
        timings.sort_by_key(|(name, _)| name.as_str());
        
        for (name, stats) in timings {
            writeln!(f, "  {}: {}", name, stats)?;
        }
        
        writeln!(f)?;
        writeln!(f, "Counters:")?;
        let mut counters: Vec<_> = self.counters.iter().collect();
        counters.sort_by_key(|(name, _)| name.as_str());
        
        for (name, value) in counters {
            writeln!(f, "  {}: {}", name, value)?;
        }
        
        writeln!(f)?;
        writeln!(f, "Memory:")?;
        writeln!(f, "  Peak: {} MB", self.peak_memory_bytes / 1_048_576)?;
        writeln!(f, "  Current: {} MB", self.current_memory_bytes / 1_048_576)?;
        
        Ok(())
    }
}

/// Memory profiler for detecting leaks
pub struct MemoryProfiler {
    baseline: usize,
    snapshots: Vec<MemorySnapshot>,
}

/// A memory snapshot
#[derive(Debug, Clone)]
pub struct MemorySnapshot {
    pub label: String,
    pub usage_bytes: usize,
    pub delta_bytes: isize,
    pub timestamp: Instant,
}

impl MemoryProfiler {
    /// Create a new memory profiler
    pub fn new() -> Self {
        Self {
            baseline: Self::current_memory_usage(),
            snapshots: Vec::new(),
        }
    }

    /// Take a memory snapshot
    pub fn snapshot(&mut self, label: &str) -> MemorySnapshot {
        let current = Self::current_memory_usage();
        let delta = current as isize - self.baseline as isize;
        
        let snapshot = MemorySnapshot {
            label: label.to_string(),
            usage_bytes: current,
            delta_bytes: delta,
            timestamp: Instant::now(),
        };
        
        self.snapshots.push(snapshot.clone());
        snapshot
    }

    /// Reset baseline to current memory usage
    pub fn reset_baseline(&mut self) {
        self.baseline = Self::current_memory_usage();
    }

    /// Get current memory usage (placeholder - would use actual memory APIs)
    fn current_memory_usage() -> usize {
        // In a real implementation, this would use platform-specific APIs
        // or a crate like `memory-stats` to get actual memory usage
        0
    }

    /// Check for potential memory leaks
    pub fn check_for_leaks(&self, threshold_mb: f64) -> Vec<PotentialLeak> {
        let mut leaks = Vec::new();
        let threshold_bytes = (threshold_mb * 1_048_576.0) as usize;
        
        // Look for continuous growth patterns
        if self.snapshots.len() >= 3 {
            let mut growing_count = 0;
            let mut total_growth = 0isize;
            
            for window in self.snapshots.windows(2) {
                let growth = window[1].usage_bytes as isize - window[0].usage_bytes as isize;
                if growth > 0 {
                    growing_count += 1;
                    total_growth += growth;
                }
            }
            
            if growing_count >= self.snapshots.len() - 1 && total_growth as usize > threshold_bytes {
                leaks.push(PotentialLeak {
                    description: "Continuous memory growth detected".to_string(),
                    growth_bytes: total_growth as usize,
                    start_snapshot: self.snapshots.first().cloned(),
                    end_snapshot: self.snapshots.last().cloned(),
                });
            }
        }
        
        leaks
    }
}

/// Potential memory leak detection
#[derive(Debug)]
pub struct PotentialLeak {
    pub description: String,
    pub growth_bytes: usize,
    pub start_snapshot: Option<MemorySnapshot>,
    pub end_snapshot: Option<MemorySnapshot>,
}

impl Default for PerformanceMonitor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_timing_measurements() {
        let monitor = PerformanceMonitor::new();
        
        // Record some timings
        monitor.record_timing("test_op", Duration::from_millis(10));
        monitor.record_timing("test_op", Duration::from_millis(20));
        monitor.record_timing("test_op", Duration::from_millis(15));
        
        let stats = monitor.timing_stats("test_op").unwrap();
        assert_eq!(stats.count, 3);
        assert_eq!(stats.average, Duration::from_millis(15));
        assert_eq!(stats.min, Duration::from_millis(10));
        assert_eq!(stats.max, Duration::from_millis(20));
    }

    #[test]
    fn test_timing_guard() {
        let monitor = PerformanceMonitor::new();
        
        {
            let _guard = monitor.start_timing("guarded_op");
            thread::sleep(Duration::from_millis(10));
        }
        
        let stats = monitor.timing_stats("guarded_op");
        assert!(stats.is_some());
        assert!(stats.unwrap().count == 1);
    }

    #[test]
    fn test_counters() {
        let monitor = PerformanceMonitor::new();
        
        monitor.increment_counter("test_counter");
        monitor.increment_counter("test_counter");
        monitor.add_to_counter("test_counter", 5);
        
        assert_eq!(monitor.get_counter("test_counter"), 7);
        assert_eq!(monitor.get_counter("non_existent"), 0);
    }

    #[test]
    fn test_memory_tracking() {
        let monitor = PerformanceMonitor::new();
        
        monitor.record_memory(1_000_000, Some("initial".to_string()));
        monitor.record_memory(2_000_000, Some("peak".to_string()));
        monitor.record_memory(1_500_000, Some("current".to_string()));
        
        assert_eq!(monitor.current_memory(), 1_500_000);
        assert_eq!(monitor.peak_memory(), 2_000_000);
    }
}