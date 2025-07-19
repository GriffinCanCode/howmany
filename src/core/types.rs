use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Statistics for a single file
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FileStats {
    pub total_lines: usize,
    pub code_lines: usize,
    pub comment_lines: usize,
    pub blank_lines: usize,
    pub file_size: u64,
    pub doc_lines: usize, // Documentation content
}

impl Default for FileStats {
    fn default() -> Self {
        Self {
            total_lines: 0,
            code_lines: 0,
            comment_lines: 0,
            blank_lines: 0,
            file_size: 0,
            doc_lines: 0,
        }
    }
}

/// Aggregated statistics for a project
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CodeStats {
    pub total_files: usize,
    pub total_lines: usize,
    pub total_code_lines: usize,
    pub total_comment_lines: usize,
    pub total_blank_lines: usize,
    pub total_size: u64,
    pub total_doc_lines: usize, // Documentation content
    pub stats_by_extension: HashMap<String, (usize, FileStats)>, // (file_count, aggregated_stats)
}

impl Default for CodeStats {
    fn default() -> Self {
        Self {
            total_files: 0,
            total_lines: 0,
            total_code_lines: 0,
            total_comment_lines: 0,
            total_blank_lines: 0,
            total_size: 0,
            total_doc_lines: 0,
            stats_by_extension: HashMap::new(),
        }
    }
} 