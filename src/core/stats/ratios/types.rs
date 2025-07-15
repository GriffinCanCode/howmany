use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Ratio and percentage statistics for analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RatioStats {
    pub code_ratio: f64,           // code lines / total lines
    pub comment_ratio: f64,        // comment lines / total lines
    pub doc_ratio: f64,            // doc lines / total lines
    pub blank_ratio: f64,          // blank lines / total lines
    pub comment_to_code_ratio: f64, // comment lines / code lines
    pub doc_to_code_ratio: f64,    // doc lines / code lines
    pub ratios_by_extension: HashMap<String, ExtensionRatios>,
    pub language_distribution: HashMap<String, f64>, // percentage of total lines by language
    pub file_distribution: HashMap<String, f64>,     // percentage of total files by language
    pub size_distribution: HashMap<String, f64>,     // percentage of total size by language
    pub quality_metrics: QualityMetrics,
}

/// Ratio statistics for a specific file extension
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionRatios {
    pub code_ratio: f64,
    pub comment_ratio: f64,
    pub doc_ratio: f64,
    pub blank_ratio: f64,
    pub comment_to_code_ratio: f64,
    pub doc_to_code_ratio: f64,
    pub lines_per_file: f64,
    pub size_per_file: f64,
}

/// Code health metrics based on ratios
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityMetrics {
    pub documentation_score: f64,   // 0-100 based on doc/comment ratios
    pub maintainability_score: f64, // 0-100 based on various factors
    pub readability_score: f64,     // 0-100 based on comment density
    pub consistency_score: f64,     // 0-100 based on ratio consistency across files
    pub overall_quality_score: f64, // 0-100 weighted average
}

/// Thresholds for quality assessment
#[derive(Debug, Clone)]
pub struct QualityThresholds {
    pub good_comment_ratio: f64,      // 0.15 = 15%
    pub good_doc_ratio: f64,          // 0.10 = 10%
    pub max_blank_ratio: f64,         // 0.30 = 30%
    pub ideal_comment_to_code: f64,   // 0.20 = 20%
    pub ideal_doc_to_code: f64,       // 0.15 = 15%
}

impl Default for QualityThresholds {
    fn default() -> Self {
        Self {
            good_comment_ratio: 0.15,
            good_doc_ratio: 0.10,
            max_blank_ratio: 0.30,
            ideal_comment_to_code: 0.20,
            ideal_doc_to_code: 0.15,
        }
    }
} 