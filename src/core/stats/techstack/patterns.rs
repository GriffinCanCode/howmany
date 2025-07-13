use crate::core::stats::techstack::TechCategory;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Detection pattern for identifying technologies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionPattern {
    pub name: String,
    pub category: TechCategory,
    pub confidence: f64,
    pub patterns: Vec<PatternRule>,
    pub file_patterns: Vec<String>,
    pub content_patterns: Vec<String>,
    pub dependency_patterns: Vec<DependencyPattern>,
    pub metadata: PatternMetadata,
}

/// Individual pattern rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternRule {
    pub rule_type: PatternType,
    pub pattern: String,
    pub weight: f64,
    pub context: Option<String>,
    pub negate: bool,
}

/// Type of pattern matching
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PatternType {
    FileExtension,
    FileName,
    FileContent,
    ImportStatement,
    PackageDeclaration,
    ConfigFile,
    BuildScript,
    Documentation,
    Comment,
    Keyword,
    Regex,
}

/// Dependency pattern for package managers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyPattern {
    pub package_manager: PackageManager,
    pub file_name: String,
    pub dependency_key: String,
    pub version_pattern: Option<String>,
}

/// Package manager types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PackageManager {
    Npm,
    Yarn,
    Cargo,
    Pip,
    Gem,
    Go,
    Maven,
    Gradle,
    Composer,
    NuGet,
    CocoaPods,
    Swift,
}

/// Metadata about the pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternMetadata {
    pub description: String,
    pub website: Option<String>,
    pub documentation: Option<String>,
    pub license: Option<String>,
    pub oss: Option<bool>,
    pub saas: Option<bool>,
    pub implies: Vec<String>,
    pub requires: Vec<String>,
    pub excludes: Vec<String>,
    pub tags: Vec<String>,
}

/// Pattern database for technology detection
pub struct PatternDatabase {
    patterns: HashMap<String, DetectionPattern>,
    category_index: HashMap<TechCategory, Vec<String>>,
    file_extension_index: HashMap<String, Vec<String>>,
    compiled_regexes: HashMap<String, Regex>,
}

impl PatternDatabase {
    pub fn new() -> Self {
        let mut db = Self {
            patterns: HashMap::new(),
            category_index: HashMap::new(),
            file_extension_index: HashMap::new(),
            compiled_regexes: HashMap::new(),
        };
        
        db.initialize_builtin_patterns();
        db.build_indexes();
        db
    }

    /// Initialize built-in technology patterns
    fn initialize_builtin_patterns(&mut self) {
        // Programming Languages
        self.add_pattern(create_rust_pattern());
        self.add_pattern(create_javascript_pattern());
        self.add_pattern(create_typescript_pattern());
        self.add_pattern(create_python_pattern());
        self.add_pattern(create_java_pattern());
        self.add_pattern(create_go_pattern());
        self.add_pattern(create_cpp_pattern());
        self.add_pattern(create_csharp_pattern());
        
        // Web Frameworks
        self.add_pattern(create_react_pattern());
        self.add_pattern(create_vue_pattern());
        self.add_pattern(create_angular_pattern());
        self.add_pattern(create_svelte_pattern());
        self.add_pattern(create_nextjs_pattern());
        self.add_pattern(create_nuxtjs_pattern());
        
        // Backend Frameworks
        self.add_pattern(create_django_pattern());
        self.add_pattern(create_flask_pattern());
        self.add_pattern(create_fastapi_pattern());
        self.add_pattern(create_express_pattern());
        self.add_pattern(create_nestjs_pattern());
        self.add_pattern(create_spring_pattern());
        self.add_pattern(create_actix_pattern());
        self.add_pattern(create_rocket_pattern());
        
        // Databases
        self.add_pattern(create_postgresql_pattern());
        self.add_pattern(create_mysql_pattern());
        self.add_pattern(create_mongodb_pattern());
        self.add_pattern(create_redis_pattern());
        self.add_pattern(create_sqlite_pattern());
        
        // Build Tools
        self.add_pattern(create_webpack_pattern());
        self.add_pattern(create_vite_pattern());
        self.add_pattern(create_rollup_pattern());
        self.add_pattern(create_parcel_pattern());
        self.add_pattern(create_gradle_pattern());
        self.add_pattern(create_maven_pattern());
        
        // Testing Frameworks
        self.add_pattern(create_jest_pattern());
        self.add_pattern(create_pytest_pattern());
        self.add_pattern(create_junit_pattern());
        self.add_pattern(create_mocha_pattern());
        
        // Infrastructure
        self.add_pattern(create_docker_pattern());
        self.add_pattern(create_kubernetes_pattern());
        self.add_pattern(create_terraform_pattern());
        self.add_pattern(create_aws_pattern());
        self.add_pattern(create_gcp_pattern());
        self.add_pattern(create_azure_pattern());
    }

