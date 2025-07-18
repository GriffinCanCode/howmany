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
    pub code_line_rate: f64,       // minutes per line of code
    pub doc_line_rate: f64,        // minutes per line of documentation
    pub comment_line_rate: f64,    // minutes per line of comments
    pub blank_line_rate: f64,      // minutes per blank line (usually 0)
}

impl Default for TimeEstimationRates {
    fn default() -> Self {
        Self {
            code_line_rate: 0.2,   // 0.2 minutes per line of code (realistic for modern dev with IDE assistance, autocomplete, copy-paste)
            doc_line_rate: 0.5,    // 0.5 minutes per line of documentation (faster than complex code)
            comment_line_rate: 0.1, // 0.1 minutes per line of comments (quick explanations)
            blank_line_rate: 0.0,  // 0 minutes per blank line
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
        Self { 
            rates,
        }
    }
    
    /// Calculate time statistics for a single file
    pub fn calculate_time_stats(&self, file_stats: &FileStats) -> Result<TimeStats> {
        let code_time_minutes = (file_stats.code_lines as f64 * self.rates.code_line_rate) as usize;
        let doc_time_minutes = (file_stats.doc_lines as f64 * self.rates.doc_line_rate) as usize;
        let comment_time_minutes = (file_stats.comment_lines as f64 * self.rates.comment_line_rate) as usize;
        let blank_time_minutes = (file_stats.blank_lines as f64 * self.rates.blank_line_rate) as usize;
        let total_time_minutes = code_time_minutes + doc_time_minutes + comment_time_minutes + blank_time_minutes;
        
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
        self.calculate_project_time_stats_with_files(code_stats, &[])
    }
    
    /// Calculate time statistics for a project with individual files
    pub fn calculate_project_time_stats_with_files(&self, code_stats: &CodeStats, _individual_files: &[(String, FileStats)]) -> Result<TimeStats> {
        let code_time_minutes = (code_stats.total_code_lines as f64 * self.rates.code_line_rate) as usize;
        let doc_time_minutes = (code_stats.total_doc_lines as f64 * self.rates.doc_line_rate) as usize;
        let comment_time_minutes = (code_stats.total_comment_lines as f64 * self.rates.comment_line_rate) as usize;
        let total_time_minutes = code_time_minutes + doc_time_minutes + comment_time_minutes;
        
        let mut time_by_extension = HashMap::new();
        
        for (ext, (file_count, file_stats)) in &code_stats.stats_by_extension {
            let ext_code_time = (file_stats.code_lines as f64 * self.rates.code_line_rate) as usize;
            let ext_doc_time = (file_stats.doc_lines as f64 * self.rates.doc_line_rate) as usize;
            let ext_comment_time = (file_stats.comment_lines as f64 * self.rates.comment_line_rate) as usize;
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
            lines_per_hour: (lines_per_hour * 1000.0).round() / 1000.0,
            code_lines_per_hour: (code_lines_per_hour * 1000.0).round() / 1000.0,
            files_per_hour: (files_per_hour * 1000.0).round() / 1000.0,
            estimated_development_days: (estimated_development_days * 1000.0).round() / 1000.0,
            estimated_development_hours: (estimated_development_hours * 1000.0).round() / 1000.0,
        }
    }
    
    /// Format time in human-readable format
    pub fn format_time_human(&self, minutes: usize) -> String {
        if minutes == 0 {
            return "0m".to_string();
        }
        
        if minutes < 60 {
            format!("{}m", minutes)
        } else if minutes < 1440 { // Less than a day
            let hours = minutes / 60;
            let remaining_minutes = minutes % 60;
            format!("{}h {}m", hours, remaining_minutes)
        } else {
            let days = minutes / 1440;
            let remaining_hours = (minutes % 1440) / 60;
            let remaining_minutes = minutes % 60;
            
            let mut result = format!("{}d", days);
            
            // Always show hours when there are days
            result.push_str(&format!(" {}h", remaining_hours));
            
            if remaining_minutes > 0 {
                result.push_str(&format!(" {}m", remaining_minutes));
            }
            
            result
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::test_utils::TestProject;
    use std::collections::HashMap;

    #[test]
    fn test_time_stats_calculator_creation() {
        let calculator = TimeStatsCalculator::new();
        assert_eq!(calculator.rates.code_line_rate, 0.2);
        assert_eq!(calculator.rates.doc_line_rate, 0.5);
        assert_eq!(calculator.rates.comment_line_rate, 0.1);
        assert_eq!(calculator.rates.blank_line_rate, 0.0);
    }

    #[test]
    fn test_time_stats_calculator_with_custom_rates() {
        let custom_rates = TimeEstimationRates {
            code_line_rate: 5.0,
            doc_line_rate: 8.0,
            comment_line_rate: 2.0,
            blank_line_rate: 0.0,
        };
        let calculator = TimeStatsCalculator::with_rates(custom_rates);
        assert_eq!(calculator.rates.code_line_rate, 5.0);
        assert_eq!(calculator.rates.doc_line_rate, 8.0);
        assert_eq!(calculator.rates.comment_line_rate, 2.0);
        assert_eq!(calculator.rates.blank_line_rate, 0.0);
    }

    #[test]
    fn test_calculate_time_stats_single_file() {
        let calculator = TimeStatsCalculator::new();
        let file_stats = FileStats {
            total_lines: 100,
            code_lines: 60,
            comment_lines: 20,
            doc_lines: 10,
            blank_lines: 10,
            file_size: 2048,
        };

        let result = calculator.calculate_time_stats(&file_stats).unwrap();

        // Expected: 60*0.2 + 20*0.1 + 10*0.5 = 12 + 2 + 5 = 19 minutes
        assert_eq!(result.total_time_minutes, 19);
        assert_eq!(result.code_time_minutes, 12);
        assert_eq!(result.comment_time_minutes, 2);
        assert_eq!(result.doc_time_minutes, 5);
        
        // Check formatted times
        assert_eq!(result.total_time_formatted, "19m");
        assert_eq!(result.code_time_formatted, "12m");
        assert_eq!(result.comment_time_formatted, "2m");
        assert_eq!(result.doc_time_formatted, "5m");
        
        // Check productivity metrics
        assert_eq!(result.productivity_metrics.lines_per_hour, 315.789); // 100 lines / 0.317 hours (rounded)
        assert_eq!(result.productivity_metrics.code_lines_per_hour, 189.474); // 60 lines / 0.317 hours (rounded)
        assert_eq!(result.productivity_metrics.files_per_hour, 3.158); // 1 file / 0.317 hours (rounded)
        assert_eq!(result.productivity_metrics.estimated_development_hours, 0.317); // 19 minutes / 60
        assert_eq!(result.productivity_metrics.estimated_development_days, 0.04); // 0.317 hours / 8
    }

    #[test]
    fn test_calculate_time_stats_empty_file() {
        let calculator = TimeStatsCalculator::new();
        let file_stats = FileStats {
            total_lines: 0,
            code_lines: 0,
            comment_lines: 0,
            doc_lines: 0,
            blank_lines: 0,
            file_size: 0,
        };

        let result = calculator.calculate_time_stats(&file_stats).unwrap();

        assert_eq!(result.total_time_minutes, 0);
        assert_eq!(result.code_time_minutes, 0);
        assert_eq!(result.comment_time_minutes, 0);
        assert_eq!(result.doc_time_minutes, 0);
        assert_eq!(result.total_time_formatted, "0m");
        assert_eq!(result.code_time_formatted, "0m");
        assert_eq!(result.comment_time_formatted, "0m");
        assert_eq!(result.doc_time_formatted, "0m");
    }

    #[test]
    fn test_calculate_project_time_stats() {
        let calculator = TimeStatsCalculator::new();
        
        let mut stats_by_extension = HashMap::new();
        stats_by_extension.insert("rs".to_string(), (2, FileStats {
            total_lines: 200,
            code_lines: 140,
            comment_lines: 40,
            doc_lines: 20,
            blank_lines: 20,
            file_size: 4000,
        }));
        stats_by_extension.insert("py".to_string(), (1, FileStats {
            total_lines: 100,
            code_lines: 70,
            comment_lines: 20,
            doc_lines: 5,
            blank_lines: 10,
            file_size: 2000,
        }));

        let code_stats = CodeStats {
            total_files: 3,
            total_lines: 300,
            total_code_lines: 210,
            total_comment_lines: 60,
            total_doc_lines: 25,
            total_blank_lines: 30,
            total_size: 6000,
            stats_by_extension,
        };

        let result = calculator.calculate_project_time_stats(&code_stats).unwrap();

        // Expected: 210*0.2 + 60*0.1 + 25*0.5 = 42 + 6 + 12.5 = 60.5 minutes
        assert_eq!(result.total_time_minutes, 60);
        assert_eq!(result.code_time_minutes, 42);
        assert_eq!(result.comment_time_minutes, 6);
        assert_eq!(result.doc_time_minutes, 12);
        
        // Check formatted times
        assert_eq!(result.total_time_formatted, "1h 0m");
        assert_eq!(result.code_time_formatted, "42m");
        assert_eq!(result.comment_time_formatted, "6m");
        assert_eq!(result.doc_time_formatted, "12m");

        // Check extension breakdown
        assert_eq!(result.time_by_extension.len(), 2);
        
        let rust_time = &result.time_by_extension["rs"];
        assert_eq!(rust_time.total_time_minutes, 42); // 140*0.2 + 40*0.1 + 20*0.5 = 28 + 4 + 10 = 42
        assert_eq!(rust_time.code_time_minutes, 28);
        assert_eq!(rust_time.comment_time_minutes, 4);
        assert_eq!(rust_time.doc_time_minutes, 10);
        assert_eq!(rust_time.average_time_per_file, 21.0);

        let python_time = &result.time_by_extension["py"];
        assert_eq!(python_time.total_time_minutes, 18); // 70*0.2 + 20*0.1 + 5*0.5 = 14 + 2 + 2.5 = 18.5 rounded to 18
        assert_eq!(python_time.code_time_minutes, 14);
        assert_eq!(python_time.comment_time_minutes, 2);
        assert_eq!(python_time.doc_time_minutes, 2); // 5*0.5 = 2.5 rounded to 2
        assert_eq!(python_time.average_time_per_file, 18.0);
    }

    #[test]
    fn test_format_time_human() {
        let calculator = TimeStatsCalculator::new();

        assert_eq!(calculator.format_time_human(0), "0m");
        assert_eq!(calculator.format_time_human(30), "30m");
        assert_eq!(calculator.format_time_human(60), "1h 0m");
        assert_eq!(calculator.format_time_human(90), "1h 30m");
        assert_eq!(calculator.format_time_human(120), "2h 0m");
        assert_eq!(calculator.format_time_human(1440), "1d 0h");
        assert_eq!(calculator.format_time_human(1500), "1d 1h");
        assert_eq!(calculator.format_time_human(10080), "7d 0h"); // 1 week = 7 days
        assert_eq!(calculator.format_time_human(43200), "30d 0h"); // 1 month ≈ 30 days
    }

    #[test]
    fn test_calculate_productivity_metrics() {
        let calculator = TimeStatsCalculator::new();

        // Test with 300 minutes (5 hours), 100 lines, 60 code lines, 2 files
        let metrics = calculator.calculate_productivity_metrics(300, 60, 100, 2);

        assert_eq!(metrics.lines_per_hour, 20.0); // 100 lines / 5 hours
        assert_eq!(metrics.code_lines_per_hour, 12.0); // 60 code lines / 5 hours
        assert_eq!(metrics.files_per_hour, 0.4); // 2 files / 5 hours
        assert_eq!(metrics.estimated_development_hours, 5.0);
        assert_eq!(metrics.estimated_development_days, 0.625); // 5 hours / 8 hours per day
    }

    #[test]
    fn test_calculate_productivity_metrics_zero_time() {
        let calculator = TimeStatsCalculator::new();

        let metrics = calculator.calculate_productivity_metrics(0, 0, 0, 0);

        assert_eq!(metrics.lines_per_hour, 0.0);
        assert_eq!(metrics.code_lines_per_hour, 0.0);
        assert_eq!(metrics.files_per_hour, 0.0);
        assert_eq!(metrics.estimated_development_hours, 0.0);
        assert_eq!(metrics.estimated_development_days, 0.0);
    }

    #[test]
    fn test_time_estimation_rates_default() {
        let rates = TimeEstimationRates::default();
        assert_eq!(rates.code_line_rate, 0.2);
        assert_eq!(rates.doc_line_rate, 0.5);
        assert_eq!(rates.comment_line_rate, 0.1);
        assert_eq!(rates.blank_line_rate, 0.0);
    }

    #[test]
    fn test_time_stats_serialization() {
        let time_stats = TimeStats {
            total_time_minutes: 300,
            code_time_minutes: 200,
            doc_time_minutes: 60,
            comment_time_minutes: 40,
            total_time_formatted: "5h 0m".to_string(),
            code_time_formatted: "3h 20m".to_string(),
            doc_time_formatted: "1h 0m".to_string(),
            comment_time_formatted: "40m".to_string(),
            time_by_extension: HashMap::new(),
            productivity_metrics: ProductivityMetrics {
                lines_per_hour: 20.0,
                code_lines_per_hour: 12.0,
                files_per_hour: 0.4,
                estimated_development_days: 0.625,
                estimated_development_hours: 5.0,
            },
        };

        // Test serialization to JSON
        let json = serde_json::to_string(&time_stats).unwrap();
        let deserialized: TimeStats = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.total_time_minutes, 300);
        assert_eq!(deserialized.code_time_minutes, 200);
        assert_eq!(deserialized.doc_time_minutes, 60);
        assert_eq!(deserialized.comment_time_minutes, 40);
        assert_eq!(deserialized.total_time_formatted, "5h 0m");
        assert_eq!(deserialized.productivity_metrics.lines_per_hour, 20.0);
    }

    #[test]
    fn test_extension_time_stats_serialization() {
        let ext_time_stats = ExtensionTimeStats {
            total_time_minutes: 180,
            code_time_minutes: 120,
            doc_time_minutes: 36,
            comment_time_minutes: 24,
            formatted_time: "3h 0m".to_string(),
            average_time_per_file: 90.0,
        };

        // Test serialization to JSON
        let json = serde_json::to_string(&ext_time_stats).unwrap();
        let deserialized: ExtensionTimeStats = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.total_time_minutes, 180);
        assert_eq!(deserialized.code_time_minutes, 120);
        assert_eq!(deserialized.doc_time_minutes, 36);
        assert_eq!(deserialized.comment_time_minutes, 24);
        assert_eq!(deserialized.formatted_time, "3h 0m");
        assert_eq!(deserialized.average_time_per_file, 90.0);
    }

    #[test]
    fn test_productivity_metrics_serialization() {
        let productivity = ProductivityMetrics {
            lines_per_hour: 25.5,
            code_lines_per_hour: 18.2,
            files_per_hour: 0.8,
            estimated_development_days: 1.25,
            estimated_development_hours: 10.0,
        };

        // Test serialization to JSON
        let json = serde_json::to_string(&productivity).unwrap();
        let deserialized: ProductivityMetrics = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.lines_per_hour, 25.5);
        assert_eq!(deserialized.code_lines_per_hour, 18.2);
        assert_eq!(deserialized.files_per_hour, 0.8);
        assert_eq!(deserialized.estimated_development_days, 1.25);
        assert_eq!(deserialized.estimated_development_hours, 10.0);
    }

