use crate::core::types::{CodeStats, FileStats};
use crate::utils::errors::Result;
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

/// Quality metrics based on ratios
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

/// Calculator for ratio and percentage statistics
pub struct RatioStatsCalculator {
    thresholds: QualityThresholds,
}

impl RatioStatsCalculator {
    pub fn new() -> Self {
        Self {
            thresholds: QualityThresholds::default(),
        }
    }
    
    pub fn with_thresholds(thresholds: QualityThresholds) -> Self {
        Self { thresholds }
    }
    
    /// Calculate ratio statistics for a single file
    pub fn calculate_ratio_stats(&self, file_stats: &FileStats) -> Result<RatioStats> {
        let total_lines = file_stats.total_lines as f64;
        
        let code_ratio = if total_lines > 0.0 { file_stats.code_lines as f64 / total_lines } else { 0.0 };
        let comment_ratio = if total_lines > 0.0 { file_stats.comment_lines as f64 / total_lines } else { 0.0 };
        let doc_ratio = if total_lines > 0.0 { file_stats.doc_lines as f64 / total_lines } else { 0.0 };
        let blank_ratio = if total_lines > 0.0 { file_stats.blank_lines as f64 / total_lines } else { 0.0 };
        
        let comment_to_code_ratio = if file_stats.code_lines > 0 {
            file_stats.comment_lines as f64 / file_stats.code_lines as f64
        } else {
            0.0
        };
        
        let doc_to_code_ratio = if file_stats.code_lines > 0 {
            file_stats.doc_lines as f64 / file_stats.code_lines as f64
        } else {
            0.0
        };
        
        let quality_metrics = self.calculate_quality_metrics(
            code_ratio, comment_ratio, doc_ratio, blank_ratio,
            comment_to_code_ratio, doc_to_code_ratio, &HashMap::new()
        );
        
        Ok(RatioStats {
            code_ratio,
            comment_ratio,
            doc_ratio,
            blank_ratio,
            comment_to_code_ratio,
            doc_to_code_ratio,
            ratios_by_extension: HashMap::new(),
            language_distribution: HashMap::new(),
            file_distribution: HashMap::new(),
            size_distribution: HashMap::new(),
            quality_metrics,
        })
    }
    
    /// Calculate ratio statistics for a project
    pub fn calculate_project_ratio_stats(&self, code_stats: &CodeStats) -> Result<RatioStats> {
        let total_lines = code_stats.total_lines as f64;
        
        let code_ratio = if total_lines > 0.0 { code_stats.total_code_lines as f64 / total_lines } else { 0.0 };
        let comment_ratio = if total_lines > 0.0 { code_stats.total_comment_lines as f64 / total_lines } else { 0.0 };
        let doc_ratio = if total_lines > 0.0 { code_stats.total_doc_lines as f64 / total_lines } else { 0.0 };
        let blank_ratio = if total_lines > 0.0 { code_stats.total_blank_lines as f64 / total_lines } else { 0.0 };
        
        let comment_to_code_ratio = if code_stats.total_code_lines > 0 {
            code_stats.total_comment_lines as f64 / code_stats.total_code_lines as f64
        } else {
            0.0
        };
        
        let doc_to_code_ratio = if code_stats.total_code_lines > 0 {
            code_stats.total_doc_lines as f64 / code_stats.total_code_lines as f64
        } else {
            0.0
        };
        
        // Calculate ratios by extension
        let mut ratios_by_extension = HashMap::new();
        for (ext, (file_count, file_stats)) in &code_stats.stats_by_extension {
            let ext_total_lines = file_stats.total_lines as f64;
            
            let ext_ratios = ExtensionRatios {
                code_ratio: if ext_total_lines > 0.0 { file_stats.code_lines as f64 / ext_total_lines } else { 0.0 },
                comment_ratio: if ext_total_lines > 0.0 { file_stats.comment_lines as f64 / ext_total_lines } else { 0.0 },
                doc_ratio: if ext_total_lines > 0.0 { file_stats.doc_lines as f64 / ext_total_lines } else { 0.0 },
                blank_ratio: if ext_total_lines > 0.0 { file_stats.blank_lines as f64 / ext_total_lines } else { 0.0 },
                comment_to_code_ratio: if file_stats.code_lines > 0 {
                    file_stats.comment_lines as f64 / file_stats.code_lines as f64
                } else {
                    0.0
                },
                doc_to_code_ratio: if file_stats.code_lines > 0 {
                    file_stats.doc_lines as f64 / file_stats.code_lines as f64
                } else {
                    0.0
                },
                lines_per_file: if *file_count > 0 { ext_total_lines / *file_count as f64 } else { 0.0 },
                size_per_file: if *file_count > 0 { file_stats.file_size as f64 / *file_count as f64 } else { 0.0 },
            };
            
            ratios_by_extension.insert(ext.clone(), ext_ratios);
        }
        
        // Calculate distributions
        let language_distribution = self.calculate_language_distribution(code_stats);
        let file_distribution = self.calculate_file_distribution(code_stats);
        let size_distribution = self.calculate_size_distribution(code_stats);
        
        let quality_metrics = self.calculate_quality_metrics(
            code_ratio, comment_ratio, doc_ratio, blank_ratio,
            comment_to_code_ratio, doc_to_code_ratio, &ratios_by_extension
        );
        
        Ok(RatioStats {
            code_ratio,
            comment_ratio,
            doc_ratio,
            blank_ratio,
            comment_to_code_ratio,
            doc_to_code_ratio,
            ratios_by_extension,
            language_distribution,
            file_distribution,
            size_distribution,
            quality_metrics,
        })
    }
    
