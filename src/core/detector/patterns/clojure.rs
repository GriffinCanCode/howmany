use regex::Regex;

pub struct ClojurePatterns {
    external_patterns: Vec<Regex>,
    cache_patterns: Vec<Regex>,
    extensions: Vec<String>,
}

impl ClojurePatterns {
    pub fn new() -> Self {
        let external_patterns = vec![
            // Leiningen artifacts
            Regex::new(r"target/").unwrap(),
            Regex::new(r"\.lein-deps-sum").unwrap(),
            Regex::new(r"\.lein-repl-history").unwrap(),
            Regex::new(r"\.lein-failures").unwrap(),
            Regex::new(r"\.nrepl-port").unwrap(),
            
            // Boot artifacts
            Regex::new(r"\.boot/").unwrap(),
            Regex::new(r"boot\.properties").unwrap(),
            
            // Clojure CLI/deps.edn artifacts
            Regex::new(r"\.cpcache/").unwrap(),
            
            // Shadow-cljs artifacts
            Regex::new(r"\.shadow-cljs/").unwrap(),
            Regex::new(r"shadow-cljs\.edn").unwrap(),
            
            // Figwheel artifacts
            Regex::new(r"figwheel_server\.log").unwrap(),
            Regex::new(r"\.figwheel/").unwrap(),
            
            // ClojureScript artifacts
            Regex::new(r"\.cljs_rhino_repl/").unwrap(),
            Regex::new(r"out/").unwrap(),
            Regex::new(r"resources/public/js/").unwrap(),
            
            // Java artifacts (common in Clojure projects)
            Regex::new(r"\.class$").unwrap(),
            Regex::new(r"\.jar$").unwrap(),
            
            // REPL artifacts
            Regex::new(r"\.rebel_readline_history").unwrap(),
            
            // Temporary files
            Regex::new(r"\.tmp/").unwrap(),
            Regex::new(r"tmp/").unwrap(),
            Regex::new(r"\.swp$").unwrap(),
            Regex::new(r"\.swo$").unwrap(),
            Regex::new(r"~$").unwrap(),
        ];

        let cache_patterns = vec![
            // Leiningen cache
            Regex::new(r"target/").unwrap(),
            Regex::new(r"\.lein-").unwrap(),
            
            // Boot cache
            Regex::new(r"\.boot/").unwrap(),
            
            // Clojure CLI cache
            Regex::new(r"\.cpcache/").unwrap(),
            
            // Shadow-cljs cache
            Regex::new(r"\.shadow-cljs/").unwrap(),
            
            // Figwheel cache
            Regex::new(r"\.figwheel/").unwrap(),
            
            // ClojureScript cache
            Regex::new(r"out/").unwrap(),
        ];

        let extensions = vec![
            // Clojure source files
            "clj".to_string(),
            
            // ClojureScript files
            "cljs".to_string(),
            
            // Clojure/ClojureScript shared files
            "cljc".to_string(),
            
            // EDN data files
            "edn".to_string(),
            
            // Boot build files
            "boot".to_string(),
            
            // Configuration files
            "json".to_string(),
            "yaml".to_string(),
            "yml".to_string(),
            
            // Documentation
            "md".to_string(),
            "rst".to_string(),
            "txt".to_string(),
            
            // Web files (for ClojureScript)
            "html".to_string(),
            "css".to_string(),
            "js".to_string(),
            
            // Scripts
            "sh".to_string(),
            
            // Java files (for interop)
            "java".to_string(),
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
            // Leiningen files
            "project.clj", "profiles.clj",
            
            // Clojure CLI files
            "deps.edn", "bb.edn",
            
            // Boot files
            "build.boot", "boot.properties",
            
            // Shadow-cljs files
            "shadow-cljs.edn",
            
            // Figwheel files
            "figwheel.edn", "figwheel-main.edn",
            
            // Babashka files
            "bb.edn", "babashka.edn",
            
            // Core files
            "core.clj", "main.clj",
            
            // Test files
            "test.clj", "tests.clj",
            
            // Build scripts
            "Makefile", "makefile", "build.sh",
            
            // Documentation
            "README.md", "CHANGELOG.md", "LICENSE",
            
            // CI/CD
            ".github/workflows/clojure.yml",
            ".travis.yml", "appveyor.yml",
            "circle.yml", ".circleci/config.yml",
            
            // Git
            ".gitignore", ".gitattributes",
            
            // Docker
            "Dockerfile", "docker-compose.yml",
            
            // Environment
            ".env", ".env.example",
            
            // REPL files
            ".nrepl-port", ".rebel_readline_history",
            
            // Common namespace files
            "user.clj", "dev.clj",
            
            // Web development
            "handler.clj", "routes.clj", "middleware.clj",
            
            // Database
            "db.clj", "schema.clj", "migrations.clj",
            
            // Configuration
            "config.clj", "settings.clj",
            
            // Utilities
            "utils.clj", "helpers.clj", "common.clj",
            
            // Examples
            "example.clj", "examples.clj",
            
            // Benchmarks
            "bench.clj", "benchmarks.clj",
        ]
    }
} 