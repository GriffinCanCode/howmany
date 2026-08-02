use super::languages::get_language_analyzer;
use super::types::{FunctionInfo, StructureInfo};
use crate::utils::errors::Result;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;

/// Everything one pass over a file yields.
///
/// Functions and structures used to be requested separately, and each request
/// opened the file and split it into lines again. Returning both from one pass
/// removes a full read and a full split per file from the report path.
#[derive(Debug, Clone, Default)]
pub struct FileAnalysis {
    pub functions: Vec<FunctionInfo>,
    pub structures: Vec<StructureInfo>,
}

/// Language-specific code analyzer
pub struct CodeAnalyzer;

impl CodeAnalyzer {
    pub fn new() -> Self {
        Self
    }

    /// Analyze `file_path` for functions and structures in a single read.
    pub fn analyze_file(&self, file_path: &str) -> Result<FileAnalysis> {
        let extension = Path::new(file_path)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("unknown")
            .to_lowercase();

        // Nothing to learn from a language with no analyzer, so the file is not
        // opened at all.
        let Some(analyzer) = get_language_analyzer(&extension) else {
            return Ok(FileAnalysis::default());
        };

        let reader = BufReader::new(fs::File::open(file_path)?);
        let lines: Vec<String> = reader
            .lines()
            .collect::<std::io::Result<Vec<_>>>()
            .unwrap_or_default();

        Ok(FileAnalysis {
            functions: analyzer.analyze_functions(&lines)?,
            structures: analyzer.analyze_structures(&lines)?,
        })
    }
}

impl Default for CodeAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}
