use std::path::Path;
use std::process::Command;
use crate::core::patterns::PatternMatcher;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SherlockLanguage {
    pub name: String,
    pub color: String,
    pub file_count: usize,
    pub files: Vec<String>,
    pub percentage: f64,
    pub total_bytes: u64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SherlockSummary {
    pub languages_detected: usize,
    pub total_bytes: u64,
    pub total_files: usize,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SherlockResult {
    pub languages: Vec<SherlockLanguage>,
    pub summary: SherlockSummary,
    pub unknown_files: Vec<String>,
}

pub struct FileDetector {
    pattern_matcher: PatternMatcher,
    sherlock_result: Option<SherlockResult>,
}

impl FileDetector {
    pub fn new() -> Self {
        Self {
            pattern_matcher: PatternMatcher::new(),
            sherlock_result: None,
        }
    }

    pub fn with_sherlock_result(mut self, sherlock_result: SherlockResult) -> Self {
        self.sherlock_result = Some(sherlock_result);
        self
    }

    pub fn is_user_created_file(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();
        
        // First check if it should be ignored based on common patterns
        if self.pattern_matcher.should_ignore_file(&path_str) {
            return false;
        }
        
        // Check if it matches build/cache patterns
        if self.pattern_matcher.matches_build_cache_pattern(&path_str) {
            return false;
        }
        
        // Check if it's a code file based on SherlockIO results or fallback
        if let Some(sherlock_result) = &self.sherlock_result {
            // Check if this file is in SherlockIO's detected files
            for language in &sherlock_result.languages {
                for file in &language.files {
                    if path_str.ends_with(file.trim_start_matches("./")) {
                        return true;
                    }
                }
            }
        }
        
        // Fallback: check if it's a code file by extension
        if let Some(extension) = path.extension() {
            let ext_str = extension.to_string_lossy().to_lowercase();
            return self.is_code_extension(&ext_str);
        }
        
        false
    }

    pub fn is_code_file(&self, path: &Path) -> bool {
        if let Some(extension) = path.extension() {
            let ext_str = extension.to_string_lossy().to_lowercase();
            return self.is_code_extension(&ext_str);
        }
        false
    }

    /// Check if an extension represents a code file
    fn is_code_extension(&self, ext: &str) -> bool {
        // Common code file extensions as fallback
        matches!(ext, 
            "rs" | "py" | "js" | "ts" | "jsx" | "tsx" | "java" | "c" | "cpp" | "cc" | "cxx" | "h" | "hpp" |
            "go" | "rb" | "php" | "cs" | "swift" | "kt" | "scala" | "hs" | "lhs" | "ex" | "exs" |
            "erl" | "hrl" | "jl" | "lua" | "zig" | "clj" | "cljs" | "cljc" | "dart" | "pl" | "pm" |
            "r" | "m" | "mlx" | "html" | "css" | "scss" | "sass" | "json" | "yaml" | "yml" | "toml" |
            "xml" | "md" | "sh" | "bash" | "zsh" | "fish" | "ps1" | "bat" | "cmd"
        )
    }

    /// Detect languages in a directory using SherlockIO
    pub fn detect_languages(&self, path: &Path) -> Result<SherlockResult, Box<dyn std::error::Error>> {
        let output = Command::new("sherlock")
            .arg(path.to_string_lossy().as_ref())
            .arg("--format")
            .arg("json")
            .output()?;

        if !output.status.success() {
            return Err(format!("SherlockIO failed: {}", String::from_utf8_lossy(&output.stderr)).into());
        }

        let result: SherlockResult = serde_json::from_slice(&output.stdout)?;
        Ok(result)
    }

    /// Get language name from file extension using SherlockIO data
    pub fn get_language_from_extension(&self, extension: &str, sherlock_result: &SherlockResult) -> Option<String> {
        // First try to find the language in SherlockIO results
        for language in &sherlock_result.languages {
            for file in &language.files {
                if let Some(file_ext) = Path::new(file).extension() {
                    if file_ext.to_string_lossy().to_lowercase() == extension.to_lowercase() {
                        return Some(language.name.clone());
                    }
                }
            }
        }
        
        // Fallback to hardcoded mapping if not found
        self.get_language_from_extension_fallback(extension)
    }

    /// Fallback language detection for when SherlockIO doesn't have the info
    fn get_language_from_extension_fallback(&self, extension: &str) -> Option<String> {
        match extension.to_lowercase().as_str() {
            "rs" => Some("Rust".to_string()),
            "py" => Some("Python".to_string()),
            "js" => Some("JavaScript".to_string()),
            "ts" => Some("TypeScript".to_string()),
            "java" => Some("Java".to_string()),
            "cpp" | "cc" | "cxx" => Some("C++".to_string()),
            "c" => Some("C".to_string()),
            "go" => Some("Go".to_string()),
            "rb" => Some("Ruby".to_string()),
            "php" => Some("PHP".to_string()),
            "cs" => Some("C#".to_string()),
            "swift" => Some("Swift".to_string()),
            "kt" => Some("Kotlin".to_string()),
            "html" => Some("HTML".to_string()),
            "css" => Some("CSS".to_string()),
            "scss" | "sass" => Some("Sass".to_string()),
            "json" => Some("JSON".to_string()),
            "yaml" | "yml" => Some("YAML".to_string()),
            "toml" => Some("TOML".to_string()),
            "md" => Some("Markdown".to_string()),
            _ => None,
        }
    }
} 