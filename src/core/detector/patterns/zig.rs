use regex::Regex;

pub struct ZigPatterns {
    external_patterns: Vec<Regex>,
    cache_patterns: Vec<Regex>,
    extensions: Vec<String>,
}

impl ZigPatterns {
    pub fn new() -> Self {
        let external_patterns = vec![
            // Zig build artifacts
            Regex::new(r"zig-cache/").unwrap(),
            Regex::new(r"zig-out/").unwrap(),
            
            // Compiled objects
            Regex::new(r"\.o$").unwrap(),
            Regex::new(r"\.obj$").unwrap(),
            Regex::new(r"\.lib$").unwrap(),
            Regex::new(r"\.a$").unwrap(),
            Regex::new(r"\.so$").unwrap(),
            Regex::new(r"\.dylib$").unwrap(),
            Regex::new(r"\.dll$").unwrap(),
            Regex::new(r"\.exe$").unwrap(),
            
            // LLVM artifacts
            Regex::new(r"\.bc$").unwrap(),
            Regex::new(r"\.ll$").unwrap(),
            
            // WebAssembly artifacts
            Regex::new(r"\.wasm$").unwrap(),
            
            // Debug info
            Regex::new(r"\.pdb$").unwrap(),
            Regex::new(r"\.dSYM/").unwrap(),
            
            // Temporary files
            Regex::new(r"\.tmp/").unwrap(),
            Regex::new(r"tmp/").unwrap(),
            Regex::new(r"\.swp$").unwrap(),
            Regex::new(r"\.swo$").unwrap(),
            Regex::new(r"~$").unwrap(),
        ];

        let cache_patterns = vec![
            // Zig cache
            Regex::new(r"zig-cache/").unwrap(),
            Regex::new(r"zig-out/").unwrap(),
            
            // Build cache
            Regex::new(r"\.o$").unwrap(),
            Regex::new(r"\.obj$").unwrap(),
        ];

        let extensions = vec![
            // Zig source files
            "zig".to_string(),
            
            // Zig build files
            "zon".to_string(),
            
            // Assembly files
            "s".to_string(),
            "asm".to_string(),
            
            // C files (often used with Zig)
            "c".to_string(),
            "h".to_string(),
            "cpp".to_string(),
            "hpp".to_string(),
            
            // Configuration files
            "json".to_string(),
            "yaml".to_string(),
            "yml".to_string(),
            "toml".to_string(),
            
            // Documentation
            "md".to_string(),
            "rst".to_string(),
            "txt".to_string(),
            
            // Scripts
            "sh".to_string(),
            
            // WebAssembly
            "wat".to_string(),
            "wast".to_string(),
        ];

        Self {
            external_patterns,
            cache_patterns,
            extensions,
        }
    }

    pub fn get_external_patterns(&self) -> &[Regex] {
        &self.external_patterns
    }

    pub fn get_cache_patterns(&self) -> &[Regex] {
        &self.cache_patterns
    }

    pub fn get_extensions(&self) -> &[String] {
        &self.extensions
    }

    pub fn get_script_names() -> Vec<&'static str> {
        vec![
            // Zig build files
            "build.zig", "build.zig.zon",
            
            // Main files
            "main.zig", "src/main.zig",
            
            // Test files
            "test.zig", "tests.zig",
            
            // Library files
            "lib.zig", "root.zig",
            
            // Build scripts
            "Makefile", "makefile", "build.sh",
            
            // Documentation
            "README.md", "CHANGELOG.md", "LICENSE",
            
            // CI/CD
            ".github/workflows/zig.yml",
            ".travis.yml", "appveyor.yml",
            "circle.yml", ".circleci/config.yml",
            
            // Git
            ".gitignore", ".gitattributes",
            
            // Docker
            "Dockerfile", "docker-compose.yml",
            
            // Environment
            ".env", ".env.example",
            
            // Configuration
            "config.zig", "settings.zig",
            
            // Common module files
            "utils.zig", "helpers.zig", "common.zig",
            
            // Examples
            "example.zig", "examples.zig",
            
            // Benchmarks
            "bench.zig", "benchmarks.zig",
            
            // Embedded development
            "linker.ld", "memory.x",
            
            // Cross-compilation
            "cross.zig", "target.zig",
        ]
    }
} 