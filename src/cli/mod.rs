use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "howmany")]
#[command(about = "Count files and lines of code in your projects, intelligently excluding dependencies and generated files")]
#[command(version = "0.1.0")]
pub struct Config {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Count files and lines of code
    Count {
        /// Directory to analyze (defaults to current directory)
        #[arg(value_name = "PATH")]
        path: Option<PathBuf>,
        
        /// Maximum directory depth to traverse
        #[arg(short, long)]
        max_depth: Option<usize>,
        
        /// Show detailed breakdown by file extension
        #[arg(short = 'v', long)]
        verbose: bool,
        
        /// Show individual file statistics
        #[arg(short = 'f', long)]
        files: bool,
        
        /// Include hidden files and directories
        #[arg(long)]
        include_hidden: bool,
        
        /// Ignore .gitignore files
        #[arg(long)]
        ignore_gitignore: bool,
        
        /// Additional patterns to ignore (can be used multiple times)
        #[arg(long = "ignore", value_name = "PATTERN")]
        custom_ignores: Vec<String>,
        
        /// Only count specific file extensions (can be used multiple times)
        #[arg(long = "ext", value_name = "EXTENSION")]
        extensions: Vec<String>,
        
        /// Output format: text, json, or csv
        #[arg(long, default_value = "text")]
        format: OutputFormat,
        
        /// Sort results by: files, lines, code, comments, or size
        #[arg(long, default_value = "files")]
        sort_by: SortBy,
        
        /// Sort in descending order
        #[arg(long)]
        descending: bool,
    },
    
    /// List all files that would be counted (useful for debugging filters)
    List {
        /// Directory to analyze (defaults to current directory)
        #[arg(value_name = "PATH")]
        path: Option<PathBuf>,
        
        /// Maximum directory depth to traverse
        #[arg(short, long)]
        max_depth: Option<usize>,
        
        /// Include hidden files and directories
        #[arg(long)]
        include_hidden: bool,
        
        /// Ignore .gitignore files
        #[arg(long)]
        ignore_gitignore: bool,
        
        /// Additional patterns to ignore (can be used multiple times)
        #[arg(long = "ignore", value_name = "PATTERN")]
        custom_ignores: Vec<String>,
        
        /// Only show specific file extensions (can be used multiple times)
        #[arg(long = "ext", value_name = "EXTENSION")]
        extensions: Vec<String>,
    },
    
    /// Interactive mode with beautiful formatting and visualization
    Interactive {
        /// Directory to analyze (defaults to current directory)
        #[arg(value_name = "PATH")]
        path: Option<PathBuf>,
        
        /// Maximum directory depth to traverse
        #[arg(short, long)]
        max_depth: Option<usize>,
        
        /// Show individual file statistics
        #[arg(short = 'f', long)]
        files: bool,
        
        /// Include hidden files and directories
        #[arg(long)]
        include_hidden: bool,
        
        /// Ignore .gitignore files
        #[arg(long)]
        ignore_gitignore: bool,
        
        /// Additional patterns to ignore (can be used multiple times)
        #[arg(long = "ignore", value_name = "PATTERN")]
        custom_ignores: Vec<String>,
        
        /// Only count specific file extensions (can be used multiple times)
        #[arg(long = "ext", value_name = "EXTENSION")]
        extensions: Vec<String>,
    },
}

#[derive(Clone, Debug)]
pub enum OutputFormat {
    Text,
    Json,
    Csv,
}

impl std::str::FromStr for OutputFormat {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "text" => Ok(OutputFormat::Text),
            "json" => Ok(OutputFormat::Json),
            "csv" => Ok(OutputFormat::Csv),
            _ => Err(format!("Invalid output format: {}. Valid options: text, json, csv", s)),
        }
    }
}

#[derive(Clone, Debug)]
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
            "files" => Ok(SortBy::Files),
            "lines" => Ok(SortBy::Lines),
            "code" => Ok(SortBy::Code),
            "comments" => Ok(SortBy::Comments),
            "size" => Ok(SortBy::Size),
            _ => Err(format!("Invalid sort option: {}. Valid options: files, lines, code, comments, size", s)),
        }
    }
}

impl Config {
    pub fn parse_args() -> Self {
        Self::parse()
    }
} 