    /// Calculate language distribution by lines
    fn calculate_language_distribution(&self, code_stats: &CodeStats) -> HashMap<String, f64> {
        let mut distribution = HashMap::new();
        let total_lines = code_stats.total_lines as f64;
        
        if total_lines > 0.0 {
            for (ext, (_, file_stats)) in &code_stats.stats_by_extension {
                let percentage = (file_stats.total_lines as f64 / total_lines) * 100.0;
                distribution.insert(ext.clone(), percentage);
            }
        }
        
        distribution
    }
    
    /// Calculate file distribution by count
    fn calculate_file_distribution(&self, code_stats: &CodeStats) -> HashMap<String, f64> {
        let mut distribution = HashMap::new();
        let total_files = code_stats.total_files as f64;
        
        if total_files > 0.0 {
            for (ext, (file_count, _)) in &code_stats.stats_by_extension {
                let percentage = (*file_count as f64 / total_files) * 100.0;
                distribution.insert(ext.clone(), percentage);
            }
        }
        
        distribution
    }
    
    /// Calculate size distribution
    fn calculate_size_distribution(&self, code_stats: &CodeStats) -> HashMap<String, f64> {
        let mut distribution = HashMap::new();
        let total_size = code_stats.total_size as f64;
        
        if total_size > 0.0 {
            for (ext, (_, file_stats)) in &code_stats.stats_by_extension {
                let percentage = (file_stats.file_size as f64 / total_size) * 100.0;
                distribution.insert(ext.clone(), percentage);
            }
        }
        
        distribution
    }
    
    /// Calculate quality metrics
    fn calculate_quality_metrics(
        &self,
        code_ratio: f64,
        comment_ratio: f64,
        doc_ratio: f64,
        blank_ratio: f64,
        comment_to_code_ratio: f64,
        doc_to_code_ratio: f64,
        ratios_by_extension: &HashMap<String, ExtensionRatios>,
    ) -> QualityMetrics {
        // Documentation score (0-100)
        let doc_score = self.calculate_documentation_score(doc_ratio, comment_ratio, doc_to_code_ratio, comment_to_code_ratio);
        
        // Maintainability score (0-100)
        let maintainability_score = self.calculate_maintainability_score(code_ratio, comment_ratio, doc_ratio, blank_ratio);
        
        // Readability score (0-100)
        let readability_score = self.calculate_readability_score(comment_ratio, blank_ratio);
        
        // Consistency score (0-100)
        let consistency_score = self.calculate_consistency_score(ratios_by_extension);
        
        // Overall quality score (weighted average)
        let overall_quality_score = (doc_score * 0.25) + (maintainability_score * 0.35) + (readability_score * 0.25) + (consistency_score * 0.15);
        
        QualityMetrics {
            documentation_score: doc_score,
            maintainability_score,
            readability_score,
            consistency_score,
            overall_quality_score,
        }
    }
    
