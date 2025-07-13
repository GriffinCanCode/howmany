use crate::core::stats::techstack::{TechCategory, ConfidenceLevel, DetectedTechnology};
use crate::utils::errors::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use ahash::{AHashMap, AHashSet};
use regex::Regex;
use rayon::prelude::*;

/// Language-specific technology pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguagePattern {
    pub name: String,
    pub description: String,
    pub category: String,
    pub website: String,
    pub confidence: u8,
    pub patterns: PatternMatchers,
    pub versions: Option<VersionPatterns>,
    pub implies: Option<Vec<String>>,
    pub oss: Option<bool>,
}

/// Pattern matchers for different detection methods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternMatchers {
    // Package manager patterns
    pub package_json: Option<Vec<String>>,
    pub requirements: Option<Vec<String>>,
    pub cargo_toml: Option<Vec<String>>,
    pub go_mod: Option<Vec<String>>,
    pub pom_xml: Option<Vec<String>>,
    pub gradle: Option<Vec<String>>,
    pub gemfile: Option<Vec<String>>,
    pub composer: Option<Vec<String>>,
    
    // Code patterns
    pub imports: Option<Vec<String>>,
    pub code: Option<Vec<String>>,
    pub annotations: Option<Vec<String>>,
    pub decorators: Option<Vec<String>>,
    pub jsx: Option<Vec<String>>,
    pub templates: Option<Vec<String>>,
    
    // File patterns
    pub files: Option<Vec<String>>,
    pub directories: Option<Vec<String>>,
    pub extensions: Option<Vec<String>>,
    
    // Infrastructure patterns
    pub dockerfile: Option<Vec<String>>,
    pub compose: Option<Vec<String>>,
    pub yaml: Option<Vec<String>>,
    pub xml: Option<Vec<String>>,
    pub json: Option<Vec<String>>,
    
    // Script patterns
    pub scripts: Option<Vec<String>>,
    pub config: Option<Vec<String>>,
    
    // Web patterns
    pub html: Option<Vec<String>>,
    pub css: Option<Vec<String>>,
    pub meta: Option<Vec<String>>,
    pub headers: Option<Vec<String>>,
}

/// Version detection patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionPatterns {
    pub patterns: Vec<String>,
    pub files: Vec<String>,
}

/// Compiled pattern for fast matching
#[derive(Debug, Clone)]
pub struct CompiledLanguagePattern {
    pub name: String,
    pub description: String,
    pub category: TechCategory,
    pub website: String,
    pub confidence: f64,
    pub language: String,
    pub compiled_patterns: CompiledPatternMatchers,
    pub version_patterns: Vec<(Regex, Vec<String>)>,
    pub implies: Vec<String>,
    pub oss: Option<bool>,
}

/// Compiled pattern matchers for maximum speed
#[derive(Debug, Clone)]
pub struct CompiledPatternMatchers {
    pub package_managers: AHashMap<String, Vec<Regex>>,
    pub code_patterns: Vec<Regex>,
    pub file_patterns: Vec<Regex>,
    pub directory_patterns: Vec<Regex>,
    pub extension_patterns: AHashSet<String>,
    pub infrastructure_patterns: AHashMap<String, Vec<Regex>>,
    pub web_patterns: Vec<Regex>,
}

/// Modular pattern loader
pub struct ModularPatternLoader {
    language_patterns: AHashMap<String, Vec<CompiledLanguagePattern>>,
    category_mapping: AHashMap<String, TechCategory>,
    pattern_cache: AHashMap<String, Regex>,
    supported_languages: AHashSet<String>,
}

impl ModularPatternLoader {
    pub fn new() -> Self {
        Self {
            language_patterns: AHashMap::new(),
            category_mapping: Self::init_category_mapping(),
            pattern_cache: AHashMap::new(),
            supported_languages: AHashSet::new(),
        }
    }
    
