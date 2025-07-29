use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "howmany")]
#[command(about = "Count files and lines of code in your projects")]
#[command(version = "2.0.0")]
pub struct Config {
    /// Directory to analyze (defaults to current directory)
    #[arg(value_name = "PATH")]
    pub path: Option<PathBuf>,
    
    /// Output format: text, json, csv, html, or sarif
    #[arg(short = 'o', long = "output", default_value = "text")]
    pub format: OutputFormat,
    
    /// Show individual file statistics
    #[arg(short = 'f', long = "files")]
    pub show_files: bool,
    
    /// Simple CLI mode - show only basic file and line counts
    #[arg(long = "cli")]
    pub cli_mode: bool,
    
    /// Disable interactive mode (interactive mode is enabled by default)
    #[arg(long = "no-interactive")]
    pub no_interactive: bool,
    
    /// Show detailed breakdown by file extension
    #[arg(short = 'v', long = "verbose")]
    pub verbose: bool,
    
    /// Maximum directory depth to traverse
    #[arg(short = 'd', long = "depth")]
    pub max_depth: Option<usize>,
    
    /// Only count specific file extensions (comma-separated: rs,py,js)
    #[arg(short = 'e', long = "ext")]
    pub extensions: Option<String>,
    
    /// Include hidden files and directories
    #[arg(long = "hidden")]
    pub include_hidden: bool,
    
    /// Sort results by: files, lines, code, comments, size, complexity, quality, functions
    #[arg(short = 's', long = "sort", default_value = "files")]
    pub sort_by: SortBy,
    
    /// Sort in descending order
    #[arg(long = "desc")]
    pub descending: bool,
    
    /// Additional patterns to ignore (comma-separated: node_modules,target,dist)
    #[arg(long = "ignore")]
    pub ignore_patterns: Option<String>,
    
    /// List files that would be counted (useful for debugging)
    #[arg(short = 'l', long = "list")]
    pub list_files: bool,
    
    // Filter options
    /// Minimum lines per file to include
    #[arg(long = "min-lines")]
    pub min_lines: Option<usize>,
    
    /// Maximum lines per file to include
    #[arg(long = "max-lines")]
    pub max_lines: Option<usize>,
    
    /// Minimum file size to include (e.g., 1KB, 500MB)
    #[arg(long = "min-size")]
    pub min_size: Option<String>,
    
    /// Maximum file size to include (e.g., 1KB, 500MB)
    #[arg(long = "max-size")]
    pub max_size: Option<String>,
    
    /// Include only these languages (comma-separated: rs,py,js)
    #[arg(long = "only")]
    pub only_languages: Option<String>,
    
    /// Exclude these languages (comma-separated: rs,py,js)
    #[arg(long = "exclude")]
    pub exclude_languages: Option<String>,
    
    // Enhanced CLI output options
    /// Show complexity information in CLI mode
    #[arg(long = "show-complexity")]
    pub show_complexity: bool,
    
    /// Show quality scores in CLI mode
    #[arg(long = "show-quality")]
    pub show_quality: bool,
    
    /// Show code ratios in CLI mode
    #[arg(long = "show-ratios")]
    pub show_ratios: bool,
    
    /// Show size information in CLI mode
    #[arg(long = "show-size")]
    pub show_size: bool,
    
    // Advanced filtering options
    /// Minimum complexity score to include (0.0-100.0)
    #[arg(long = "min-complexity")]
    pub min_complexity: Option<f64>,
    
    /// Maximum complexity score to include (0.0-100.0)
    #[arg(long = "max-complexity")]
    pub max_complexity: Option<f64>,
    
    /// Minimum functions per file to include
    #[arg(long = "min-functions")]
    pub min_functions: Option<usize>,
    
    /// Maximum functions per file to include
    #[arg(long = "max-functions")]
    pub max_functions: Option<usize>,
    
    /// Minimum quality score to include (0-100)
    #[arg(long = "min-quality")]
    pub min_quality_score: Option<f64>,
    
    /// Maximum quality score to include (0-100)
    #[arg(long = "max-quality")]
    pub max_quality_score: Option<f64>,
    
    /// Minimum documentation ratio to include (0.0-1.0)
    #[arg(long = "min-doc-ratio")]
    pub min_doc_ratio: Option<f64>,
    
    /// Maximum documentation ratio to include (0.0-1.0)
    #[arg(long = "max-doc-ratio")]
    pub max_doc_ratio: Option<f64>,
    
    // Advanced filter shortcuts
    /// Only show files with high complexity (complexity > 10)
    #[arg(long = "high-complexity")]
    pub high_complexity_only: bool,
    
    /// Only show files with low quality scores (quality < 60)
    #[arg(long = "low-quality")]
    pub low_quality_only: bool,
    
    /// Only show files with poor documentation (doc ratio < 0.1)
    #[arg(long = "undocumented")]
    pub undocumented_only: bool,
    
    // Output enhancement options
    /// Show time estimates in CLI mode
    #[arg(long = "show-time")]
    pub show_time_estimates: bool,
    
    /// Compact output mode
    #[arg(long = "compact")]
    pub compact_output: bool,
    
    /// Show only summary (no per-extension breakdown)
    #[arg(long = "summary-only")]
    pub summary_only: bool,
    
    /// Show top N results only
    #[arg(long = "top")]
    pub top_n: Option<usize>,
    
