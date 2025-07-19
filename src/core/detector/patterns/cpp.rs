use regex::Regex;

pub struct CppPatterns {
    external_patterns: Vec<Regex>,
    cache_patterns: Vec<Regex>,
    extensions: Vec<String>,
}

impl CppPatterns {
    pub fn new() -> Self {
        let external_patterns = vec![
            Regex::new(r"\.o$").unwrap(),
            Regex::new(r"\.so$").unwrap(),
            Regex::new(r"\.a$").unwrap(),
            Regex::new(r"\.dylib$").unwrap(),
            Regex::new(r"\.dll$").unwrap(),
            Regex::new(r"\.exe$").unwrap(),
            Regex::new(r"\.obj$").unwrap(),
            Regex::new(r"\.lib$").unwrap(),
            Regex::new(r"\.pdb$").unwrap(),
        ];

        let cache_patterns = vec![
            Regex::new(r"\.ccache/").unwrap(),
            Regex::new(r"\.sccache/").unwrap(),
            Regex::new(r"build/").unwrap(),
            Regex::new(r"cmake-build-").unwrap(),
            Regex::new(r"CMakeFiles/").unwrap(),
            Regex::new(r"CMakeCache\.txt").unwrap(),
        ];

        let extensions = vec![
            "c".to_string(), "cpp".to_string(), "cc".to_string(),
            "cxx".to_string(), "h".to_string(), "hpp".to_string(),
            "hxx".to_string(), "hh".to_string(), "inl".to_string(),
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
            "Makefile", "makefile", "GNUmakefile", "CMakeLists.txt",
            "configure", "autogen.sh", "bootstrap", "build.sh",
            "meson.build", "SConstruct", "SConscript",
        ]
    }
} 