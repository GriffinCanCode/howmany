use crate::core::stats::techstack::{TechCategory, ConfidenceLevel};
use crate::core::stats::techstack::loader::{ModularPatternLoader, initialize_modular_loader};
use crate::utils::errors::Result;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, BTreeMap};
use std::sync::OnceLock;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;
use regex::Regex;
use ahash::{AHashMap, AHashSet};

/// Technology detection pattern from JSON database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechPattern {
    pub name: String,
    pub description: Option<String>,
    pub website: Option<String>,
    pub cats: Vec<u32>,
    pub icon: Option<String>,
    pub cpe: Option<String>,
    pub headers: Option<AHashMap<String, String>>,
    pub html: Option<Vec<String>>,
    pub scripts: Option<Vec<String>>,
    pub js: Option<AHashMap<String, String>>,
    pub cookies: Option<AHashMap<String, String>>,
    pub meta: Option<AHashMap<String, String>>,
    pub implies: Option<Vec<String>>,
    pub requires: Option<Vec<String>>,
    pub excludes: Option<Vec<String>>,
    pub dom: Option<serde_json::Value>,
    pub url: Option<Vec<String>>,
    pub xhr: Option<Vec<String>>,
    pub oss: Option<bool>,
    pub saas: Option<bool>,
    pub pricing: Option<Vec<String>>,
    pub confidence: Option<f64>,
}

/// Categories mapping from Wappalyzer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechCategories {
    #[serde(flatten)]
    pub categories: AHashMap<String, Category>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    pub name: String,
    pub priority: Option<u32>,
}

/// Compiled detection patterns for maximum speed
#[derive(Debug, Clone)]
pub struct CompiledPattern {
    pub name: String,
    pub category: TechCategory,
    pub confidence: f64,
    pub headers: Vec<(String, Regex)>,
    pub html_patterns: Vec<Regex>,
    pub script_patterns: Vec<Regex>,
    pub js_patterns: Vec<(String, Regex)>,
    pub cookie_patterns: Vec<(String, Regex)>,
    pub meta_patterns: Vec<(String, Regex)>,
    pub url_patterns: Vec<Regex>,
    pub implies: Vec<String>,
    pub requires: Vec<String>,
    pub excludes: Vec<String>,
    pub description: String,
    pub website: Option<String>,
    pub oss: Option<bool>,
    pub saas: Option<bool>,
}

/// Detection evidence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionEvidence {
    pub source: EvidenceSource,
    pub file_path: String,
    pub line_number: Option<usize>,
    pub content: String,
    pub weight: f64,
    pub pattern_name: String,
}

/// Source of detection evidence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EvidenceSource {
    PackageJson,
    CargoToml,
    RequirementsTxt,
    Gemfile,
    GoMod,
    ImportStatement,
    FileExtension,
    ConfigFile,
    DockerFile,
    BuildScript,
    SourceCode,
    Documentation,
    HttpHeader,
    HtmlContent,
    JavascriptCode,
    CookieValue,
    MetaTag,
    UrlPattern,
}

/// Detected technology with comprehensive information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedTechnology {
    pub name: String,
    pub version: Option<String>,
    pub category: TechCategory,
    pub confidence: f64,
    pub evidence: Vec<DetectionEvidence>,
    pub description: String,
    pub website: Option<String>,
    pub documentation: Option<String>,
    pub license: Option<String>,
    pub oss: Option<bool>,
    pub saas: Option<bool>,
    pub pricing: Option<Vec<String>>,
    pub implies: Vec<String>,
    pub requires: Vec<String>,
    pub excludes: Vec<String>,
}

/// Complete techstack inventory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechStackInventory {
    pub technologies: Vec<DetectedTechnology>,
    pub total_files_analyzed: usize,
    pub overall_confidence: f64,
    pub analysis_summary: AnalysisSummary,
}

/// Analysis summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisSummary {
    pub total_technologies: usize,
    pub primary_language: Option<String>,
    pub architecture_type: ArchitectureType,
    pub deployment_type: DeploymentType,
    pub security_posture: SecurityPosture,
    pub modernization_score: f64,
}