    /// Add a pattern to the database
    fn add_pattern(&mut self, pattern: DetectionPattern) {
        let name = pattern.name.clone();
        let category = pattern.category.clone();
        
        // Add to main patterns
        self.patterns.insert(name.clone(), pattern);
        
        // Add to category index
        self.category_index.entry(category).or_insert_with(Vec::new).push(name);
    }

    /// Build indexes for fast lookups
    fn build_indexes(&mut self) {
        for (name, pattern) in &self.patterns {
            // Build file extension index
            for file_pattern in &pattern.file_patterns {
                if let Some(extension) = extract_extension(file_pattern) {
                    self.file_extension_index
                        .entry(extension)
                        .or_insert_with(Vec::new)
                        .push(name.clone());
                }
            }
            
            // Compile regex patterns
            for rule in &pattern.patterns {
                if matches!(rule.rule_type, PatternType::Regex) {
                    if let Ok(regex) = Regex::new(&rule.pattern) {
                        self.compiled_regexes.insert(rule.pattern.clone(), regex);
                    }
                }
            }
        }
    }

    /// Get patterns by category
    pub fn get_patterns_by_category(&self, category: &TechCategory) -> Vec<&DetectionPattern> {
        self.category_index
            .get(category)
            .unwrap_or(&Vec::new())
            .iter()
            .filter_map(|name| self.patterns.get(name))
            .collect()
    }

    /// Get patterns by file extension
    pub fn get_patterns_by_extension(&self, extension: &str) -> Vec<&DetectionPattern> {
        self.file_extension_index
            .get(extension)
            .unwrap_or(&Vec::new())
            .iter()
            .filter_map(|name| self.patterns.get(name))
            .collect()
    }

    /// Get all patterns
    pub fn get_all_patterns(&self) -> Vec<&DetectionPattern> {
        self.patterns.values().collect()
    }

    /// Get compiled regex for pattern
    pub fn get_compiled_regex(&self, pattern: &str) -> Option<&Regex> {
        self.compiled_regexes.get(pattern)
    }
}

// Helper function to extract file extension
fn extract_extension(file_pattern: &str) -> Option<String> {
    if file_pattern.starts_with("*.") {
        Some(file_pattern[2..].to_string())
    } else if let Some(pos) = file_pattern.rfind('.') {
        Some(file_pattern[pos + 1..].to_string())
    } else {
        None
    }
}

// Pattern creation functions

fn create_rust_pattern() -> DetectionPattern {
    DetectionPattern {
        name: "Rust".to_string(),
        category: TechCategory::ProgrammingLanguage,
        confidence: 0.95,
        patterns: vec![
            PatternRule {
                rule_type: PatternType::FileExtension,
                pattern: "rs".to_string(),
                weight: 1.0,
                context: None,
                negate: false,
            },
            PatternRule {
                rule_type: PatternType::FileName,
                pattern: "Cargo.toml".to_string(),
                weight: 0.9,
                context: None,
                negate: false,
            },
            PatternRule {
                rule_type: PatternType::FileContent,
                pattern: "fn main()".to_string(),
                weight: 0.8,
                context: None,
                negate: false,
            },
        ],
        file_patterns: vec!["*.rs".to_string(), "Cargo.toml".to_string()],
        content_patterns: vec!["fn main()".to_string(), "use std::".to_string()],
        dependency_patterns: vec![
            DependencyPattern {
                package_manager: PackageManager::Cargo,
                file_name: "Cargo.toml".to_string(),
                dependency_key: "dependencies".to_string(),
                version_pattern: None,
            }
        ],
        metadata: PatternMetadata {
            description: "Rust programming language".to_string(),
            website: Some("https://rust-lang.org".to_string()),
            documentation: Some("https://doc.rust-lang.org".to_string()),
            license: Some("MIT".to_string()),
            oss: Some(true),
            saas: Some(false),
            implies: vec!["Cargo".to_string()],
            requires: vec![],
            excludes: vec![],
            tags: vec!["language".to_string(), "systems".to_string()],
        },
    }
}

