use regex::Regex;

pub struct HaskellPatterns {
    external_patterns: Vec<Regex>,
    cache_patterns: Vec<Regex>,
    extensions: Vec<String>,
}

impl HaskellPatterns {
    pub fn new() -> Self {
        let external_patterns = vec![
            // Stack build artifacts
            Regex::new(r"\.stack-work/").unwrap(),
            Regex::new(r"stack\.yaml\.lock").unwrap(),
            
            // Cabal build artifacts
            Regex::new(r"dist/").unwrap(),
            Regex::new(r"dist-newstyle/").unwrap(),
            Regex::new(r"cabal\.project\.local").unwrap(),
            Regex::new(r"\.cabal-sandbox/").unwrap(),
            Regex::new(r"cabal\.sandbox\.config").unwrap(),
            
            // GHC artifacts
            Regex::new(r"\.hi$").unwrap(),
            Regex::new(r"\.o$").unwrap(),
            Regex::new(r"\.dyn_hi$").unwrap(),
            Regex::new(r"\.dyn_o$").unwrap(),
            Regex::new(r"\.p_hi$").unwrap(),
            Regex::new(r"\.p_o$").unwrap(),
            
            // Haddock documentation
            Regex::new(r"\.haddock/").unwrap(),
            Regex::new(r"haddock\.html").unwrap(),
            
            // Nix build artifacts
            Regex::new(r"result").unwrap(),
            Regex::new(r"result-*").unwrap(),
            
            // IDE files
            Regex::new(r"\.ghc\.environment\.").unwrap(),
            Regex::new(r"\.hie/").unwrap(),
            Regex::new(r"\.hiedb").unwrap(),
            
            // Package databases
            Regex::new(r"package\.conf\.d/").unwrap(),
            
            // Profiling files
            Regex::new(r"\.prof$").unwrap(),
            Regex::new(r"\.hp$").unwrap(),
            Regex::new(r"\.ps$").unwrap(),
            Regex::new(r"\.aux$").unwrap(),
            
            // Test coverage
            Regex::new(r"\.tix$").unwrap(),
            Regex::new(r"hpc_index\.html").unwrap(),
            
            // Temporary files
            Regex::new(r"\.swp$").unwrap(),
            Regex::new(r"\.swo$").unwrap(),
            Regex::new(r"~$").unwrap(),
        ];

        let cache_patterns = vec![
            // Stack cache
            Regex::new(r"\.stack-work/").unwrap(),
            
            // Cabal cache
            Regex::new(r"dist/").unwrap(),
            Regex::new(r"dist-newstyle/").unwrap(),
            
            // GHC cache
            Regex::new(r"\.ghc/").unwrap(),
            
            // HIE cache
            Regex::new(r"\.hie/").unwrap(),
        ];

        let extensions = vec![
            // Haskell source files
            "hs".to_string(),
            
            // Literate Haskell
            "lhs".to_string(),
            
            // Haskell C files
            "hsc".to_string(),
            
            // Cabal files
            "cabal".to_string(),
            
            // C files (often used with Haskell)
            "c".to_string(),
            "h".to_string(),
            
            // Configuration files
            "yaml".to_string(),
            "yml".to_string(),
            "json".to_string(),
            
            // Documentation
            "md".to_string(),
            "rst".to_string(),
            "txt".to_string(),
            
            // Nix files (common in Haskell projects)
            "nix".to_string(),
            
            // Shell scripts
            "sh".to_string(),
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
            // Stack files
            "stack.yaml", "stack.yaml.lock",
            
            // Cabal files
            "cabal.project", "cabal.project.local",
            "Setup.hs", "Setup.lhs",
            
            // Package files
            "package.yaml", "hpack.yaml",
            
            // Build scripts
            "Makefile", "makefile", "build.sh",
            
            // Nix files
            "default.nix", "shell.nix", "release.nix",
            
            // Documentation
            "README.md", "CHANGELOG.md", "LICENSE",
            
            // Configuration
            ".ghci", ".ghc-flags", ".hspec",
            
            // CI/CD
            ".github/workflows/haskell.yml",
            ".travis.yml", "appveyor.yml",
            "circle.yml", ".circleci/config.yml",
            
            // Git
            ".gitignore", ".gitattributes",
            
            // Editor
            ".dir-locals.el", ".projectile",
            
            // Benchmarks
            "bench.hs", "benchmarks.hs",
            
            // Tests
            "test.hs", "tests.hs", "Spec.hs",
            
            // Examples
            "example.hs", "examples.hs",
            
            // Main files
            "Main.hs", "main.hs",
            
            // Library files
            "Lib.hs", "lib.hs",
        ]
    }
} 