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