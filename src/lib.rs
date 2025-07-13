// Core functionality modules
pub mod core {
    pub mod detector;
    pub mod counter;
    pub mod filters;
}

// User interface modules
pub mod ui {
    pub mod cli;
    pub mod interactive;
}

// Utility modules
pub mod utils {
    pub mod errors;
    pub mod config;
    pub mod progress;
    pub mod cache;
    pub mod metrics;
}

// Testing utilities (only available in test builds)
#[cfg(test)]
pub mod testing {
    pub mod test_utils;
}

// Re-export commonly used types for convenience
pub use core::detector::FileDetector;
pub use core::counter::CodeCounter;
pub use core::filters::FileFilter;
pub use ui::cli::Config;
pub use ui::interactive::InteractiveDisplay;
pub use utils::errors::{HowManyError, Result};
pub use utils::config::HowManyConfig;
pub use utils::progress::ProgressReporter;
pub use utils::cache::FileCache;
pub use utils::metrics::{PerformanceMetrics, MetricsCollector}; 