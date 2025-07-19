use crate::utils::errors::Result;
use super::types::{FunctionInfo, StructureInfo};
use super::languages::get_language_analyzer;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;

/// Language-specific code analyzer
pub struct CodeAnalyzer;

impl CodeAnalyzer {
    pub fn new() -> Self {
        Self
    }

    /// Analyze structures in a file (classes, interfaces, etc.)
    pub fn analyze_file_structures(&self, file_path: &str) -> Result<Vec<StructureInfo>> {
        let file = fs::File::open(file_path)?;
        let reader = BufReader::new(file);
        let lines: Vec<String> = reader.lines().collect::<std::io::Result<Vec<_>>>()?;
        
        let extension = Path::new(file_path)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("unknown")
            .to_lowercase();
        
        if let Some(analyzer) = get_language_analyzer(&extension) {
            analyzer.analyze_structures(&lines)
        } else {
            Ok(Vec::new()) // Unsupported language
        }
    }
    
    /// Analyze functions in a file for complexity metrics
    pub fn analyze_file_functions(&self, file_path: &str) -> Result<Vec<FunctionInfo>> {
        let file = fs::File::open(file_path)?;
        let reader = BufReader::new(file);
        let lines: Vec<String> = reader.lines().collect::<std::io::Result<Vec<_>>>()?;
        
        let extension = Path::new(file_path)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("unknown")
            .to_lowercase();
        
        if let Some(analyzer) = get_language_analyzer(&extension) {
            analyzer.analyze_functions(&lines)
        } else {
            Ok(Vec::new()) // Unsupported language
        }
    }
}

impl Default for CodeAnalyzer {
    fn default() -> Self {
        Self::new()
    }
} 