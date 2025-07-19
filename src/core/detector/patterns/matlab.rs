use regex::Regex;

pub struct MatlabPatterns {
    external_patterns: Vec<Regex>,
    cache_patterns: Vec<Regex>,
    extensions: Vec<String>,
}

impl MatlabPatterns {
    pub fn new() -> Self {
        let external_patterns = vec![
            // MATLAB compiled files
            Regex::new(r"\.p$").unwrap(),
            Regex::new(r"\.mex").unwrap(),
            Regex::new(r"\.mexw32$").unwrap(),
            Regex::new(r"\.mexw64$").unwrap(),
            Regex::new(r"\.mexa64$").unwrap(),
            Regex::new(r"\.mexmaci64$").unwrap(),
            
            // MATLAB cache
            Regex::new(r"\.asv$").unwrap(),
            Regex::new(r"\.autosave$").unwrap(),
            
            // MATLAB temporary files
            Regex::new(r"matlab_crash_dump\.").unwrap(),
            Regex::new(r"\.tmp$").unwrap(),
            
            // Simulink files
            Regex::new(r"\.slxc$").unwrap(),
            Regex::new(r"\.slx\.autosave$").unwrap(),
            Regex::new(r"\.mdl\.autosave$").unwrap(),
            
            // MATLAB projects
            Regex::new(r"\.prj$").unwrap(),
            
            // Code generation
            Regex::new(r"codegen/").unwrap(),
            Regex::new(r"slprj/").unwrap(),
            
            // MATLAB Compiler
            Regex::new(r"\.ctf$").unwrap(),
            Regex::new(r"\.exe$").unwrap(),
            Regex::new(r"\.app$").unwrap(),
            
            // MATLAB App Designer
            Regex::new(r"\.mlappinstall$").unwrap(),
            
            // MATLAB Reports
            Regex::new(r"\.rpt$").unwrap(),
            
            // MATLAB Profiler
            Regex::new(r"profile_results").unwrap(),
            
            // MATLAB Coder
            Regex::new(r"html/").unwrap(),
            
            // Temporary files
            Regex::new(r"\.tmp/").unwrap(),
            Regex::new(r"tmp/").unwrap(),
            Regex::new(r"\.swp$").unwrap(),
            Regex::new(r"\.swo$").unwrap(),
            Regex::new(r"~$").unwrap(),
            
            // MATLAB specific temp files
            Regex::new(r"\.bak$").unwrap(),
            Regex::new(r"\.orig$").unwrap(),
            
            // MATLAB logs
            Regex::new(r"\.log$").unwrap(),
            Regex::new(r"logs/").unwrap(),
        ];

        let cache_patterns = vec![
            // MATLAB cache
            Regex::new(r"\.asv$").unwrap(),
            
            // Code generation cache
            Regex::new(r"codegen/").unwrap(),
            Regex::new(r"slprj/").unwrap(),
            
            // Compiled files
            Regex::new(r"\.p$").unwrap(),
            Regex::new(r"\.mex").unwrap(),
            
            // Reports cache
            Regex::new(r"html/").unwrap(),
        ];

        let extensions = vec![
            // MATLAB source files
            "m".to_string(),
            
            // MATLAB Live Scripts
            "mlx".to_string(),
            
            // MATLAB App Designer
            "mlapp".to_string(),
            
            // Simulink models
            "slx".to_string(),
            "mdl".to_string(),
            
            // MATLAB Functions
            "p".to_string(),
            
            // MATLAB Data
            "mat".to_string(),
            
            // MATLAB Figure
            "fig".to_string(),
            
            // MATLAB Projects
            "prj".to_string(),
            
            // Configuration files
            "yml".to_string(),
            "yaml".to_string(),
            "json".to_string(),
            "xml".to_string(),
            
            // Documentation
            "md".to_string(),
            "rst".to_string(),
            "txt".to_string(),
            
            // Data files
            "csv".to_string(),
            "tsv".to_string(),
            "xlsx".to_string(),
            "xls".to_string(),
            
            // Image files
            "png".to_string(),
            "jpg".to_string(),
            "jpeg".to_string(),
            "gif".to_string(),
            "bmp".to_string(),
            "tiff".to_string(),
            "eps".to_string(),
            "pdf".to_string(),
            
            // C/C++ (for MEX files)
            "c".to_string(),
            "cpp".to_string(),
            "h".to_string(),
            "hpp".to_string(),
            
            // Fortran (for MEX files)
            "f".to_string(),
            "f90".to_string(),
            "f95".to_string(),
            
            // Shell scripts
            "sh".to_string(),
            "bash".to_string(),
            "bat".to_string(),
            "ps1".to_string(),
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
            // MATLAB project files
            "startup.m", "finish.m", "pathdef.m",
            
            // MATLAB toolbox files
            "Contents.m", "info.xml",
            
            // MATLAB build files
            "make.m", "build.m", "setup.m",
            
            // MATLAB test files
            "runtests.m", "test.m",
            
            // MATLAB App files
            "App.mlapp", "main.mlapp",
            
            // Simulink files
            "model.slx", "library.slx",
            
            // MATLAB Compiler files
            "compile.m", "package.m",
            
            // CI/CD
            ".github/workflows/matlab.yml",
            ".travis.yml", "appveyor.yml",
            "azure-pipelines.yml",
            
            // Docker
            "Dockerfile", "docker-compose.yml",
            
            // Editor
            ".editorconfig", ".gitignore",
            ".gitattributes",
            
            // Scripts
            "run.sh", "build.sh", "deploy.sh",
            "install.sh", "test.sh",
            "run.bat", "build.bat",
            
            // Documentation
            "README.md", "CHANGELOG.md",
            "LICENSE", "LICENCE",
            
            // MATLAB specific
            "license.txt", "getting_started.mlx",
            "examples.mlx", "demo.m",
        ]
    }

    pub fn get_common_directories() -> Vec<&'static str> {
        vec![
            // Source
            "src/", "lib/", "functions/",
            
            // Tests
            "tests/", "test/", "unit_tests/",
            
            // Documentation
            "doc/", "docs/", "help/",
            "examples/", "demos/",
            
            // Build
            "build/", "dist/", "codegen/",
            "slprj/", "html/",
            
            // Data
            "data/", "datasets/", "results/",
            
            // Resources
            "resources/", "assets/", "images/",
            
            // Tools
            "tools/", "utils/", "utilities/",
            
            // Configuration
            "config/", "settings/",
            
            // Apps
            "apps/", "gui/",
            
            // Models
            "models/", "simulink/",
            
            // Scripts
            "scripts/", "batch/",
            
            // Output
            "output/", "figures/", "plots/",
        ]
    }
} 