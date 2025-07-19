use crate::core::types::{CodeStats, FileStats};
use crate::utils::errors::Result;
use super::types::{RatioStats, QualityThresholds};
use super::calculator::RatioStatsCalculator;
use super::insights::InsightsAnalyzer;

/// Main interface for ratio statistics functionality
/// This serves as the primary entry point for all ratio-related operations
pub struct RatioStatsManager {
    calculator: RatioStatsCalculator,
    insights_analyzer: InsightsAnalyzer,
}

impl RatioStatsManager {
    /// Create a new ratio stats manager with default settings
    pub fn new() -> Self {
        Self {
            calculator: RatioStatsCalculator::new(),
            insights_analyzer: InsightsAnalyzer::new(),
        }
    }
    
    /// Create a new ratio stats manager with custom thresholds
    pub fn with_thresholds(thresholds: QualityThresholds) -> Self {
        Self {
            calculator: RatioStatsCalculator::with_thresholds(thresholds),
            insights_analyzer: InsightsAnalyzer::new(),
        }
    }
    
    /// Calculate ratio statistics for a single file
    pub fn calculate_file_ratios(&self, file_stats: &FileStats) -> Result<RatioStats> {
        self.calculator.calculate_ratio_stats(file_stats)
    }
    
    /// Calculate ratio statistics for a project
    pub fn calculate_project_ratios(&self, code_stats: &CodeStats) -> Result<RatioStats> {
        self.calculator.calculate_project_ratio_stats(code_stats)
    }
    
    /// Get insights based on ratio statistics
    pub fn get_insights(&self, stats: &RatioStats) -> Vec<String> {
        self.insights_analyzer.get_ratio_insights(stats)
    }
    
    /// Get most documented language
    pub fn get_most_documented_language(&self, stats: &RatioStats) -> Option<(String, f64)> {
        self.insights_analyzer.get_most_documented_language(stats)
    }
    
    /// Get most efficient language (highest code ratio)
    pub fn get_most_efficient_language(&self, stats: &RatioStats) -> Option<(String, f64)> {
        self.insights_analyzer.get_most_efficient_language(stats)
    }
    
    /// Get quality level description
    pub fn get_quality_level(&self, score: f64) -> String {
        self.calculator.get_quality_level(score)
    }
    
    /// Get quality level CSS class
    pub fn get_quality_class(&self, score: f64) -> String {
        self.calculator.get_quality_class(score)
    }
    
    /// Get current thresholds
    pub fn get_thresholds(&self) -> &QualityThresholds {
        self.calculator.get_thresholds()
    }
    
    /// Update thresholds
    pub fn set_thresholds(&mut self, thresholds: QualityThresholds) {
        self.calculator.set_thresholds(thresholds);
    }
    
    /// Get the calculator instance for advanced usage
    pub fn calculator(&self) -> &RatioStatsCalculator {
        &self.calculator
    }
    
    /// Get the insights analyzer instance for advanced usage
    pub fn insights_analyzer(&self) -> &InsightsAnalyzer {
        &self.insights_analyzer
    }
}

impl Default for RatioStatsManager {
    fn default() -> Self {
        Self::new()
    }
} 