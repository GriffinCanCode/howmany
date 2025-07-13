use std::path::Path;
use regex::Regex;

pub struct FileDetector {
    // Common patterns for external/generated files
    external_patterns: Vec<Regex>,
    // Common patterns for cache/build directories
    cache_patterns: Vec<Regex>,
    // Programming language extensions we care about
    code_extensions: Vec<String>,
}

impl FileDetector {
    pub fn new() -> Self {
        let external_patterns = vec![
            // Node.js
            Regex::new(r"node_modules").unwrap(),
            Regex::new(r"package-lock\.json").unwrap(),
            Regex::new(r"yarn\.lock").unwrap(),
            
            // Python
            Regex::new(r"__pycache__").unwrap(),
            Regex::new(r"\.pyc$").unwrap(),
            Regex::new(r"\.pyo$").unwrap(),
            Regex::new(r"site-packages").unwrap(),
            Regex::new(r"\.egg-info").unwrap(),
            
            // Rust
            Regex::new(r"target/").unwrap(),
            Regex::new(r"Cargo\.lock").unwrap(),
            
            // Java
            Regex::new(r"\.class$").unwrap(),
            Regex::new(r"target/").unwrap(),
            Regex::new(r"\.m2/").unwrap(),
            
            // C/C++
            Regex::new(r"\.o$").unwrap(),
            Regex::new(r"\.so$").unwrap(),
            Regex::new(r"\.a$").unwrap(),
            Regex::new(r"\.dylib$").unwrap(),
            
            // General build artifacts
            Regex::new(r"build/").unwrap(),
            Regex::new(r"dist/").unwrap(),
            Regex::new(r"\.git/").unwrap(),
            Regex::new(r"\.svn/").unwrap(),
            Regex::new(r"\.hg/").unwrap(),
        ];

        let cache_patterns = vec![
            // Generic cache patterns
            Regex::new(r"(?i)/cache/").unwrap(),
            Regex::new(r"(?i)/temp/").unwrap(),
            Regex::new(r"(?i)/tmp/").unwrap(),
            Regex::new(r"\.cache").unwrap(),
            Regex::new(r"\.tmp$").unwrap(),
            
            // IDE/Editor files
            Regex::new(r"\.vscode").unwrap(),
            Regex::new(r"\.idea").unwrap(),
            Regex::new(r"\.DS_Store").unwrap(),
            Regex::new(r"Thumbs\.db").unwrap(),
            
            // Log files
            Regex::new(r"\.log$").unwrap(),
            Regex::new(r"logs/").unwrap(),
        ];

        let code_extensions = vec![
            // Common programming languages
            "rs".to_string(), "py".to_string(), "js".to_string(), "ts".to_string(),
            "jsx".to_string(), "tsx".to_string(), "java".to_string(), "c".to_string(),
            "cpp".to_string(), "cc".to_string(), "cxx".to_string(), "h".to_string(),
            "hpp".to_string(), "cs".to_string(), "php".to_string(), "rb".to_string(),
            "go".to_string(), "swift".to_string(), "kt".to_string(), "scala".to_string(),
            "clj".to_string(), "cljs".to_string(), "hs".to_string(), "ml".to_string(), "fs".to_string(),
            "elm".to_string(), "dart".to_string(), "lua".to_string(), "pl".to_string(),
            "r".to_string(), "m".to_string(), "mm".to_string(), "erl".to_string(), "ex".to_string(),
            "exs".to_string(), "jl".to_string(), "sql".to_string(), "zig".to_string(),
            
            // Web technologies
            "html".to_string(), "css".to_string(), "scss".to_string(), "sass".to_string(),
            "less".to_string(), "vue".to_string(), "svelte".to_string(),
            
            // Config/markup files that might contain user code
            "json".to_string(), "xml".to_string(), "yaml".to_string(), "yml".to_string(),
            "toml".to_string(), "ini".to_string(), "cfg".to_string(), "conf".to_string(),
            
            // Scripts
            "sh".to_string(), "bash".to_string(), "zsh".to_string(), "fish".to_string(),
            "ps1".to_string(), "bat".to_string(), "cmd".to_string(),
            
            // Documentation that might be user-created
            "md".to_string(), "rst".to_string(), "txt".to_string(), "adoc".to_string(),
            "asciidoc".to_string(),
        ];

        Self {
            external_patterns,
            cache_patterns,
            code_extensions,
        }
    }

    pub fn is_user_created_file(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();
        
        // Check if it matches external/dependency patterns
        for pattern in &self.external_patterns {
            if pattern.is_match(&path_str) {
                return false;
            }
        }
        
        // Check if it matches cache patterns
        for pattern in &self.cache_patterns {
            if pattern.is_match(&path_str) {
                return false;
            }
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
            let script_names = vec![
                "Makefile", "Dockerfile", "Rakefile", "Gemfile", "Pipfile",
                "requirements.txt", "setup.py", "pyproject.toml", "poetry.lock",
                "package.json", "tsconfig.json", "webpack.config.js",
                "Cargo.toml", "build.rs", "main.rs", "lib.rs",
            ];
            
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