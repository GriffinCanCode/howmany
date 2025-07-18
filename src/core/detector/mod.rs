use std::path::Path;
use crate::core::patterns::PatternMatcher;

pub mod patterns;
use patterns::{ExternalPatterns, CodeExtensions};

pub struct FileDetector {
    external_patterns: ExternalPatterns,
    code_extensions: CodeExtensions,
    pattern_matcher: PatternMatcher,
}

impl FileDetector {
    pub fn new() -> Self {
        Self {
            external_patterns: ExternalPatterns::new(),
            code_extensions: CodeExtensions::new(),
            pattern_matcher: PatternMatcher::new(),
        }
    }

    pub fn is_user_created_file(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();
        
        // First check if it should be ignored based on common patterns
        if self.pattern_matcher.should_ignore_file(&path_str) {
            return false;
        }
        
        // Check if it matches external/dependency patterns
        if self.external_patterns.matches(&path_str) {
            return false;
        }
        
        // Check if it matches build/cache patterns
        if self.pattern_matcher.matches_build_cache_pattern(&path_str) {
            return false;
        }
        
        // Check if it's a code file we care about
        if let Some(extension) = path.extension() {
            let ext_str = extension.to_string_lossy().to_lowercase();
            return self.code_extensions.contains(&ext_str);
        }
        
        // If no extension, check if it might be a script or config file
        if let Some(filename) = path.file_name() {
            let filename_str = filename.to_string_lossy();
            
            // Common script names without extensions
            let script_names = CodeExtensions::get_script_names();
            
            for script_name in script_names {
                if filename_str.eq_ignore_ascii_case(script_name) {
                    return true;
                }
            }
        }
        
        false
    }

    pub fn is_code_file(&self, path: &Path) -> bool {
        if let Some(extension) = path.extension() {
            let ext_str = extension.to_string_lossy().to_lowercase();
            return self.code_extensions.contains(&ext_str);
        }
        false
    }
} 