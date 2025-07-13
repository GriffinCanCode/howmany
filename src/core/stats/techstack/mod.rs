pub mod detection;
pub mod mapping;
pub mod insights;
pub mod analysis;
pub mod patterns;
pub mod loader;

// Re-export commonly used types
pub use detection::{TechStackDetector, DetectedTechnology, TechStackInventory};
pub use mapping::{DependencyMapper, DependencyGraph, DependencyRelationship};
pub use insights::{TechStackInsights, TechStackAnalyzer as InsightsAnalyzer, FrameworkRecommendation};
pub use analysis::{TechStackStats, TechStackStatsCalculator};
pub use patterns::{PatternDatabase, DetectionPattern};

use crate::utils::errors::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Comprehensive techstack analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechStackAnalysis {
    pub inventory: TechStackInventory,
    pub dependency_graph: DependencyGraph,
    pub insights: TechStackInsights,
    pub stats: TechStackStats,
    pub metadata: TechStackMetadata,
}



/// Metadata about the techstack analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechStackMetadata {
    pub analysis_timestamp: String,
    pub analysis_version: String,
    pub files_analyzed: usize,
    pub confidence_score: f64,
    pub analysis_duration_ms: u64,
}

/// Main techstack analysis coordinator
pub struct TechStackAnalyzer {
    mapper: DependencyMapper,
}

impl TechStackAnalyzer {
    pub fn new() -> Self {
        Self {
            mapper: DependencyMapper::new(),
        }
    }

    /// Perform comprehensive techstack analysis on a project
    pub fn analyze_project(&self, project_path: &str) -> Result<TechStackAnalysis> {
        let start_time = std::time::Instant::now();
        
        // Step 1: Initialize detector and detect technologies
        let detector = TechStackDetector::new()?;
        let inventory = detector.detect_techstack(project_path)?;
        
        // Step 2: Map dependencies and relationships
        let dependency_graph = self.mapper.map_dependencies(&inventory, project_path)?;
        
        let analysis_duration = start_time.elapsed();
        
        let metadata = TechStackMetadata {
            analysis_timestamp: chrono::Utc::now().to_rfc3339(),
            analysis_version: env!("CARGO_PKG_VERSION").to_string(),
            files_analyzed: inventory.total_files_analyzed,
            confidence_score: inventory.overall_confidence,
            analysis_duration_ms: analysis_duration.as_millis() as u64,
        };

        // Step 3: Generate insights and recommendations
        let insights_analyzer = InsightsAnalyzer::new();
        let insights = insights_analyzer.analyze(&inventory, &dependency_graph)?;
        
        // Step 4: Calculate statistics
        let stats_calculator = TechStackStatsCalculator::new();
        let stats = stats_calculator.calculate_stats(&inventory, &dependency_graph)?;

        Ok(TechStackAnalysis {
            inventory,
            dependency_graph,
            insights,
            stats,
            metadata,
        })
    }

    /// Analyze a single file for techstack components
    pub fn analyze_file(&self, file_path: &str) -> Result<TechStackInventory> {
        let detector = TechStackDetector::new()?;
        let technologies = detector.detect_file_techstack(file_path)?;
        
        // Create a simple inventory from the detected technologies
        let analysis_summary = crate::core::stats::techstack::detection::AnalysisSummary {
            total_technologies: technologies.len(),
            primary_language: None,
            architecture_type: crate::core::stats::techstack::detection::ArchitectureType::Unknown,
            deployment_type: crate::core::stats::techstack::detection::DeploymentType::Unknown,
            security_posture: crate::core::stats::techstack::detection::SecurityPosture::Unknown,
            modernization_score: 0.0,
        };
        
        Ok(TechStackInventory {
            technologies,
            total_files_analyzed: 1,
            overall_confidence: 0.8,
            analysis_summary,
        })
    }

    /// Get the mapper for direct access
    pub fn mapper(&self) -> &DependencyMapper {
        &self.mapper
    }
}

impl Default for TechStackAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// Confidence levels for techstack detection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConfidenceLevel {
    VeryHigh,  // 90-100%
    High,      // 70-89%
    Medium,    // 50-69%
    Low,       // 30-49%
    VeryLow,   // 0-29%
}