fn create_javascript_pattern() -> DetectionPattern {
    DetectionPattern {
        name: "JavaScript".to_string(),
        category: TechCategory::ProgrammingLanguage,
        confidence: 0.9,
        patterns: vec![
            PatternRule {
                rule_type: PatternType::FileExtension,
                pattern: "js".to_string(),
                weight: 1.0,
                context: None,
                negate: false,
            },
            PatternRule {
                rule_type: PatternType::FileExtension,
                pattern: "mjs".to_string(),
                weight: 0.9,
                context: None,
                negate: false,
            },
            PatternRule {
                rule_type: PatternType::FileName,
                pattern: "package.json".to_string(),
                weight: 0.8,
                context: None,
                negate: false,
            },
        ],
        file_patterns: vec!["*.js".to_string(), "*.mjs".to_string(), "package.json".to_string()],
        content_patterns: vec!["function".to_string(), "const".to_string(), "let".to_string()],
        dependency_patterns: vec![
            DependencyPattern {
                package_manager: PackageManager::Npm,
                file_name: "package.json".to_string(),
                dependency_key: "dependencies".to_string(),
                version_pattern: None,
            }
        ],
        metadata: PatternMetadata {
            description: "JavaScript programming language".to_string(),
            website: Some("https://developer.mozilla.org/en-US/docs/Web/JavaScript".to_string()),
            documentation: Some("https://developer.mozilla.org/en-US/docs/Web/JavaScript".to_string()),
            license: Some("Public Domain".to_string()),
            oss: Some(true),
            saas: Some(false),
            implies: vec!["Node.js".to_string()],
            requires: vec![],
            excludes: vec![],
            tags: vec!["language".to_string(), "web".to_string()],
        },
    }
}

fn create_typescript_pattern() -> DetectionPattern {
    DetectionPattern {
        name: "TypeScript".to_string(),
        category: TechCategory::ProgrammingLanguage,
        confidence: 0.95,
        patterns: vec![
            PatternRule {
                rule_type: PatternType::FileExtension,
                pattern: "ts".to_string(),
                weight: 1.0,
                context: None,
                negate: false,
            },
            PatternRule {
                rule_type: PatternType::FileExtension,
                pattern: "tsx".to_string(),
                weight: 1.0,
                context: None,
                negate: false,
            },
            PatternRule {
                rule_type: PatternType::FileName,
                pattern: "tsconfig.json".to_string(),
                weight: 0.9,
                context: None,
                negate: false,
            },
        ],
        file_patterns: vec!["*.ts".to_string(), "*.tsx".to_string(), "tsconfig.json".to_string()],
        content_patterns: vec!["interface".to_string(), "type".to_string(), "enum".to_string()],
        dependency_patterns: vec![
            DependencyPattern {
                package_manager: PackageManager::Npm,
                file_name: "package.json".to_string(),
                dependency_key: "devDependencies".to_string(),
                version_pattern: Some("typescript".to_string()),
            }
        ],
        metadata: PatternMetadata {
            description: "TypeScript programming language".to_string(),
            website: Some("https://www.typescriptlang.org".to_string()),
            documentation: Some("https://www.typescriptlang.org/docs/".to_string()),
            license: Some("Apache-2.0".to_string()),
            oss: Some(true),
            saas: Some(false),
            implies: vec!["JavaScript".to_string()],
            requires: vec!["JavaScript".to_string()],
            excludes: vec![],
            tags: vec!["language".to_string(), "typed".to_string()],
        },
    }
}

