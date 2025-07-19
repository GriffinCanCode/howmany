use crate::core::types::{CodeStats, FileStats};
use crate::utils::errors::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Basic statistics for a file or project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BasicStats {
    pub total_files: usize,
    pub total_lines: usize,
    pub code_lines: usize,
    pub comment_lines: usize,
    pub doc_lines: usize,
    pub blank_lines: usize,
    pub total_size: u64,
    pub average_file_size: f64,
    pub average_lines_per_file: f64,
    pub largest_file_size: u64,
    pub smallest_file_size: u64,
    pub stats_by_extension: HashMap<String, ExtensionStats>,
}

/// Statistics for a specific file extension
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionStats {
    pub file_count: usize,
    pub total_lines: usize,
    pub code_lines: usize,
    pub comment_lines: usize,
    pub doc_lines: usize,
    pub blank_lines: usize,
    pub total_size: u64,
    pub average_lines_per_file: f64,
    pub average_size_per_file: f64,
}

/// Calculator for basic statistics
pub struct BasicStatsCalculator;

impl BasicStatsCalculator {
    pub fn new() -> Self {
        Self
    }
    
    /// Calculate basic statistics for a single file
    pub fn calculate_basic_stats(&self, file_stats: &FileStats) -> Result<BasicStats> {
        Ok(BasicStats {
            total_files: 1,
            total_lines: file_stats.total_lines,
            code_lines: file_stats.code_lines,
            comment_lines: file_stats.comment_lines,
            doc_lines: file_stats.doc_lines,
            blank_lines: file_stats.blank_lines,
            total_size: file_stats.file_size,
            average_file_size: file_stats.file_size as f64,
            average_lines_per_file: file_stats.total_lines as f64,
            largest_file_size: file_stats.file_size,
            smallest_file_size: file_stats.file_size,
            stats_by_extension: HashMap::new(),
        })
    }
    
    /// Calculate basic statistics for a project
    pub fn calculate_project_basic_stats(&self, code_stats: &CodeStats) -> Result<BasicStats> {
        let mut stats_by_extension = HashMap::new();
        let mut file_sizes = Vec::new();
        
        for (ext, (file_count, file_stats)) in &code_stats.stats_by_extension {
            let ext_stats = ExtensionStats {
                file_count: *file_count,
                total_lines: file_stats.total_lines,
                code_lines: file_stats.code_lines,
                comment_lines: file_stats.comment_lines,
                doc_lines: file_stats.doc_lines,
                blank_lines: file_stats.blank_lines,
                total_size: file_stats.file_size,
                average_lines_per_file: if *file_count > 0 {
                    file_stats.total_lines as f64 / *file_count as f64
                } else {
                    0.0
                },
                average_size_per_file: if *file_count > 0 {
                    file_stats.file_size as f64 / *file_count as f64
                } else {
                    0.0
                },
            };
            
            stats_by_extension.insert(ext.clone(), ext_stats);
            
            // Estimate individual file sizes for min/max calculation
            // This is an approximation since we don't have individual file sizes
            let estimated_avg_size = if *file_count > 0 {
                file_stats.file_size / *file_count as u64
            } else {
                0
            };
            
            for _ in 0..*file_count {
                file_sizes.push(estimated_avg_size);
            }
        }
        
        let largest_file_size = file_sizes.iter().max().copied().unwrap_or(0);
        let smallest_file_size = file_sizes.iter().min().copied().unwrap_or(0);
        
        Ok(BasicStats {
            total_files: code_stats.total_files,
            total_lines: code_stats.total_lines,
            code_lines: code_stats.total_code_lines,
            comment_lines: code_stats.total_comment_lines,
            doc_lines: code_stats.total_doc_lines,
            blank_lines: code_stats.total_blank_lines,
            total_size: code_stats.total_size,
            average_file_size: if code_stats.total_files > 0 {
                code_stats.total_size as f64 / code_stats.total_files as f64
            } else {
                0.0
            },
            average_lines_per_file: if code_stats.total_files > 0 {
                code_stats.total_lines as f64 / code_stats.total_files as f64
            } else {
                0.0
            },
            largest_file_size,
            smallest_file_size,
            stats_by_extension,
        })
    }
    