    /// Calculate documentation score
    fn calculate_documentation_score(&self, doc_ratio: f64, comment_ratio: f64, doc_to_code_ratio: f64, comment_to_code_ratio: f64) -> f64 {
        let mut score = 0.0;
        
        // Documentation ratio score (0-40 points)
        if doc_ratio >= self.thresholds.good_doc_ratio {
            score += 40.0;
        } else {
            score += (doc_ratio / self.thresholds.good_doc_ratio) * 40.0;
        }
        
        // Comment ratio score (0-30 points)
        if comment_ratio >= self.thresholds.good_comment_ratio {
            score += 30.0;
        } else {
            score += (comment_ratio / self.thresholds.good_comment_ratio) * 30.0;
        }
        
        // Doc to code ratio score (0-15 points)
        if doc_to_code_ratio >= self.thresholds.ideal_doc_to_code {
            score += 15.0;
        } else {
            score += (doc_to_code_ratio / self.thresholds.ideal_doc_to_code) * 15.0;
        }
        
        // Comment to code ratio score (0-15 points)
        if comment_to_code_ratio >= self.thresholds.ideal_comment_to_code {
            score += 15.0;
        } else {
            score += (comment_to_code_ratio / self.thresholds.ideal_comment_to_code) * 15.0;
        }
        
        score.min(100.0)
    }
    
    /// Calculate maintainability score
    fn calculate_maintainability_score(&self, code_ratio: f64, comment_ratio: f64, doc_ratio: f64, blank_ratio: f64) -> f64 {
        let mut score = 0.0;
        
        // Code ratio score (0-40 points) - higher is better
        score += code_ratio * 40.0;
        
        // Comment ratio score (0-25 points)
        if comment_ratio >= self.thresholds.good_comment_ratio {
            score += 25.0;
        } else {
            score += (comment_ratio / self.thresholds.good_comment_ratio) * 25.0;
        }
        
        // Documentation ratio score (0-25 points)
        if doc_ratio >= self.thresholds.good_doc_ratio {
            score += 25.0;
        } else {
            score += (doc_ratio / self.thresholds.good_doc_ratio) * 25.0;
        }
        
        // Blank ratio penalty (0-10 points) - too many blanks is bad
        if blank_ratio <= self.thresholds.max_blank_ratio {
            score += 10.0;
        } else {
            let penalty = (blank_ratio - self.thresholds.max_blank_ratio) * 20.0;
            score += (10.0 - penalty).max(0.0);
        }
        
        score.min(100.0)
    }
    
    /// Calculate readability score
    fn calculate_readability_score(&self, comment_ratio: f64, blank_ratio: f64) -> f64 {
        let mut score = 0.0;
        
        // Comment ratio score (0-70 points)
        if comment_ratio >= self.thresholds.good_comment_ratio {
            score += 70.0;
        } else {
            score += (comment_ratio / self.thresholds.good_comment_ratio) * 70.0;
        }
        
        // Blank ratio score (0-30 points) - some blanks are good for readability
        let ideal_blank_ratio = 0.15; // 15% blank lines is ideal
        if blank_ratio <= ideal_blank_ratio {
            score += (blank_ratio / ideal_blank_ratio) * 30.0;
        } else {
            let penalty = (blank_ratio - ideal_blank_ratio) * 60.0;
            score += (30.0 - penalty).max(0.0);
        }
        
        score.min(100.0)
    }
    
    /// Calculate consistency score
    fn calculate_consistency_score(&self, ratios_by_extension: &HashMap<String, ExtensionRatios>) -> f64 {
        if ratios_by_extension.len() <= 1 {
            return 100.0; // Perfect consistency with one or no languages
        }
        
        let mut comment_ratios = Vec::new();
        let mut doc_ratios = Vec::new();
        
        for ext_ratios in ratios_by_extension.values() {
            comment_ratios.push(ext_ratios.comment_ratio);
            doc_ratios.push(ext_ratios.doc_ratio);
        }
        
        let comment_consistency = self.calculate_variance_score(&comment_ratios);
        let doc_consistency = self.calculate_variance_score(&doc_ratios);
        
        (comment_consistency + doc_consistency) / 2.0
    }
    
