use super::types::RatioStats;

/// Insights analyzer for ratio statistics
pub struct InsightsAnalyzer;

impl InsightsAnalyzer {
    pub fn new() -> Self {
        Self
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
}

impl Default for InsightsAnalyzer {
    fn default() -> Self {
        Self::new()
    }
} 