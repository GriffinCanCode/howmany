pub mod calculator;
pub mod insights;
pub mod manager;
pub mod quality;
pub mod types;

// Re-export the main types and structs for easy access
pub use calculator::RatioStatsCalculator;
pub use insights::InsightsAnalyzer;
pub use manager::RatioStatsManager;
pub use quality::QualityCalculator;
pub use types::{ExtensionRatios, LineRatios, QualityMetrics, QualityThresholds, RatioStats};
