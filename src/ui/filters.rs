use crate::core::types::FileStats;
use crate::core::stats::aggregation::AggregatedStats;
use crate::core::stats::basic::ExtensionStats;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Filter options for CLI output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterOptions {
    // Size filters
    pub min_lines: Option<usize>,
    pub max_lines: Option<usize>,
    pub min_size_bytes: Option<u64>,
    pub max_size_bytes: Option<u64>,
    
    // Complexity filters
    pub min_complexity: Option<f64>,
    pub max_complexity: Option<f64>,
    pub min_functions: Option<usize>,
    pub max_functions: Option<usize>,
    
    // Quality filters
    pub min_quality_score: Option<f64>,
    pub max_quality_score: Option<f64>,
    pub min_doc_ratio: Option<f64>,
    pub max_doc_ratio: Option<f64>,
    
    // Language/extension filters
    pub include_languages: Vec<String>,
    pub exclude_languages: Vec<String>,
    
    // Output customization
    pub show_complexity: bool,
    pub show_quality: bool,
    pub show_ratios: bool,
    pub show_size_info: bool,
    pub compact_output: bool,
}

impl Default for FilterOptions {
    fn default() -> Self {
        Self {
            min_lines: None,
            max_lines: None,
            min_size_bytes: None,
            max_size_bytes: None,
            min_complexity: None,
            max_complexity: None,
            min_functions: None,
            max_functions: None,
            min_quality_score: None,
            max_quality_score: None,
            min_doc_ratio: None,
            max_doc_ratio: None,
            include_languages: Vec::new(),
            exclude_languages: Vec::new(),
            show_complexity: false,
            show_quality: false,
            show_ratios: false,
            show_size_info: false,
            compact_output: false,
        }
    }
}

/// Filter for individual files
pub struct FileFilter {
    options: FilterOptions,
}

impl FileFilter {
    pub fn new(options: FilterOptions) -> Self {
        Self { options }
    }
    
    /// Check if a file passes all filters
    pub fn passes_filter(&self, file_path: &str, file_stats: &FileStats) -> bool {
        // Size filters
        if let Some(min_lines) = self.options.min_lines {
            if file_stats.total_lines < min_lines {
                return false;
            }
        }
        
        if let Some(max_lines) = self.options.max_lines {
            if file_stats.total_lines > max_lines {
                return false;
            }
        }
        
        if let Some(min_size) = self.options.min_size_bytes {
            if file_stats.file_size < min_size {
                return false;
            }
        }
        
        if let Some(max_size) = self.options.max_size_bytes {
            if file_stats.file_size > max_size {
                return false;
            }
        }
        
        // Language/extension filters
        let extension = std::path::Path::new(file_path)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("no_ext")
            .to_lowercase();
        
        if !self.options.include_languages.is_empty() {
            if !self.options.include_languages.iter().any(|lang| lang.to_lowercase() == extension) {
                return false;
            }
        }
        
        if !self.options.exclude_languages.is_empty() {
            if self.options.exclude_languages.iter().any(|lang| lang.to_lowercase() == extension) {
                return false;
            }
        }
        
        // Documentation ratio filter
        if let Some(min_doc_ratio) = self.options.min_doc_ratio {
            let doc_ratio = if file_stats.total_lines > 0 {
                file_stats.doc_lines as f64 / file_stats.total_lines as f64
            } else {
                0.0
            };
            if doc_ratio < min_doc_ratio {
                return false;
            }
        }
        
        if let Some(max_doc_ratio) = self.options.max_doc_ratio {
            let doc_ratio = if file_stats.total_lines > 0 {
                file_stats.doc_lines as f64 / file_stats.total_lines as f64
            } else {
                0.0
            };
            if doc_ratio > max_doc_ratio {
                return false;
            }
        }
        
        true
    }
}

/// Project-level filter for aggregated stats
pub struct ProjectFilter {
    options: FilterOptions,
}