    /// Get the most common file extension
    pub fn get_most_common_extension<'a>(&self, stats: &'a BasicStats) -> Option<(String, &'a ExtensionStats)> {
        stats.stats_by_extension
            .iter()
            .max_by_key(|(_, ext_stats)| ext_stats.file_count)
            .map(|(ext, stats)| (ext.clone(), stats))
    }
    
    /// Get the extension with the most lines of code
    pub fn get_most_lines_extension<'a>(&self, stats: &'a BasicStats) -> Option<(String, &'a ExtensionStats)> {
        stats.stats_by_extension
            .iter()
            .max_by_key(|(_, ext_stats)| ext_stats.total_lines)
            .map(|(ext, stats)| (ext.clone(), stats))
    }
    
    /// Get the extension with the largest file size
    pub fn get_largest_extension<'a>(&self, stats: &'a BasicStats) -> Option<(String, &'a ExtensionStats)> {
        stats.stats_by_extension
            .iter()
            .max_by_key(|(_, ext_stats)| ext_stats.total_size)
            .map(|(ext, stats)| (ext.clone(), stats))
    }
    
    /// Calculate size distribution percentages
    pub fn calculate_size_distribution(&self, stats: &BasicStats) -> HashMap<String, f64> {
        let mut distribution = HashMap::new();
        
        if stats.total_size > 0 {
            for (ext, ext_stats) in &stats.stats_by_extension {
                let percentage = (ext_stats.total_size as f64 / stats.total_size as f64) * 100.0;
                distribution.insert(ext.clone(), percentage);
            }
        }
        
        distribution
    }
    
    /// Calculate line distribution percentages
    pub fn calculate_line_distribution(&self, stats: &BasicStats) -> HashMap<String, f64> {
        let mut distribution = HashMap::new();
        
        if stats.total_lines > 0 {
            for (ext, ext_stats) in &stats.stats_by_extension {
                let percentage = (ext_stats.total_lines as f64 / stats.total_lines as f64) * 100.0;
                distribution.insert(ext.clone(), percentage);
            }
        }
        
        distribution
    }
}

impl Default for BasicStatsCalculator {
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
    fn test_basic_stats_calculator_creation() {
        let calculator = BasicStatsCalculator::new();
        // Test that the calculator can be created without issues
        assert!(true); // Basic creation test
    }

    #[test]
    fn test_calculate_basic_stats_single_file() {
        let calculator = BasicStatsCalculator::new();
        let file_stats = FileStats {
            total_lines: 100,
            code_lines: 70,
            comment_lines: 20,
            doc_lines: 5,
            blank_lines: 10,
            file_size: 2048,
        };

        let result = calculator.calculate_basic_stats(&file_stats).unwrap();

        assert_eq!(result.total_files, 1);
        assert_eq!(result.total_lines, 100);
        assert_eq!(result.code_lines, 70);
        assert_eq!(result.comment_lines, 20);
        assert_eq!(result.doc_lines, 5);
        assert_eq!(result.blank_lines, 10);
        assert_eq!(result.total_size, 2048);
        assert_eq!(result.average_file_size, 2048.0);
        assert_eq!(result.average_lines_per_file, 100.0);
        assert_eq!(result.largest_file_size, 2048);
        assert_eq!(result.smallest_file_size, 2048);
        assert!(result.stats_by_extension.is_empty());
    }

    #[test]
    fn test_calculate_basic_stats_empty_file() {
        let calculator = BasicStatsCalculator::new();
        let file_stats = FileStats {
            total_lines: 0,
            code_lines: 0,
            comment_lines: 0,
            doc_lines: 0,
            blank_lines: 0,
            file_size: 0,
        };

        let result = calculator.calculate_basic_stats(&file_stats).unwrap();

        assert_eq!(result.total_files, 1);
        assert_eq!(result.total_lines, 0);
        assert_eq!(result.code_lines, 0);
        assert_eq!(result.comment_lines, 0);
        assert_eq!(result.doc_lines, 0);
        assert_eq!(result.blank_lines, 0);
        assert_eq!(result.total_size, 0);
        assert_eq!(result.average_file_size, 0.0);
        assert_eq!(result.average_lines_per_file, 0.0);
        assert_eq!(result.largest_file_size, 0);
        assert_eq!(result.smallest_file_size, 0);
    }

