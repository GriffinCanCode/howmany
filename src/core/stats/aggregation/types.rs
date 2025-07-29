use crate::core::stats::basic::BasicStats;
use crate::core::stats::complexity::ComplexityStats;
use crate::core::stats::ratios::RatioStats;
use serde::{Deserialize, Serialize};

/// Aggregated statistics containing all types of statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedStats {
    pub basic: BasicStats,
    pub complexity: ComplexityStats,
    pub ratios: RatioStats,
    pub metadata: StatsMetadata,
}

/// Metadata about the statistics calculation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatsMetadata {
    pub calculation_time_ms: u64,
    pub version: String,
    pub timestamp: String,
    pub file_count_analyzed: usize,
    pub total_bytes_analyzed: u64,
    pub languages_detected: Vec<String>,
    pub analysis_depth: AnalysisDepth,
}

/// Depth of analysis performed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnalysisDepth {
    Basic,      // Only basic line counting
    Standard,   // Basic + ratios
    Advanced,   // Standard + complexity analysis
    Complete,   // Advanced + all insights and quality metrics
} 