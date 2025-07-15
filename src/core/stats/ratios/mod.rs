pub mod types;
pub mod calculator;
pub mod quality;
pub mod insights;
pub mod manager;

// Re-export the main types and structs for easy access
pub use types::{RatioStats, ExtensionRatios, QualityMetrics, QualityThresholds};
pub use calculator::RatioStatsCalculator;
pub use quality::QualityCalculator;
pub use insights::InsightsAnalyzer;
pub use manager::RatioStatsManager; 