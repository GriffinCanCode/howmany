use crate::core::stats::basic::BasicStats;
use crate::core::stats::complexity::ComplexityStats;
use crate::core::stats::ratios::RatioStats;
use crate::utils::errors::Result;
use super::types::{AggregatedStats, StatsMetadata, AnalysisDepth};
use super::merging::StatsMerger;
use std::collections::HashMap;

/// Aggregator for combining different types of statistics
pub struct StatsAggregator {
    version: String,
    merger: StatsMerger,
}

impl StatsAggregator {
    pub fn new() -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
            merger: StatsMerger::new(),
        }
    }
    
    /// Aggregate statistics for a single file
    pub fn aggregate_file_stats(
        &self,
        basic: BasicStats,
        complexity: ComplexityStats,
        ratios: RatioStats,
    ) -> AggregatedStats {
        let metadata = StatsMetadata {
            calculation_time_ms: 0, // Will be set by caller
            version: self.version.clone(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            file_count_analyzed: 1,
            total_bytes_analyzed: basic.total_size,
            languages_detected: vec!["unknown".to_string()], // Will be updated by caller
            analysis_depth: AnalysisDepth::Complete,
        };
        
        AggregatedStats {
            basic,
            complexity,
            ratios,
            metadata,
        }
    }
    
    /// Aggregate statistics for a project
    pub fn aggregate_project_stats(
        &self,
        basic: BasicStats,
        complexity: ComplexityStats,
        ratios: RatioStats,
    ) -> AggregatedStats {
        let languages_detected: Vec<String> = basic.stats_by_extension.keys().cloned().collect();
        
        let metadata = StatsMetadata {
            calculation_time_ms: 0, // Will be set by caller
            version: self.version.clone(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            file_count_analyzed: basic.total_files,
            total_bytes_analyzed: basic.total_size,
            languages_detected,
            analysis_depth: AnalysisDepth::Complete,
        };
        
        AggregatedStats {
            basic,
            complexity,
            ratios,
            metadata,
        }
    }
    
    /// Merge multiple aggregated statistics
    pub fn merge_stats(&self, stats_list: Vec<AggregatedStats>) -> Result<AggregatedStats> {
        self.merger.merge_stats(stats_list)
    }
    
    /// Get summary statistics
    pub fn get_summary(&self, stats: &AggregatedStats) -> HashMap<String, String> {
        let mut summary = HashMap::new();
        
        summary.insert("total_files".to_string(), stats.basic.total_files.to_string());
        summary.insert("total_lines".to_string(), stats.basic.total_lines.to_string());
        summary.insert("code_lines".to_string(), stats.basic.code_lines.to_string());
        summary.insert("functions".to_string(), stats.complexity.function_count.to_string());
        summary.insert("avg_complexity".to_string(), format!("{:.1}", stats.complexity.cyclomatic_complexity));
        summary.insert("quality_score".to_string(), format!("{:.1}", stats.ratios.quality_metrics.overall_quality_score));
        summary.insert("languages".to_string(), stats.metadata.languages_detected.len().to_string());
        
        summary
    }
    
    /// Update metadata with timing information
    pub fn update_timing(stats: &mut AggregatedStats, start_time: std::time::Instant) {
        stats.metadata.calculation_time_ms = start_time.elapsed().as_millis() as u64;
    }
    
    /// Get access to the internal merger for advanced operations
    pub fn merger(&self) -> &StatsMerger {
        &self.merger
    }
}

impl Default for StatsAggregator {
    fn default() -> Self {
        Self::new()
    }
} 