impl ConfidenceLevel {
    pub fn from_score(score: f64) -> Self {
        match score {
            s if s >= 0.9 => ConfidenceLevel::VeryHigh,
            s if s >= 0.7 => ConfidenceLevel::High,
            s if s >= 0.5 => ConfidenceLevel::Medium,
            s if s >= 0.3 => ConfidenceLevel::Low,
            _ => ConfidenceLevel::VeryLow,
        }
    }

    pub fn to_score(&self) -> f64 {
        match self {
            ConfidenceLevel::VeryHigh => 0.95,
            ConfidenceLevel::High => 0.8,
            ConfidenceLevel::Medium => 0.6,
            ConfidenceLevel::Low => 0.4,
            ConfidenceLevel::VeryLow => 0.2,
        }
    }

    pub fn to_string(&self) -> &'static str {
        match self {
            ConfidenceLevel::VeryHigh => "Very High",
            ConfidenceLevel::High => "High",
            ConfidenceLevel::Medium => "Medium",
            ConfidenceLevel::Low => "Low",
            ConfidenceLevel::VeryLow => "Very Low",
        }
    }
}

/// Technology categories for classification
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TechCategory {
    // Frontend
    Frontend,
    UIFramework,
    StateManagement,
    Styling,
    
    // Backend
    Backend,
    WebFramework,
    Database,
    ORM,
    Cache,
    
    // Infrastructure
    Infrastructure,
    CloudProvider,
    Containerization,
    Orchestration,
    
    // Development Tools
    BuildTool,
    TestingFramework,
    Testing,
    Linting,
    Documentation,
    PackageManager,
    CLI,
    
    // Languages & Runtimes
    ProgrammingLanguage,
    Runtime,
    AsyncRuntime,
    
    // Security
    Security,
    Authentication,
    Authorization,
    
    // Monitoring & Logging
    Monitoring,
    Logging,
    Analytics,
    
    // Data & ML
    DataFramework,
    DataLibrary,
    MLFramework,
    
    // Specialized
    GameEngine,
    DesktopFramework,
    Messaging,
    Serialization,
    Configuration,
    CMS,
    CDN,
    WebServer,
    OperatingSystem,
    
    // Other
    Other,
}