    /// Load all language patterns from the data directory
    pub fn load_all_patterns(&mut self, data_dir: &str) -> Result<()> {
        let languages_dir = Path::new(data_dir).join("languages");
        
        if !languages_dir.exists() {
            return Err(crate::utils::errors::HowManyError::FileNotFound(
                languages_dir.to_string_lossy().to_string()
            ));
        }
        
        // Dynamically discover available languages
        let discovered_languages = self.discover_available_languages(&languages_dir)?;
        self.supported_languages = discovered_languages;
        
        println!("🔍 Discovered {} languages: {:?}", 
                 self.supported_languages.len(), 
                 self.supported_languages.iter().collect::<Vec<_>>());
        
        // Load patterns for each discovered language sequentially to avoid borrow checker issues
        let mut results: Vec<Result<(String, Vec<CompiledLanguagePattern>)>> = Vec::new();
        let languages_to_process: Vec<String> = self.supported_languages.iter().cloned().collect();
        for language in languages_to_process {
            let patterns = self.load_language_patterns(&languages_dir, &language)?;
            results.push(Ok((language, patterns)));
        }
        
        // Collect results
        for result in results {
            let (language, patterns) = result?;
            if !patterns.is_empty() {
                self.language_patterns.insert(language, patterns);
            }
        }
        
        println!("✅ Loaded patterns for {} languages", self.language_patterns.len());
        self.print_statistics();
        
        Ok(())
    }
    
