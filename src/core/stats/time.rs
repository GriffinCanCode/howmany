use crate::core::types::{CodeStats, FileStats};
use crate::utils::errors::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Time-based statistics for development estimates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeStats {
    pub total_time_minutes: usize,
    pub code_time_minutes: usize,
    pub doc_time_minutes: usize,
    pub comment_time_minutes: usize,
    pub total_time_formatted: String,
    pub code_time_formatted: String,
    pub doc_time_formatted: String,
    pub comment_time_formatted: String,
    pub time_by_extension: HashMap<String, ExtensionTimeStats>,
    pub productivity_metrics: ProductivityMetrics,
}

/// Time statistics for a specific file extension
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionTimeStats {
    pub total_time_minutes: usize,
    pub code_time_minutes: usize,
    pub doc_time_minutes: usize,
    pub comment_time_minutes: usize,
    pub formatted_time: String,
    pub average_time_per_file: f64,
}

/// Productivity metrics based on time estimates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductivityMetrics {
    pub lines_per_hour: f64,
    pub code_lines_per_hour: f64,
    pub files_per_hour: f64,
    pub estimated_development_days: f64,
    pub estimated_development_hours: f64,
}

/// Time estimation rates (minutes per line)
#[derive(Debug, Clone)]
pub struct TimeEstimationRates {
    pub code_line_rate: usize,     // minutes per line of code
    pub doc_line_rate: usize,      // minutes per line of documentation
    pub comment_line_rate: usize,  // minutes per line of comments
    pub blank_line_rate: usize,    // minutes per blank line (usually 0)
}

impl Default for TimeEstimationRates {
    fn default() -> Self {
        Self {
            code_line_rate: 2,     // 2 minutes per line of code
            doc_line_rate: 3,      // 3 minutes per line of documentation
            comment_line_rate: 1,  // 1 minute per line of comments
            blank_line_rate: 0,    // 0 minutes per blank line
        }
    }
}

/// Calculator for time-based statistics
pub struct TimeStatsCalculator {
    rates: TimeEstimationRates,
}

impl TimeStatsCalculator {
    pub fn new() -> Self {
        Self {
            rates: TimeEstimationRates::default(),
        }
    }
    
    pub fn with_rates(rates: TimeEstimationRates) -> Self {
        Self { rates }
    }
    
    /// Calculate time statistics for a single file
    pub fn calculate_time_stats(&self, file_stats: &FileStats) -> Result<TimeStats> {
        let code_time_minutes = file_stats.code_lines * self.rates.code_line_rate;
        let doc_time_minutes = file_stats.doc_lines * self.rates.doc_line_rate;
        let comment_time_minutes = file_stats.comment_lines * self.rates.comment_line_rate;
        let total_time_minutes = code_time_minutes + doc_time_minutes + comment_time_minutes;
        
        let productivity_metrics = self.calculate_productivity_metrics(
            total_time_minutes,
            file_stats.code_lines,
            file_stats.total_lines,
            1, // single file
        );
        
        Ok(TimeStats {
            total_time_minutes,
            code_time_minutes,
            doc_time_minutes,
            comment_time_minutes,
            total_time_formatted: self.format_time_human(total_time_minutes),
            code_time_formatted: self.format_time_human(code_time_minutes),
            doc_time_formatted: self.format_time_human(doc_time_minutes),
            comment_time_formatted: self.format_time_human(comment_time_minutes),
            time_by_extension: HashMap::new(),
            productivity_metrics,
        })
    }
    
