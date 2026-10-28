use crate::core::languages::{Breakdown, Category, CategoryTotals};
use crate::core::stats::basic::BasicStats;
use crate::core::stats::complexity::ComplexityStats;
use crate::core::stats::ratios::RatioStats;
use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize, Serializer};

/// Aggregated statistics containing all types of statistics
#[derive(Debug, Clone, Deserialize)]
pub struct AggregatedStats {
    pub basic: BasicStats,
    pub complexity: ComplexityStats,
    pub ratios: RatioStats,
    pub metadata: StatsMetadata,
}

impl AggregatedStats {
    /// Content categories derived from the current extension totals.
    ///
    /// Deriving this on demand keeps merged and filtered reports consistent
    /// without retaining a second mutable copy of the same statistics.
    pub fn breakdown(&self) -> Breakdown {
        Breakdown::from_extensions(&self.basic.stats_by_extension)
    }

    /// Source-file totals, excluding configuration, data and prose files.
    pub fn source(&self) -> CategoryTotals {
        self.breakdown().totals_in(Category::Code)
    }
}

impl Serialize for AggregatedStats {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let breakdown = self.breakdown();
        let mut report = serializer.serialize_struct("AggregatedStats", 6)?;
        // These four sections are consumed by the editor and Action integrations.
        report.serialize_field("basic", &self.basic)?;
        report.serialize_field("complexity", &self.complexity)?;
        report.serialize_field("ratios", &self.ratios)?;
        report.serialize_field("metadata", &self.metadata)?;
        report.serialize_field("source", &breakdown.totals_in(Category::Code))?;
        report.serialize_field("breakdown", &breakdown)?;
        report.end()
    }
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
    Basic,    // Only basic line counting
    Standard, // Basic + ratios
    Advanced, // Standard + complexity analysis
    Complete, // Advanced + all insights and quality metrics
}