    #[test]
    fn test_time_stats_edge_cases() {
        let calculator = TimeStatsCalculator::new();

        // Test with extremely large values
        let large_file_stats = FileStats {
            total_lines: 1000000,
            code_lines: 800000,
            comment_lines: 150000,
            doc_lines: 50000,
            blank_lines: 50000,
            file_size: 50000000,
        };

        let result = calculator.calculate_time_stats(&large_file_stats).unwrap();

        // Expected: 800000*0.2 + 150000*0.1 + 50000*0.5 = 160000 + 15000 + 25000 = 200000 minutes
        assert_eq!(result.total_time_minutes, 200000);
        assert_eq!(result.code_time_minutes, 160000);
        assert_eq!(result.comment_time_minutes, 15000);
        assert_eq!(result.doc_time_minutes, 25000);
        
        // Check that it handles large numbers correctly
        assert!(result.productivity_metrics.estimated_development_days > 0.0);
        assert!(result.productivity_metrics.estimated_development_hours > 0.0);
    }

    #[test]
    fn test_time_stats_with_custom_rates() {
        let custom_rates = TimeEstimationRates {
            code_line_rate: 10.0,
            doc_line_rate: 15.0,
            comment_line_rate: 5.0,
            blank_line_rate: 1.0,
        };
        let calculator = TimeStatsCalculator::with_rates(custom_rates);
        
        let file_stats = FileStats {
            total_lines: 100,
            code_lines: 60,
            comment_lines: 20,
            doc_lines: 10,
            blank_lines: 10,
            file_size: 2048,
        };

        let result = calculator.calculate_time_stats(&file_stats).unwrap();

        // Expected: 60*10 + 20*5 + 10*15 + 10*1 = 600 + 100 + 150 + 10 = 860 minutes
        assert_eq!(result.total_time_minutes, 860);
        assert_eq!(result.code_time_minutes, 600);
        assert_eq!(result.comment_time_minutes, 100);
        assert_eq!(result.doc_time_minutes, 150);
    }