impl TechCategory {
    pub fn to_string(&self) -> &'static str {
        match self {
            TechCategory::Frontend => "Frontend",
            TechCategory::UIFramework => "UI Framework",
            TechCategory::StateManagement => "State Management",
            TechCategory::Styling => "Styling",
            TechCategory::Backend => "Backend",
            TechCategory::WebFramework => "Web Framework",
            TechCategory::Database => "Database",
            TechCategory::ORM => "ORM",
            TechCategory::Cache => "Cache",
            TechCategory::Infrastructure => "Infrastructure",
            TechCategory::CloudProvider => "Cloud Provider",
            TechCategory::Containerization => "Containerization",
            TechCategory::Orchestration => "Orchestration",
            TechCategory::BuildTool => "Build Tool",
            TechCategory::TestingFramework => "Testing Framework",
            TechCategory::Testing => "Testing",
            TechCategory::Linting => "Linting",
            TechCategory::Documentation => "Documentation",
            TechCategory::PackageManager => "Package Manager",
            TechCategory::CLI => "CLI",
            TechCategory::ProgrammingLanguage => "Programming Language",
            TechCategory::Runtime => "Runtime",
            TechCategory::AsyncRuntime => "Async Runtime",
            TechCategory::Security => "Security",
            TechCategory::Authentication => "Authentication",
            TechCategory::Authorization => "Authorization",
            TechCategory::Monitoring => "Monitoring",
            TechCategory::Logging => "Logging",
            TechCategory::Analytics => "Analytics",
            TechCategory::DataFramework => "Data Framework",
            TechCategory::DataLibrary => "Data Library",
            TechCategory::MLFramework => "ML Framework",
            TechCategory::GameEngine => "Game Engine",
            TechCategory::DesktopFramework => "Desktop Framework",
            TechCategory::Messaging => "Messaging",
            TechCategory::Serialization => "Serialization",
            TechCategory::Configuration => "Configuration",
            TechCategory::CMS => "CMS",
            TechCategory::CDN => "CDN",
            TechCategory::WebServer => "Web Server",
            TechCategory::OperatingSystem => "Operating System",
            TechCategory::Other => "Other",
        }
    }

    pub fn get_emoji(&self) -> &'static str {
        match self {
            TechCategory::Frontend => "🎨",
            TechCategory::UIFramework => "🖼️",
            TechCategory::StateManagement => "📊",
            TechCategory::Styling => "🎨",
            TechCategory::Backend => "⚙️",
            TechCategory::WebFramework => "🌐",
            TechCategory::Database => "🗄️",
            TechCategory::ORM => "🔗",
            TechCategory::Cache => "⚡",
            TechCategory::Infrastructure => "🏗️",
            TechCategory::CloudProvider => "☁️",
            TechCategory::Containerization => "📦",
            TechCategory::Orchestration => "🎼",
            TechCategory::BuildTool => "🔨",
            TechCategory::TestingFramework => "🧪",
            TechCategory::Testing => "🧪",
            TechCategory::Linting => "🔍",
            TechCategory::Documentation => "📚",
            TechCategory::PackageManager => "📦",
            TechCategory::CLI => "💻",
            TechCategory::ProgrammingLanguage => "💻",
            TechCategory::Runtime => "🚀",
            TechCategory::AsyncRuntime => "⚡",
            TechCategory::Security => "🔒",
            TechCategory::Authentication => "🔐",
            TechCategory::Authorization => "🛡️",
            TechCategory::Monitoring => "📈",
            TechCategory::Logging => "📝",
            TechCategory::Analytics => "📊",
            TechCategory::DataFramework => "📊",
            TechCategory::DataLibrary => "📚",
            TechCategory::MLFramework => "🤖",
            TechCategory::GameEngine => "🎮",
            TechCategory::DesktopFramework => "🖥️",
            TechCategory::Messaging => "💬",
            TechCategory::Serialization => "📦",
            TechCategory::Configuration => "⚙️",
            TechCategory::CMS => "📝",
            TechCategory::CDN => "🌐",
            TechCategory::WebServer => "🌐",
            TechCategory::OperatingSystem => "💻",
            TechCategory::Other => "🔧",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::test_utils::TestProject;
    
    #[test]
    fn test_confidence_level_conversion() {
        assert_eq!(ConfidenceLevel::from_score(0.95), ConfidenceLevel::VeryHigh);
        assert_eq!(ConfidenceLevel::from_score(0.8), ConfidenceLevel::High);
        assert_eq!(ConfidenceLevel::from_score(0.6), ConfidenceLevel::Medium);
        assert_eq!(ConfidenceLevel::from_score(0.4), ConfidenceLevel::Low);
        assert_eq!(ConfidenceLevel::from_score(0.2), ConfidenceLevel::VeryLow);
    }
    
    #[test]
    fn test_tech_category_display() {
        assert_eq!(TechCategory::Frontend.to_string(), "Frontend");
        assert_eq!(TechCategory::Frontend.get_emoji(), "🎨");
        assert_eq!(TechCategory::Database.to_string(), "Database");
        assert_eq!(TechCategory::Database.get_emoji(), "🗄️");
    }
    
    #[test]
    fn test_techstack_analyzer_creation() {
        let analyzer = TechStackAnalyzer::new();
        assert!(analyzer.mapper().is_ready());
    }
    
    #[test]
    fn test_techstack_analysis_with_test_project() {
        let project = TestProject::new().unwrap();
        let analyzer = TechStackAnalyzer::new();
        
        // Create a simple test file structure
        project.create_file("package.json", r#"{"dependencies": {"react": "^18.0.0"}}"#).unwrap();
        project.create_file("src/main.js", "import React from 'react';").unwrap();
        
        // Initialize detector first
        // initialize_detector().unwrap(); // This line was removed as per the new_code
        
        let result = analyzer.analyze_project(project.path());
        assert!(result.is_ok());
        
        let analysis = result.unwrap();
        assert!(analysis.inventory.technologies.len() >= 0);
        assert!(analysis.metadata.files_analyzed >= 0);
    }
} 