use std::path::Path;
use ignore::{WalkBuilder, DirEntry};
use crate::core::patterns::PatternMatcher;

pub struct FileFilter {
    // Use gitignore-style filtering
    respect_gitignore: bool,
    respect_hidden: bool,
    max_depth: Option<usize>,
    custom_ignores: Vec<String>,
    pattern_matcher: PatternMatcher,
}

impl FileFilter {
    pub fn new() -> Self {
        Self {
            respect_gitignore: true,
            respect_hidden: true,
            max_depth: None,
            custom_ignores: Vec::new(),
            pattern_matcher: PatternMatcher::new(),
        }
    }
    
    pub fn with_max_depth(mut self, depth: usize) -> Self {
        self.max_depth = Some(depth);
        self
    }
    
    pub fn with_custom_ignores(mut self, ignores: Vec<String>) -> Self {
        self.custom_ignores.extend(ignores);
        self
    }
    
    pub fn respect_gitignore(mut self, respect: bool) -> Self {
        self.respect_gitignore = respect;
        self
    }
    
    pub fn respect_hidden(mut self, respect: bool) -> Self {
        self.respect_hidden = respect;
        self
    }
    
    pub fn walk_directory<P: AsRef<Path>>(&self, path: P) -> impl Iterator<Item = DirEntry> {
        let path_ref = path.as_ref();
        let mut builder = WalkBuilder::new(path_ref);
        
        builder
            .git_ignore(self.respect_gitignore)
            .hidden(self.respect_hidden)
            .parents(true)
            .ignore(true);
        
        if let Some(depth) = self.max_depth {
            builder.max_depth(Some(depth));
        }
        
        // Add custom ignore patterns directly to the builder
        for pattern in &self.custom_ignores {
            builder.add_custom_ignore_filename(pattern);
        }
        
        builder.build().filter_map(|entry| entry.ok())
    }
    
    pub fn should_include_file(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();
        
        // Check if file should be ignored based on common patterns
        if self.pattern_matcher.should_ignore_file(&path_str) {
            return false;
        }
        
        // Check if it matches build/cache patterns
        if self.pattern_matcher.matches_build_cache_pattern(&path_str) {
            return false;
        }
        
        // Check against custom ignore patterns
        for pattern in &self.custom_ignores {
            if self.matches_pattern(&path_str, pattern) {
                return false;
            }
        }
        
        // Check if it's a binary file
        if let Some(extension) = path.extension() {
            let ext_str = extension.to_string_lossy();
            if self.pattern_matcher.is_binary_file(&ext_str) {
                return false;
            }
        }
        
        // Check if it's a generated file
        if let Some(filename) = path.file_name() {
            let filename_str = filename.to_string_lossy();
            if self.pattern_matcher.is_generated_file(&filename_str) {
                return false;
            }
        }
        
        true
    }
    
    fn matches_pattern(&self, path: &str, pattern: &str) -> bool {
        // Simple glob-like matching for custom patterns
        if pattern.ends_with('/') {
            // Directory pattern
            let dir_pattern = &pattern[..pattern.len() - 1];
            return path.contains(dir_pattern);
        } else if pattern.starts_with("*.") {
            // Extension pattern
            let ext = &pattern[2..];
            return path.ends_with(ext);
        } else if pattern.contains('*') {
            // Wildcard pattern - simple implementation
            let parts: Vec<&str> = pattern.split('*').collect();
            if parts.len() == 2 {
                return path.starts_with(parts[0]) && path.ends_with(parts[1]);
            }
        } else {
            // Exact match
            return path.contains(pattern);
        }
        false
    }
} 