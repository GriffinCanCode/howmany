use crate::core::types::{CodeStats, FileStats};
use crate::utils::errors::Result;
use super::types::{RatioStats, ExtensionRatios, QualityThresholds};
use super::quality::QualityCalculator;
use std::collections::HashMap;

/// Helper function to round a floating-point value to 2 decimal places
fn round_to_2_decimals(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

/// Calculator for ratio and percentage statistics
pub struct RatioStatsCalculator {
    thresholds: QualityThresholds,
    quality_calculator: QualityCalculator,
}

impl RatioStatsCalculator {
    pub fn new() -> Self {
        let thresholds = QualityThresholds::default();
        let quality_calculator = QualityCalculator::new(thresholds.clone());
        Self {
            thresholds,
            quality_calculator,
        }
    }
    
    pub fn with_thresholds(thresholds: QualityThresholds) -> Self {
        let quality_calculator = QualityCalculator::new(thresholds.clone());
        Self { 
            thresholds,
            quality_calculator,
        }
    }
    
    /// Calculate ratio statistics for a single file
    pub fn calculate_ratio_stats(&self, file_stats: &FileStats) -> Result<RatioStats> {
        let total_lines = file_stats.total_lines as f64;
        
        // Calculate ratios as 0-1 values (not percentages) for quality calculations
        let code_ratio = if total_lines > 0.0 { 
            round_to_2_decimals(file_stats.code_lines as f64 / total_lines)
        } else { 
            0.0 
        };
        let comment_ratio = if total_lines > 0.0 { 
            round_to_2_decimals(file_stats.comment_lines as f64 / total_lines)
        } else { 
            0.0 
        };
        let doc_ratio = if total_lines > 0.0 { 
            round_to_2_decimals(file_stats.doc_lines as f64 / total_lines)
        } else { 
            0.0 
        };
        let blank_ratio = if total_lines > 0.0 { 
            round_to_2_decimals(file_stats.blank_lines as f64 / total_lines)
        } else { 
            0.0 
        };
        
        let comment_to_code_ratio = if file_stats.code_lines > 0 {
            round_to_2_decimals(file_stats.comment_lines as f64 / file_stats.code_lines as f64)
        } else {
            0.0
        };
        
        let doc_to_code_ratio = if file_stats.code_lines > 0 {
            round_to_2_decimals(file_stats.doc_lines as f64 / file_stats.code_lines as f64)
        } else {
            0.0
        };
        
        let quality_metrics = self.quality_calculator.calculate_quality_metrics(
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
        
        // Calculate ratios as 0-1 values (not percentages) for quality calculations
        let code_ratio = if total_lines > 0.0 { 
            round_to_2_decimals(code_stats.total_code_lines as f64 / total_lines)
        } else { 
            0.0 
        };
        let comment_ratio = if total_lines > 0.0 { 
            round_to_2_decimals(code_stats.total_comment_lines as f64 / total_lines)
        } else { 
            0.0 
        };
        let doc_ratio = if total_lines > 0.0 { 
            round_to_2_decimals(code_stats.total_doc_lines as f64 / total_lines)
        } else { 
            0.0 
        };
        let blank_ratio = if total_lines > 0.0 { 
            round_to_2_decimals(code_stats.total_blank_lines as f64 / total_lines)
        } else { 
            0.0 
        };
        
        let comment_to_code_ratio = if code_stats.total_code_lines > 0 {
            round_to_2_decimals(code_stats.total_comment_lines as f64 / code_stats.total_code_lines as f64)
        } else {
            0.0
        };
        
        let doc_to_code_ratio = if code_stats.total_code_lines > 0 {
            round_to_2_decimals(code_stats.total_doc_lines as f64 / code_stats.total_code_lines as f64)
        } else {
            0.0
        };
        
        // Calculate per-extension ratios
        let mut ratios_by_extension = HashMap::new();
        
        for (ext, (file_count, file_stats)) in &code_stats.stats_by_extension {
            let ext_total_lines = file_stats.total_lines as f64;
            
            let ext_ratios = ExtensionRatios {
                code_ratio: if ext_total_lines > 0.0 { 
                    round_to_2_decimals(file_stats.code_lines as f64 / ext_total_lines)
                } else { 
                    0.0 
                },
                comment_ratio: if ext_total_lines > 0.0 { 
                    round_to_2_decimals(file_stats.comment_lines as f64 / ext_total_lines)
                } else { 
                    0.0 
                },
                doc_ratio: if ext_total_lines > 0.0 { 
                    round_to_2_decimals(file_stats.doc_lines as f64 / ext_total_lines)
                } else { 
                    0.0 
                },
                blank_ratio: if ext_total_lines > 0.0 { 
                    round_to_2_decimals(file_stats.blank_lines as f64 / ext_total_lines)
                } else { 
                    0.0 
                },
                comment_to_code_ratio: if file_stats.code_lines > 0 {
                    round_to_2_decimals(file_stats.comment_lines as f64 / file_stats.code_lines as f64)
                } else {
                    0.0
                },
                doc_to_code_ratio: if file_stats.code_lines > 0 {
                    round_to_2_decimals(file_stats.doc_lines as f64 / file_stats.code_lines as f64)
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
        
        let quality_metrics = self.quality_calculator.calculate_quality_metrics(
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
                let percentage = ((file_stats.total_lines as f64 / total_lines) * 100.0 * 100.0).round() / 100.0;
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
                let percentage = ((*file_count as f64 / total_files) * 100.0 * 100.0).round() / 100.0;
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
                let percentage = ((file_stats.file_size as f64 / total_size) * 100.0 * 100.0).round() / 100.0;
                distribution.insert(ext.clone(), percentage);
            }
        }
        
        distribution
    }
    
    /// Get thresholds for customization
    pub fn get_thresholds(&self) -> &QualityThresholds {
        &self.thresholds
    }
    
    /// Update thresholds
    pub fn set_thresholds(&mut self, thresholds: QualityThresholds) {
        self.thresholds = thresholds.clone();
        self.quality_calculator = QualityCalculator::new(thresholds);
    }
    
    /// Get quality level description
    pub fn get_quality_level(&self, score: f64) -> String {
        self.quality_calculator.get_quality_level(score)
    }
    
    /// Get quality level CSS class
    pub fn get_quality_class(&self, score: f64) -> String {
        self.quality_calculator.get_quality_class(score)
    }
}

impl Default for RatioStatsCalculator {
    fn default() -> Self {
        Self::new()
    }
} 

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::stats::ratios::types::QualityMetrics;
    use crate::testing::test_utils::TestProject;
    use std::collections::HashMap;

    #[test]
    fn test_ratio_stats_calculator_creation() {
        let calculator = RatioStatsCalculator::new();
        // Test that the calculator can be created without issues
        assert!(true); // Basic creation test
    }

    #[test]
    fn test_ratio_stats_calculator_with_custom_thresholds() {
        let custom_thresholds = QualityThresholds {
            good_comment_ratio: 0.25,
            good_doc_ratio: 0.15,
            max_blank_ratio: 0.35,
            ideal_comment_to_code: 0.25,
            ideal_doc_to_code: 0.20,
        };
        let calculator = RatioStatsCalculator::with_thresholds(custom_thresholds);
        assert_eq!(calculator.thresholds.good_comment_ratio, 0.25);
        assert_eq!(calculator.thresholds.good_doc_ratio, 0.15);
    }

    #[test]
    fn test_calculate_ratio_stats_single_file() {
        let calculator = RatioStatsCalculator::new();
        let file_stats = FileStats {
            total_lines: 100,
            code_lines: 60,
            comment_lines: 20,
            doc_lines: 10,
            blank_lines: 10,
            file_size: 2048,
        };

        let result = calculator.calculate_ratio_stats(&file_stats).unwrap();

        assert_eq!(result.code_ratio, 0.6); // 60/100
        assert_eq!(result.comment_ratio, 0.2); // 20/100
        assert_eq!(result.doc_ratio, 0.1); // 10/100
        assert_eq!(result.blank_ratio, 0.1); // 10/100
        assert!((result.comment_to_code_ratio - 0.33).abs() < 0.01); // 20/60 rounded to 2 decimal places
        assert!((result.doc_to_code_ratio - 0.17).abs() < 0.01); // 10/60 rounded to 2 decimal places
        
        // Check that ratios_by_extension is empty for single file
        assert!(result.ratios_by_extension.is_empty());
        
        // Check that distributions are empty for single file
        assert!(result.language_distribution.is_empty());
        assert!(result.file_distribution.is_empty());
        assert!(result.size_distribution.is_empty());
        
        // Check quality metrics are calculated
        assert!(result.quality_metrics.overall_quality_score > 0.0);
        assert!(result.quality_metrics.documentation_score > 0.0);
        assert!(result.quality_metrics.maintainability_score > 0.0);
        assert!(result.quality_metrics.readability_score > 0.0);
        assert!(result.quality_metrics.consistency_score > 0.0);
    }

    #[test]
    fn test_calculate_ratio_stats_empty_file() {
        let calculator = RatioStatsCalculator::new();
        let file_stats = FileStats {
            total_lines: 0,
            code_lines: 0,
            comment_lines: 0,
            doc_lines: 0,
            blank_lines: 0,
            file_size: 0,
        };

        let result = calculator.calculate_ratio_stats(&file_stats).unwrap();

        assert_eq!(result.code_ratio, 0.0);
        assert_eq!(result.comment_ratio, 0.0);
        assert_eq!(result.doc_ratio, 0.0);
        assert_eq!(result.blank_ratio, 0.0);
        assert_eq!(result.comment_to_code_ratio, 0.0);
        assert_eq!(result.doc_to_code_ratio, 0.0);
        
        // Quality metrics should handle zero values gracefully
        assert!(result.quality_metrics.overall_quality_score >= 0.0);
        assert!(result.quality_metrics.documentation_score >= 0.0);
        assert!(result.quality_metrics.maintainability_score >= 0.0);
        assert!(result.quality_metrics.readability_score >= 0.0);
        assert!(result.quality_metrics.consistency_score >= 0.0);
    }

    #[test]
    fn test_calculate_project_ratio_stats() {
        let calculator = RatioStatsCalculator::new();
        
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

        let result = calculator.calculate_project_ratio_stats(&code_stats).unwrap();

        assert_eq!(result.code_ratio, 0.7); // 210/300
        assert_eq!(result.comment_ratio, 0.2); // 60/300
        assert!((result.doc_ratio - 0.08).abs() < 0.01); // 25/300 rounded to 2 decimal places
        assert_eq!(result.blank_ratio, 0.1); // 30/300
        assert!((result.comment_to_code_ratio - 0.29).abs() < 0.01); // 60/210 rounded to 2 decimal places
        assert!((result.doc_to_code_ratio - 0.12).abs() < 0.01); // 25/210 rounded to 2 decimal places

        // Check extension ratios
        assert_eq!(result.ratios_by_extension.len(), 2);
        
        let rust_ratios = &result.ratios_by_extension["rs"];
        assert_eq!(rust_ratios.code_ratio, 0.7); // 140/200
        assert_eq!(rust_ratios.comment_ratio, 0.2); // 40/200
        assert_eq!(rust_ratios.doc_ratio, 0.1); // 20/200
        assert_eq!(rust_ratios.blank_ratio, 0.1); // 20/200
        assert!((rust_ratios.comment_to_code_ratio - 0.29).abs() < 0.01); // 40/140 rounded
        assert!((rust_ratios.doc_to_code_ratio - 0.14).abs() < 0.01); // 20/140 rounded
        assert_eq!(rust_ratios.lines_per_file, 100.0); // 200/2
        assert_eq!(rust_ratios.size_per_file, 2000.0); // 4000/2

        let python_ratios = &result.ratios_by_extension["py"];
        assert_eq!(python_ratios.code_ratio, 0.7); // 70/100
        assert_eq!(python_ratios.comment_ratio, 0.2); // 20/100
        assert_eq!(python_ratios.doc_ratio, 0.05); // 5/100
        assert_eq!(python_ratios.blank_ratio, 0.1); // 10/100
        assert_eq!(python_ratios.comment_to_code_ratio, 0.29); // 20/70 rounded
        assert_eq!(python_ratios.doc_to_code_ratio, 0.07); // 5/70 rounded
        assert_eq!(python_ratios.lines_per_file, 100.0); // 100/1
        assert_eq!(python_ratios.size_per_file, 2000.0); // 2000/1

        // Check language distribution
        assert_eq!(result.language_distribution.len(), 2);
        assert_eq!(result.language_distribution["rs"], 66.67); // 200/300 * 100 rounded
        assert_eq!(result.language_distribution["py"], 33.33); // 100/300 * 100 rounded

        // Check file distribution
        assert_eq!(result.file_distribution.len(), 2);
        assert_eq!(result.file_distribution["rs"], 66.67); // 2/3 * 100 rounded
        assert_eq!(result.file_distribution["py"], 33.33); // 1/3 * 100 rounded

        // Check size distribution
        assert_eq!(result.size_distribution.len(), 2);
        assert_eq!(result.size_distribution["rs"], 66.67); // 4000/6000 * 100 rounded
        assert_eq!(result.size_distribution["py"], 33.33); // 2000/6000 * 100 rounded
    }

    #[test]
    fn test_calculate_project_ratio_stats_empty_project() {
        let calculator = RatioStatsCalculator::new();
        let code_stats = CodeStats {
            total_files: 0,
            total_lines: 0,
            total_code_lines: 0,
            total_comment_lines: 0,
            total_doc_lines: 0,
            total_blank_lines: 0,
            total_size: 0,
            stats_by_extension: HashMap::new(),
        };

        let result = calculator.calculate_project_ratio_stats(&code_stats).unwrap();

        assert_eq!(result.code_ratio, 0.0);
        assert_eq!(result.comment_ratio, 0.0);
        assert_eq!(result.doc_ratio, 0.0);
        assert_eq!(result.blank_ratio, 0.0);
        assert_eq!(result.comment_to_code_ratio, 0.0);
        assert_eq!(result.doc_to_code_ratio, 0.0);
        
        assert!(result.ratios_by_extension.is_empty());
        assert!(result.language_distribution.is_empty());
        assert!(result.file_distribution.is_empty());
        assert!(result.size_distribution.is_empty());
    }

    #[test]
    fn test_calculate_project_ratio_stats_single_extension() {
        let calculator = RatioStatsCalculator::new();
        
        let mut stats_by_extension = HashMap::new();
        stats_by_extension.insert("js".to_string(), (3, FileStats {
            total_lines: 300,
            code_lines: 200,
            comment_lines: 60,
            doc_lines: 20,
            blank_lines: 40,
            file_size: 6000,
        }));

        let code_stats = CodeStats {
            total_files: 3,
            total_lines: 300,
            total_code_lines: 200,
            total_comment_lines: 60,
            total_doc_lines: 20,
            total_blank_lines: 40,
            total_size: 6000,
            stats_by_extension,
        };

        let result = calculator.calculate_project_ratio_stats(&code_stats).unwrap();

        assert!((result.code_ratio - 0.67).abs() < 0.01); // 200/300 rounded
        assert_eq!(result.comment_ratio, 0.2); // 60/300
        assert!((result.doc_ratio - 0.07).abs() < 0.01); // 20/300 rounded
        assert!((result.blank_ratio - 0.13).abs() < 0.01); // 40/300 rounded
        assert_eq!(result.comment_to_code_ratio, 0.3); // 60/200
        assert_eq!(result.doc_to_code_ratio, 0.1); // 20/200

        // Check single extension
        assert_eq!(result.ratios_by_extension.len(), 1);
        let js_ratios = &result.ratios_by_extension["js"];
        assert!((js_ratios.code_ratio - 0.67).abs() < 0.01);
        assert_eq!(js_ratios.comment_ratio, 0.2);
        assert_eq!(js_ratios.doc_ratio, 0.07);
        assert_eq!(js_ratios.blank_ratio, 0.13);
        assert_eq!(js_ratios.lines_per_file, 100.0);
        assert_eq!(js_ratios.size_per_file, 2000.0);

        // Check distributions
        assert_eq!(result.language_distribution.len(), 1);
        assert_eq!(result.language_distribution["js"], 100.0);
        assert_eq!(result.file_distribution.len(), 1);
        assert_eq!(result.file_distribution["js"], 100.0);
        assert_eq!(result.size_distribution.len(), 1);
        assert_eq!(result.size_distribution["js"], 100.0);
    }

    #[test]
    fn test_extension_ratios_creation() {
        let ext_ratios = ExtensionRatios {
            code_ratio: 0.7,
            comment_ratio: 0.2,
            doc_ratio: 0.05,
            blank_ratio: 0.05,
            comment_to_code_ratio: 0.29,
            doc_to_code_ratio: 0.07,
            lines_per_file: 100.0,
            size_per_file: 2000.0,
        };

        assert_eq!(ext_ratios.code_ratio, 0.7);
        assert_eq!(ext_ratios.comment_ratio, 0.2);
        assert_eq!(ext_ratios.doc_ratio, 0.05);
        assert_eq!(ext_ratios.blank_ratio, 0.05);
        assert_eq!(ext_ratios.comment_to_code_ratio, 0.29);
        assert_eq!(ext_ratios.doc_to_code_ratio, 0.07);
        assert_eq!(ext_ratios.lines_per_file, 100.0);
        assert_eq!(ext_ratios.size_per_file, 2000.0);
    }

    #[test]
    fn test_ratio_stats_serialization() {
        let ratio_stats = RatioStats {
            code_ratio: 0.7,
            comment_ratio: 0.2,
            doc_ratio: 0.05,
            blank_ratio: 0.05,
            comment_to_code_ratio: 0.29,
            doc_to_code_ratio: 0.07,
            ratios_by_extension: HashMap::new(),
            language_distribution: HashMap::new(),
            file_distribution: HashMap::new(),
            size_distribution: HashMap::new(),
            quality_metrics: QualityMetrics {
                documentation_score: 75.0,
                maintainability_score: 80.0,
                readability_score: 70.0,
                consistency_score: 85.0,
                overall_quality_score: 77.5,
            },
        };

        // Test serialization to JSON
        let json = serde_json::to_string(&ratio_stats).unwrap();
        let deserialized: RatioStats = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.code_ratio, 0.7);
        assert_eq!(deserialized.comment_ratio, 0.2);
        assert_eq!(deserialized.doc_ratio, 0.05);
        assert_eq!(deserialized.blank_ratio, 0.05);
        assert_eq!(deserialized.comment_to_code_ratio, 0.29);
        assert_eq!(deserialized.doc_to_code_ratio, 0.07);
        assert_eq!(deserialized.quality_metrics.documentation_score, 75.0);
        assert_eq!(deserialized.quality_metrics.maintainability_score, 80.0);
        assert_eq!(deserialized.quality_metrics.readability_score, 70.0);
        assert_eq!(deserialized.quality_metrics.consistency_score, 85.0);
        assert_eq!(deserialized.quality_metrics.overall_quality_score, 77.5);
    }

    #[test]
    fn test_extension_ratios_serialization() {
        let ext_ratios = ExtensionRatios {
            code_ratio: 0.8,
            comment_ratio: 0.15,
            doc_ratio: 0.03,
            blank_ratio: 0.02,
            comment_to_code_ratio: 0.19,
            doc_to_code_ratio: 0.04,
            lines_per_file: 120.0,
            size_per_file: 2400.0,
        };

        // Test serialization to JSON
        let json = serde_json::to_string(&ext_ratios).unwrap();
        let deserialized: ExtensionRatios = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.code_ratio, 0.8);
        assert_eq!(deserialized.comment_ratio, 0.15);
        assert_eq!(deserialized.doc_ratio, 0.03);
        assert_eq!(deserialized.blank_ratio, 0.02);
        assert_eq!(deserialized.comment_to_code_ratio, 0.19);
        assert_eq!(deserialized.doc_to_code_ratio, 0.04);
        assert_eq!(deserialized.lines_per_file, 120.0);
        assert_eq!(deserialized.size_per_file, 2400.0);
    }

    #[test]
    fn test_quality_metrics_serialization() {
        let quality_metrics = QualityMetrics {
            documentation_score: 85.0,
            maintainability_score: 90.0,
            readability_score: 80.0,
            consistency_score: 95.0,
            overall_quality_score: 87.5,
        };

        // Test serialization to JSON
        let json = serde_json::to_string(&quality_metrics).unwrap();
        let deserialized: QualityMetrics = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.documentation_score, 85.0);
        assert_eq!(deserialized.maintainability_score, 90.0);
        assert_eq!(deserialized.readability_score, 80.0);
        assert_eq!(deserialized.consistency_score, 95.0);
        assert_eq!(deserialized.overall_quality_score, 87.5);
    }

    #[test]
    fn test_ratio_stats_edge_cases() {
        let calculator = RatioStatsCalculator::new();

        // Test with all code, no comments
        let code_only_stats = FileStats {
            total_lines: 100,
            code_lines: 100,
            comment_lines: 0,
            doc_lines: 0,
            blank_lines: 0,
            file_size: 2000,
        };

        let result = calculator.calculate_ratio_stats(&code_only_stats).unwrap();
        assert_eq!(result.code_ratio, 1.0);
        assert_eq!(result.comment_ratio, 0.0);
        assert_eq!(result.doc_ratio, 0.0);
        assert_eq!(result.blank_ratio, 0.0);
        assert_eq!(result.comment_to_code_ratio, 0.0);
        assert_eq!(result.doc_to_code_ratio, 0.0);

        // Test with all comments, no code
        let comments_only_stats = FileStats {
            total_lines: 100,
            code_lines: 0,
            comment_lines: 100,
            doc_lines: 0,
            blank_lines: 0,
            file_size: 2000,
        };

        let result = calculator.calculate_ratio_stats(&comments_only_stats).unwrap();
        assert_eq!(result.code_ratio, 0.0);
        assert_eq!(result.comment_ratio, 1.0);
        assert_eq!(result.doc_ratio, 0.0);
        assert_eq!(result.blank_ratio, 0.0);
        assert_eq!(result.comment_to_code_ratio, 0.0); // No code to divide by
        assert_eq!(result.doc_to_code_ratio, 0.0); // No code to divide by
    }

    #[test]
    fn test_ratio_stats_with_real_project() {
        let project = TestProject::new("test_project").unwrap();
        
        // Create a realistic project structure
        project.create_rust_file("src/main.rs", 15, 8).unwrap();
        project.create_rust_file("src/lib.rs", 25, 12).unwrap();
        project.create_python_file("script.py", 20).unwrap();
        project.create_javascript_file("app.js", 18).unwrap();

        let calculator = RatioStatsCalculator::new();
        
        // Simulate realistic project stats
        let mut stats_by_extension = HashMap::new();
        stats_by_extension.insert("rs".to_string(), (2, FileStats {
            total_lines: 200,
            code_lines: 120,
            comment_lines: 50,
            doc_lines: 20,
            blank_lines: 30,
            file_size: 4000,
        }));
        stats_by_extension.insert("py".to_string(), (1, FileStats {
            total_lines: 100,
            code_lines: 75,
            comment_lines: 15,
            doc_lines: 5,
            blank_lines: 10,
            file_size: 2000,
        }));
        stats_by_extension.insert("js".to_string(), (1, FileStats {
            total_lines: 120,
            code_lines: 85,
            comment_lines: 20,
            doc_lines: 10,
            blank_lines: 15,
            file_size: 2400,
        }));

        let code_stats = CodeStats {
            total_files: 4,
            total_lines: 420,
            total_code_lines: 280,
            total_comment_lines: 85,
            total_doc_lines: 35,
            total_blank_lines: 55,
            total_size: 8400,
            stats_by_extension,
        };

        let result = calculator.calculate_project_ratio_stats(&code_stats).unwrap();

        // Check overall ratios
        assert!((result.code_ratio - 0.67).abs() < 0.01); // 280/420 rounded
        assert!((result.comment_ratio - 0.2).abs() < 0.01); // 85/420 rounded
        assert!((result.doc_ratio - 0.08).abs() < 0.01); // 35/420 rounded
        assert_eq!(result.blank_ratio, 0.13); // 55/420 rounded
        assert_eq!(result.comment_to_code_ratio, 0.3); // 85/280 rounded
        assert!((result.doc_to_code_ratio - 0.13).abs() < 0.01); // 35/280 rounded

        // Check that all extensions are present
        assert_eq!(result.ratios_by_extension.len(), 3);
        assert!(result.ratios_by_extension.contains_key("rs"));
        assert!(result.ratios_by_extension.contains_key("py"));
        assert!(result.ratios_by_extension.contains_key("js"));

        // Check distributions
        assert_eq!(result.language_distribution.len(), 3);
        assert_eq!(result.file_distribution.len(), 3);
        assert_eq!(result.size_distribution.len(), 3);

        // Check that quality metrics are reasonable
        assert!(result.quality_metrics.overall_quality_score > 0.0);
        assert!(result.quality_metrics.documentation_score > 0.0);
        assert!(result.quality_metrics.maintainability_score > 0.0);
        assert!(result.quality_metrics.readability_score > 0.0);
        assert!(result.quality_metrics.consistency_score > 0.0);
    }

    #[test]
    fn test_quality_thresholds_default() {
        let thresholds = QualityThresholds::default();
        assert_eq!(thresholds.good_comment_ratio, 0.15);
        assert_eq!(thresholds.good_doc_ratio, 0.10);
        assert_eq!(thresholds.max_blank_ratio, 0.30);
        assert_eq!(thresholds.ideal_comment_to_code, 0.20);
        assert_eq!(thresholds.ideal_doc_to_code, 0.15);
    }
} 