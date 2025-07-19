use regex::Regex;

pub struct PerlPatterns {
    external_patterns: Vec<Regex>,
    cache_patterns: Vec<Regex>,
    extensions: Vec<String>,
}

impl PerlPatterns {
    pub fn new() -> Self {
        let external_patterns = vec![
            // CPAN build artifacts
            Regex::new(r"blib/").unwrap(),
            Regex::new(r"_build/").unwrap(),
            Regex::new(r"Build$").unwrap(),
            Regex::new(r"\.build/").unwrap(),
            
            // Module::Build artifacts
            Regex::new(r"MYMETA\.yml").unwrap(),
            Regex::new(r"MYMETA\.json").unwrap(),
            Regex::new(r"META\.yml").unwrap(),
            Regex::new(r"META\.json").unwrap(),
            
            // ExtUtils::MakeMaker artifacts
            Regex::new(r"Makefile$").unwrap(),
            Regex::new(r"Makefile\.old").unwrap(),
            Regex::new(r"pm_to_blib").unwrap(),
            
            // PAR/PAR::Packer artifacts
            Regex::new(r"\.par$").unwrap(),
            Regex::new(r"\.pp$").unwrap(),
            
            // Perl archives
            Regex::new(r"\.phar$").unwrap(),
            
            // Coverage
            Regex::new(r"cover_db/").unwrap(),
            Regex::new(r"\.gcov$").unwrap(),
            
            // Temporary files
            Regex::new(r"\.tmp/").unwrap(),
            Regex::new(r"tmp/").unwrap(),
            Regex::new(r"\.swp$").unwrap(),
            Regex::new(r"\.swo$").unwrap(),
            Regex::new(r"~$").unwrap(),
            
            // Perl specific temp files
            Regex::new(r"\.bak$").unwrap(),
            Regex::new(r"\.orig$").unwrap(),
            
            // Carton (Perl dependency manager)
            Regex::new(r"local/").unwrap(),
            Regex::new(r"\.carton/").unwrap(),
            
            // Perlbrew
            Regex::new(r"\.perlbrew/").unwrap(),
            
            // Dist::Zilla
            Regex::new(r"\.build/").unwrap(),
            
            // Minilla
            Regex::new(r"\.minilla/").unwrap(),
            
            // Logs
            Regex::new(r"\.log$").unwrap(),
            Regex::new(r"logs/").unwrap(),
        ];

        let cache_patterns = vec![
            // Build cache
            Regex::new(r"blib/").unwrap(),
            Regex::new(r"_build/").unwrap(),
            
            // Coverage cache
            Regex::new(r"cover_db/").unwrap(),
            
            // Carton cache
            Regex::new(r"local/").unwrap(),
            
            // Dist::Zilla cache
            Regex::new(r"\.build/").unwrap(),
        ];

        let extensions = vec![
            // Perl source files
            "pl".to_string(),
            "pm".to_string(),
            "pod".to_string(),
            "t".to_string(),
            
            // Perl scripts
            "perl".to_string(),
            "cgi".to_string(),
            
            // Perl 6/Raku (for compatibility)
            "p6".to_string(),
            "pl6".to_string(),
            "pm6".to_string(),
            "raku".to_string(),
            "rakumod".to_string(),
            
            // Configuration files
            "yml".to_string(),
            "yaml".to_string(),
            "json".to_string(),
            "ini".to_string(),
            "conf".to_string(),
            
            // Documentation
            "md".to_string(),
            "rst".to_string(),
            "txt".to_string(),
            
            // Web files (for Perl web apps)
            "html".to_string(),
            "css".to_string(),
            "js".to_string(),
            
            // Shell scripts
            "sh".to_string(),
            "bash".to_string(),
            
            // SQL
            "sql".to_string(),
            
            // XML
            "xml".to_string(),
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
            // CPAN files
            "Makefile.PL", "Build.PL", "META.yml", "META.json",
            "MYMETA.yml", "MYMETA.json", "MANIFEST", "MANIFEST.SKIP",
            
            // Carton files
            "cpanfile", "cpanfile.snapshot",
            
            // Dist::Zilla
            "dist.ini", "weaver.ini",
            
            // Minilla
            "minil.toml",
            
            // App::cpm
            "cpm.yml",
            
            // Perl version managers
            ".perl-version", ".perlbrew",
            
            // Build files
            "Makefile", "GNUmakefile", "makefile",
            
            // CI/CD
            ".github/workflows/perl.yml",
            ".travis.yml", "appveyor.yml",
            
            // Docker
            "Dockerfile", "docker-compose.yml",
            
            // Editor
            ".editorconfig", ".gitignore",
            ".perltidyrc", ".perlcriticrc",
            
            // Scripts
            "run.sh", "build.sh", "deploy.sh",
            "install.sh", "test.sh",
            
            // Web server configs
            ".htaccess", "httpd.conf",
            
            // Perl specific
            "perllocal.pod", "perldoc.pod",
        ]
    }

    pub fn get_common_directories() -> Vec<&'static str> {
        vec![
            // Source
            "lib/", "bin/", "script/", "scripts/",
            
            // Tests
            "t/", "test/", "tests/", "xt/",
            
            // Documentation
            "doc/", "docs/", "eg/", "examples/",
            
            // Build
            "blib/", "_build/", ".build/",
            
            // Dependencies
            "local/", "extlib/", "fatlib/",
            
            // Web
            "public/", "static/", "htdocs/", "www/",
            "templates/", "views/",
            
            // Configuration
            "config/", "conf/", "etc/",
            
            // Logs
            "logs/", "log/",
            
            // Data
            "data/", "share/",
            
            // Tools
            "tools/", "util/", "utils/",
            
            // Resources
            "res/", "resources/",
        ]
    }
} 