impl ProjectFilter {
    pub fn new(options: FilterOptions) -> Self {
        Self { options }
    }
    
    /// Filter extensions based on criteria
    pub fn filter_extensions(&self, stats_by_extension: &HashMap<String, ExtensionStats>) -> HashMap<String, ExtensionStats> {
        let mut filtered = HashMap::new();
        
        for (ext, stats) in stats_by_extension {
            // Language filters
            if !self.options.include_languages.is_empty() {
                if !self.options.include_languages.iter().any(|lang| lang.to_lowercase() == ext.to_lowercase()) {
                    continue;
                }
            }
            
            if !self.options.exclude_languages.is_empty() {
                if self.options.exclude_languages.iter().any(|lang| lang.to_lowercase() == ext.to_lowercase()) {
                    continue;
                }
            }
            
            // Size filters
            if let Some(min_lines) = self.options.min_lines {
                if stats.total_lines < min_lines {
                    continue;
                }
            }
            
            if let Some(max_lines) = self.options.max_lines {
                if stats.total_lines > max_lines {
                    continue;
                }
            }
            
            if let Some(min_size) = self.options.min_size_bytes {
                if stats.total_size < min_size {
                    continue;
                }
            }
            
            if let Some(max_size) = self.options.max_size_bytes {
                if stats.total_size > max_size {
                    continue;
                }
            }
            
            filtered.insert(ext.clone(), stats.clone());
        }
        
        filtered
    }
}

/// Utility functions for filter parsing
pub struct FilterParser;

impl FilterParser {
    /// Parse a size string like "1KB", "500MB", "2GB" into bytes
    pub fn parse_size(size_str: &str) -> Option<u64> {
        let size_str = size_str.trim().to_uppercase();
        
        let (number_part, unit_part) = if size_str.ends_with("KB") {
            (size_str.trim_end_matches("KB"), 1024)
        } else if size_str.ends_with("MB") {
            (size_str.trim_end_matches("MB"), 1024 * 1024)
        } else if size_str.ends_with("GB") {
            (size_str.trim_end_matches("GB"), 1024 * 1024 * 1024)
        } else if size_str.ends_with("B") {
            (size_str.trim_end_matches("B"), 1)
        } else {
            (size_str.as_str(), 1)
        };
        
        number_part.parse::<f64>().ok().map(|n| (n * unit_part as f64) as u64)
    }
    
    /// Parse a comma-separated list of languages
    pub fn parse_languages(lang_str: &str) -> Vec<String> {
        lang_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }
}

/// Format filtered output with additional information
pub struct FilteredOutputFormatter;

impl FilteredOutputFormatter {
    pub fn format_enhanced_cli_output(
        aggregated_stats: &AggregatedStats,
        individual_files: &[(String, FileStats)],
        options: &FilterOptions,
    ) -> String {
        let mut output = String::new();
        
        // Basic counts
        output.push_str(&format!("{} files, {} lines", 
            aggregated_stats.basic.total_files, 
            aggregated_stats.basic.total_lines
        ));
        
        if options.show_size_info {
            let size_mb = aggregated_stats.basic.total_size as f64 / (1024.0 * 1024.0);
            output.push_str(&format!(", {:.1} MB", size_mb));
        }
        
        if options.show_complexity && aggregated_stats.complexity.function_count > 0 {
            output.push_str(&format!(", {:.1} avg complexity", aggregated_stats.complexity.cyclomatic_complexity));
        }
        
        if options.show_quality {
            output.push_str(&format!(", {:.1}/100 quality", aggregated_stats.ratios.quality_metrics.overall_quality_score));
        }
        
        if options.show_ratios {
            output.push_str(&format!(", {:.1}% code", aggregated_stats.ratios.code_ratio * 100.0));
            if aggregated_stats.ratios.comment_ratio > 0.0 {
                output.push_str(&format!(", {:.1}% comments", aggregated_stats.ratios.comment_ratio * 100.0));
            }
        }
        
        output
    }
}