    /// Show file-level complexity details
    #[arg(long = "show-functions")]
    pub show_function_details: bool,
    
    // Format options
    /// Disable colors in output
    #[arg(long = "no-color")]
    pub no_color: bool,
    
    /// Output preset (compact, detailed, minimal)
    #[arg(long = "preset")]
    pub output_preset: Option<String>,
    
    // Developer experience
    /// Quiet mode - minimal output
    #[arg(short = 'q', long = "quiet")]
    pub quiet: bool,
    
    /// Explain why files were included/excluded
    #[arg(long = "explain")]
    pub explain_filtering: bool,
}

#[derive(Clone)]
pub enum OutputFormat {
    Text,
    Json,
    Csv,
    Html,
    Sarif,
}

impl std::str::FromStr for OutputFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "text" | "txt" => Ok(OutputFormat::Text),
            "json" => Ok(OutputFormat::Json),
            "csv" => Ok(OutputFormat::Csv),
            "html" => Ok(OutputFormat::Html),
            "sarif" => Ok(OutputFormat::Sarif),
            _ => Err(format!("Invalid output format: {}", s)),
        }
    }
}

#[derive(Clone, Copy)]
pub enum SortBy {
    Files,
    Lines,
    Code,
    Comments,
    Size,
    Complexity,
    Quality,
    Functions,
    DocRatio,
}

impl std::str::FromStr for SortBy {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "files" | "file" => Ok(SortBy::Files),
            "lines" | "line" => Ok(SortBy::Lines),
            "code" => Ok(SortBy::Code),
            "comments" | "comment" => Ok(SortBy::Comments),
            "size" => Ok(SortBy::Size),
            "complexity" | "complex" => Ok(SortBy::Complexity),
            "quality" => Ok(SortBy::Quality),
            "functions" | "function" | "func" => Ok(SortBy::Functions),
            "doc-ratio" | "docs" | "documentation" => Ok(SortBy::DocRatio),
            _ => Err(format!("Invalid sort option: {}", s)),
        }
    }
}

impl Config {
    pub fn parse_args() -> Self {
        Self::parse()
    }
    
    /// Check if interactive mode should be enabled (default true, unless --no-interactive is passed)
    pub fn interactive(&self) -> bool {
        !self.no_interactive
    }
    
    /// Convert comma-separated extensions string to Vec
    pub fn get_extensions(&self) -> Vec<String> {
        self.extensions
            .as_ref()
            .map(|s| s.split(',').map(|ext| ext.trim().to_string()).collect())
            .unwrap_or_default()
    }
    
    /// Convert comma-separated ignore patterns string to Vec
    pub fn get_ignore_patterns(&self) -> Vec<String> {
        self.ignore_patterns
            .as_ref()
            .map(|s| s.split(',').map(|pattern| pattern.trim().to_string()).collect())
            .unwrap_or_default()
    }
    
    /// Apply advanced filter shortcuts to set specific filter values
    pub fn apply_advanced_filter_shortcuts(&mut self) {
        if self.high_complexity_only {
            self.min_complexity = Some(10.0);
        }
        
        if self.low_quality_only {
            self.max_quality_score = Some(60.0);
        }
        
        if self.undocumented_only {
            self.max_doc_ratio = Some(0.1);
        }
    }
    
    /// Get output preset configuration
    pub fn apply_output_preset(&mut self) {
        if let Some(preset) = &self.output_preset {
            match preset.to_lowercase().as_str() {
                "compact" => {
                    self.compact_output = true;
                    self.no_color = true;
                    self.top_n = Some(10);
                },
                "detailed" => {
                    self.verbose = true;
                    self.show_complexity = true;
                    self.show_quality = true;
                    self.show_ratios = true;
                    self.show_size = true;
                    self.show_time_estimates = true;
                    self.show_function_details = true;
                },
                "minimal" => {
                    self.quiet = true;
                    self.summary_only = true;
                    self.no_color = true;
                    self.compact_output = true;
                },
                _ => {} // Unknown preset, ignore
            }
        }
    }
    
    /// Convert CLI options to FilterOptions
    pub fn get_filter_options(&self) -> crate::ui::filters::FilterOptions {
        use crate::ui::filters::{FilterOptions, FilterParser};
        
        FilterOptions {
            min_lines: self.min_lines,
            max_lines: self.max_lines,
            min_size_bytes: self.min_size.as_ref().and_then(|s| FilterParser::parse_size(s)),
            max_size_bytes: self.max_size.as_ref().and_then(|s| FilterParser::parse_size(s)),
            min_complexity: self.min_complexity,
            max_complexity: self.max_complexity,
            min_functions: self.min_functions,
            max_functions: self.max_functions,
            min_quality_score: self.min_quality_score,
            max_quality_score: self.max_quality_score,
            min_doc_ratio: self.min_doc_ratio,
            max_doc_ratio: self.max_doc_ratio,
            include_languages: self.only_languages
                .as_ref()
                .map(|s| FilterParser::parse_languages(s))
                .unwrap_or_default(),
            exclude_languages: self.exclude_languages
                .as_ref()
                .map(|s| FilterParser::parse_languages(s))
                .unwrap_or_default(),
            show_complexity: self.show_complexity,
            show_quality: self.show_quality,
            show_ratios: self.show_ratios,
            show_size_info: self.show_size,
            compact_output: self.compact_output,
        }
    }
} 