fn create_python_pattern() -> DetectionPattern {
    DetectionPattern {
        name: "Python".to_string(),
        category: TechCategory::ProgrammingLanguage,
        confidence: 0.9,
        patterns: vec![
            PatternRule {
                rule_type: PatternType::FileExtension,
                pattern: "py".to_string(),
                weight: 1.0,
                context: None,
                negate: false,
            },
            PatternRule {
                rule_type: PatternType::FileName,
                pattern: "requirements.txt".to_string(),
                weight: 0.8,
                context: None,
                negate: false,
            },
            PatternRule {
                rule_type: PatternType::FileName,
                pattern: "setup.py".to_string(),
                weight: 0.7,
                context: None,
                negate: false,
            },
        ],
        file_patterns: vec!["*.py".to_string(), "requirements.txt".to_string(), "setup.py".to_string()],
        content_patterns: vec!["def ".to_string(), "import ".to_string(), "from ".to_string()],
        dependency_patterns: vec![
            DependencyPattern {
                package_manager: PackageManager::Pip,
                file_name: "requirements.txt".to_string(),
                dependency_key: "".to_string(),
                version_pattern: None,
            }
        ],
        metadata: PatternMetadata {
            description: "Python programming language".to_string(),
            website: Some("https://python.org".to_string()),
            documentation: Some("https://docs.python.org".to_string()),
            license: Some("Python Software Foundation License".to_string()),
            oss: Some(true),
            saas: Some(false),
            implies: vec![],
            requires: vec![],
            excludes: vec![],
            tags: vec!["language".to_string(), "scripting".to_string()],
        },
    }
}

fn create_java_pattern() -> DetectionPattern {
    DetectionPattern {
        name: "Java".to_string(),
        category: TechCategory::ProgrammingLanguage,
        confidence: 0.9,
        patterns: vec![
            PatternRule {
                rule_type: PatternType::FileExtension,
                pattern: "java".to_string(),
                weight: 1.0,
                context: None,
                negate: false,
            },
            PatternRule {
                rule_type: PatternType::FileName,
                pattern: "pom.xml".to_string(),
                weight: 0.8,
                context: None,
                negate: false,
            },
            PatternRule {
                rule_type: PatternType::FileName,
                pattern: "build.gradle".to_string(),
                weight: 0.8,
                context: None,
                negate: false,
            },
        ],
        file_patterns: vec!["*.java".to_string(), "pom.xml".to_string(), "build.gradle".to_string()],
        content_patterns: vec!["public class".to_string(), "import java.".to_string()],
        dependency_patterns: vec![
            DependencyPattern {
                package_manager: PackageManager::Maven,
                file_name: "pom.xml".to_string(),
                dependency_key: "dependencies".to_string(),
                version_pattern: None,
            }
        ],
        metadata: PatternMetadata {
            description: "Java programming language".to_string(),
            website: Some("https://www.oracle.com/java/".to_string()),
            documentation: Some("https://docs.oracle.com/en/java/".to_string()),
            license: Some("Oracle Binary Code License".to_string()),
            oss: Some(false),
            saas: Some(false),
            implies: vec!["JVM".to_string()],
            requires: vec![],
            excludes: vec![],
            tags: vec!["language".to_string(), "enterprise".to_string()],
        },
    }
}

