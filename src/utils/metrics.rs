use std::time::{Duration, Instant};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub total_duration: Duration,
    pub files_processed: usize,
    pub lines_processed: usize,
    pub bytes_processed: u64,
    pub cache_hits: usize,
    pub cache_misses: usize,
    pub parallel_workers: usize,
    pub memory_usage_mb: f64,
    pub phase_timings: HashMap<String, Duration>,
}

impl PerformanceMetrics {
    pub fn new() -> Self {
        Self {
            total_duration: Duration::new(0, 0),
            files_processed: 0,
            lines_processed: 0,
            bytes_processed: 0,
            cache_hits: 0,
            cache_misses: 0,
            parallel_workers: rayon::current_num_threads(),
            memory_usage_mb: 0.0,
            phase_timings: HashMap::new(),
        }
    }
    
    pub fn files_per_second(&self) -> f64 {
        if self.total_duration.as_secs_f64() > 0.0 {
            self.files_processed as f64 / self.total_duration.as_secs_f64()
        } else {
            0.0
        }
    }
    
    pub fn lines_per_second(&self) -> f64 {
        if self.total_duration.as_secs_f64() > 0.0 {
            self.lines_processed as f64 / self.total_duration.as_secs_f64()
        } else {
            0.0
        }
    }
    
    pub fn bytes_per_second(&self) -> f64 {
        if self.total_duration.as_secs_f64() > 0.0 {
            self.bytes_processed as f64 / self.total_duration.as_secs_f64()
        } else {
            0.0
        }
    }
    
    pub fn cache_hit_rate(&self) -> f64 {
        let total_requests = self.cache_hits + self.cache_misses;
        if total_requests > 0 {
            self.cache_hits as f64 / total_requests as f64
        } else {
            0.0
        }
    }
    
    pub fn add_phase_timing(&mut self, phase: &str, duration: Duration) {
        self.phase_timings.insert(phase.to_string(), duration);
    }
    
    pub fn print_summary(&self) {
        println!("\n📊 Performance Summary");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("⏱️  Total time: {:.2}s", self.total_duration.as_secs_f64());
        println!("📁 Files processed: {}", self.files_processed);
        println!("📏 Lines processed: {}", self.lines_processed);
        println!("💾 Bytes processed: {:.2} MB", self.bytes_processed as f64 / (1024.0 * 1024.0));
        println!("🚀 Throughput:");
        println!("   • {:.0} files/sec", self.files_per_second());
        println!("   • {:.0} lines/sec", self.lines_per_second());
        println!("   • {:.2} MB/sec", self.bytes_per_second() / (1024.0 * 1024.0));
        
        if self.cache_hits + self.cache_misses > 0 {
            println!("💾 Cache performance:");
            println!("   • Hit rate: {:.1}%", self.cache_hit_rate() * 100.0);
            println!("   • Hits: {}", self.cache_hits);
            println!("   • Misses: {}", self.cache_misses);
        }
        
        println!("🔧 System:");
        println!("   • Parallel workers: {}", self.parallel_workers);
        println!("   • Memory usage: {:.1} MB", self.memory_usage_mb);
        
        if !self.phase_timings.is_empty() {
            println!("⏱️  Phase timings:");
            let mut phases: Vec<_> = self.phase_timings.iter().collect();
            phases.sort_by_key(|(_, duration)| *duration);
            phases.reverse();
            
            for (phase, duration) in phases {
                let percentage = (duration.as_secs_f64() / self.total_duration.as_secs_f64()) * 100.0;
                println!("   • {}: {:.2}s ({:.1}%)", phase, duration.as_secs_f64(), percentage);
            }
        }
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    }
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Timer {
    start: Instant,
    name: String,
}

impl Timer {
    pub fn new(name: &str) -> Self {
        Self {
            start: Instant::now(),
            name: name.to_string(),
        }
    }
    
    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }
    
    pub fn finish(self) -> (String, Duration) {
        let elapsed = self.elapsed();
        (self.name, elapsed)
    }
}

pub struct MetricsCollector {
    metrics: PerformanceMetrics,
    start_time: Instant,
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            metrics: PerformanceMetrics::new(),
            start_time: Instant::now(),
        }
    }
    
    pub fn record_file_processed(&mut self, lines: usize, bytes: u64) {
        self.metrics.files_processed += 1;
        self.metrics.lines_processed += lines;
        self.metrics.bytes_processed += bytes;
    }
    
    pub fn record_cache_hit(&mut self) {
        self.metrics.cache_hits += 1;
    }
    
    pub fn record_cache_miss(&mut self) {
        self.metrics.cache_misses += 1;
    }
    
    pub fn add_phase_timing(&mut self, phase: &str, duration: Duration) {
        self.metrics.add_phase_timing(phase, duration);
    }
    
    pub fn finish(mut self) -> PerformanceMetrics {
        self.metrics.total_duration = self.start_time.elapsed();
        
        // Estimate memory usage (rough approximation)
        self.metrics.memory_usage_mb = estimate_memory_usage(&self.metrics);
        
        self.metrics
    }
    
    pub fn create_timer(&self, name: &str) -> Timer {
        Timer::new(name)
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

fn estimate_memory_usage(metrics: &PerformanceMetrics) -> f64 {
    // Rough estimation based on processed data
    // This is a simplified calculation
    let base_memory = 10.0; // Base application memory in MB
    let file_overhead = metrics.files_processed as f64 * 0.001; // ~1KB per file
    let data_overhead = metrics.bytes_processed as f64 / (1024.0 * 1024.0 * 10.0); // 10% of data size
    
    base_memory + file_overhead + data_overhead
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    
    #[test]
    fn test_metrics_creation() {
        let metrics = PerformanceMetrics::new();
        assert_eq!(metrics.files_processed, 0);
        assert_eq!(metrics.lines_processed, 0);
        assert_eq!(metrics.bytes_processed, 0);
    }
    
    #[test]
    fn test_metrics_calculations() {
        let mut metrics = PerformanceMetrics::new();
        metrics.total_duration = Duration::from_secs(2);
        metrics.files_processed = 100;
        metrics.lines_processed = 10000;
        metrics.bytes_processed = 1024 * 1024; // 1MB
        
        assert_eq!(metrics.files_per_second(), 50.0);
        assert_eq!(metrics.lines_per_second(), 5000.0);
        assert_eq!(metrics.bytes_per_second(), 524288.0);
    }
    
    #[test]
    fn test_cache_hit_rate() {
        let mut metrics = PerformanceMetrics::new();
        metrics.cache_hits = 80;
        metrics.cache_misses = 20;
        
        assert_eq!(metrics.cache_hit_rate(), 0.8);
    }
    
    #[test]
    fn test_timer() {
        let timer = Timer::new("test");
        thread::sleep(Duration::from_millis(10));
        let (name, duration) = timer.finish();
        
        assert_eq!(name, "test");
        assert!(duration.as_millis() >= 10);
    }
    
    #[test]
    fn test_metrics_collector() {
        let mut collector = MetricsCollector::new();
        
        collector.record_file_processed(100, 1024);
        collector.record_cache_hit();
        collector.record_cache_miss();
        
        let metrics = collector.finish();
        
        assert_eq!(metrics.files_processed, 1);
        assert_eq!(metrics.lines_processed, 100);
        assert_eq!(metrics.bytes_processed, 1024);
        assert_eq!(metrics.cache_hits, 1);
        assert_eq!(metrics.cache_misses, 1);
    }
} 