//! Statistics aggregation module
//!
//! This module provides comprehensive statistics aggregation capabilities,
//! combining different types of statistics (basic, complexity, time, ratios)
//! into unified aggregated statistics.
//!
//! ## Structure
//!
//! - `types` - Core data structures for aggregated statistics
//! - `aggregator` - Main aggregator interface for combining statistics
//! - `merging` - Logic for merging multiple statistics together
//!
//! ## Usage
//!
//! ```rust
//! use howmany::core::stats::aggregation::StatsAggregator;
//!
//! let aggregator = StatsAggregator::new();
//! // The aggregator is used to combine statistics from different calculators
//! // In real usage, you would pass actual stats from BasicStats, ComplexityStats, etc.
//! ```

pub mod aggregator;
pub mod merging;
pub mod types;

// Re-export the main types and functionality
pub use aggregator::StatsAggregator;
pub use merging::StatsMerger;
pub use types::{AggregatedStats, AnalysisDepth, StatsMetadata};

// Convenience re-exports for common operations
pub use aggregator::StatsAggregator as Aggregator;
pub use types::AggregatedStats as Stats;

/// Update timing information for aggregated statistics
///
/// This is a convenience function that delegates to StatsAggregator::update_timing
pub fn update_timing(stats: &mut AggregatedStats, start_time: std::time::Instant) {
    StatsAggregator::update_timing(stats, start_time);
}