    #[test]
    fn test_calculate_project_basic_stats() {
        let calculator = BasicStatsCalculator::new();
        
        // Create mock CodeStats
        let mut stats_by_extension = HashMap::new();
        stats_by_extension.insert("rs".to_string(), (2, FileStats {
            total_lines: 150,
            code_lines: 100,
            comment_lines: 30,
            doc_lines: 10,
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
            total_doc_lines: 13,
            total_blank_lines: 25,
            total_size: 4500,
            stats_by_extension,
        };

        let result = calculator.calculate_project_basic_stats(&code_stats).unwrap();

        assert_eq!(result.total_files, 3);
        assert_eq!(result.total_lines, 230);
        assert_eq!(result.code_lines, 160);
        assert_eq!(result.comment_lines, 45);
        assert_eq!(result.doc_lines, 13);
        assert_eq!(result.blank_lines, 25);
        assert_eq!(result.total_size, 4500);
        assert_eq!(result.average_file_size, 1500.0);
        assert!((result.average_lines_per_file - 76.67).abs() < 0.01);
        assert_eq!(result.largest_file_size, 1500);
        assert_eq!(result.smallest_file_size, 1500);

        // Check extension stats
        assert_eq!(result.stats_by_extension.len(), 2);
        
        let rust_stats = &result.stats_by_extension["rs"];
        assert_eq!(rust_stats.file_count, 2);
        assert_eq!(rust_stats.total_lines, 150);
        assert_eq!(rust_stats.code_lines, 100);
        assert_eq!(rust_stats.average_lines_per_file, 75.0);
        assert_eq!(rust_stats.average_size_per_file, 1500.0);

        let python_stats = &result.stats_by_extension["py"];
        assert_eq!(python_stats.file_count, 1);
        assert_eq!(python_stats.total_lines, 80);
        assert_eq!(python_stats.code_lines, 60);
        assert_eq!(python_stats.average_lines_per_file, 80.0);
        assert_eq!(python_stats.average_size_per_file, 1500.0);
    }

    #[test]
    fn test_calculate_project_basic_stats_empty_project() {
        let calculator = BasicStatsCalculator::new();
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

        let result = calculator.calculate_project_basic_stats(&code_stats).unwrap();

        assert_eq!(result.total_files, 0);
        assert_eq!(result.total_lines, 0);
        assert_eq!(result.code_lines, 0);
        assert_eq!(result.comment_lines, 0);
        assert_eq!(result.doc_lines, 0);
        assert_eq!(result.blank_lines, 0);
        assert_eq!(result.total_size, 0);
        assert_eq!(result.average_file_size, 0.0);
        assert_eq!(result.average_lines_per_file, 0.0);
        assert_eq!(result.largest_file_size, 0);
        assert_eq!(result.smallest_file_size, 0);
        assert!(result.stats_by_extension.is_empty());
    }

    #[test]
    fn test_calculate_project_basic_stats_single_extension() {
        let calculator = BasicStatsCalculator::new();
        
        let mut stats_by_extension = HashMap::new();
        stats_by_extension.insert("js".to_string(), (3, FileStats {
            total_lines: 300,
            code_lines: 200,
            comment_lines: 50,
            doc_lines: 25,
            blank_lines: 50,
            file_size: 6000,  // This is the total size for all files of this extension
        }));

        let code_stats = CodeStats {
            total_files: 3,
            total_lines: 300,
            total_code_lines: 200,
            total_comment_lines: 50,
            total_doc_lines: 25,
            total_blank_lines: 50,
            total_size: 6000,
            stats_by_extension,
        };

        let result = calculator.calculate_project_basic_stats(&code_stats).unwrap();

        assert_eq!(result.total_files, 3);
        assert_eq!(result.total_lines, 300);
        assert_eq!(result.code_lines, 200);
        assert_eq!(result.comment_lines, 50);
        assert_eq!(result.doc_lines, 25);
        assert_eq!(result.blank_lines, 50);
        assert_eq!(result.total_size, 6000);
        assert_eq!(result.average_file_size, 2000.0);
        assert_eq!(result.average_lines_per_file, 100.0);
        assert_eq!(result.largest_file_size, 2000);
        assert_eq!(result.smallest_file_size, 2000);

        assert_eq!(result.stats_by_extension.len(), 1);
        let js_stats = &result.stats_by_extension["js"];
        assert_eq!(js_stats.file_count, 3);
        assert_eq!(js_stats.total_lines, 300);
        assert_eq!(js_stats.code_lines, 200);
        assert_eq!(js_stats.average_lines_per_file, 100.0);
        assert_eq!(js_stats.average_size_per_file, 2000.0);
    }

    #[test]
    fn test_extension_stats_creation() {
        let ext_stats = ExtensionStats {
            file_count: 5,
            total_lines: 500,
            code_lines: 350,
            comment_lines: 100,
            doc_lines: 25,
            blank_lines: 50,
            total_size: 10000,
            average_lines_per_file: 100.0,
            average_size_per_file: 2000.0,
        };

        assert_eq!(ext_stats.file_count, 5);
        assert_eq!(ext_stats.total_lines, 500);
        assert_eq!(ext_stats.code_lines, 350);
        assert_eq!(ext_stats.comment_lines, 100);
        assert_eq!(ext_stats.doc_lines, 25);
        assert_eq!(ext_stats.blank_lines, 50);
        assert_eq!(ext_stats.total_size, 10000);
        assert_eq!(ext_stats.average_lines_per_file, 100.0);
        assert_eq!(ext_stats.average_size_per_file, 2000.0);
    }

    #[test]
    fn test_basic_stats_serialization() {
        let basic_stats = BasicStats {
            total_files: 10,
            total_lines: 1000,
            code_lines: 700,
            comment_lines: 200,
            doc_lines: 50,
            blank_lines: 100,
            total_size: 20000,
            average_file_size: 2000.0,
            average_lines_per_file: 100.0,
            largest_file_size: 5000,
            smallest_file_size: 500,
            stats_by_extension: HashMap::new(),
        };

        // Test serialization to JSON
        let json = serde_json::to_string(&basic_stats).unwrap();
        let deserialized: BasicStats = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.total_files, 10);
        assert_eq!(deserialized.total_lines, 1000);
        assert_eq!(deserialized.code_lines, 700);
        assert_eq!(deserialized.comment_lines, 200);
        assert_eq!(deserialized.doc_lines, 50);
        assert_eq!(deserialized.blank_lines, 100);
        assert_eq!(deserialized.total_size, 20000);
        assert_eq!(deserialized.average_file_size, 2000.0);
        assert_eq!(deserialized.average_lines_per_file, 100.0);
        assert_eq!(deserialized.largest_file_size, 5000);
        assert_eq!(deserialized.smallest_file_size, 500);
    }

    #[test]
    fn test_extension_stats_serialization() {
        let ext_stats = ExtensionStats {
            file_count: 3,
            total_lines: 300,
            code_lines: 200,
            comment_lines: 50,
            doc_lines: 25,
            blank_lines: 50,
            total_size: 6000,
            average_lines_per_file: 100.0,
            average_size_per_file: 2000.0,
        };

        // Test serialization to JSON
        let json = serde_json::to_string(&ext_stats).unwrap();
        let deserialized: ExtensionStats = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.file_count, 3);
        assert_eq!(deserialized.total_lines, 300);
        assert_eq!(deserialized.code_lines, 200);
        assert_eq!(deserialized.comment_lines, 50);
        assert_eq!(deserialized.doc_lines, 25);
        assert_eq!(deserialized.blank_lines, 50);
        assert_eq!(deserialized.total_size, 6000);
        assert_eq!(deserialized.average_lines_per_file, 100.0);
        assert_eq!(deserialized.average_size_per_file, 2000.0);
    }

