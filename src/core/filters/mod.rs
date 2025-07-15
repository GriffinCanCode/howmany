use std::path::Path;
use ignore::{WalkBuilder, DirEntry};

pub struct FileFilter {
    // Use gitignore-style filtering
    respect_gitignore: bool,
    respect_hidden: bool,
    max_depth: Option<usize>,
    custom_ignores: Vec<String>,
}

impl FileFilter {
    pub fn new() -> Self {
        Self {
            respect_gitignore: true,
            respect_hidden: true,
            max_depth: None,
            custom_ignores: vec![
                // Additional patterns beyond what's in .gitignore
                "*.tmp".to_string(),
                "*.temp".to_string(),
                "*.log".to_string(),
                "*.cache".to_string(),
                "*cache*".to_string(),
                "thumbs.db".to_string(),
                ".DS_Store".to_string(),
                "desktop.ini".to_string(),
                "*.swp".to_string(),
                "*.swo".to_string(),
                "*~".to_string(),
                ".#*".to_string(),
                "#*#".to_string(),
                
                // IDE and editor files
                ".vscode/".to_string(),
                ".idea/".to_string(),
                "*.sublime-*".to_string(),
                ".atom/".to_string(),
                
                // OS generated files
                ".Spotlight-V100".to_string(),
                ".Trashes".to_string(),
                "ehthumbs.db".to_string(),
                
                // Language specific build/cache directories
                "node_modules/".to_string(),
                "__pycache__/".to_string(),
                "*.pyc".to_string(),
                "*.pyo".to_string(),
                ".pytest_cache/".to_string(),
                ".tox/".to_string(),
                ".coverage".to_string(),
                "htmlcov/".to_string(),
                ".nyc_output/".to_string(),
                "coverage/".to_string(),
                
                // Rust
                "target/".to_string(),
                "Cargo.lock".to_string(),
                
                // Java
                "*.class".to_string(),
                "*.jar".to_string(),
                "*.war".to_string(),
                "*.ear".to_string(),
                ".gradle/".to_string(),
                "build/".to_string(),
                
                // C/C++
                "*.o".to_string(),
                "*.so".to_string(),
                "*.a".to_string(),
                "*.dylib".to_string(),
                "*.dll".to_string(),
                "*.exe".to_string(),
                
                // Web development
                "dist/".to_string(),
                ".next/".to_string(),
                ".nuxt/".to_string(),
                ".output/".to_string(),
                ".vercel/".to_string(),
                ".netlify/".to_string(),
                
                // Package managers
                "yarn.lock".to_string(),
                "package-lock.json".to_string(),
                "pnpm-lock.yaml".to_string(),
                "Pipfile.lock".to_string(),
                "poetry.lock".to_string(),
                "Gemfile.lock".to_string(),
                "composer.lock".to_string(),
                
                // Version control
                ".git/".to_string(),
                ".svn/".to_string(),
                ".hg/".to_string(),
                ".bzr/".to_string(),
                
                // Documentation build outputs
                "_site/".to_string(),
                "site/".to_string(),
                "docs/_build/".to_string(),
                ".docusaurus/".to_string(),
                
                // Databases
                "*.db".to_string(),
                "*.sqlite".to_string(),
                "*.sqlite3".to_string(),
                
                // Archives
                "*.zip".to_string(),
                "*.tar".to_string(),
                "*.tar.gz".to_string(),
                "*.tar.bz2".to_string(),
                "*.rar".to_string(),
                "*.7z".to_string(),
                
                // Images (usually not code)
                "*.jpg".to_string(),
                "*.jpeg".to_string(),
                "*.png".to_string(),
                "*.gif".to_string(),
                "*.bmp".to_string(),
                "*.tiff".to_string(),
                "*.ico".to_string(),
                
                // Audio/Video
                "*.mp3".to_string(),
                "*.mp4".to_string(),
                "*.avi".to_string(),
                "*.mov".to_string(),
                "*.wmv".to_string(),
                "*.flv".to_string(),
                "*.wav".to_string(),
                "*.flac".to_string(),
                
                // Fonts
                "*.ttf".to_string(),
                "*.otf".to_string(),
                "*.woff".to_string(),
                "*.woff2".to_string(),
                "*.eot".to_string(),
            ],
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
        
        // Use override matcher for custom ignore patterns instead of add_ignore
        if !self.custom_ignores.is_empty() {
            use ignore::overrides::OverrideBuilder;
            let mut override_builder = OverrideBuilder::new(path_ref);
            
            for pattern in &self.custom_ignores {
                // Add patterns as ignore patterns (prefixed with !)
                let ignore_pattern = if pattern.starts_with('!') {
                    pattern.clone()
                } else {
                    format!("!{}", pattern)
                };
                
                // Ignore errors from invalid patterns
                let _ = override_builder.add(&ignore_pattern);
            }
            
            if let Ok(overrides) = override_builder.build() {
                builder.overrides(overrides);
            }
        }
        
        builder.build().filter_map(|entry| entry.ok())
    }
    
    pub fn should_include_file(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy().to_lowercase();
        
        // Check against custom ignore patterns
        for pattern in &self.custom_ignores {
            if self.matches_pattern(&path_str, pattern) {
                return false;
            }
        }
        
        // Additional intelligent checks
        if self.is_likely_generated_file(path) {
            return false;
        }
        
        if self.is_likely_binary_file(path) {
            return false;
        }
        
        true
    }
    
    fn matches_pattern(&self, path: &str, pattern: &str) -> bool {
        // Simple glob-like matching
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
    
    fn is_likely_generated_file(&self, path: &Path) -> bool {
        if let Some(filename) = path.file_name() {
            let filename_str = filename.to_string_lossy().to_lowercase();
            
            // Common generated file patterns
            let generated_patterns = vec![
                "generated", "auto", "autogen", "codegen", "_gen", ".gen",
                "build", "dist", "out", "output", "bin", "obj",
                "bundle", "minified", ".min.", "compiled",
                "protobuf", ".pb.", "thrift", ".thrift.",
                "swagger", "openapi", "schema",
            ];
            
            for pattern in generated_patterns {
                if filename_str.contains(pattern) {
                    return true;
                }
            }
        }
        
        false
    }
    
    fn is_likely_binary_file(&self, path: &Path) -> bool {
        if let Some(extension) = path.extension() {
            let ext_str = extension.to_string_lossy().to_lowercase();
            
            let binary_extensions = vec![
                "exe", "dll", "so", "dylib", "a", "lib", "o", "obj",
                "bin", "dat", "db", "sqlite", "sqlite3",
                "jpg", "jpeg", "png", "gif", "bmp", "tiff", "ico",
                "mp3", "mp4", "avi", "mov", "wmv", "flv", "wav", "flac",
                "zip", "tar", "gz", "bz2", "rar", "7z", "dmg", "iso",
                "ttf", "otf", "woff", "woff2", "eot",
                "pdf", "doc", "docx", "xls", "xlsx", "ppt", "pptx",
                "class", "jar", "war", "ear", "pyc", "pyo",
            ];
            
            return binary_extensions.contains(&ext_str.as_str());
        }
        
        false
    }
} 