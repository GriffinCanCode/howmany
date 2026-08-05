// Core functionality modules
pub mod core {
    pub mod counter;
    pub mod detector;
    pub mod engine;
    pub mod filters;
    pub mod languages;
    pub mod patterns;
    pub mod stats;
    pub mod types;
}

// User interface modules
pub mod ui {
    pub mod cli;
    pub mod filters;
    pub mod html;
    pub mod interactive;
    pub mod sarif;
}

// Utility modules
pub mod utils {
    pub mod cache;
    pub mod config;
    pub mod errors;
    pub mod metrics;
    pub mod progress;
}

// Testing utilities (only available in test builds)
#[cfg(test)]
pub mod testing;

// Re-export commonly used types for convenience
pub use core::counter::{CachedCodeCounter, CodeCounter, CommentPattern};
pub use core::detector::{FileDetector, SherlockLanguage, SherlockResult, SherlockSummary};
pub use core::engine::{
    Analysis, AnalysisOptions, AnalysisReport, DetectionMode, Engine, Parallelism,
};
pub use core::filters::FileFilter;
pub use core::languages::{Category, Language};
pub use core::patterns::PatternMatcher;
pub use core::stats::StatsCalculator;
pub use core::types::{CodeStats, FileStats};

pub use ui::cli::Config;
pub use ui::html::HtmlReporter;
pub use ui::interactive::InteractiveDisplay;
pub use ui::sarif::SarifReporter;
pub use utils::cache::FileCache;
pub use utils::config::HowManyConfig;
pub use utils::errors::{HowManyError, Result};
pub use utils::metrics::{MetricsCollector, PerformanceMetrics};
pub use utils::progress::ProgressReporter;