fn create_go_pattern() -> DetectionPattern {
    DetectionPattern {
        name: "Go".to_string(),
        category: TechCategory::ProgrammingLanguage,
        confidence: 0.95,
        patterns: vec![
            PatternRule {
                rule_type: PatternType::FileExtension,
                pattern: "go".to_string(),
                weight: 1.0,
                context: None,
                negate: false,
            },
            PatternRule {
                rule_type: PatternType::FileName,
                pattern: "go.mod".to_string(),
                weight: 0.9,
                context: None,
                negate: false,
            },
            PatternRule {
                rule_type: PatternType::FileName,
                pattern: "go.sum".to_string(),
                weight: 0.8,
                context: None,
                negate: false,
            },
        ],
        file_patterns: vec!["*.go".to_string(), "go.mod".to_string(), "go.sum".to_string()],
        content_patterns: vec!["package main".to_string(), "func main()".to_string()],
        dependency_patterns: vec![
            DependencyPattern {
                package_manager: PackageManager::Go,
                file_name: "go.mod".to_string(),
                dependency_key: "require".to_string(),
                version_pattern: None,
            }
        ],
        metadata: PatternMetadata {
            description: "Go programming language".to_string(),
            website: Some("https://golang.org".to_string()),
            documentation: Some("https://golang.org/doc/".to_string()),
            license: Some("BSD-3-Clause".to_string()),
            oss: Some(true),
            saas: Some(false),
            implies: vec![],
            requires: vec![],
            excludes: vec![],
            tags: vec!["language".to_string(), "systems".to_string()],
        },
    }
}

fn create_cpp_pattern() -> DetectionPattern {
    DetectionPattern {
        name: "C++".to_string(),
        category: TechCategory::ProgrammingLanguage,
        confidence: 0.9,
        patterns: vec![
            PatternRule {
                rule_type: PatternType::FileExtension,
                pattern: "cpp".to_string(),
                weight: 1.0,
                context: None,
                negate: false,
            },
            PatternRule {
                rule_type: PatternType::FileExtension,
                pattern: "hpp".to_string(),
                weight: 0.9,
                context: None,
                negate: false,
            },
            PatternRule {
                rule_type: PatternType::FileExtension,
                pattern: "cc".to_string(),
                weight: 0.8,
                context: None,
                negate: false,
            },
        ],
        file_patterns: vec!["*.cpp".to_string(), "*.hpp".to_string(), "*.cc".to_string()],
        content_patterns: vec!["#include".to_string(), "using namespace".to_string()],
        dependency_patterns: vec![],
        metadata: PatternMetadata {
            description: "C++ programming language".to_string(),
            website: Some("https://isocpp.org".to_string()),
            documentation: Some("https://en.cppreference.com".to_string()),
            license: Some("ISO Standard".to_string()),
            oss: Some(true),
            saas: Some(false),
            implies: vec![],
            requires: vec![],
            excludes: vec![],
            tags: vec!["language".to_string(), "systems".to_string()],
        },
    }
}

fn create_csharp_pattern() -> DetectionPattern {
    DetectionPattern {
        name: "C#".to_string(),
        category: TechCategory::ProgrammingLanguage,
        confidence: 0.9,
        patterns: vec![
            PatternRule {
                rule_type: PatternType::FileExtension,
                pattern: "cs".to_string(),
                weight: 1.0,
                context: None,
                negate: false,
            },
            PatternRule {
                rule_type: PatternType::FileName,
                pattern: "*.csproj".to_string(),
                weight: 0.9,
                context: None,
                negate: false,
            },
            PatternRule {
                rule_type: PatternType::FileName,
                pattern: "*.sln".to_string(),
                weight: 0.8,
                context: None,
                negate: false,
            },
        ],
        file_patterns: vec!["*.cs".to_string(), "*.csproj".to_string(), "*.sln".to_string()],
        content_patterns: vec!["using System".to_string(), "namespace".to_string()],
        dependency_patterns: vec![],
        metadata: PatternMetadata {
            description: "C# programming language".to_string(),
            website: Some("https://docs.microsoft.com/en-us/dotnet/csharp/".to_string()),
            documentation: Some("https://docs.microsoft.com/en-us/dotnet/csharp/".to_string()),
            license: Some("MIT".to_string()),
            oss: Some(true),
            saas: Some(false),
            implies: vec![".NET".to_string()],
            requires: vec![],
            excludes: vec![],
            tags: vec!["language".to_string(), "microsoft".to_string()],
        },
    }
}