    /// Calculate variance score (lower variance = higher score)
    fn calculate_variance_score(&self, values: &[f64]) -> f64 {
        if values.len() <= 1 {
            return 100.0;
        }
        
        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / values.len() as f64;
        let std_dev = variance.sqrt();
        
        // Convert standard deviation to score (lower std_dev = higher score)
        let max_std_dev = 0.2; // 20% standard deviation is considered high
        let score = ((max_std_dev - std_dev.min(max_std_dev)) / max_std_dev) * 100.0;
        score.max(0.0)
    }
    
    /// Get quality level description
    pub fn get_quality_level(&self, score: f64) -> String {
        match score as usize {
            90..=100 => "Excellent".to_string(),
            80..=89 => "Good".to_string(),
            70..=79 => "Fair".to_string(),
            60..=69 => "Poor".to_string(),
            _ => "Very Poor".to_string(),
        }
    }
    
    /// Get quality level CSS class
    pub fn get_quality_class(&self, score: f64) -> String {
        match score as usize {
            90..=100 => "quality-excellent".to_string(),
            80..=89 => "quality-good".to_string(),
            70..=79 => "quality-fair".to_string(),
            60..=69 => "quality-poor".to_string(),
            _ => "quality-very-poor".to_string(),
        }
    }
    
    /// Get insights based on ratios
    pub fn get_ratio_insights(&self, stats: &RatioStats) -> Vec<String> {
        let mut insights = Vec::new();
        
        // Code ratio insights
        if stats.code_ratio > 0.8 {
            insights.push("High code density - very efficient!".to_string());
        } else if stats.code_ratio < 0.4 {
            insights.push("Low code density - lots of documentation or comments".to_string());
        }
        
        // Comment insights
        if stats.comment_ratio > 0.3 {
            insights.push("Very well commented code".to_string());
        } else if stats.comment_ratio < 0.05 {
            insights.push("Could use more comments for clarity".to_string());
        }
        
        // Documentation insights
        if stats.doc_ratio > 0.2 {
            insights.push("Excellent documentation coverage".to_string());
        } else if stats.doc_ratio < 0.05 {
            insights.push("Consider adding more documentation".to_string());
        }
        
        // Blank line insights
        if stats.blank_ratio > 0.4 {
            insights.push("Very spacious code - lots of breathing room".to_string());
        } else if stats.blank_ratio < 0.1 {
            insights.push("Dense code - consider adding blank lines for readability".to_string());
        }
        
        insights
    }
    
    /// Get most documented language
    pub fn get_most_documented_language(&self, stats: &RatioStats) -> Option<(String, f64)> {
        stats.ratios_by_extension
            .iter()
            .max_by(|a, b| {
                let a_doc_score = a.1.doc_ratio + a.1.comment_ratio;
                let b_doc_score = b.1.doc_ratio + b.1.comment_ratio;
                a_doc_score.partial_cmp(&b_doc_score).unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(ext, ratios)| (ext.clone(), ratios.doc_ratio + ratios.comment_ratio))
    }
    
    /// Get most efficient language (highest code ratio)
    pub fn get_most_efficient_language(&self, stats: &RatioStats) -> Option<(String, f64)> {
        stats.ratios_by_extension
            .iter()
            .max_by_key(|(_, ratios)| (ratios.code_ratio * 1000.0) as usize)
            .map(|(ext, ratios)| (ext.clone(), ratios.code_ratio))
    }
    
    /// Get thresholds for customization
    pub fn get_thresholds(&self) -> &QualityThresholds {
        &self.thresholds
    }
    
    /// Update thresholds
    pub fn set_thresholds(&mut self, thresholds: QualityThresholds) {
        self.thresholds = thresholds;
    }
}

impl Default for RatioStatsCalculator {
    fn default() -> Self {
        Self::new()
    }
} 