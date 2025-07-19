use regex::Regex;

pub struct GeneralPatterns {
    external_patterns: Vec<Regex>,
    cache_patterns: Vec<Regex>,
    extensions: Vec<String>,
}

impl GeneralPatterns {
    pub fn new() -> Self {
        let external_patterns = vec![
            // Package managers (general)
            Regex::new(r"vendor/").unwrap(),
            Regex::new(r"deps/").unwrap(),
            Regex::new(r"packages/").unwrap(),
        ];

        let cache_patterns = vec![
            // Generic cache patterns (case-insensitive)
            Regex::new(r"(?i)/cache/").unwrap(),
            Regex::new(r"(?i)/temp/").unwrap(),
            Regex::new(r"(?i)/tmp/").unwrap(),
            
            // Log files
            Regex::new(r"logs/").unwrap(),
        ];

        let extensions = vec![
            // Configuration files
            "json".to_string(), "xml".to_string(), "yaml".to_string(),
            "yml".to_string(), "toml".to_string(), "ini".to_string(),
            "cfg".to_string(), "conf".to_string(), "config".to_string(),
            
            // Scripts
            "sh".to_string(), "bash".to_string(), "zsh".to_string(),
            "fish".to_string(), "ps1".to_string(), "bat".to_string(),
            "cmd".to_string(),
            
            // Documentation
            "md".to_string(), "rst".to_string(), "txt".to_string(),
            "adoc".to_string(), "asciidoc".to_string(),
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
            "Dockerfile", "docker-compose.yml", "docker-compose.yaml",
            "Containerfile", "Vagrantfile", "Procfile", ".gitignore",
            ".dockerignore", ".editorconfig", "LICENSE", "README.md",
            "CHANGELOG.md", "CONTRIBUTING.md", "CODE_OF_CONDUCT.md",
        ]
    }
} 