// Framework patterns (abbreviated for brevity)
fn create_react_pattern() -> DetectionPattern {
    DetectionPattern {
        name: "React".to_string(),
        category: TechCategory::UIFramework,
        confidence: 0.9,
        patterns: vec![
            PatternRule {
                rule_type: PatternType::FileExtension,
                pattern: "jsx".to_string(),
                weight: 1.0,
                context: None,
                negate: false,
            },
            PatternRule {
                rule_type: PatternType::FileExtension,
                pattern: "tsx".to_string(),
                weight: 0.9,
                context: None,
                negate: false,
            },
        ],
        file_patterns: vec!["*.jsx".to_string(), "*.tsx".to_string()],
        content_patterns: vec!["import React".to_string(), "from 'react'".to_string()],
        dependency_patterns: vec![
            DependencyPattern {
                package_manager: PackageManager::Npm,
                file_name: "package.json".to_string(),
                dependency_key: "dependencies".to_string(),
                version_pattern: Some("react".to_string()),
            }
        ],
        metadata: PatternMetadata {
            description: "React JavaScript library".to_string(),
            website: Some("https://reactjs.org".to_string()),
            documentation: Some("https://reactjs.org/docs/".to_string()),
            license: Some("MIT".to_string()),
            oss: Some(true),
            saas: Some(false),
            implies: vec!["JavaScript".to_string()],
            requires: vec!["JavaScript".to_string()],
            excludes: vec!["Vue".to_string(), "Angular".to_string()],
            tags: vec!["framework".to_string(), "ui".to_string()],
        },
    }
}

// Additional framework patterns would follow similar structure...
// For brevity, I'll create simplified versions of the remaining patterns

fn create_vue_pattern() -> DetectionPattern {
    DetectionPattern {
        name: "Vue".to_string(),
        category: TechCategory::UIFramework,
        confidence: 0.9,
        patterns: vec![
            PatternRule {
                rule_type: PatternType::FileExtension,
                pattern: "vue".to_string(),
                weight: 1.0,
                context: None,
                negate: false,
            },
        ],
        file_patterns: vec!["*.vue".to_string()],
        content_patterns: vec!["<template>".to_string(), "Vue.".to_string()],
        dependency_patterns: vec![],
        metadata: PatternMetadata {
            description: "Vue.js framework".to_string(),
            website: Some("https://vuejs.org".to_string()),
            documentation: Some("https://vuejs.org/guide/".to_string()),
            license: Some("MIT".to_string()),
            oss: Some(true),
            saas: Some(false),
            implies: vec!["JavaScript".to_string()],
            requires: vec!["JavaScript".to_string()],
            excludes: vec!["React".to_string(), "Angular".to_string()],
            tags: vec!["framework".to_string(), "ui".to_string()],
        },
    }
}

