use regex::Regex;

pub struct RPatterns {
    external_patterns: Vec<Regex>,
    cache_patterns: Vec<Regex>,
    extensions: Vec<String>,
}

impl RPatterns {
    pub fn new() -> Self {
        let external_patterns = vec![
            // R package build artifacts
            Regex::new(r"\.Rcheck/").unwrap(),
            Regex::new(r"\.Rhistory").unwrap(),
            Regex::new(r"\.RData").unwrap(),
            Regex::new(r"\.Ruserdata").unwrap(),
            
            // R session files
            Regex::new(r"\.Rproj\.user/").unwrap(),
            Regex::new(r"\.Rapp\.history").unwrap(),
            
            // R package files
            Regex::new(r"\.tar\.gz$").unwrap(),
            Regex::new(r"\.tgz$").unwrap(),
            
            // renv (R environment management)
            Regex::new(r"renv/").unwrap(),
            Regex::new(r"renv\.lock").unwrap(),
            
            // packrat (legacy R package management)
            Regex::new(r"packrat/").unwrap(),
            
            // R documentation
            Regex::new(r"\.Rd2pdf").unwrap(),
            
            // R cache
            Regex::new(r"\.cache/").unwrap(),
            Regex::new(r"cache/").unwrap(),
            
            // R Markdown cache
            Regex::new(r"_cache/").unwrap(),
            Regex::new(r"[^/]*_cache/").unwrap(),
            
            // R output files
            Regex::new(r"\.out$").unwrap(),
            Regex::new(r"\.log$").unwrap(),
            
            // Shiny app files
            Regex::new(r"rsconnect/").unwrap(),
            
            // R plots
            Regex::new(r"Rplots\.pdf").unwrap(),
            
            // Temporary files
            Regex::new(r"\.tmp/").unwrap(),
            Regex::new(r"tmp/").unwrap(),
            Regex::new(r"\.swp$").unwrap(),
            Regex::new(r"\.swo$").unwrap(),
            Regex::new(r"~$").unwrap(),
            
            // R specific temp files
            Regex::new(r"\.bak$").unwrap(),
            Regex::new(r"\.orig$").unwrap(),
            
            // R Studio files
            Regex::new(r"\.Rproj$").unwrap(),
            
            // R build files
            Regex::new(r"src/\*\.o$").unwrap(),
            Regex::new(r"src/\*\.so$").unwrap(),
            Regex::new(r"src/\*\.dll$").unwrap(),
        ];

        let cache_patterns = vec![
            // R package cache
            Regex::new(r"\.Rcheck/").unwrap(),
            
            // R session cache
            Regex::new(r"\.Rproj\.user/").unwrap(),
            
            // renv cache
            Regex::new(r"renv/").unwrap(),
            
            // R Markdown cache
            Regex::new(r"_cache/").unwrap(),
            
            // General cache
            Regex::new(r"\.cache/").unwrap(),
        ];

        let extensions = vec![
            // R source files
            "r".to_string(),
            "R".to_string(),
            
            // R Markdown
            "rmd".to_string(),
            "Rmd".to_string(),
            
            // R Notebook
            "rnw".to_string(),
            "Rnw".to_string(),
            
            // R Sweave
            "rtex".to_string(),
            "Rtex".to_string(),
            
            // R Documentation
            "rd".to_string(),
            "Rd".to_string(),
            
            // R Data files
            "rda".to_string(),
            "RData".to_string(),
            "rds".to_string(),
            
            // Configuration files
            "yml".to_string(),
            "yaml".to_string(),
            "json".to_string(),
            "toml".to_string(),
            
            // Documentation
            "md".to_string(),
            "rst".to_string(),
            "txt".to_string(),
            
            // Web files (for Shiny apps)
            "html".to_string(),
            "css".to_string(),
            "js".to_string(),
            
            // Data files
            "csv".to_string(),
            "tsv".to_string(),
            "xlsx".to_string(),
            "json".to_string(),
            "xml".to_string(),
            
            // SQL
            "sql".to_string(),
            
            // Shell scripts
            "sh".to_string(),
            "bash".to_string(),
            
            // C/C++ (for R packages)
            "c".to_string(),
            "cpp".to_string(),
            "h".to_string(),
            "hpp".to_string(),
            
            // Fortran (for R packages)
            "f".to_string(),
            "f90".to_string(),
            "f95".to_string(),
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
            // R package files
            "DESCRIPTION", "NAMESPACE", "NEWS", "NEWS.md",
            "README", "README.md", "README.Rmd",
            "LICENSE", "LICENCE", "COPYING",
            
            // R project files
            ".Rprofile", ".Renviron", ".Rproj",
            
            // renv files
            "renv.lock", ".Rprofile",
            
            // packrat files
            "packrat.lock", ".Rprofile",
            
            // R build files
            "configure", "configure.ac", "configure.in",
            "Makefile", "Makefile.in", "Makefile.win",
            "cleanup", "cleanup.win",
            
            // CI/CD
            ".github/workflows/r.yml",
            ".github/workflows/R-CMD-check.yaml",
            ".travis.yml", "appveyor.yml",
            
            // Docker
            "Dockerfile", "docker-compose.yml",
            
            // Editor
            ".editorconfig", ".gitignore",
            ".Rbuildignore", ".Rinstignore",
            
            // Scripts
            "run.sh", "build.sh", "deploy.sh",
            "install.sh", "test.sh",
            
            // Shiny
            "app.R", "ui.R", "server.R",
            "global.R", "www/",
            
            // R Markdown
            "index.Rmd", "_site.yml", "_bookdown.yml",
            
            // pkgdown
            "_pkgdown.yml", "pkgdown.yml",
        ]
    }

    pub fn get_common_directories() -> Vec<&'static str> {
        vec![
            // R package structure
            "R/", "man/", "vignettes/", "inst/",
            "data/", "data-raw/", "exec/",
            
            // Tests
            "tests/", "testthat/",
            
            // Documentation
            "doc/", "docs/", "vignettes/",
            
            // Build
            ".Rcheck/", "src/",
            
            // Dependencies
            "renv/", "packrat/",
            
            // Shiny
            "www/", "R/", "data/",
            
            // R Markdown
            "_site/", "_book/", "_bookdown_files/",
            
            // Configuration
            "config/", "conf/",
            
            // Logs
            "logs/", "log/",
            
            // Output
            "output/", "results/", "figures/",
            
            // Tools
            "tools/", "scripts/",
            
            // Resources
            "resources/", "assets/",
        ]
    }
} 