    /// Calculate time statistics for a project
    pub fn calculate_project_time_stats(&self, code_stats: &CodeStats) -> Result<TimeStats> {
        let code_time_minutes = code_stats.total_code_lines * self.rates.code_line_rate;
        let doc_time_minutes = code_stats.total_doc_lines * self.rates.doc_line_rate;
        let comment_time_minutes = code_stats.total_comment_lines * self.rates.comment_line_rate;
        let total_time_minutes = code_time_minutes + doc_time_minutes + comment_time_minutes;
        
        let mut time_by_extension = HashMap::new();
        
        for (ext, (file_count, file_stats)) in &code_stats.stats_by_extension {
            let ext_code_time = file_stats.code_lines * self.rates.code_line_rate;
            let ext_doc_time = file_stats.doc_lines * self.rates.doc_line_rate;
            let ext_comment_time = file_stats.comment_lines * self.rates.comment_line_rate;
            let ext_total_time = ext_code_time + ext_doc_time + ext_comment_time;
            
            let ext_time_stats = ExtensionTimeStats {
                total_time_minutes: ext_total_time,
                code_time_minutes: ext_code_time,
                doc_time_minutes: ext_doc_time,
                comment_time_minutes: ext_comment_time,
                formatted_time: self.format_time_human(ext_total_time),
                average_time_per_file: if *file_count > 0 {
                    ext_total_time as f64 / *file_count as f64
                } else {
                    0.0
                },
            };
            
            time_by_extension.insert(ext.clone(), ext_time_stats);
        }
        
        let productivity_metrics = self.calculate_productivity_metrics(
            total_time_minutes,
            code_stats.total_code_lines,
            code_stats.total_lines,
            code_stats.total_files,
        );
        
        Ok(TimeStats {
            total_time_minutes,
            code_time_minutes,
            doc_time_minutes,
            comment_time_minutes,
            total_time_formatted: self.format_time_human(total_time_minutes),
            code_time_formatted: self.format_time_human(code_time_minutes),
            doc_time_formatted: self.format_time_human(doc_time_minutes),
            comment_time_formatted: self.format_time_human(comment_time_minutes),
            time_by_extension,
            productivity_metrics,
        })
    }
    
    /// Calculate productivity metrics
    fn calculate_productivity_metrics(
        &self,
        total_time_minutes: usize,
        code_lines: usize,
        total_lines: usize,
        total_files: usize,
    ) -> ProductivityMetrics {
        let total_hours = total_time_minutes as f64 / 60.0;
        
        let lines_per_hour = if total_hours > 0.0 {
            total_lines as f64 / total_hours
        } else {
            0.0
        };
        
        let code_lines_per_hour = if total_hours > 0.0 {
            code_lines as f64 / total_hours
        } else {
            0.0
        };
        
        let files_per_hour = if total_hours > 0.0 {
            total_files as f64 / total_hours
        } else {
            0.0
        };
        
        // Assuming 8 hours per working day
        let estimated_development_hours = total_hours;
        let estimated_development_days = total_hours / 8.0;
        
        ProductivityMetrics {
            lines_per_hour,
            code_lines_per_hour,
            files_per_hour,
            estimated_development_days,
            estimated_development_hours,
        }
    }
    
    /// Format time in human-readable format
    pub fn format_time_human(&self, minutes: usize) -> String {
        if minutes == 0 {
            return "0min".to_string();
        }
        
        if minutes < 60 {
            format!("{}min", minutes)
        } else if minutes < 1440 { // Less than a day
            let hours = minutes / 60;
            let remaining_minutes = minutes % 60;
            if remaining_minutes == 0 {
                format!("{}h", hours)
            } else {
                format!("{}h {}min", hours, remaining_minutes)
            }
        } else {
            let days = minutes / 1440;
            let remaining_hours = (minutes % 1440) / 60;
            let remaining_minutes = minutes % 60;
            
            let mut result = if days == 1 {
                "1 day".to_string()
            } else {
                format!("{} days", days)
            };
            
            if remaining_hours > 0 {
                result.push_str(&format!(" {}h", remaining_hours));
            }
            
            if remaining_minutes > 0 {
                result.push_str(&format!(" {}min", remaining_minutes));
            }
            
            result
        }
    }
    
