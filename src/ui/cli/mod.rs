use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "howmany")]
#[command(about = "Count files and lines of code in your projects")]
#[command(version = "0.3.2")]
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
    
    /// Sort results by: files, lines, code, comments, or size
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

#[derive(Clone)]
pub enum SortBy {
    Files,
    Lines,
    Code,
    Comments,
    Size,
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
} 