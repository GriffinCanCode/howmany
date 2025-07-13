pub mod types;
pub mod counter;
pub mod detector;
pub mod filters;
pub mod stats;

pub use types::{CodeStats, FileStats};
pub use counter::CodeCounter;
pub use detector::FileDetector;
pub use filters::FileFilter;
pub use stats::StatsCalculator;

// Re-export techstack types for easy access
pub use stats::{
    TechStackAnalyzer, TechStackDetector, TechStackInventory, DetectedTechnology,
    TechCategory, ConfidenceLevel, TechStackInsights, TechStackStats,
    DependencyGraph, DependencyMapper, FrameworkRecommendation
}; 