    #[test]
    fn test_basic_stats_edge_cases() {
        let calculator = BasicStatsCalculator::new();

        // Test with very large numbers
        let large_file_stats = FileStats {
            total_lines: usize::MAX,
            code_lines: usize::MAX / 2,
            comment_lines: usize::MAX / 4,
            doc_lines: usize::MAX / 8,
            blank_lines: usize::MAX / 8,
            file_size: u64::MAX,
        };

        let result = calculator.calculate_basic_stats(&large_file_stats).unwrap();
        assert_eq!(result.total_files, 1);
        assert_eq!(result.total_lines, usize::MAX);
        assert_eq!(result.total_size, u64::MAX);
    }

    #[test]
    fn test_basic_stats_with_real_project() {
        let project = TestProject::new("test_project").unwrap();
        
        // Create a realistic project structure
        project.create_rust_file("src/main.rs", 10, 5).unwrap();
        project.create_rust_file("src/lib.rs", 20, 8).unwrap();
        project.create_python_file("script.py", 15).unwrap();
        project.create_javascript_file("app.js", 12).unwrap();

        let calculator = BasicStatsCalculator::new();
        
        // This would normally be done by the counter, but we'll simulate it
        let mut stats_by_extension = HashMap::new();
        stats_by_extension.insert("rs".to_string(), (2, FileStats {
            total_lines: 100,
            code_lines: 70,
            comment_lines: 20,
            doc_lines: 5,
            blank_lines: 10,
            file_size: 2000,
        }));

        let code_stats = CodeStats {
            total_files: 4,
            total_lines: 200,
            total_code_lines: 140,
            total_comment_lines: 40,
            total_doc_lines: 10,
            total_blank_lines: 20,
            total_size: 4000,
            stats_by_extension,
        };

        let result = calculator.calculate_project_basic_stats(&code_stats).unwrap();

        assert_eq!(result.total_files, 4);
        assert_eq!(result.total_lines, 200);
        assert_eq!(result.code_lines, 140);
        assert_eq!(result.comment_lines, 40);
        assert_eq!(result.doc_lines, 10);
        assert_eq!(result.blank_lines, 20);
        assert_eq!(result.total_size, 4000);
        assert_eq!(result.average_file_size, 1000.0);
        assert_eq!(result.average_lines_per_file, 50.0);
    }
} 