/// Architecture type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ArchitectureType {
    Monolithic,
    Microservices,
    Serverless,
    Hybrid,
    Unknown,
}

/// Deployment type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeploymentType {
    OnPremise,
    Cloud,
    Hybrid,
    ContainerBased,
    Unknown,
}

/// Security posture assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityPosture {
    Excellent,
    Good,
    Fair,
    Poor,
    Unknown,
}

/// Global technology database singleton
static TECH_DATABASE: OnceLock<TechStackDetector> = OnceLock::new();

/// Ultra-fast technology detection engine
pub struct TechStackDetector {
    // Modular pattern loader for comprehensive coverage
    pattern_loader: ModularPatternLoader,
    
    // Fast lookups using AHashMap (faster than std::HashMap)
    name_to_pattern: AHashMap<String, usize>,
    category_patterns: AHashMap<TechCategory, Vec<usize>>,
    
    // File extension mappings for quick filtering
    extension_patterns: AHashMap<String, Vec<usize>>,
    
    // Filename patterns for quick matching
    filename_patterns: AHashMap<String, Vec<usize>>,
    
    // Performance metrics
    cache_hits: std::sync::atomic::AtomicU64,
    cache_misses: std::sync::atomic::AtomicU64,
}

impl TechStackDetector {
    /// Create a new detector instance
    pub fn new() -> Result<Self> {
        let pattern_loader = initialize_modular_loader("data")?;
        let mut detector = Self::new_with_loader(pattern_loader);
        detector.build_indices()?;
        Ok(detector)
    }

