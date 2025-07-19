// Type definitions for complexity analysis
// Main implementation is in src/core/stats/complexity.rs
// This module contains only the type definitions to avoid circular dependencies


use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Complexity statistics for a file or project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityStats {
    pub function_count: usize,
    pub class_count: usize,
    pub interface_count: usize,
    pub trait_count: usize,
    pub enum_count: usize,
    pub struct_count: usize,
    pub module_count: usize,
    pub total_structures: usize,
    pub cyclomatic_complexity: f64,
    pub cognitive_complexity: f64,
    pub maintainability_index: f64,
    pub average_function_length: f64,
    pub max_function_length: usize,
    pub min_function_length: usize,
    pub max_nesting_depth: usize,
    pub average_nesting_depth: f64,
    pub methods_per_class: f64,
    pub average_parameters_per_function: f64,
    pub max_parameters_per_function: usize,
    pub complexity_by_extension: HashMap<String, ExtensionComplexity>,
    pub complexity_distribution: ComplexityDistribution,
    pub structure_distribution: StructureDistribution,
    pub function_complexity_details: Vec<FunctionComplexityDetail>,
    pub quality_metrics: QualityMetrics,
}

/// Code health metrics for practical developer insights
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityMetrics {
    pub code_health_score: f64,        // Overall code health (0-100)
    pub maintainability_index: f64,    // Industry-standard maintainability index (0-100)
    pub documentation_coverage: f64,   // Percentage of code with documentation (0-100)
    pub avg_complexity: f64,           // Average cyclomatic complexity per function
    pub function_size_health: f64,     // Health score based on function sizes (0-100)
    pub nesting_depth_health: f64,     // Health score based on nesting depth (0-100)
    pub code_duplication_ratio: f64,   // Estimated code duplication percentage (0-100)
    pub technical_debt_ratio: f64,     // Estimated technical debt ratio (0-100)
}

/// Detailed complexity information for individual functions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionComplexityDetail {
    pub name: String,
    pub file_path: String,
    pub start_line: usize,
    pub end_line: usize,
    pub line_count: usize,
    pub cyclomatic_complexity: usize,
    pub cognitive_complexity: usize,
    pub parameter_count: usize,
    pub return_path_count: usize,
    pub nesting_depth: usize,
    pub is_method: bool,
    pub parent_class: Option<String>,
    pub local_variable_count: usize,
    pub has_recursion: bool,
    pub has_exception_handling: bool,
    pub complexity_level: ComplexityLevel,
    pub maintainability_concerns: Vec<String>,
}

/// Complexity level classification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ComplexityLevel {
    VeryLow,    // 1-5
    Low,        // 6-10
    Medium,     // 11-20
    High,       // 21-50
    VeryHigh,   // 51+
}

/// Distribution of different structure types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructureDistribution {
    pub classes: usize,
    pub interfaces: usize,
    pub traits: usize,
    pub enums: usize,
    pub structs: usize,
    pub modules: usize,
}

/// Complexity statistics for a specific file extension
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionComplexity {
    pub function_count: usize,
    pub class_count: usize,
    pub interface_count: usize,
    pub trait_count: usize,
    pub enum_count: usize,
    pub struct_count: usize,
    pub total_structures: usize,
    pub cyclomatic_complexity: f64,
    pub cognitive_complexity: f64,
    pub maintainability_index: f64,
    pub average_function_length: f64,
    pub max_nesting_depth: usize,
    pub average_nesting_depth: f64,
    pub methods_per_class: f64,
    pub average_parameters_per_function: f64,
    pub quality_score: f64,
}

/// Distribution of complexity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityDistribution {
    pub very_low_complexity: usize,  // 1-5
    pub low_complexity: usize,       // 6-10
    pub medium_complexity: usize,    // 11-20
    pub high_complexity: usize,      // 21-50
    pub very_high_complexity: usize, // 51+
}

/// Enhanced function information for complexity analysis
#[derive(Debug, Clone)]
pub struct FunctionInfo {
    pub name: String,
    pub line_count: usize,
    pub cyclomatic_complexity: usize,
    pub cognitive_complexity: usize,
    pub nesting_depth: usize,
    pub parameter_count: usize,
    pub return_path_count: usize,
    pub start_line: usize,
    pub end_line: usize,
    pub is_method: bool,
    pub parent_class: Option<String>,
    pub local_variable_count: usize,
    pub has_recursion: bool,
    pub has_exception_handling: bool,
    pub visibility: Visibility,
}

/// Structure information (classes, interfaces, enums, etc.)
#[derive(Debug, Clone)]
pub struct StructureInfo {
    pub name: String,
    pub structure_type: StructureType,
    pub line_count: usize,
    pub start_line: usize,
    pub end_line: usize,
    pub methods: Vec<FunctionInfo>,
    pub properties: usize,
    pub visibility: Visibility,
    pub inheritance_depth: usize,
    pub interface_count: usize,
}

/// Type of code structure
#[derive(Debug, Clone, PartialEq)]
pub enum StructureType {
    Class,
    Interface,
    Trait,
    Enum,
    Struct,
    Module,
    Namespace,
}

/// Visibility of code structure
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Visibility {
    Public,
    Private,
    Protected,
    Internal,
    Unknown,
} 