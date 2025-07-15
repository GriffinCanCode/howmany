use super::types::{QualityMetrics, QualityThresholds, ExtensionRatios};
use std::collections::HashMap;

/// Quality metrics calculator
pub struct QualityCalculator {
    thresholds: QualityThresholds,
}

impl QualityCalculator {
    pub fn new(thresholds: QualityThresholds) -> Self {
        Self { thresholds }
    }
    
    /// Calculate quality metrics
    pub fn calculate_quality_metrics(
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
} 