    /// Discover available languages by scanning the languages directory
    fn discover_available_languages(&self, languages_dir: &Path) -> Result<AHashSet<String>> {
        let mut languages = AHashSet::new();
        
        if let Ok(entries) = fs::read_dir(languages_dir) {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.is_dir() {
                        if let Some(language_name) = path.file_name().and_then(|n| n.to_str()) {
                            // Check if the directory contains any JSON files
                            if self.has_pattern_files(&path) {
                                languages.insert(language_name.to_string());
                            }
                        }
                    }
                }
            }
        }
        
        Ok(languages)
    }
    
    /// Check if a language directory contains pattern files
    fn has_pattern_files(&self, language_dir: &Path) -> bool {
        if let Ok(entries) = fs::read_dir(language_dir) {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.is_file() && path.extension().and_then(|e| e.to_str()) == Some("json") {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Load patterns for a specific language
    fn load_language_patterns(&mut self, languages_dir: &Path, language: &str) -> Result<Vec<CompiledLanguagePattern>> {
        let language_dir = languages_dir.join(language);
        let mut compiled_patterns = Vec::new();
        
        if !language_dir.exists() {
            return Ok(compiled_patterns);
        }
        
        // Dynamically discover all JSON files in the language directory
        if let Ok(entries) = fs::read_dir(&language_dir) {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.is_file() && path.extension().and_then(|e| e.to_str()) == Some("json") {
                        let patterns = self.load_pattern_file(&path, language)?;
                        compiled_patterns.extend(patterns);
                    }
                }
            }
        }
        
        Ok(compiled_patterns)
    }
    
    /// Load a specific pattern file
    fn load_pattern_file(&mut self, file_path: &Path, language: &str) -> Result<Vec<CompiledLanguagePattern>> {
        let content = fs::read_to_string(file_path)
            .map_err(|e| crate::utils::errors::HowManyError::FileNotFound(
                format!("{}: {}", file_path.display(), e)
            ))?;
        
        let patterns: AHashMap<String, LanguagePattern> = serde_json::from_str(&content)
            .map_err(|e| crate::utils::errors::HowManyError::ParseError(
                format!("Failed to parse {}: {}", file_path.display(), e)
            ))?;
        
        let mut compiled_patterns = Vec::new();
        
        for (key, pattern) in patterns {
            match self.compile_pattern(pattern, language) {
                Ok(compiled) => compiled_patterns.push(compiled),
                Err(e) => {
                    eprintln!("Warning: Failed to compile pattern '{}' in {}: {}", key, file_path.display(), e);
                }
            }
        }
        
        Ok(compiled_patterns)
    }
    
    /// Compile a language pattern for fast matching
    fn compile_pattern(&mut self, pattern: LanguagePattern, language: &str) -> Result<CompiledLanguagePattern> {
        let category = self.category_mapping.get(&pattern.category)
            .cloned()
            .unwrap_or(TechCategory::Other);
        
        let compiled_matchers = self.compile_pattern_matchers(&pattern.patterns)?;
        let version_patterns = self.compile_version_patterns(&pattern.versions)?;
        
        Ok(CompiledLanguagePattern {
            name: pattern.name,
            description: pattern.description,
            category,
            website: pattern.website,
            confidence: pattern.confidence as f64 / 100.0,
            language: language.to_string(),
            compiled_patterns: compiled_matchers,
            version_patterns,
            implies: pattern.implies.unwrap_or_default(),
            oss: pattern.oss,
        })
    }
    
    /// Compile pattern matchers
    fn compile_pattern_matchers(&mut self, patterns: &PatternMatchers) -> Result<CompiledPatternMatchers> {
        let mut package_managers = AHashMap::new();
        let mut code_patterns = Vec::new();
        let mut file_patterns = Vec::new();
        let mut directory_patterns = Vec::new();
        let mut extension_patterns = AHashSet::new();
        let mut infrastructure_patterns = AHashMap::new();
        let mut web_patterns = Vec::new();
        
        // Compile package manager patterns
        self.compile_package_manager_patterns(patterns, &mut package_managers)?;
        
        // Compile code patterns
        if let Some(imports) = &patterns.imports {
            for pattern in imports {
                if let Ok(regex) = self.get_or_compile_regex(pattern) {
                    code_patterns.push(regex);
                }
            }
        }
        
        if let Some(code) = &patterns.code {
            for pattern in code {
                if let Ok(regex) = self.get_or_compile_regex(pattern) {
                    code_patterns.push(regex);
                }
            }
        }
        
        // Compile file patterns
        if let Some(files) = &patterns.files {
            for pattern in files {
                if let Ok(regex) = self.get_or_compile_regex(pattern) {
                    file_patterns.push(regex);
                }
            }
        }
        
        // Compile directory patterns
        if let Some(directories) = &patterns.directories {
            for pattern in directories {
                if let Ok(regex) = self.get_or_compile_regex(pattern) {
                    directory_patterns.push(regex);
                }
            }
        }
        
        // Compile extension patterns
        if let Some(extensions) = &patterns.extensions {
            for ext in extensions {
                extension_patterns.insert(ext.clone());
            }
        }
        
        // Compile infrastructure patterns
        self.compile_infrastructure_patterns(patterns, &mut infrastructure_patterns)?;
        
        // Compile web patterns
        if let Some(html) = &patterns.html {
            for pattern in html {
                if let Ok(regex) = self.get_or_compile_regex(pattern) {
                    web_patterns.push(regex);
                }
            }
        }
        
        Ok(CompiledPatternMatchers {
            package_managers,
            code_patterns,
            file_patterns,
            directory_patterns,
            extension_patterns,
            infrastructure_patterns,
            web_patterns,
        })
    }
    
    /// Compile package manager patterns
    fn compile_package_manager_patterns(
        &mut self,
        patterns: &PatternMatchers,
        package_managers: &mut AHashMap<String, Vec<Regex>>,
    ) -> Result<()> {
        let pm_patterns = [
            ("package_json", &patterns.package_json),
            ("requirements", &patterns.requirements),
            ("cargo_toml", &patterns.cargo_toml),
            ("go_mod", &patterns.go_mod),
            ("pom_xml", &patterns.pom_xml),
            ("gradle", &patterns.gradle),
            ("gemfile", &patterns.gemfile),
            ("composer", &patterns.composer),
        ];
        
        for (pm_type, patterns_opt) in pm_patterns {
            if let Some(patterns_list) = patterns_opt {
                let mut compiled_patterns = Vec::new();
                for pattern in patterns_list {
                    if let Ok(regex) = self.get_or_compile_regex(pattern) {
                        compiled_patterns.push(regex);
                    }
                }
                if !compiled_patterns.is_empty() {
                    package_managers.insert(pm_type.to_string(), compiled_patterns);
                }
            }
        }
        
        Ok(())
    }
    
    /// Compile infrastructure patterns
    fn compile_infrastructure_patterns(
        &mut self,
        patterns: &PatternMatchers,
        infrastructure_patterns: &mut AHashMap<String, Vec<Regex>>,
    ) -> Result<()> {
        let infra_patterns = [
            ("dockerfile", &patterns.dockerfile),
            ("compose", &patterns.compose),
            ("yaml", &patterns.yaml),
            ("xml", &patterns.xml),
            ("json", &patterns.json),
        ];
        
        for (infra_type, patterns_opt) in infra_patterns {
            if let Some(patterns_list) = patterns_opt {
                let mut compiled_patterns = Vec::new();
                for pattern in patterns_list {
                    if let Ok(regex) = self.get_or_compile_regex(pattern) {
                        compiled_patterns.push(regex);
                    }
                }
                if !compiled_patterns.is_empty() {
                    infrastructure_patterns.insert(infra_type.to_string(), compiled_patterns);
                }
            }
        }
        
        Ok(())
    }
    
    /// Compile version patterns
    fn compile_version_patterns(&mut self, version_patterns: &Option<VersionPatterns>) -> Result<Vec<(Regex, Vec<String>)>> {
        let mut compiled = Vec::new();
        
        if let Some(versions) = version_patterns {
            for pattern in &versions.patterns {
                if let Ok(regex) = self.get_or_compile_regex(pattern) {
                    compiled.push((regex, versions.files.clone()));
                }
            }
        }
        
        Ok(compiled)
    }
    
    /// Get or compile regex with caching
    fn get_or_compile_regex(&mut self, pattern: &str) -> Result<Regex> {
        if let Some(cached) = self.pattern_cache.get(pattern) {
            return Ok(cached.clone());
        }
        
        let regex = Regex::new(pattern)
            .map_err(|e| crate::utils::errors::HowManyError::ParseError(
                format!("Invalid regex pattern '{}': {}", pattern, e)
            ))?;
        
        self.pattern_cache.insert(pattern.to_string(), regex.clone());
        Ok(regex)
    }
    
    /// Initialize category mapping
    fn init_category_mapping() -> AHashMap<String, TechCategory> {
        let mut mapping = AHashMap::new();
        
        // Web & UI
        mapping.insert("web_framework".to_string(), TechCategory::WebFramework);
        mapping.insert("ui_framework".to_string(), TechCategory::UIFramework);
        mapping.insert("framework".to_string(), TechCategory::WebFramework);
        mapping.insert("frontend".to_string(), TechCategory::Frontend);
        mapping.insert("backend".to_string(), TechCategory::Backend);
        
        // Data & Storage
        mapping.insert("orm".to_string(), TechCategory::ORM);
        mapping.insert("database".to_string(), TechCategory::Database);
        mapping.insert("cache".to_string(), TechCategory::Cache);
        mapping.insert("data_framework".to_string(), TechCategory::DataFramework);
        mapping.insert("data_library".to_string(), TechCategory::DataLibrary);
        
        // Development Tools
        mapping.insert("testing".to_string(), TechCategory::Testing);
        mapping.insert("build_tool".to_string(), TechCategory::BuildTool);
        mapping.insert("package_manager".to_string(), TechCategory::PackageManager);
        mapping.insert("cli".to_string(), TechCategory::CLI);
        mapping.insert("linting".to_string(), TechCategory::Linting);
        mapping.insert("documentation".to_string(), TechCategory::Documentation);
        
        // Languages & Runtimes
        mapping.insert("programming_language".to_string(), TechCategory::ProgrammingLanguage);
        mapping.insert("runtime".to_string(), TechCategory::Runtime);
        mapping.insert("async_runtime".to_string(), TechCategory::AsyncRuntime);
        
        // Infrastructure
        mapping.insert("containerization".to_string(), TechCategory::Containerization);
        mapping.insert("orchestration".to_string(), TechCategory::Orchestration);
        mapping.insert("cloud_provider".to_string(), TechCategory::CloudProvider);
        mapping.insert("infrastructure".to_string(), TechCategory::Infrastructure);
        
        // Monitoring & Logging
        mapping.insert("logging".to_string(), TechCategory::Logging);
        mapping.insert("monitoring".to_string(), TechCategory::Monitoring);
        mapping.insert("analytics".to_string(), TechCategory::Analytics);
        
        // Specialized
        mapping.insert("ml_framework".to_string(), TechCategory::MLFramework);
        mapping.insert("game_engine".to_string(), TechCategory::GameEngine);
        mapping.insert("desktop_framework".to_string(), TechCategory::DesktopFramework);
        mapping.insert("messaging".to_string(), TechCategory::Messaging);
        mapping.insert("serialization".to_string(), TechCategory::Serialization);
        mapping.insert("configuration".to_string(), TechCategory::Configuration);
        mapping.insert("config".to_string(), TechCategory::Configuration);
        mapping.insert("utility".to_string(), TechCategory::Other);
        mapping.insert("graphics".to_string(), TechCategory::Other);
        mapping.insert("networking".to_string(), TechCategory::Other);
        mapping.insert("security".to_string(), TechCategory::Security);
        
        // Web Infrastructure
        mapping.insert("web_server".to_string(), TechCategory::WebServer);
        mapping.insert("cdn".to_string(), TechCategory::CDN);
        mapping.insert("cms".to_string(), TechCategory::CMS);
        
        mapping
    }
    
    /// Get all compiled patterns
    pub fn get_all_patterns(&self) -> Vec<&CompiledLanguagePattern> {
        self.language_patterns
            .values()
            .flat_map(|patterns| patterns.iter())
            .collect()
    }
    
    /// Get patterns for a specific language
    pub fn get_language_patterns(&self, language: &str) -> Option<&Vec<CompiledLanguagePattern>> {
        self.language_patterns.get(language)
    }
    
    /// Get patterns by category
    pub fn get_patterns_by_category(&self, category: TechCategory) -> Vec<&CompiledLanguagePattern> {
        self.language_patterns
            .values()
            .flat_map(|patterns| patterns.iter())
            .filter(|pattern| pattern.category == category)
            .collect()
    }
    
    /// Print loading statistics
    fn print_statistics(&self) {
        use owo_colors::OwoColorize;
        
        let total_patterns: usize = self.language_patterns.values().map(|v| v.len()).sum();
        let total_regexes = self.pattern_cache.len();
        
        println!();
        println!("{}", "📊 TECHSTACK PATTERN ANALYSIS".bright_green().bold());
        println!("{}", "─".repeat(80).bright_blue());
        println!();
        
        // Basic statistics
        println!("{}", "📋 Overview:".bright_cyan().bold());
        println!("   📂 Languages discovered: {}", self.supported_languages.len().to_string().bright_yellow());
        println!("   🔍 Languages with patterns: {}", self.language_patterns.len().to_string().bright_yellow());
        println!("   📊 Total patterns: {}", total_patterns.to_string().bright_yellow());
        println!("   ⚡ Compiled regexes: {}", total_regexes.to_string().bright_yellow());
        println!();
        
        // Language breakdown with proper formatting
        println!("{}", "🌐 Language Breakdown:".bright_cyan().bold());
        let mut language_stats: Vec<_> = self.language_patterns.iter().collect();
        language_stats.sort_by(|a, b| b.1.len().cmp(&a.1.len()));
        
        for (language, patterns) in language_stats {
            if !patterns.is_empty() {
                let categories: std::collections::HashSet<_> = patterns.iter()
                    .map(|p| format!("{:?}", p.category))
                    .collect();
                println!("   • {}: {} patterns ({} categories)", 
                         language.bright_white().bold(), 
                         patterns.len().to_string().bright_green(), 
                         categories.len().to_string().bright_blue());
            }
        }
        println!();
        
        // Category distribution with proper formatting
        let mut category_counts = std::collections::HashMap::new();
        for patterns in self.language_patterns.values() {
            for pattern in patterns {
                *category_counts.entry(pattern.category.clone()).or_insert(0) += 1;
            }
        }
        
        println!("{}", "📈 Category Distribution:".bright_cyan().bold());
        let mut sorted_categories: Vec<_> = category_counts.iter().collect();
        sorted_categories.sort_by(|a, b| b.1.cmp(a.1));
        
        for (category, count) in sorted_categories.iter().take(10) {
            println!("   • {}: {} patterns", 
                     format!("{:?}", category).bright_white().bold(), 
                     count.to_string().bright_green());
        }
        
        println!();
        println!("{}", format!("✅ Built indices for {} patterns", total_patterns.to_string().bright_yellow()).bright_green().bold());
        println!();
    }
}

/// Initialize the modular pattern loader
pub fn initialize_modular_loader(data_dir: &str) -> Result<ModularPatternLoader> {
    let mut loader = ModularPatternLoader::new();
    loader.load_all_patterns(data_dir)?;
    Ok(loader)
} 