    /// Get time breakdown percentages
    pub fn get_time_breakdown_percentages(&self, stats: &TimeStats) -> HashMap<String, f64> {
        let mut breakdown = HashMap::new();
        
        if stats.total_time_minutes > 0 {
            let total = stats.total_time_minutes as f64;
            breakdown.insert("code".to_string(), (stats.code_time_minutes as f64 / total) * 100.0);
            breakdown.insert("documentation".to_string(), (stats.doc_time_minutes as f64 / total) * 100.0);
            breakdown.insert("comments".to_string(), (stats.comment_time_minutes as f64 / total) * 100.0);
        }
        
        breakdown
    }
    
    /// Get the most time-consuming extension
    pub fn get_most_time_consuming_extension<'a>(&self, stats: &'a TimeStats) -> Option<(String, &'a ExtensionTimeStats)> {
        stats.time_by_extension
            .iter()
            .max_by_key(|(_, ext_stats)| ext_stats.total_time_minutes)
            .map(|(ext, stats)| (ext.clone(), stats))
    }
    
    /// Get alternative activities for time spent
    pub fn get_alternative_activities(&self, minutes: usize) -> Vec<String> {
        let mut activities = Vec::new();
        
        match minutes {
            0..=30 => {
                activities.push("Had a coffee break".to_string());
                activities.push("Read a few Stack Overflow answers".to_string());
            }
            31..=60 => {
                activities.push("Watched a coding tutorial".to_string());
                activities.push("Fixed a small bug".to_string());
            }
            61..=120 => {
                activities.push("Learned a new Git command".to_string());
                activities.push("Refactored a function".to_string());
            }
            121..=240 => {
                activities.push("Implemented a small feature".to_string());
                activities.push("Wrote comprehensive tests".to_string());
            }
            241..=480 => {
                activities.push("Learned a new framework".to_string());
                activities.push("Set up a new development environment".to_string());
            }
            481..=960 => {
                activities.push("Built a small project".to_string());
                activities.push("Mastered a new programming language".to_string());
            }
            _ => {
                activities.push("Become a senior developer".to_string());
                activities.push("Started your own tech company".to_string());
            }
        }
        
        activities
    }
    
    /// Calculate time efficiency score (0-100)
    pub fn calculate_efficiency_score(&self, stats: &TimeStats) -> f64 {
        if stats.total_time_minutes == 0 {
            return 0.0;
        }
        
        let code_percentage = (stats.code_time_minutes as f64 / stats.total_time_minutes as f64) * 100.0;
        
        // Score based on code percentage (higher is better)
        // 80%+ code = 100 points
        // 60-79% code = 80 points
        // 40-59% code = 60 points
        // 20-39% code = 40 points
        // <20% code = 20 points
        match code_percentage {
            p if p >= 80.0 => 100.0,
            p if p >= 60.0 => 80.0,
            p if p >= 40.0 => 60.0,
            p if p >= 20.0 => 40.0,
            _ => 20.0,
        }
    }
    
    /// Get time waste level description
    pub fn get_time_waste_level(&self, stats: &TimeStats) -> (String, String) {
        let efficiency = self.calculate_efficiency_score(stats);
        
        match efficiency as usize {
            90..=100 => ("Highly Efficient".to_string(), "waste-low".to_string()),
            70..=89 => ("Moderately Efficient".to_string(), "waste-medium".to_string()),
            50..=69 => ("Somewhat Wasteful".to_string(), "waste-high".to_string()),
            _ => ("Time Waster".to_string(), "waste-extreme".to_string()),
        }
    }
    
    /// Calculate development velocity (lines per day)
    pub fn calculate_development_velocity(&self, stats: &TimeStats, total_lines: usize) -> f64 {
        if stats.productivity_metrics.estimated_development_days > 0.0 {
            total_lines as f64 / stats.productivity_metrics.estimated_development_days
        } else {
            0.0
        }
    }
    
    /// Get rates for customization
    pub fn get_rates(&self) -> &TimeEstimationRates {
        &self.rates
    }
    
    /// Update rates
    pub fn set_rates(&mut self, rates: TimeEstimationRates) {
        self.rates = rates;
    }
}

impl Default for TimeStatsCalculator {
    fn default() -> Self {
        Self::new()
    }
} 