    /// Initialize the global detector instance
    pub fn initialize() -> Result<&'static Self> {
        TECH_DATABASE.get_or_init(|| {
            let pattern_loader = initialize_modular_loader("data").unwrap();
            let mut detector = Self::new_with_loader(pattern_loader);
            detector.build_indices().unwrap();
            detector
        });
        Ok(TECH_DATABASE.get().unwrap())
    }
    
    /// Get the global detector instance
    pub fn instance() -> &'static Self {
        TECH_DATABASE.get().expect("TechStackDetector not initialized")
    }
    
    fn new_with_loader(pattern_loader: ModularPatternLoader) -> Self {
        Self {
            pattern_loader,
            name_to_pattern: AHashMap::with_capacity(10000),
            category_patterns: AHashMap::with_capacity(100),
            extension_patterns: AHashMap::with_capacity(200),
            filename_patterns: AHashMap::with_capacity(500),
            cache_hits: std::sync::atomic::AtomicU64::new(0),
            cache_misses: std::sync::atomic::AtomicU64::new(0),
        }
    }
    
    /// Build indices for fast pattern lookup
    fn build_indices(&mut self) -> Result<()> {
        let patterns = self.pattern_loader.get_all_patterns();
        
        for (index, pattern) in patterns.iter().enumerate() {
            // Build name to pattern index
            self.name_to_pattern.insert(pattern.name.clone(), index);
            
            // Build category to patterns index
            self.category_patterns.entry(pattern.category.clone()).or_insert_with(Vec::new).push(index);
            
            // Build extension patterns index
            for ext in &pattern.compiled_patterns.extension_patterns {
                self.extension_patterns.entry(ext.clone()).or_insert_with(Vec::new).push(index);
            }
        }
        
        println!("✅ Built indices for {} patterns", patterns.len());
        Ok(())
    }
    
    /// Get patterns for comprehensive detection
    pub fn get_all_patterns(&self) -> Vec<&crate::core::stats::techstack::loader::CompiledLanguagePattern> {
        self.pattern_loader.get_all_patterns()
    }
    
    /// Get patterns by language
    pub fn get_language_patterns(&self, language: &str) -> Option<&Vec<crate::core::stats::techstack::loader::CompiledLanguagePattern>> {
        self.pattern_loader.get_language_patterns(language)
    }
    
    /// Get patterns by category
    pub fn get_patterns_by_category(&self, category: TechCategory) -> Vec<&crate::core::stats::techstack::loader::CompiledLanguagePattern> {
        self.pattern_loader.get_patterns_by_category(category)
    }
    
        /// Ultra-fast technology detection using modular patterns
    pub fn detect_techstack(&self, project_path: &str) -> Result<TechStackInventory> {
        let mut detected_technologies = Vec::new();
        let mut files_analyzed = 0;

        // Multi-threaded file analysis for speed
        self.analyze_project_comprehensive(project_path, &mut detected_technologies, &mut files_analyzed)?;
        
        // Post-process detections
        self.resolve_implications(&mut detected_technologies);
        self.remove_conflicts(&mut detected_technologies);
        self.calculate_confidence(&mut detected_technologies);
        self.deduplicate_technologies(&mut detected_technologies);
        
        let analysis_summary = self.generate_analysis_summary(&detected_technologies);
        let overall_confidence = self.calculate_overall_confidence(&detected_technologies);

        Ok(TechStackInventory {
            technologies: detected_technologies,
            total_files_analyzed: files_analyzed,
            overall_confidence,
            analysis_summary,
        })
    }

    /// Detect technologies for a single file using modular patterns
    pub fn detect_file_techstack(&self, file_path: &str) -> Result<Vec<DetectedTechnology>> {
        let mut detected = Vec::new();
        
        // Read file content once for all pattern matching
        let content = match fs::read_to_string(file_path) {
            Ok(content) => content,
            Err(_) => return Ok(detected), // Skip files that can't be read
        };
        
        let extension = Path::new(file_path)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        
        let filename = Path::new(file_path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");
        
        // Get all patterns for comprehensive detection
        let all_patterns = self.get_all_patterns();
        
        // Analyze file against all patterns
        for pattern in all_patterns {
            if let Some(tech) = self.match_pattern_comprehensive(pattern, file_path, &content, extension, filename)? {
                detected.push(tech);
            }
        }
        
        Ok(detected)
    }

        /// Comprehensive project analysis using modular patterns
    fn analyze_project_comprehensive(
        &self,
        project_path: &str,
        detected_technologies: &mut Vec<DetectedTechnology>,
        files_analyzed: &mut usize,
    ) -> Result<()> {
        use std::sync::{Arc, Mutex};
        use rayon::prelude::*;
        
        let path = Path::new(project_path);
        if !path.exists() {
            return Err(crate::utils::errors::HowManyError::FileNotFound(project_path.to_string()));
        }

        let technologies = Arc::new(Mutex::new(Vec::new()));
        let file_count = Arc::new(Mutex::new(0));
        
        // Collect all files first
        let mut files_to_analyze = Vec::new();
        self.collect_files_comprehensive(path, &mut files_to_analyze)?;
        
        // Get all patterns for comprehensive detection
        let all_patterns = self.get_all_patterns();
        
        // Analyze files in parallel using all patterns
        files_to_analyze.par_iter().for_each(|file_path| {
            if let Ok(mut file_technologies) = self.analyze_file_comprehensive(file_path, &all_patterns) {
                let mut tech_lock = technologies.lock().unwrap();
                tech_lock.append(&mut file_technologies);
                
                let mut count_lock = file_count.lock().unwrap();
                *count_lock += 1;
            }
        });
        
        let final_technologies = technologies.lock().unwrap();
        detected_technologies.extend(final_technologies.clone());
        *files_analyzed = *file_count.lock().unwrap();

        Ok(())
    }

    /// Comprehensive file analysis using all modular patterns
    fn analyze_file_comprehensive(
        &self,
        file_path: &str,
        patterns: &[&crate::core::stats::techstack::loader::CompiledLanguagePattern]
    ) -> Result<Vec<DetectedTechnology>> {
        let mut detected = Vec::new();
        
        // Read file content once for all pattern matching
        let content = match fs::read_to_string(file_path) {
            Ok(content) => content,
            Err(_) => return Ok(detected), // Skip files that can't be read
        };
        
        let extension = Path::new(file_path)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        
        let filename = Path::new(file_path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");
        
        // Match against all patterns comprehensively
        for pattern in patterns {
            if let Some(tech) = self.match_pattern_comprehensive(pattern, file_path, &content, extension, filename)? {
                detected.push(tech);
            }
        }
        
        Ok(detected)
    }
    
    /// Comprehensive pattern matching using modular patterns
    fn match_pattern_comprehensive(
        &self,
        pattern: &crate::core::stats::techstack::loader::CompiledLanguagePattern,
        file_path: &str,
        content: &str,
        extension: &str,
        filename: &str
    ) -> Result<Option<DetectedTechnology>> {
        let mut evidence = Vec::new();
        let mut confidence = 0.0;
        
        // Check extension patterns
        if pattern.compiled_patterns.extension_patterns.contains(extension) {
            evidence.push(DetectionEvidence {
                source: EvidenceSource::FileExtension,
                        file_path: file_path.to_string(),
                        line_number: None,
                content: format!("File extension: {}", extension),
                weight: 0.6,
                pattern_name: pattern.name.clone(),
            });
            confidence += 0.6;
        }
        
        // Check file patterns
        for file_regex in &pattern.compiled_patterns.file_patterns {
            if file_regex.is_match(filename) {
                evidence.push(DetectionEvidence {
                    source: EvidenceSource::ConfigFile,
                        file_path: file_path.to_string(),
                        line_number: None,
                    content: format!("Filename match: {}", filename),
                        weight: 0.8,
                    pattern_name: pattern.name.clone(),
                });
                confidence += 0.8;
                break;
            }
        }
        
        // Check package manager patterns
        for (pm_type, regexes) in &pattern.compiled_patterns.package_managers {
            for regex in regexes {
                if regex.is_match(content) {
                    evidence.push(DetectionEvidence {
                        source: match pm_type.as_str() {
                            "package_json" => EvidenceSource::PackageJson,
                            "cargo_toml" => EvidenceSource::CargoToml,
                            "requirements" => EvidenceSource::RequirementsTxt,
                            "gemfile" => EvidenceSource::Gemfile,
                            "go_mod" => EvidenceSource::GoMod,
                            _ => EvidenceSource::ConfigFile,
                        },
                        file_path: file_path.to_string(),
                        line_number: None,
                        content: format!("Package manager pattern match in {}", pm_type),
                        weight: 0.9,
                        pattern_name: pattern.name.clone(),
                    });
                    confidence += 0.9;
                    break;
                }
            }
        }
        
        // Check code patterns
        for code_regex in &pattern.compiled_patterns.code_patterns {
            if code_regex.is_match(content) {
                evidence.push(DetectionEvidence {
                    source: EvidenceSource::SourceCode,
                    file_path: file_path.to_string(),
                    line_number: None,
                    content: "Code pattern match".to_string(),
                    weight: 0.7,
                    pattern_name: pattern.name.clone(),
                });
                confidence += 0.7;
                break;
            }
        }
        
        // Check infrastructure patterns
        for (infra_type, regexes) in &pattern.compiled_patterns.infrastructure_patterns {
            for regex in regexes {
                if regex.is_match(content) {
                    evidence.push(DetectionEvidence {
                        source: match infra_type.as_str() {
                            "dockerfile" => EvidenceSource::DockerFile,
                            "compose" => EvidenceSource::DockerFile,
                            "yaml" => EvidenceSource::ConfigFile,
                            "xml" => EvidenceSource::ConfigFile,
                            "json" => EvidenceSource::ConfigFile,
                            _ => EvidenceSource::ConfigFile,
                        },
                        file_path: file_path.to_string(),
                        line_number: None,
                        content: format!("Infrastructure pattern match in {}", infra_type),
                        weight: 0.8,
                        pattern_name: pattern.name.clone(),
                    });
                    confidence += 0.8;
                    break;
                }
            }
        }
        
        // Check web patterns
        for web_regex in &pattern.compiled_patterns.web_patterns {
            if web_regex.is_match(content) {
                evidence.push(DetectionEvidence {
                    source: EvidenceSource::HtmlContent,
                    file_path: file_path.to_string(),
                    line_number: None,
                    content: "Web pattern match".to_string(),
                    weight: 0.7,
                    pattern_name: pattern.name.clone(),
                });
                confidence += 0.7;
                break;
            }
        }
        
        // Extract version if patterns exist
        let version = self.extract_version(pattern, content);
        
        // Apply base confidence from pattern
        confidence = (confidence * pattern.confidence).min(1.0);
        
        if confidence > 0.3 {
            Ok(Some(DetectedTechnology {
                name: pattern.name.clone(),
                version,
                category: pattern.category.clone(),
                confidence,
                evidence,
                description: pattern.description.clone(),
                website: Some(pattern.website.clone()),
                documentation: None,
                license: None,
                oss: pattern.oss,
                saas: None,
                pricing: None,
                implies: pattern.implies.clone(),
                requires: Vec::new(),
                excludes: Vec::new(),
            }))
        } else {
            Ok(None)
        }
    }
    
        /// Extract version from content using version patterns
    fn extract_version(&self, pattern: &crate::core::stats::techstack::loader::CompiledLanguagePattern, content: &str) -> Option<String> {
        for (version_regex, _files) in &pattern.version_patterns {
            if let Some(captures) = version_regex.captures(content) {
                if let Some(version_match) = captures.get(1) {
                    return Some(version_match.as_str().to_string());
                }
            }
        }
        None
    }

    /// Comprehensive file collection for all supported file types
    fn collect_files_comprehensive(&self, dir: &Path, files: &mut Vec<String>) -> Result<()> {
        use walkdir::WalkDir;
        
        for entry in WalkDir::new(dir)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            
            // Skip common ignored directories
            if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                if file_name.starts_with('.') || 
                   matches!(file_name, "node_modules" | "target" | "build" | "dist" | "__pycache__" | ".git") {
                    continue;
                }
            }
            
            if path.is_file() {
                if let Some(path_str) = path.to_str() {
                    files.push(path_str.to_string());
                }
            }
        }

        Ok(())
    }

    /// Deduplicate detected technologies
    fn deduplicate_technologies(&self, technologies: &mut Vec<DetectedTechnology>) {
        use std::collections::HashMap;
        
        let mut seen = HashMap::new();
        let mut deduped = Vec::new();
        
        for tech in technologies.drain(..) {
            if let Some(existing) = seen.get_mut(&tech.name) {
                // Merge evidence and take higher confidence
                let existing_tech: &mut DetectedTechnology = existing;
                existing_tech.evidence.extend(tech.evidence);
                if tech.confidence > existing_tech.confidence {
                    existing_tech.confidence = tech.confidence;
                    existing_tech.version = tech.version;
                }
        } else {
                seen.insert(tech.name.clone(), tech.clone());
                deduped.push(tech);
            }
        }
        
        *technologies = deduped;
    }
    
    fn resolve_implications(&self, technologies: &mut Vec<DetectedTechnology>) {
        let mut implied_technologies = Vec::new();
        
        for tech in technologies.iter() {
            for implied_name in &tech.implies {
                // Check if implied technology is already detected
                if !technologies.iter().any(|t| t.name == *implied_name) {
                    // Create implied technology with lower confidence
                    implied_technologies.push(DetectedTechnology {
                        name: implied_name.clone(),
                    version: None,
                        category: self.infer_category_from_name(implied_name),
                        confidence: tech.confidence * 0.6, // Reduce confidence for implied
                    evidence: vec![DetectionEvidence {
                            source: EvidenceSource::SourceCode,
                            file_path: "implied".to_string(),
                        line_number: None,
                            content: format!("Implied by {}", tech.name),
                            weight: 0.5,
                            pattern_name: tech.name.clone(),
                        }],
                        description: format!("Implied by {}", tech.name),
                        website: None,
                        documentation: None,
                        license: None,
                        oss: None,
                        saas: None,
                        pricing: None,
                        implies: Vec::new(),
                        requires: Vec::new(),
                        excludes: Vec::new(),
                });
            }
        }
        }
        
        technologies.extend(implied_technologies);
    }
    
    fn remove_conflicts(&self, technologies: &mut Vec<DetectedTechnology>) {
        let mut to_remove = Vec::new();
        
        for (i, tech) in technologies.iter().enumerate() {
            for excluded_name in &tech.excludes {
                // Find and mark conflicting technologies for removal
                for (j, other_tech) in technologies.iter().enumerate() {
                    if i != j && other_tech.name == *excluded_name {
                        // Keep the one with higher confidence
                        if tech.confidence > other_tech.confidence {
                            to_remove.push(j);
                        } else {
                            to_remove.push(i);
                        }
                    }
                }
            }
        }
        
        // Remove conflicts (in reverse order to maintain indices)
        to_remove.sort_unstable();
        to_remove.dedup();
        for &index in to_remove.iter().rev() {
            if index < technologies.len() {
                technologies.remove(index);
            }
        }
    }
    
    fn calculate_confidence(&self, technologies: &mut Vec<DetectedTechnology>) {
        for tech in technologies.iter_mut() {
            // Normalize confidence based on evidence quality
            let evidence_weight: f64 = tech.evidence.iter().map(|e| e.weight).sum();
            let evidence_count = tech.evidence.len() as f64;
            
            if evidence_count > 0.0 {
                // Boost confidence for multiple evidence sources
                let evidence_boost = (evidence_count.sqrt() - 1.0) * 0.1;
                tech.confidence = ((tech.confidence + evidence_boost) * (evidence_weight / evidence_count)).min(1.0);
            }
            
            // Apply minimum confidence threshold
            if tech.confidence < 0.1 {
                tech.confidence = 0.1;
            }
        }
    }
    
    fn generate_analysis_summary(&self, technologies: &[DetectedTechnology]) -> AnalysisSummary {
        let primary_language = self.determine_primary_language(technologies);
        let architecture_type = self.determine_architecture_type(technologies);
        let deployment_type = self.determine_deployment_type(technologies);
        let security_posture = self.assess_security_posture(technologies);
        let modernization_score = self.calculate_modernization_score(technologies);
        
        AnalysisSummary {
            total_technologies: technologies.len(),
            primary_language,
            architecture_type,
            deployment_type,
            security_posture,
            modernization_score,
        }
    }
    
    fn calculate_overall_confidence(&self, technologies: &[DetectedTechnology]) -> f64 {
        if technologies.is_empty() {
            return 0.0;
        }
        
        // Weight confidence by evidence quality
        let weighted_sum: f64 = technologies.iter()
            .map(|t| t.confidence * t.evidence.len() as f64)
            .sum();
        let total_evidence: f64 = technologies.iter()
            .map(|t| t.evidence.len() as f64)
            .sum();
        
        if total_evidence > 0.0 {
            weighted_sum / total_evidence
        } else {
            technologies.iter().map(|t| t.confidence).sum::<f64>() / technologies.len() as f64
        }
    }
    
    fn infer_category_from_name(&self, name: &str) -> TechCategory {
        match name.to_lowercase().as_str() {
            "javascript" | "typescript" | "python" | "rust" | "java" | "go" | "php" | "ruby" => TechCategory::ProgrammingLanguage,
            "node.js" | "deno" => TechCategory::Runtime,
            "docker" | "podman" => TechCategory::Containerization,
            "kubernetes" | "k8s" => TechCategory::Orchestration,
            "aws" | "azure" | "gcp" => TechCategory::CloudProvider,
            "mysql" | "postgresql" | "mongodb" => TechCategory::Database,
            "redis" | "memcached" => TechCategory::Cache,
            _ => TechCategory::Other,
        }
    }
    
    fn determine_primary_language(&self, technologies: &[DetectedTechnology]) -> Option<String> {
        let mut language_scores = std::collections::HashMap::new();
        
        for tech in technologies {
            if tech.category == TechCategory::ProgrammingLanguage {
                *language_scores.entry(tech.name.clone()).or_insert(0.0) += tech.confidence;
            }
        }
        
        language_scores.into_iter()
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(name, _)| name)
    }
    
    fn determine_architecture_type(&self, technologies: &[DetectedTechnology]) -> ArchitectureType {
        let has_microservices = technologies.iter().any(|t| {
            matches!(t.name.as_str(), "Docker" | "Kubernetes" | "Istio" | "Consul" | "Envoy")
        });
        
        let has_serverless = technologies.iter().any(|t| {
            matches!(t.name.as_str(), "AWS Lambda" | "Azure Functions" | "Vercel" | "Netlify")
        });
        
        let has_monolith_indicators = technologies.iter().any(|t| {
            matches!(t.name.as_str(), "Spring Boot" | "Django" | "Rails" | "Express.js")
        });
        
        if has_serverless {
            ArchitectureType::Serverless
        } else if has_microservices {
            ArchitectureType::Microservices
        } else if has_monolith_indicators {
            ArchitectureType::Monolithic
        } else {
            ArchitectureType::Unknown
        }
    }
    
    fn determine_deployment_type(&self, technologies: &[DetectedTechnology]) -> DeploymentType {
        let has_cloud = technologies.iter().any(|t| {
            t.category == TechCategory::CloudProvider || 
            matches!(t.name.as_str(), "AWS" | "Azure" | "GCP" | "Heroku" | "Vercel")
        });
        
        let has_containers = technologies.iter().any(|t| {
            t.category == TechCategory::Containerization
        });
        
        if has_containers {
            DeploymentType::ContainerBased
        } else if has_cloud {
            DeploymentType::Cloud
        } else {
            DeploymentType::Unknown
        }
    }
    
    fn assess_security_posture(&self, technologies: &[DetectedTechnology]) -> SecurityPosture {
        let security_tools = technologies.iter()
            .filter(|t| t.category == TechCategory::Security)
            .count();
        
        let outdated_technologies = technologies.iter()
            .filter(|t| self.is_technology_outdated(t))
            .count();
        
        let total_technologies = technologies.len();
        
        if security_tools > 2 && outdated_technologies == 0 {
            SecurityPosture::Excellent
        } else if security_tools > 0 && outdated_technologies < total_technologies / 4 {
            SecurityPosture::Good
        } else if outdated_technologies < total_technologies / 2 {
            SecurityPosture::Fair
        } else {
            SecurityPosture::Poor
        }
    }
    
    fn calculate_modernization_score(&self, technologies: &[DetectedTechnology]) -> f64 {
        let modern_technologies = technologies.iter()
            .filter(|t| self.is_technology_modern(t))
            .count();
        
        let total_technologies = technologies.len();
        
        if total_technologies == 0 {
            return 0.0;
        }
        
        (modern_technologies as f64 / total_technologies as f64) * 100.0
    }
    
    fn is_technology_outdated(&self, technology: &DetectedTechnology) -> bool {
        // Simple heuristic for outdated technologies
        matches!(technology.name.as_str(), 
            "jQuery" | "AngularJS" | "Bower" | "Grunt" | "Internet Explorer" | 
            "Python 2" | "PHP 5" | "Java 8" | "Node.js 10"
        )
    }
    
    fn is_technology_modern(&self, technology: &DetectedTechnology) -> bool {
        // Simple heuristic for modern technologies
        matches!(technology.name.as_str(),
            "React" | "Vue.js" | "Angular" | "Svelte" | "Next.js" | "Nuxt.js" |
            "TypeScript" | "Rust" | "Go" | "Kotlin" | "Swift" |
            "Docker" | "Kubernetes" | "GraphQL" | "WebAssembly" |
            "Vite" | "esbuild" | "SWC" | "Deno" | "Bun"
        )
    }
}

/// Initialize the global detector
pub fn initialize_detector() -> Result<()> {
    TechStackDetector::initialize()?;
    Ok(())
}

/// Get detection statistics
pub fn get_detection_stats() -> (u64, u64) {
    let detector = TechStackDetector::instance();
    (
        detector.cache_hits.load(std::sync::atomic::Ordering::Relaxed),
        detector.cache_misses.load(std::sync::atomic::Ordering::Relaxed),
    )
} 