use regex::Regex;

pub struct ErlangPatterns {
    external_patterns: Vec<Regex>,
    cache_patterns: Vec<Regex>,
    extensions: Vec<String>,
}

impl ErlangPatterns {
    pub fn new() -> Self {
        let external_patterns = vec![
            // Rebar3 build artifacts
            Regex::new(r"_build/").unwrap(),
            Regex::new(r"rebar\.lock").unwrap(),
            Regex::new(r"rebar3\.crashdump").unwrap(),
            
            // Compiled BEAM files
            Regex::new(r"\.beam$").unwrap(),
            Regex::new(r"ebin/").unwrap(),
            
            // Dialyzer
            Regex::new(r"\.plt$").unwrap(),
            Regex::new(r"\.dialyzer_plt").unwrap(),
            
            // Documentation
            Regex::new(r"doc/").unwrap(),
            Regex::new(r"docs/").unwrap(),
            Regex::new(r"edoc-info").unwrap(),
            
            // Logs
            Regex::new(r"logs/").unwrap(),
            Regex::new(r"log/").unwrap(),
            Regex::new(r"\.log$").unwrap(),
            
            // Releases
            Regex::new(r"_rel/").unwrap(),
            Regex::new(r"rel/").unwrap(),
            
            // Common Test artifacts
            Regex::new(r"ct_logs/").unwrap(),
            Regex::new(r"\.ct$").unwrap(),
            
            // Coverage
            Regex::new(r"\.coverdata$").unwrap(),
            Regex::new(r"cover/").unwrap(),
            
            // Mnesia database
            Regex::new(r"Mnesia\.").unwrap(),
            Regex::new(r"\.DAT$").unwrap(),
            Regex::new(r"\.LOG$").unwrap(),
            Regex::new(r"\.TMP$").unwrap(),
            
            // Temporary files
            Regex::new(r"\.tmp/").unwrap(),
            Regex::new(r"tmp/").unwrap(),
            Regex::new(r"\.swp$").unwrap(),
            Regex::new(r"\.swo$").unwrap(),
            Regex::new(r"~$").unwrap(),
            
            // Crashdumps
            Regex::new(r"erl_crash\.dump").unwrap(),
            Regex::new(r"\.crashdump$").unwrap(),
        ];

        let cache_patterns = vec![
            // Rebar3 cache
            Regex::new(r"_build/").unwrap(),
            
            // Compiled files
            Regex::new(r"\.beam$").unwrap(),
            Regex::new(r"ebin/").unwrap(),
            
            // Dialyzer cache
            Regex::new(r"\.plt$").unwrap(),
            
            // Documentation cache
            Regex::new(r"doc/").unwrap(),
            
            // Coverage cache
            Regex::new(r"\.coverdata$").unwrap(),
        ];

        let extensions = vec![
            // Erlang source files
            "erl".to_string(),
            
            // Erlang header files
            "hrl".to_string(),
            
            // Erlang script files
            "escript".to_string(),
            
            // Configuration files
            "config".to_string(),
            "sys.config".to_string(),
            "vm.args".to_string(),
            
            // Application files
            "app".to_string(),
            "app.src".to_string(),
            
            // Release files
            "rel".to_string(),
            
            // Common formats in Erlang projects
            "json".to_string(),
            "yaml".to_string(),
            "yml".to_string(),
            "xml".to_string(),
            
            // Documentation
            "md".to_string(),
            "rst".to_string(),
            
            // Shell scripts
            "sh".to_string(),
            
            // Makefile
            "mk".to_string(),
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
            // Rebar files
            "rebar.config", "rebar.lock", "rebar3",
            
            // Erlang release files
            "relx.config", "sys.config", "vm.args",
            
            // Common scripts
            "start.sh", "stop.sh", "restart.sh",
            
            // Build files
            "Makefile", "GNUmakefile", "makefile",
            "Emakefile",
            
            // Application files
            "app.config",
            
            // CI/CD
            ".github/workflows/erlang.yml",
            ".travis.yml", "appveyor.yml",
            
            // Docker
            "Dockerfile", "docker-compose.yml",
            
            // Editor
            ".editorconfig", ".gitignore",
            
            // OTP release
            "start_erl.data", "RELEASES",
        ]
    }

    pub fn get_common_directories() -> Vec<&'static str> {
        vec![
            // Source
            "src/", "include/", "priv/",
            
            // Tests
            "test/", "tests/", "ct/",
            
            // Documentation
            "doc/", "docs/",
            
            // Build
            "_build/", "ebin/", "deps/",
            
            // Config
            "config/", "priv/",
            
            // Release
            "_rel/", "rel/", "releases/",
            
            // Logs
            "logs/", "log/",
            
            // Scripts
            "scripts/", "bin/",
            
            // Examples
            "examples/", "samples/",
        ]
    }
} 