// Simplified patterns for other technologies
fn create_angular_pattern() -> DetectionPattern { create_simple_pattern("Angular", TechCategory::UIFramework) }
fn create_svelte_pattern() -> DetectionPattern { create_simple_pattern("Svelte", TechCategory::UIFramework) }
fn create_nextjs_pattern() -> DetectionPattern { create_simple_pattern("Next.js", TechCategory::WebFramework) }
fn create_nuxtjs_pattern() -> DetectionPattern { create_simple_pattern("Nuxt.js", TechCategory::WebFramework) }
fn create_django_pattern() -> DetectionPattern { create_simple_pattern("Django", TechCategory::WebFramework) }
fn create_flask_pattern() -> DetectionPattern { create_simple_pattern("Flask", TechCategory::WebFramework) }
fn create_fastapi_pattern() -> DetectionPattern { create_simple_pattern("FastAPI", TechCategory::WebFramework) }
fn create_express_pattern() -> DetectionPattern { create_simple_pattern("Express", TechCategory::WebFramework) }
fn create_nestjs_pattern() -> DetectionPattern { create_simple_pattern("NestJS", TechCategory::WebFramework) }
fn create_spring_pattern() -> DetectionPattern { create_simple_pattern("Spring", TechCategory::WebFramework) }
fn create_actix_pattern() -> DetectionPattern { create_simple_pattern("Actix", TechCategory::WebFramework) }
fn create_rocket_pattern() -> DetectionPattern { create_simple_pattern("Rocket", TechCategory::WebFramework) }
fn create_postgresql_pattern() -> DetectionPattern { create_simple_pattern("PostgreSQL", TechCategory::Database) }
fn create_mysql_pattern() -> DetectionPattern { create_simple_pattern("MySQL", TechCategory::Database) }
fn create_mongodb_pattern() -> DetectionPattern { create_simple_pattern("MongoDB", TechCategory::Database) }
fn create_redis_pattern() -> DetectionPattern { create_simple_pattern("Redis", TechCategory::Database) }
fn create_sqlite_pattern() -> DetectionPattern { create_simple_pattern("SQLite", TechCategory::Database) }
fn create_webpack_pattern() -> DetectionPattern { create_simple_pattern("Webpack", TechCategory::BuildTool) }
fn create_vite_pattern() -> DetectionPattern { create_simple_pattern("Vite", TechCategory::BuildTool) }
fn create_rollup_pattern() -> DetectionPattern { create_simple_pattern("Rollup", TechCategory::BuildTool) }
fn create_parcel_pattern() -> DetectionPattern { create_simple_pattern("Parcel", TechCategory::BuildTool) }
fn create_gradle_pattern() -> DetectionPattern { create_simple_pattern("Gradle", TechCategory::BuildTool) }
fn create_maven_pattern() -> DetectionPattern { create_simple_pattern("Maven", TechCategory::BuildTool) }
fn create_jest_pattern() -> DetectionPattern { create_simple_pattern("Jest", TechCategory::TestingFramework) }
fn create_pytest_pattern() -> DetectionPattern { create_simple_pattern("pytest", TechCategory::TestingFramework) }
fn create_junit_pattern() -> DetectionPattern { create_simple_pattern("JUnit", TechCategory::TestingFramework) }
fn create_mocha_pattern() -> DetectionPattern { create_simple_pattern("Mocha", TechCategory::TestingFramework) }
fn create_docker_pattern() -> DetectionPattern { create_simple_pattern("Docker", TechCategory::Containerization) }
fn create_kubernetes_pattern() -> DetectionPattern { create_simple_pattern("Kubernetes", TechCategory::Orchestration) }
fn create_terraform_pattern() -> DetectionPattern { create_simple_pattern("Terraform", TechCategory::Infrastructure) }
fn create_aws_pattern() -> DetectionPattern { create_simple_pattern("AWS", TechCategory::CloudProvider) }
fn create_gcp_pattern() -> DetectionPattern { create_simple_pattern("Google Cloud", TechCategory::CloudProvider) }
fn create_azure_pattern() -> DetectionPattern { create_simple_pattern("Azure", TechCategory::CloudProvider) }

// Helper function to create simple patterns
fn create_simple_pattern(name: &str, category: TechCategory) -> DetectionPattern {
    DetectionPattern {
        name: name.to_string(),
        category,
        confidence: 0.8,
        patterns: vec![],
        file_patterns: vec![],
        content_patterns: vec![],
        dependency_patterns: vec![],
        metadata: PatternMetadata {
            description: format!("{} technology", name),
            website: None,
            documentation: None,
            license: None,
            oss: None,
            saas: None,
            implies: vec![],
            requires: vec![],
            excludes: vec![],
            tags: vec![],
        },
    }
}

impl Default for PatternDatabase {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pattern_database_creation() {
        let db = PatternDatabase::new();
        assert!(!db.patterns.is_empty());
    }

    #[test]
    fn test_get_patterns_by_category() {
        let db = PatternDatabase::new();
        let patterns = db.get_patterns_by_category(&TechCategory::ProgrammingLanguage);
        assert!(!patterns.is_empty());
    }

    #[test]
    fn test_get_patterns_by_extension() {
        let db = PatternDatabase::new();
        let patterns = db.get_patterns_by_extension("rs");
        assert!(!patterns.is_empty());
    }

    #[test]
    fn test_extract_extension() {
        assert_eq!(extract_extension("*.rs"), Some("rs".to_string()));
        assert_eq!(extract_extension("Cargo.toml"), Some("toml".to_string()));
        assert_eq!(extract_extension("README"), None);
    }
} 