    #[test]
    fn test_time_stats_with_real_project() {
        let project = TestProject::new("test_project").unwrap();
        
        // Create a realistic project structure
        project.create_rust_file("src/main.rs", 20, 10).unwrap();
        project.create_rust_file("src/lib.rs", 30, 15).unwrap();
        project.create_python_file("script.py", 25).unwrap();

        let calculator = TimeStatsCalculator::new();
        
        // Simulate realistic stats
        let mut stats_by_extension = HashMap::new();
        stats_by_extension.insert("rs".to_string(), (2, FileStats {
            total_lines: 150,
            code_lines: 100,
            comment_lines: 30,
            doc_lines: 15,
            blank_lines: 20,
            file_size: 3000,
        }));
        stats_by_extension.insert("py".to_string(), (1, FileStats {
            total_lines: 80,
            code_lines: 60,
            comment_lines: 15,
            doc_lines: 3,
            blank_lines: 5,
            file_size: 1500,
        }));

        let code_stats = CodeStats {
            total_files: 3,
            total_lines: 230,
            total_code_lines: 160,
            total_comment_lines: 45,
            total_doc_lines: 18,
            total_blank_lines: 25,
            total_size: 4500,
            stats_by_extension,
        };

        let result = calculator.calculate_project_time_stats(&code_stats).unwrap();

        // Expected: 160*0.2 + 45*0.1 + 18*0.5 = 32 + 4.5 + 9 = 45.5 minutes
        assert_eq!(result.total_time_minutes, 45);
        assert_eq!(result.code_time_minutes, 32);
        assert_eq!(result.comment_time_minutes, 4);
        assert_eq!(result.doc_time_minutes, 9);
        
        // Check that extension breakdown is correct
        assert_eq!(result.time_by_extension.len(), 2);
        assert!(result.time_by_extension.contains_key("rs"));
        assert!(result.time_by_extension.contains_key("py"));
        
        // Check productivity metrics are reasonable
        assert!(result.productivity_metrics.lines_per_hour > 0.0);
        assert!(result.productivity_metrics.code_lines_per_hour > 0.0);
        assert!(result.productivity_metrics.files_per_hour > 0.0);
        assert!(result.productivity_metrics.estimated_development_days > 0.0);
        assert!(result.productivity_metrics.estimated_development_hours > 0.0);
    }
} 