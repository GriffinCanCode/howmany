use crate::core::types::{CodeStats, FileStats};
use crate::utils::errors::Result;

// Re-export all public types
pub use types::*;

// Internal modules
mod types;
mod analyzer;
mod quality;
mod calculator;
mod languages;

// Main interface - this is the public API that other modules will use
pub struct ComplexityStatsCalculator {
    calculator: calculator::ComplexityCalculator,
}

impl ComplexityStatsCalculator {
    pub fn new() -> Self {
        Self {
            calculator: calculator::ComplexityCalculator::new(),
        }
    }
    
    /// Calculate complexity statistics for a single file
    pub fn calculate_complexity_stats(&self, file_stats: &FileStats, file_path: &str) -> Result<ComplexityStats> {
        self.calculator.calculate_complexity_stats(file_stats, file_path)
    }
    
    /// Calculate complexity statistics for a project
    pub fn calculate_project_complexity_stats(&self, code_stats: &CodeStats, individual_files: &[(String, FileStats)]) -> Result<ComplexityStats> {
        self.calculator.calculate_project_complexity_stats(code_stats, individual_files)
    }
    
    /// Get complexity level description
    pub fn get_complexity_level(&self, complexity: f64) -> String {
        self.calculator.get_complexity_level(complexity)
    }
    
    /// Get complexity level CSS class
    pub fn get_complexity_class(&self, complexity: f64) -> String {
        self.calculator.get_complexity_class(complexity)
    }
}

impl Default for ComplexityStatsCalculator {
    fn default() -> Self {
        Self::new()
    }
} 