use regex::Regex;

pub struct JuliaPatterns {
    external_patterns: Vec<Regex>,
    cache_patterns: Vec<Regex>,
    extensions: Vec<String>,
}

impl JuliaPatterns {
    pub fn new() -> Self {
        let external_patterns = vec![
            // Julia package artifacts
            Regex::new(r"Manifest\.toml").unwrap(),
            Regex::new(r"\.julia/").unwrap(),
            
            // Compiled artifacts
            Regex::new(r"\.ji$").unwrap(),
            Regex::new(r"\.so$").unwrap(),
            Regex::new(r"\.dylib$").unwrap(),
            Regex::new(r"\.dll$").unwrap(),
            
            // Package development
            Regex::new(r"deps/build\.log").unwrap(),
            Regex::new(r"deps/deps\.jl").unwrap(),
            
            // Documentation
            Regex::new(r"docs/build/").unwrap(),
            Regex::new(r"docs/site/").unwrap(),
            
            // Test artifacts
            Regex::new(r"\.coverage").unwrap(),
            Regex::new(r"lcov\.info").unwrap(),
            
            // Jupyter notebooks (often used with Julia)
            Regex::new(r"\.ipynb_checkpoints/").unwrap(),
            
            // Temporary files
            Regex::new(r"\.tmp/").unwrap(),
            Regex::new(r"tmp/").unwrap(),
            Regex::new(r"\.swp$").unwrap(),
            Regex::new(r"\.swo$").unwrap(),
            Regex::new(r"~$").unwrap(),
        ];

        let cache_patterns = vec![
            // Julia package cache
            Regex::new(r"\.julia/").unwrap(),
            
            // Documentation cache
            Regex::new(r"docs/build/").unwrap(),
            
            // Test cache
            Regex::new(r"\.coverage").unwrap(),
            
            // Compiled cache
            Regex::new(r"\.ji$").unwrap(),
        ];

        let extensions = vec![
            // Julia source files
            "jl".to_string(),
            
            // Jupyter notebooks
            "ipynb".to_string(),
            
            // Configuration files
            "toml".to_string(),
            "json".to_string(),
            "yaml".to_string(),
            "yml".to_string(),
            
            // Documentation
            "md".to_string(),
            "rst".to_string(),
            
            // Data files (common in Julia projects)
            "csv".to_string(),
            "tsv".to_string(),
            "json".to_string(),
            "hdf5".to_string(),
            "h5".to_string(),
            
            // Scripts
            "sh".to_string(),
            
            // C/C++ files (for Julia packages with C extensions)
            "c".to_string(),
            "cpp".to_string(),
            "h".to_string(),
            "hpp".to_string(),
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
            // Package files
            "Project.toml", "Manifest.toml",
            
            // Build files
            "deps/build.jl", "deps/deps.jl",
            
            // Documentation
            "docs/make.jl", "docs/Project.toml",
            
            // Test files
            "test/runtests.jl", "runtests.jl",
            
            // Benchmark files
            "benchmark/benchmarks.jl",
            
            // Examples
            "examples/example.jl",
            
            // Build scripts
            "Makefile", "makefile", "build.sh",
            
            // CI/CD
            ".github/workflows/ci.yml",
            ".github/workflows/julia.yml",
            ".travis.yml", "appveyor.yml",
            "circle.yml", ".circleci/config.yml",
            
            // Git
            ".gitignore", ".gitattributes",
            
            // Documentation
            "README.md", "CHANGELOG.md", "LICENSE",
            
            // Configuration
            ".juliarc.jl", "startup.jl",
            
            // Docker
            "Dockerfile", "docker-compose.yml",
            
            // Jupyter
            "jupyter_notebook_config.py",
            
            // Environment
            ".env", ".env.example",
            
            // Coverage
            ".codecov.yml", "codecov.yml",
            
            // Package development
            "REQUIRE", "METADATA.jl",
            
            // Main files
            "main.jl", "Main.jl",
            
            // Common module files
            "src/MyPackage.jl",
        ]
    }
} 