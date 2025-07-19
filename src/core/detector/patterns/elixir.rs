use regex::Regex;

pub struct ElixirPatterns {
    external_patterns: Vec<Regex>,
    cache_patterns: Vec<Regex>,
    extensions: Vec<String>,
}

impl ElixirPatterns {
    pub fn new() -> Self {
        let external_patterns = vec![
            // Mix build artifacts
            Regex::new(r"_build/").unwrap(),
            Regex::new(r"deps/").unwrap(),
            Regex::new(r"mix\.lock").unwrap(),
            
            // Hex packages
            Regex::new(r"\.hex/").unwrap(),
            Regex::new(r"hex_metadata\.config").unwrap(),
            
            // Compiled files
            Regex::new(r"\.beam$").unwrap(),
            Regex::new(r"\.plt$").unwrap(),
            
            // OTP releases
            Regex::new(r"_rel/").unwrap(),
            Regex::new(r"rel/").unwrap(),
            
            // Documentation
            Regex::new(r"doc/").unwrap(),
            Regex::new(r"docs/").unwrap(),
            
            // Coverage reports
            Regex::new(r"cover/").unwrap(),
            Regex::new(r"coverage/").unwrap(),
            
            // Logs
            Regex::new(r"logs/").unwrap(),
            Regex::new(r"log/").unwrap(),
            
            // Temporary files
            Regex::new(r"\.tmp/").unwrap(),
            Regex::new(r"tmp/").unwrap(),
            
            // Dialyzer
            Regex::new(r"\.dialyzer_plt").unwrap(),
            
            // Phoenix specific
            Regex::new(r"priv/static/").unwrap(),
            Regex::new(r"priv/gettext/").unwrap(),
            
            // Node.js artifacts (for Phoenix projects)
            Regex::new(r"node_modules/").unwrap(),
            Regex::new(r"package-lock\.json").unwrap(),
            
            // Temporary files
            Regex::new(r"\.swp$").unwrap(),
            Regex::new(r"\.swo$").unwrap(),
            Regex::new(r"~$").unwrap(),
        ];

        let cache_patterns = vec![
            // Mix cache
            Regex::new(r"_build/").unwrap(),
            
            // Dependencies cache
            Regex::new(r"deps/").unwrap(),
            
            // Hex cache
            Regex::new(r"\.hex/").unwrap(),
            
            // Dialyzer cache
            Regex::new(r"\.dialyzer_plt").unwrap(),
            
            // Coverage cache
            Regex::new(r"cover/").unwrap(),
            
            // Documentation cache
            Regex::new(r"doc/").unwrap(),
        ];

        let extensions = vec![
            // Elixir source files
            "ex".to_string(),
            
            // Elixir script files
            "exs".to_string(),
            
            // Elixir templates
            "eex".to_string(),
            "heex".to_string(),
            "leex".to_string(),
            
            // Configuration files
            "json".to_string(),
            "yaml".to_string(),
            "yml".to_string(),
            
            // Documentation
            "md".to_string(),
            "markdown".to_string(),
            
            // Web assets (Phoenix)
            "js".to_string(),
            "css".to_string(),
            "scss".to_string(),
            "html".to_string(),
            
            // Database files
            "sql".to_string(),
            
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
            // Mix files
            "mix.exs", "mix.lock",
            
            // Configuration
            "config.exs", "dev.exs", "prod.exs", "test.exs",
            "runtime.exs", "releases.exs",
            
            // Phoenix files
            "phoenix_static_buildpack.config",
            "elixir_buildpack.config",
            
            // OTP applications
            "rebar.config", "rebar.lock",
            
            // Documentation
            "README.md", "CHANGELOG.md", "LICENSE",
            
            // Build scripts
            "Makefile", "makefile", "build.sh",
            
            // CI/CD
            ".github/workflows/elixir.yml",
            ".travis.yml", "appveyor.yml",
            "circle.yml", ".circleci/config.yml",
            
            // Git
            ".gitignore", ".gitattributes",
            
            // Editor
            ".formatter.exs", ".credo.exs",
            
            // Deployment
            "Dockerfile", "docker-compose.yml",
            "Procfile", "app.json",
            
            // Environment
            ".env", ".env.example",
            
            // Database
            "seeds.exs", "structure.sql",
            
            // Tests
            "test_helper.exs",
            
            // Benchmarks
            "bench.exs", "benchmarks.exs",
            
            // Examples
            "example.exs", "examples.exs",
            
            // Assets (Phoenix)
            "package.json", "webpack.config.js",
            "brunch-config.js", "postcss.config.js",
        ]
    }
} 