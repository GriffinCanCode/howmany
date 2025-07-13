use crate::core::types::{CodeStats, FileStats};
use crate::utils::errors::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;

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

/// Quality metrics for code assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityMetrics {
    pub overall_quality_score: f64,
    pub maintainability_score: f64,
    pub readability_score: f64,
    pub testability_score: f64,
    pub code_duplication_ratio: f64,
    pub comment_coverage_ratio: f64,
    pub function_size_score: f64,
    pub complexity_score: f64,
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
#[derive(Debug, Clone, PartialEq)]
pub enum Visibility {
    Public,
    Private,
    Protected,
    Internal,
    Unknown,
}

/// Calculator for complexity statistics
pub struct ComplexityStatsCalculator;

impl ComplexityStatsCalculator {
    pub fn new() -> Self {
        Self
    }
    
    /// Calculate complexity statistics for a single file
    pub fn calculate_complexity_stats(&self, file_stats: &FileStats, file_path: &str) -> Result<ComplexityStats> {
        let functions = self.analyze_file_functions(file_path)?;
        let structures = self.analyze_file_structures(file_path)?;
        
        let function_count = functions.len();
        
        // Calculate cyclomatic complexity
        let total_cyclomatic = functions.iter().map(|f| f.cyclomatic_complexity as f64).sum::<f64>();
        let cyclomatic_complexity = if function_count > 0 { total_cyclomatic / function_count as f64 } else { 0.0 };
        
        // Calculate cognitive complexity
        let total_cognitive = functions.iter().map(|f| f.cognitive_complexity as f64).sum::<f64>();
        let cognitive_complexity = if function_count > 0 { total_cognitive / function_count as f64 } else { 0.0 };
        
        // Calculate maintainability index
        let maintainability_index = self.calculate_maintainability_index(&functions, file_stats);
        
        let average_function_length = if function_count > 0 {
            functions.iter().map(|f| f.line_count as f64).sum::<f64>() / function_count as f64
        } else {
            0.0
        };
        
        let max_function_length = functions.iter().map(|f| f.line_count).max().unwrap_or(0);
        let min_function_length = functions.iter().map(|f| f.line_count).min().unwrap_or(0);
        let max_nesting_depth = functions.iter().map(|f| f.nesting_depth).max().unwrap_or(0);
        let average_nesting_depth = if function_count > 0 {
            functions.iter().map(|f| f.nesting_depth as f64).sum::<f64>() / function_count as f64
        } else {
            0.0
        };
        
        let average_parameters_per_function = if function_count > 0 {
            functions.iter().map(|f| f.parameter_count as f64).sum::<f64>() / function_count as f64
        } else {
            0.0
        };
        
        let max_parameters_per_function = functions.iter().map(|f| f.parameter_count).max().unwrap_or(0);
        
        let complexity_distribution = self.calculate_complexity_distribution(&functions);
        let structure_distribution = self.calculate_structure_distribution(&structures);
        
        let class_count = structures.iter().filter(|s| s.structure_type == StructureType::Class).count();
        let interface_count = structures.iter().filter(|s| s.structure_type == StructureType::Interface).count();
        let trait_count = structures.iter().filter(|s| s.structure_type == StructureType::Trait).count();
        let enum_count = structures.iter().filter(|s| s.structure_type == StructureType::Enum).count();
        let struct_count = structures.iter().filter(|s| s.structure_type == StructureType::Struct).count();
        let module_count = structures.iter().filter(|s| s.structure_type == StructureType::Module || s.structure_type == StructureType::Namespace).count();
        let total_structures = structures.len();
        
        let methods_per_class = if class_count > 0 {
            structures.iter()
                .filter(|s| s.structure_type == StructureType::Class)
                .map(|s| s.methods.len())
                .sum::<usize>() as f64 / class_count as f64
        } else {
            0.0
        };
        
        let function_complexity_details = self.create_function_complexity_details(&functions, file_path);
        let quality_metrics = self.calculate_quality_metrics(&functions, file_stats, &structures);
        
        Ok(ComplexityStats {
            function_count,
            class_count,
            interface_count,
            trait_count,
            enum_count,
            struct_count,
            module_count,
            total_structures,
            cyclomatic_complexity,
            cognitive_complexity,
            maintainability_index,
            average_function_length,
            max_function_length,
            min_function_length,
            max_nesting_depth,
            average_nesting_depth,
            methods_per_class,
            average_parameters_per_function,
            max_parameters_per_function,
            complexity_by_extension: HashMap::new(),
            complexity_distribution,
            structure_distribution,
            function_complexity_details,
            quality_metrics,
        })
    }
    
    /// Calculate complexity statistics for a project
    pub fn calculate_project_complexity_stats(&self, code_stats: &CodeStats, individual_files: &[(String, FileStats)]) -> Result<ComplexityStats> {
        let mut total_functions = 0;
        let mut total_classes = 0;
        let mut total_interfaces = 0;
        let mut total_traits = 0;
        let mut total_enums = 0;
        let mut total_structs = 0;
        let mut total_modules = 0;
        let mut total_complexity = 0.0;
        let mut total_function_lines = 0;
        let mut max_function_length = 0;
        let mut min_function_length = usize::MAX;
        let mut max_nesting_depth = 0;
        let mut total_nesting_depth = 0.0;
        let mut complexity_by_extension = HashMap::new();
        let mut all_functions = Vec::new();
        let mut all_structures = Vec::new();
        
        // Analyze individual files for detailed complexity metrics
        for (file_path, _) in individual_files {
            if let Ok(functions) = self.analyze_file_functions(file_path) {
                all_functions.extend(functions.clone());
            }
            
            if let Ok(structures) = self.analyze_file_structures(file_path) {
                all_structures.extend(structures.clone());
                
                total_classes += structures.iter().filter(|s| s.structure_type == StructureType::Class).count();
                total_interfaces += structures.iter().filter(|s| s.structure_type == StructureType::Interface).count();
                total_traits += structures.iter().filter(|s| s.structure_type == StructureType::Trait).count();
                total_enums += structures.iter().filter(|s| s.structure_type == StructureType::Enum).count();
                total_structs += structures.iter().filter(|s| s.structure_type == StructureType::Struct).count();
                total_modules += structures.iter().filter(|s| s.structure_type == StructureType::Module || s.structure_type == StructureType::Namespace).count();
            }
            
            if let Ok(functions) = self.analyze_file_functions(file_path) {
                let extension = Path::new(file_path)
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .unwrap_or("unknown")
                    .to_lowercase();
                
                let function_count = functions.len();
                if function_count > 0 {
                    let ext_complexity = functions.iter().map(|f| f.cyclomatic_complexity as f64).sum::<f64>() / function_count as f64;
                    let ext_avg_length = functions.iter().map(|f| f.line_count as f64).sum::<f64>() / function_count as f64;
                    let ext_max_nesting = functions.iter().map(|f| f.nesting_depth).max().unwrap_or(0);
                    let ext_avg_nesting = functions.iter().map(|f| f.nesting_depth as f64).sum::<f64>() / function_count as f64;
                    
                    let entry = complexity_by_extension.entry(extension).or_insert(ExtensionComplexity {
                        function_count: 0,
                        class_count: 0,
                        interface_count: 0,
                        trait_count: 0,
                        enum_count: 0,
                        struct_count: 0,
                        total_structures: 0,
                        cyclomatic_complexity: 0.0,
                        cognitive_complexity: 0.0,
                        maintainability_index: 0.0,
                        average_function_length: 0.0,
                        max_nesting_depth: 0,
                        average_nesting_depth: 0.0,
                        methods_per_class: 0.0,
                        average_parameters_per_function: 0.0,
                        quality_score: 0.0,
                    });
                    
                    entry.function_count += function_count;
                    entry.cyclomatic_complexity = (entry.cyclomatic_complexity * (entry.function_count - function_count) as f64 + ext_complexity * function_count as f64) / entry.function_count as f64;
                    entry.average_function_length = (entry.average_function_length * (entry.function_count - function_count) as f64 + ext_avg_length * function_count as f64) / entry.function_count as f64;
                    entry.max_nesting_depth = entry.max_nesting_depth.max(ext_max_nesting);
                    entry.average_nesting_depth = (entry.average_nesting_depth * (entry.function_count - function_count) as f64 + ext_avg_nesting * function_count as f64) / entry.function_count as f64;
                }
                
                all_functions.extend(functions);
            }
        }
        
        // Calculate aggregate statistics
        total_functions = all_functions.len();
        if total_functions > 0 {
            total_complexity = all_functions.iter().map(|f| f.cyclomatic_complexity as f64).sum::<f64>();
            total_function_lines = all_functions.iter().map(|f| f.line_count).sum();
            max_function_length = all_functions.iter().map(|f| f.line_count).max().unwrap_or(0);
            min_function_length = all_functions.iter().map(|f| f.line_count).min().unwrap_or(0);
            max_nesting_depth = all_functions.iter().map(|f| f.nesting_depth).max().unwrap_or(0);
            total_nesting_depth = all_functions.iter().map(|f| f.nesting_depth as f64).sum::<f64>();
        }
        
        // Calculate cognitive complexity and other metrics
        let total_cognitive_complexity = all_functions.iter().map(|f| f.cognitive_complexity as f64).sum::<f64>();
        let cognitive_complexity = if total_functions > 0 { total_cognitive_complexity / total_functions as f64 } else { 0.0 };
        
        let total_parameters = all_functions.iter().map(|f| f.parameter_count).sum::<usize>();
        let average_parameters_per_function = if total_functions > 0 { total_parameters as f64 / total_functions as f64 } else { 0.0 };
        let max_parameters_per_function = all_functions.iter().map(|f| f.parameter_count).max().unwrap_or(0);
        
        // Calculate maintainability index for the project
        let maintainability_index = if total_functions > 0 {
            let avg_complexity = total_complexity / total_functions as f64;
            let avg_length = total_function_lines as f64 / total_functions as f64;
            let avg_cognitive = cognitive_complexity;
            let avg_params = average_parameters_per_function;
            
            // Simplified maintainability calculation
            let length_score = (50.0 - avg_length).max(0.0);
            let complexity_score = (30.0 - avg_complexity * 2.0).max(0.0);
            let cognitive_score = (30.0 - avg_cognitive * 2.0).max(0.0);
            let param_score = (20.0 - avg_params * 3.0).max(0.0);
            
            (length_score + complexity_score + cognitive_score + param_score).min(100.0).max(0.0)
        } else {
            100.0
        };
        
        let complexity_distribution = self.calculate_complexity_distribution(&all_functions);
        let structure_distribution = self.calculate_structure_distribution(&all_structures);
        
        let total_structures = all_structures.len();
        let methods_per_class = if total_classes > 0 {
            all_structures.iter()
                .filter(|s| s.structure_type == StructureType::Class)
                .map(|s| s.methods.len())
                .sum::<usize>() as f64 / total_classes as f64
        } else {
            0.0
        };
        
        // Calculate quality metrics for the project
        let quality_metrics = self.calculate_project_quality_metrics(&all_functions, code_stats, &all_structures);
        
        Ok(ComplexityStats {
            function_count: total_functions,
            class_count: total_classes,
            interface_count: total_interfaces,
            trait_count: total_traits,
            enum_count: total_enums,
            struct_count: total_structs,
            module_count: total_modules,
            total_structures,
            cyclomatic_complexity: if total_functions > 0 { total_complexity / total_functions as f64 } else { 0.0 },
            cognitive_complexity,
            maintainability_index,
            average_function_length: if total_functions > 0 { total_function_lines as f64 / total_functions as f64 } else { 0.0 },
            max_function_length,
            min_function_length: if min_function_length == usize::MAX { 0 } else { min_function_length },
            max_nesting_depth,
            average_nesting_depth: if total_functions > 0 { total_nesting_depth / total_functions as f64 } else { 0.0 },
            methods_per_class,
            average_parameters_per_function,
            max_parameters_per_function,
            complexity_by_extension,
            complexity_distribution,
            structure_distribution,
            function_complexity_details: Vec::new(), // Will be populated by calling code if needed
            quality_metrics,
        })
    }
    
    /// Analyze structures in a file (classes, interfaces, etc.)
    fn analyze_file_structures(&self, file_path: &str) -> Result<Vec<StructureInfo>> {
        let file = fs::File::open(file_path)?;
        let reader = BufReader::new(file);
        let lines: Vec<String> = reader.lines().collect::<std::io::Result<Vec<_>>>()?;
        
        let extension = Path::new(file_path)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("unknown")
            .to_lowercase();
        
        match extension.as_str() {
            "rs" => self.analyze_rust_structures(&lines),
            "py" => self.analyze_python_structures(&lines),
            "js" | "jsx" | "ts" | "tsx" => self.analyze_javascript_structures(&lines),
            "java" => self.analyze_java_structures(&lines),
            "cpp" | "cc" | "cxx" | "c" | "h" | "hpp" => self.analyze_cpp_structures(&lines),
            "go" => self.analyze_go_structures(&lines),
            "cs" => self.analyze_csharp_structures(&lines),
            "php" => self.analyze_php_structures(&lines),
            "rb" => self.analyze_ruby_structures(&lines),
            "swift" => self.analyze_swift_structures(&lines),
            "kt" => self.analyze_kotlin_structures(&lines),
            _ => Ok(Vec::new()), // Unsupported language
        }
    }
    
    /// Analyze functions in a file for complexity metrics
    fn analyze_file_functions(&self, file_path: &str) -> Result<Vec<FunctionInfo>> {
        let file = fs::File::open(file_path)?;
        let reader = BufReader::new(file);
        let lines: Vec<String> = reader.lines().collect::<std::io::Result<Vec<_>>>()?;
        
        let extension = Path::new(file_path)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("unknown")
            .to_lowercase();
        
        match extension.as_str() {
            "rs" => self.analyze_rust_functions(&lines),
            "py" => self.analyze_python_functions(&lines),
            "js" | "jsx" | "ts" | "tsx" => self.analyze_javascript_functions(&lines),
            "java" => self.analyze_java_functions(&lines),
            "cpp" | "cc" | "cxx" | "c" => self.analyze_cpp_functions(&lines),
            "go" => self.analyze_go_functions(&lines),
            _ => Ok(Vec::new()), // Unsupported language
        }
    }
    
    /// Analyze Rust functions
    fn analyze_rust_functions(&self, lines: &[String]) -> Result<Vec<FunctionInfo>> {
        let mut functions = Vec::new();
        let mut current_function: Option<FunctionInfo> = None;
        let mut brace_count = 0;
        let mut in_function = false;
        
        for (line_num, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            
            // Skip comments and empty lines
            if trimmed.starts_with("//") || trimmed.is_empty() {
                continue;
            }
            
            // Function declaration detection
            if trimmed.starts_with("fn ") || trimmed.contains(" fn ") {
                if let Some(func_name) = self.extract_rust_function_name(trimmed) {
                    current_function = Some(FunctionInfo {
                        name: func_name,
                        line_count: 0,
                        cyclomatic_complexity: 1, // Base complexity
                        cognitive_complexity: 1, // Base cognitive complexity
                        nesting_depth: 0,
                        parameter_count: 0,
                        return_path_count: 0,
                        start_line: line_num + 1,
                        end_line: line_num + 1,
                        is_method: false,
                        parent_class: None,
                        local_variable_count: 0,
                        has_recursion: false,
                        has_exception_handling: false,
                    });
                    in_function = true;
                    brace_count = 0;
                }
            }
            
            if in_function {
                if let Some(ref mut func) = current_function {
                    func.line_count += 1;
                    func.end_line = line_num + 1;
                    
                    // Count braces for nesting depth
                    let open_braces = trimmed.matches('{').count();
                    let close_braces = trimmed.matches('}').count();
                    brace_count += open_braces as i32 - close_braces as i32;
                    func.nesting_depth = func.nesting_depth.max(brace_count.max(0) as usize);
                    
                    // Calculate cyclomatic complexity
                    func.cyclomatic_complexity += self.count_rust_complexity_keywords(trimmed);
                    
                    // Calculate cognitive complexity
                    func.cognitive_complexity += self.count_rust_cognitive_complexity(trimmed, brace_count);
                    
                    // Count parameters
                    if trimmed.contains('(') && func.parameter_count == 0 {
                        func.parameter_count = self.count_function_parameters(trimmed);
                    }
                    
                    // Count return paths
                    if trimmed.contains("return") {
                        func.return_path_count += 1;
                    }
                    
                    // Check for recursion
                    if trimmed.contains(&func.name) && !trimmed.starts_with("fn ") {
                        func.has_recursion = true;
                    }
                    
                    // Check for exception handling
                    if trimmed.contains("try") || trimmed.contains("catch") || trimmed.contains("?") {
                        func.has_exception_handling = true;
                    }
                    
                    // Function end detection
                    if brace_count <= 0 && close_braces > 0 {
                        functions.push(func.clone());
                        current_function = None;
                        in_function = false;
                    }
                }
            }
        }
        
        Ok(functions)
    }
    
    /// Analyze Python functions
    fn analyze_python_functions(&self, lines: &[String]) -> Result<Vec<FunctionInfo>> {
        let mut functions = Vec::new();
        let mut current_function: Option<FunctionInfo> = None;
        let mut indent_level = 0;
        let mut function_indent = 0;
        
        for (line_num, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            
            // Skip comments and empty lines
            if trimmed.starts_with("#") || trimmed.is_empty() {
                continue;
            }
            
            // Calculate indentation
            let current_indent = line.len() - line.trim_start().len();
            
            // Function declaration detection
            if trimmed.starts_with("def ") {
                if let Some(func_name) = self.extract_python_function_name(trimmed) {
                    // Save previous function if exists
                    if let Some(func) = current_function.take() {
                        functions.push(func);
                    }
                    
                    current_function = Some(FunctionInfo {
                        name: func_name,
                        line_count: 0,
                        cyclomatic_complexity: 1, // Base complexity
                        cognitive_complexity: 1, // Base cognitive complexity
                        nesting_depth: 0,
                        parameter_count: 0,
                        return_path_count: 0,
                        start_line: line_num + 1,
                        end_line: line_num + 1,
                        is_method: trimmed.contains("self"),
                        parent_class: None,
                        local_variable_count: 0,
                        has_recursion: false,
                        has_exception_handling: false,
                    });
                    function_indent = current_indent;
                    indent_level = current_indent;
                }
            }
            
            if let Some(ref mut func) = current_function {
                // Check if we're still in the function
                if current_indent <= function_indent && line_num > func.start_line - 1 && !trimmed.is_empty() {
                    // Function ended
                    functions.push(func.clone());
                    current_function = None;
                    continue;
                }
                
                if current_indent > function_indent {
                    func.line_count += 1;
                    func.end_line = line_num + 1;
                    
                    // Calculate nesting depth
                    let relative_indent = (current_indent - function_indent) / 4; // Assuming 4-space indentation
                    func.nesting_depth = func.nesting_depth.max(relative_indent);
                    
                    // Calculate cyclomatic complexity
                    func.cyclomatic_complexity += self.count_python_complexity_keywords(trimmed);
                    
                    // Calculate cognitive complexity
                    func.cognitive_complexity += self.count_python_cognitive_complexity(trimmed, relative_indent);
                    
                    // Count parameters
                    if trimmed.contains('(') && func.parameter_count == 0 {
                        func.parameter_count = self.count_function_parameters(trimmed);
                    }
                    
                    // Count return paths
                    if trimmed.contains("return") {
                        func.return_path_count += 1;
                    }
                    
                    // Check for recursion
                    if trimmed.contains(&func.name) && !trimmed.starts_with("def ") {
                        func.has_recursion = true;
                    }
                    
                    // Check for exception handling
                    if trimmed.contains("try:") || trimmed.contains("except") || trimmed.contains("finally") {
                        func.has_exception_handling = true;
                    }
                }
            }
        }
        
        // Add the last function if exists
        if let Some(func) = current_function {
            functions.push(func);
        }
        
        Ok(functions)
    }
    
    /// Analyze JavaScript/TypeScript functions
    fn analyze_javascript_functions(&self, lines: &[String]) -> Result<Vec<FunctionInfo>> {
        let mut functions = Vec::new();
        let mut current_function: Option<FunctionInfo> = None;
        let mut brace_count = 0;
        let mut in_function = false;
        
        for (line_num, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            
            // Skip comments and empty lines
            if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.is_empty() {
                continue;
            }
            
            // Function declaration detection
            if self.is_javascript_function_declaration(trimmed) {
                if let Some(func_name) = self.extract_javascript_function_name(trimmed) {
                    current_function = Some(FunctionInfo {
                        name: func_name,
                        line_count: 0,
                        cyclomatic_complexity: 1, // Base complexity
                        cognitive_complexity: 1, // Base cognitive complexity
                        nesting_depth: 0,
                        parameter_count: 0,
                        return_path_count: 0,
                        start_line: line_num + 1,
                        end_line: line_num + 1,
                        is_method: false,
                        parent_class: None,
                        local_variable_count: 0,
                        has_recursion: false,
                        has_exception_handling: false,
                    });
                    in_function = true;
                    brace_count = 0;
                }
            }
            
            if in_function {
                if let Some(ref mut func) = current_function {
                    func.line_count += 1;
                    func.end_line = line_num + 1;
                    
                    // Count braces for nesting depth
                    let open_braces = trimmed.matches('{').count();
                    let close_braces = trimmed.matches('}').count();
                    brace_count += open_braces as i32 - close_braces as i32;
                    func.nesting_depth = func.nesting_depth.max(brace_count.max(0) as usize);
                    
                    // Calculate cyclomatic complexity
                    func.cyclomatic_complexity += self.count_javascript_complexity_keywords(trimmed);
                    
                    // Calculate cognitive complexity
                    func.cognitive_complexity += self.count_javascript_cognitive_complexity(trimmed, brace_count);
                    
                    // Count parameters
                    if trimmed.contains('(') && func.parameter_count == 0 {
                        func.parameter_count = self.count_function_parameters(trimmed);
                    }
                    
                    // Count return paths
                    if trimmed.contains("return") {
                        func.return_path_count += 1;
                    }
                    
                    // Check for recursion
                    if trimmed.contains(&func.name) && !trimmed.starts_with("function ") {
                        func.has_recursion = true;
                    }
                    
                    // Check for exception handling
                    if trimmed.contains("try") || trimmed.contains("catch") || trimmed.contains("throw") {
                        func.has_exception_handling = true;
                    }
                    
                    // Function end detection
                    if brace_count <= 0 && close_braces > 0 {
                        functions.push(func.clone());
                        current_function = None;
                        in_function = false;
                    }
                }
            }
        }
        
        Ok(functions)
    }
    
    /// Analyze Java functions
    fn analyze_java_functions(&self, lines: &[String]) -> Result<Vec<FunctionInfo>> {
        // Similar to JavaScript but with Java-specific patterns
        self.analyze_javascript_functions(lines) // Simplified for now
    }
    
    /// Analyze C++ functions
    fn analyze_cpp_functions(&self, lines: &[String]) -> Result<Vec<FunctionInfo>> {
        // Similar to JavaScript but with C++-specific patterns
        self.analyze_javascript_functions(lines) // Simplified for now
    }
    
    /// Analyze Go functions
    fn analyze_go_functions(&self, lines: &[String]) -> Result<Vec<FunctionInfo>> {
        // Similar to JavaScript but with Go-specific patterns
        self.analyze_javascript_functions(lines) // Simplified for now
    }
    
    /// Extract function name from Rust function declaration
    fn extract_rust_function_name(&self, line: &str) -> Option<String> {
        if let Some(start) = line.find("fn ") {
            let after_fn = &line[start + 3..];
            if let Some(end) = after_fn.find('(') {
                Some(after_fn[..end].trim().to_string())
            } else {
                None
            }
        } else {
            None
        }
    }
    
    /// Extract function name from Python function declaration
    fn extract_python_function_name(&self, line: &str) -> Option<String> {
        if let Some(start) = line.find("def ") {
            let after_def = &line[start + 4..];
            if let Some(end) = after_def.find('(') {
                Some(after_def[..end].trim().to_string())
            } else {
                None
            }
        } else {
            None
        }
    }
    
    /// Check if line is a JavaScript function declaration
    fn is_javascript_function_declaration(&self, line: &str) -> bool {
        line.contains("function ") || 
        line.contains("=> ") || 
        (line.contains("(") && line.contains(")") && line.contains("{"))
    }
    
    /// Extract function name from JavaScript function declaration
    fn extract_javascript_function_name(&self, line: &str) -> Option<String> {
        if line.contains("function ") {
            if let Some(start) = line.find("function ") {
                let after_function = &line[start + 9..];
                if let Some(end) = after_function.find('(') {
                    return Some(after_function[..end].trim().to_string());
                }
            }
        }
        
        // Handle arrow functions and method declarations
        if let Some(arrow_pos) = line.find("=>") {
            let before_arrow = &line[..arrow_pos];
            if let Some(equals_pos) = before_arrow.rfind('=') {
                let name_part = &before_arrow[..equals_pos];
                if let Some(name) = name_part.split_whitespace().last() {
                    return Some(name.to_string());
                }
            }
        }
        
        Some("anonymous".to_string())
    }
    
    /// Count complexity keywords in Rust code
    fn count_rust_complexity_keywords(&self, line: &str) -> usize {
        let keywords = ["if", "else if", "match", "while", "for", "loop", "&&", "||", "?"];
        keywords.iter().map(|&keyword| line.matches(keyword).count()).sum()
    }
    
    /// Count complexity keywords in Python code
    fn count_python_complexity_keywords(&self, line: &str) -> usize {
        let keywords = ["if", "elif", "while", "for", "and", "or", "except", "finally"];
        keywords.iter().map(|&keyword| line.matches(keyword).count()).sum()
    }
    
    /// Count complexity keywords in JavaScript code
    fn count_javascript_complexity_keywords(&self, line: &str) -> usize {
        let keywords = ["if", "else if", "while", "for", "switch", "case", "catch", "finally", "&&", "||", "?"];
        keywords.iter().map(|&keyword| line.matches(keyword).count()).sum()
    }
    
    /// Count cognitive complexity for Rust code
    fn count_rust_cognitive_complexity(&self, line: &str, nesting_level: i32) -> usize {
        let mut complexity = 0;
        let nesting_multiplier = (nesting_level as usize).max(1);
        
        // Basic control structures
        if line.contains("if") { complexity += 1 * nesting_multiplier; }
        if line.contains("else") { complexity += 1; }
        if line.contains("match") { complexity += 1 * nesting_multiplier; }
        if line.contains("while") { complexity += 1 * nesting_multiplier; }
        if line.contains("for") { complexity += 1 * nesting_multiplier; }
        if line.contains("loop") { complexity += 1 * nesting_multiplier; }
        
        // Logical operators
        complexity += line.matches("&&").count() * nesting_multiplier;
        complexity += line.matches("||").count() * nesting_multiplier;
        
        // Recursion penalty
        if line.contains("self.") && line.contains("(") { complexity += 1; }
        
        complexity
    }
    
    /// Count cognitive complexity for Python code
    fn count_python_cognitive_complexity(&self, line: &str, nesting_level: usize) -> usize {
        let mut complexity = 0;
        let nesting_multiplier = nesting_level.max(1);
        
        // Basic control structures
        if line.contains("if ") { complexity += 1 * nesting_multiplier; }
        if line.contains("elif ") { complexity += 1; }
        if line.contains("else:") { complexity += 1; }
        if line.contains("while ") { complexity += 1 * nesting_multiplier; }
        if line.contains("for ") { complexity += 1 * nesting_multiplier; }
        if line.contains("try:") { complexity += 1 * nesting_multiplier; }
        if line.contains("except") { complexity += 1; }
        if line.contains("finally") { complexity += 1; }
        
        // Logical operators
        complexity += line.matches(" and ").count() * nesting_multiplier;
        complexity += line.matches(" or ").count() * nesting_multiplier;
        
        // Comprehensions add complexity
        if line.contains(" for ") && (line.contains("[") || line.contains("{")) {
            complexity += 1;
        }
        
        complexity
    }
    
    /// Count cognitive complexity for JavaScript code
    fn count_javascript_cognitive_complexity(&self, line: &str, nesting_level: i32) -> usize {
        let mut complexity = 0;
        let nesting_multiplier = (nesting_level as usize).max(1);
        
        // Basic control structures
        if line.contains("if") { complexity += 1 * nesting_multiplier; }
        if line.contains("else") { complexity += 1; }
        if line.contains("while") { complexity += 1 * nesting_multiplier; }
        if line.contains("for") { complexity += 1 * nesting_multiplier; }
        if line.contains("switch") { complexity += 1 * nesting_multiplier; }
        if line.contains("case") { complexity += 1; }
        if line.contains("catch") { complexity += 1; }
        if line.contains("finally") { complexity += 1; }
        
        // Logical operators
        complexity += line.matches("&&").count() * nesting_multiplier;
        complexity += line.matches("||").count() * nesting_multiplier;
        
        // Ternary operator
        complexity += line.matches("?").count() * nesting_multiplier;
        
        // Recursion penalty
        if line.contains("(") && line.contains(")") { 
            // Simple heuristic for recursive calls
            complexity += line.matches("()").count().min(1);
        }
        
        complexity
    }
    
    /// Count function parameters
    fn count_function_parameters(&self, line: &str) -> usize {
        if let Some(start) = line.find('(') {
            if let Some(end) = line.find(')') {
                let params_str = &line[start + 1..end];
                if params_str.trim().is_empty() {
                    return 0;
                }
                
                // Simple parameter counting (split by comma)
                let param_count = params_str.split(',').count();
                
                // Adjust for common patterns
                if params_str.contains("self") || params_str.contains("this") {
                    return param_count.saturating_sub(1);
                }
                
                return param_count;
            }
        }
        0
    }
    
    /// Calculate complexity distribution
    fn calculate_complexity_distribution(&self, functions: &[FunctionInfo]) -> ComplexityDistribution {
        let mut distribution = ComplexityDistribution {
            very_low_complexity: 0,
            low_complexity: 0,
            medium_complexity: 0,
            high_complexity: 0,
            very_high_complexity: 0,
        };
        
        for func in functions {
            match func.cyclomatic_complexity {
                1..=5 => distribution.very_low_complexity += 1,
                6..=10 => distribution.low_complexity += 1,
                11..=20 => distribution.medium_complexity += 1,
                21..=50 => distribution.high_complexity += 1,
                _ => distribution.very_high_complexity += 1,
            }
        }
        
        distribution
    }
    
    /// Get complexity level as string
    pub fn get_complexity_level(&self, complexity: f64) -> String {
        match complexity as usize {
            1..=5 => "Very Low".to_string(),
            6..=10 => "Low".to_string(),
            11..=20 => "Medium".to_string(),
            21..=50 => "High".to_string(),
            _ => "Very High".to_string(),
        }
    }
    
    /// Get complexity level CSS class
    pub fn get_complexity_class(&self, complexity: f64) -> String {
        match complexity as usize {
            1..=5 => "complexity-very-low".to_string(),
            6..=10 => "complexity-low".to_string(),
            11..=20 => "complexity-medium".to_string(),
            21..=50 => "complexity-high".to_string(),
            _ => "complexity-very-high".to_string(),
        }
    }
    
    /// Calculate structure distribution
    fn calculate_structure_distribution(&self, structures: &[StructureInfo]) -> StructureDistribution {
        StructureDistribution {
            classes: structures.iter().filter(|s| s.structure_type == StructureType::Class).count(),
            interfaces: structures.iter().filter(|s| s.structure_type == StructureType::Interface).count(),
            traits: structures.iter().filter(|s| s.structure_type == StructureType::Trait).count(),
            enums: structures.iter().filter(|s| s.structure_type == StructureType::Enum).count(),
            structs: structures.iter().filter(|s| s.structure_type == StructureType::Struct).count(),
            modules: structures.iter().filter(|s| s.structure_type == StructureType::Module || s.structure_type == StructureType::Namespace).count(),
        }
    }
    
    /// Analyze Rust structures
    fn analyze_rust_structures(&self, lines: &[String]) -> Result<Vec<StructureInfo>> {
        let mut structures = Vec::new();
        let mut current_structure: Option<StructureInfo> = None;
        let mut brace_count = 0;
        let mut in_structure = false;
        
        for (line_num, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            
            // Skip comments and empty lines
            if trimmed.starts_with("//") || trimmed.is_empty() {
                continue;
            }
            
            // Structure declaration detection
            if let Some((structure_type, name, visibility)) = self.detect_rust_structure(trimmed) {
                current_structure = Some(StructureInfo {
                    name,
                    structure_type,
                    line_count: 0,
                    start_line: line_num + 1,
                    end_line: line_num + 1,
                    methods: Vec::new(),
                    properties: 0,
                    visibility,
                    inheritance_depth: 0,
                    interface_count: 0,
                });
                in_structure = true;
                brace_count = 0;
            }
            
            if in_structure {
                if let Some(ref mut structure) = current_structure {
                    structure.line_count += 1;
                    structure.end_line = line_num + 1;
                    
                    // Count braces for structure end detection
                    let open_braces = trimmed.matches('{').count();
                    let close_braces = trimmed.matches('}').count();
                    brace_count += open_braces as i32 - close_braces as i32;
                    
                    // Count properties (field declarations)
                    if trimmed.contains(':') && !trimmed.contains("fn ") && !trimmed.contains("//") {
                        structure.properties += 1;
                    }
                    
                    // Structure end detection
                    if brace_count <= 0 && close_braces > 0 {
                        structures.push(structure.clone());
                        current_structure = None;
                        in_structure = false;
                    }
                }
            }
        }
        
        Ok(structures)
    }
    
    /// Detect Rust structure type and name
    fn detect_rust_structure(&self, line: &str) -> Option<(StructureType, String, Visibility)> {
        let visibility = if line.starts_with("pub ") {
            Visibility::Public
        } else {
            Visibility::Private
        };
        
        if line.contains("struct ") {
            if let Some(name) = self.extract_structure_name(line, "struct ") {
                return Some((StructureType::Struct, name, visibility));
            }
        }
        
        if line.contains("enum ") {
            if let Some(name) = self.extract_structure_name(line, "enum ") {
                return Some((StructureType::Enum, name, visibility));
            }
        }
        
        if line.contains("trait ") {
            if let Some(name) = self.extract_structure_name(line, "trait ") {
                return Some((StructureType::Trait, name, visibility));
            }
        }
        
        if line.contains("impl ") {
            if let Some(name) = self.extract_structure_name(line, "impl ") {
                return Some((StructureType::Class, name, visibility)); // Treat impl as class-like
            }
        }
        
        if line.contains("mod ") {
            if let Some(name) = self.extract_structure_name(line, "mod ") {
                return Some((StructureType::Module, name, visibility));
            }
        }
        
        None
    }
    
    /// Extract structure name from declaration
    fn extract_structure_name(&self, line: &str, keyword: &str) -> Option<String> {
        if let Some(start) = line.find(keyword) {
            let after_keyword = &line[start + keyword.len()..];
            let name_part = after_keyword.split_whitespace().next()?;
            let name = name_part.split('<').next()?.split('{').next()?.trim();
            if !name.is_empty() {
                Some(name.to_string())
            } else {
                None
            }
        } else {
            None
        }
    }
    
    /// Analyze Python structures
    fn analyze_python_structures(&self, lines: &[String]) -> Result<Vec<StructureInfo>> {
        let mut structures = Vec::new();
        let mut current_structure: Option<StructureInfo> = None;
        let mut structure_indent = 0;
        
        for (line_num, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            
            // Skip comments and empty lines
            if trimmed.starts_with("#") || trimmed.is_empty() {
                continue;
            }
            
            // Calculate indentation
            let current_indent = line.len() - line.trim_start().len();
            
            // Structure declaration detection
            if let Some((structure_type, name)) = self.detect_python_structure(trimmed) {
                // Save previous structure if exists
                if let Some(structure) = current_structure.take() {
                    structures.push(structure);
                }
                
                current_structure = Some(StructureInfo {
                    name,
                    structure_type,
                    line_count: 0,
                    start_line: line_num + 1,
                    end_line: line_num + 1,
                    methods: Vec::new(),
                    properties: 0,
                    visibility: Visibility::Public, // Python doesn't have strict visibility
                    inheritance_depth: 0,
                    interface_count: 0,
                });
                structure_indent = current_indent;
            }
            
            if let Some(ref mut structure) = current_structure {
                // Check if we're still in the structure
                if current_indent <= structure_indent && line_num > structure.start_line - 1 && !trimmed.is_empty() {
                    // Structure ended
                    structures.push(structure.clone());
                    current_structure = None;
                    continue;
                }
                
                if current_indent > structure_indent {
                    structure.line_count += 1;
                    structure.end_line = line_num + 1;
                    
                    // Count properties (self.property assignments)
                    if trimmed.starts_with("self.") && trimmed.contains('=') {
                        structure.properties += 1;
                    }
                }
            }
        }
        
        // Add the last structure if exists
        if let Some(structure) = current_structure {
            structures.push(structure);
        }
        
        Ok(structures)
    }
    
    /// Detect Python structure type and name
    fn detect_python_structure(&self, line: &str) -> Option<(StructureType, String)> {
        if line.starts_with("class ") {
            if let Some(name) = self.extract_python_class_name(line) {
                return Some((StructureType::Class, name));
            }
        }
        
        // Python doesn't have interfaces, but we can detect ABC classes
        if line.contains("ABC") && line.starts_with("class ") {
            if let Some(name) = self.extract_python_class_name(line) {
                return Some((StructureType::Interface, name));
            }
        }
        
        None
    }
    
    /// Extract Python class name
    fn extract_python_class_name(&self, line: &str) -> Option<String> {
        if let Some(start) = line.find("class ") {
            let after_class = &line[start + 6..];
            let name_part = after_class.split('(').next()?.split(':').next()?.trim();
            if !name_part.is_empty() {
                Some(name_part.to_string())
            } else {
                None
            }
        } else {
            None
        }
    }
    
    /// Analyze JavaScript/TypeScript structures
    fn analyze_javascript_structures(&self, lines: &[String]) -> Result<Vec<StructureInfo>> {
        let mut structures = Vec::new();
        let mut current_structure: Option<StructureInfo> = None;
        let mut brace_count = 0;
        let mut in_structure = false;
        
        for (line_num, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            
            // Skip comments and empty lines
            if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.is_empty() {
                continue;
            }
            
            // Structure declaration detection
            if let Some((structure_type, name, visibility)) = self.detect_javascript_structure(trimmed) {
                current_structure = Some(StructureInfo {
                    name,
                    structure_type,
                    line_count: 0,
                    start_line: line_num + 1,
                    end_line: line_num + 1,
                    methods: Vec::new(),
                    properties: 0,
                    visibility,
                    inheritance_depth: 0,
                    interface_count: 0,
                });
                in_structure = true;
                brace_count = 0;
            }
            
            if in_structure {
                if let Some(ref mut structure) = current_structure {
                    structure.line_count += 1;
                    structure.end_line = line_num + 1;
                    
                    // Count braces for structure end detection
                    let open_braces = trimmed.matches('{').count();
                    let close_braces = trimmed.matches('}').count();
                    brace_count += open_braces as i32 - close_braces as i32;
                    
                    // Count properties (this.property or property:)
                    if (trimmed.contains("this.") && trimmed.contains('=')) || 
                       (trimmed.contains(':') && !trimmed.contains("function") && !trimmed.contains("=>")) {
                        structure.properties += 1;
                    }
                    
                    // Structure end detection
                    if brace_count <= 0 && close_braces > 0 {
                        structures.push(structure.clone());
                        current_structure = None;
                        in_structure = false;
                    }
                }
            }
        }
        
        Ok(structures)
    }
    
    /// Detect JavaScript/TypeScript structure type and name
    fn detect_javascript_structure(&self, line: &str) -> Option<(StructureType, String, Visibility)> {
        let visibility = if line.contains("export ") {
            Visibility::Public
        } else {
            Visibility::Private
        };
        
        if line.contains("class ") {
            if let Some(name) = self.extract_javascript_class_name(line) {
                return Some((StructureType::Class, name, visibility));
            }
        }
        
        if line.contains("interface ") {
            if let Some(name) = self.extract_structure_name(line, "interface ") {
                return Some((StructureType::Interface, name, visibility));
            }
        }
        
        if line.contains("enum ") {
            if let Some(name) = self.extract_structure_name(line, "enum ") {
                return Some((StructureType::Enum, name, visibility));
            }
        }
        
        None
    }
    
    /// Extract JavaScript/TypeScript class name
    fn extract_javascript_class_name(&self, line: &str) -> Option<String> {
        if let Some(start) = line.find("class ") {
            let after_class = &line[start + 6..];
            let name_part = after_class.split_whitespace().next()?;
            let name = name_part.split('{').next()?.split('(').next()?.trim();
            if !name.is_empty() {
                Some(name.to_string())
            } else {
                None
            }
        } else {
            None
        }
    }
    
    // Placeholder implementations for other languages
    fn analyze_java_structures(&self, _lines: &[String]) -> Result<Vec<StructureInfo>> {
        Ok(Vec::new()) // TODO: Implement Java structure analysis
    }
    
    fn analyze_cpp_structures(&self, _lines: &[String]) -> Result<Vec<StructureInfo>> {
        Ok(Vec::new()) // TODO: Implement C++ structure analysis
    }
    
    fn analyze_go_structures(&self, _lines: &[String]) -> Result<Vec<StructureInfo>> {
        Ok(Vec::new()) // TODO: Implement Go structure analysis
    }
    
    fn analyze_csharp_structures(&self, _lines: &[String]) -> Result<Vec<StructureInfo>> {
        Ok(Vec::new()) // TODO: Implement C# structure analysis
    }
    
    fn analyze_php_structures(&self, _lines: &[String]) -> Result<Vec<StructureInfo>> {
        Ok(Vec::new()) // TODO: Implement PHP structure analysis
    }
    
    fn analyze_ruby_structures(&self, _lines: &[String]) -> Result<Vec<StructureInfo>> {
        Ok(Vec::new()) // TODO: Implement Ruby structure analysis
    }
    
    fn analyze_swift_structures(&self, _lines: &[String]) -> Result<Vec<StructureInfo>> {
        Ok(Vec::new()) // TODO: Implement Swift structure analysis
    }
    
    fn analyze_kotlin_structures(&self, _lines: &[String]) -> Result<Vec<StructureInfo>> {
        Ok(Vec::new()) // TODO: Implement Kotlin structure analysis
    }

    /// Calculate maintainability index (simplified version)
    fn calculate_maintainability_index(&self, functions: &[FunctionInfo], file_stats: &FileStats) -> f64 {
        if functions.is_empty() {
            return 100.0; // Perfect score for empty files
        }

        let mut total_score = 0.0;
        
        for func in functions {
            // Simplified maintainability calculation based on:
            // - Function length (shorter is better)
            // - Cyclomatic complexity (lower is better)
            // - Cognitive complexity (lower is better)
            // - Parameter count (fewer is better)
            
            let length_score = (50.0 - func.line_count as f64).max(0.0);
            let cyclomatic_score = (30.0 - func.cyclomatic_complexity as f64 * 2.0).max(0.0);
            let cognitive_score = (30.0 - func.cognitive_complexity as f64 * 2.0).max(0.0);
            let param_score = (20.0 - func.parameter_count as f64 * 3.0).max(0.0);
            
            total_score += length_score + cyclomatic_score + cognitive_score + param_score;
        }
        
        (total_score / functions.len() as f64).min(100.0).max(0.0)
    }
    
    /// Create detailed complexity information for functions
    fn create_function_complexity_details(&self, functions: &[FunctionInfo], file_path: &str) -> Vec<FunctionComplexityDetail> {
        functions.iter().map(|func| {
            let complexity_level = self.classify_complexity_level(func.cyclomatic_complexity);
            let maintainability_concerns = self.identify_maintainability_concerns(func);
            
            FunctionComplexityDetail {
                name: func.name.clone(),
                file_path: file_path.to_string(),
                start_line: func.start_line,
                end_line: func.end_line,
                line_count: func.line_count,
                cyclomatic_complexity: func.cyclomatic_complexity,
                cognitive_complexity: func.cognitive_complexity,
                parameter_count: func.parameter_count,
                return_path_count: func.return_path_count,
                nesting_depth: func.nesting_depth,
                is_method: func.is_method,
                parent_class: func.parent_class.clone(),
                local_variable_count: 0, // Placeholder, needs actual analysis
                has_recursion: func.has_recursion,
                has_exception_handling: func.has_exception_handling,
                complexity_level,
                maintainability_concerns,
            }
        }).collect()
    }
    
    /// Classify complexity level based on cyclomatic complexity
    fn classify_complexity_level(&self, complexity: usize) -> ComplexityLevel {
        match complexity {
            1..=5 => ComplexityLevel::VeryLow,
            6..=10 => ComplexityLevel::Low,
            11..=20 => ComplexityLevel::Medium,
            21..=50 => ComplexityLevel::High,
            _ => ComplexityLevel::VeryHigh,
        }
    }
    
    /// Identify maintainability concerns for a function
    fn identify_maintainability_concerns(&self, func: &FunctionInfo) -> Vec<String> {
        let mut concerns = Vec::new();
        
        if func.line_count > 50 {
            concerns.push("Function is too long (>50 lines)".to_string());
        }
        
        if func.cyclomatic_complexity > 10 {
            concerns.push("High cyclomatic complexity".to_string());
        }
        
        if func.cognitive_complexity > 15 {
            concerns.push("High cognitive complexity".to_string());
        }
        
        if func.parameter_count > 5 {
            concerns.push("Too many parameters".to_string());
        }
        
        if func.nesting_depth > 4 {
            concerns.push("Deep nesting detected".to_string());
        }
        
        if func.has_recursion {
            concerns.push("Contains recursion".to_string());
        }
        
        if func.return_path_count > 5 {
            concerns.push("Multiple return paths".to_string());
        }
        
        concerns
    }
    
    /// Calculate quality metrics for the codebase
    fn calculate_quality_metrics(&self, functions: &[FunctionInfo], file_stats: &FileStats, structures: &[StructureInfo]) -> QualityMetrics {
        let overall_quality_score = self.calculate_overall_quality_score(functions, file_stats);
        let maintainability_score = self.calculate_maintainability_score(functions);
        let readability_score = self.calculate_readability_score(functions, file_stats);
        let testability_score = self.calculate_testability_score(functions);
        let code_duplication_ratio = self.estimate_code_duplication(file_stats);
        let comment_coverage_ratio = self.calculate_comment_coverage(file_stats);
        let function_size_score = self.calculate_function_size_score(functions);
        let complexity_score = self.calculate_complexity_score(functions);
        
        QualityMetrics {
            overall_quality_score,
            maintainability_score,
            readability_score,
            testability_score,
            code_duplication_ratio,
            comment_coverage_ratio,
            function_size_score,
            complexity_score,
        }
    }
    
    /// Calculate overall quality score
    fn calculate_overall_quality_score(&self, functions: &[FunctionInfo], file_stats: &FileStats) -> f64 {
        let maintainability = self.calculate_maintainability_score(functions);
        let readability = self.calculate_readability_score(functions, file_stats);
        let testability = self.calculate_testability_score(functions);
        let complexity = self.calculate_complexity_score(functions);
        
        // Weighted average of different quality aspects
        (maintainability * 0.3 + readability * 0.25 + testability * 0.25 + complexity * 0.2).min(100.0).max(0.0)
    }
    
    /// Calculate maintainability score
    fn calculate_maintainability_score(&self, functions: &[FunctionInfo]) -> f64 {
        if functions.is_empty() {
            return 100.0;
        }
        
        let mut total_score = 0.0;
        
        for func in functions {
            let mut func_score = 100.0;
            
            // Penalize based on function length
            if func.line_count > 20 {
                func_score -= (func.line_count - 20) as f64 * 1.5;
            }
            
            // Penalize based on complexity
            func_score -= func.cyclomatic_complexity as f64 * 3.0;
            func_score -= func.cognitive_complexity as f64 * 2.0;
            
            // Penalize based on parameters
            if func.parameter_count > 3 {
                func_score -= (func.parameter_count - 3) as f64 * 5.0;
            }
            
            total_score += func_score.max(0.0);
        }
        
        (total_score / functions.len() as f64).min(100.0).max(0.0)
    }
    
    /// Calculate readability score
    fn calculate_readability_score(&self, functions: &[FunctionInfo], file_stats: &FileStats) -> f64 {
        let mut score = 100.0;
        
        // Comment ratio (higher is better)
        let comment_ratio = file_stats.comment_lines as f64 / file_stats.total_lines.max(1) as f64;
        score += comment_ratio * 20.0;
        
        // Average function length (shorter is better)
        if !functions.is_empty() {
            let avg_length = functions.iter().map(|f| f.line_count).sum::<usize>() as f64 / functions.len() as f64;
            if avg_length > 20.0 {
                score -= (avg_length - 20.0) * 2.0;
            }
        }
        
        // Nesting depth (lower is better)
        for func in functions {
            if func.nesting_depth > 3 {
                score -= (func.nesting_depth - 3) as f64 * 5.0;
            }
        }
        
        score.min(100.0).max(0.0)
    }
    
    /// Calculate testability score
    fn calculate_testability_score(&self, functions: &[FunctionInfo]) -> f64 {
        if functions.is_empty() {
            return 100.0;
        }
        
        let mut total_score = 0.0;
        
        for func in functions {
            let mut func_score = 100.0;
            
            // Functions with fewer parameters are more testable
            if func.parameter_count > 4 {
                func_score -= (func.parameter_count - 4) as f64 * 10.0;
            }
            
            // Lower complexity is more testable
            func_score -= func.cyclomatic_complexity as f64 * 4.0;
            
            // Functions with fewer return paths are more testable
            if func.return_path_count > 3 {
                func_score -= (func.return_path_count - 3) as f64 * 8.0;
            }
            
            // Exception handling makes testing more complex
            if func.has_exception_handling {
                func_score -= 10.0;
            }
            
            total_score += func_score.max(0.0);
        }
        
        (total_score / functions.len() as f64).min(100.0).max(0.0)
    }
    
    /// Estimate code duplication ratio
    fn estimate_code_duplication(&self, file_stats: &FileStats) -> f64 {
        // Simplified estimation based on file size and structure
        // In a real implementation, this would involve more sophisticated analysis
        let ratio = if file_stats.total_lines > 1000 {
            0.15 // Assume 15% duplication in large files
        } else if file_stats.total_lines > 500 {
            0.10 // Assume 10% duplication in medium files
        } else {
            0.05 // Assume 5% duplication in small files
        };
        
        ratio * 100.0 // Return as percentage
    }
    
    /// Calculate comment coverage ratio
    fn calculate_comment_coverage(&self, file_stats: &FileStats) -> f64 {
        let total_non_blank = file_stats.total_lines - file_stats.blank_lines;
        if total_non_blank == 0 {
            return 0.0;
        }
        
        ((file_stats.comment_lines + file_stats.doc_lines) as f64 / total_non_blank as f64) * 100.0
    }
    
    /// Calculate function size score
    fn calculate_function_size_score(&self, functions: &[FunctionInfo]) -> f64 {
        if functions.is_empty() {
            return 100.0;
        }
        
        let mut score = 100.0;
        let avg_size = functions.iter().map(|f| f.line_count).sum::<usize>() as f64 / functions.len() as f64;
        
        // Ideal function size is 10-20 lines
        if avg_size > 20.0 {
            score -= (avg_size - 20.0) * 2.0;
        } else if avg_size < 5.0 {
            score -= (5.0 - avg_size) * 3.0;
        }
        
        // Penalize very large functions
        for func in functions {
            if func.line_count > 100 {
                score -= 10.0;
            } else if func.line_count > 50 {
                score -= 5.0;
            }
        }
        
        score.min(100.0).max(0.0)
    }
    
    /// Calculate complexity score
    fn calculate_complexity_score(&self, functions: &[FunctionInfo]) -> f64 {
        if functions.is_empty() {
            return 100.0;
        }
        
        let mut score = 100.0;
        let avg_cyclomatic = functions.iter().map(|f| f.cyclomatic_complexity).sum::<usize>() as f64 / functions.len() as f64;
        let avg_cognitive = functions.iter().map(|f| f.cognitive_complexity).sum::<usize>() as f64 / functions.len() as f64;
        
        // Penalize high complexity
        score -= avg_cyclomatic * 3.0;
        score -= avg_cognitive * 2.0;
        
        // Extra penalty for very complex functions
        for func in functions {
            if func.cyclomatic_complexity > 20 {
                score -= 15.0;
            } else if func.cyclomatic_complexity > 10 {
                score -= 5.0;
            }
        }
        
        score.min(100.0).max(0.0)
    }
    
    /// Calculate quality metrics for the entire project
    fn calculate_project_quality_metrics(&self, functions: &[FunctionInfo], code_stats: &CodeStats, structures: &[StructureInfo]) -> QualityMetrics {
        // Create a synthetic FileStats for project-level calculations
        let project_file_stats = FileStats {
            total_lines: code_stats.total_lines,
            code_lines: code_stats.total_code_lines,
            comment_lines: code_stats.total_comment_lines,
            doc_lines: code_stats.total_doc_lines,
            blank_lines: code_stats.total_blank_lines,
            file_size: code_stats.total_size,
        };
        
        let overall_quality_score = self.calculate_overall_quality_score(functions, &project_file_stats);
        let maintainability_score = self.calculate_maintainability_score(functions);
        let readability_score = self.calculate_readability_score(functions, &project_file_stats);
        let testability_score = self.calculate_testability_score(functions);
        let code_duplication_ratio = self.estimate_project_code_duplication(code_stats);
        let comment_coverage_ratio = self.calculate_comment_coverage(&project_file_stats);
        let function_size_score = self.calculate_function_size_score(functions);
        let complexity_score = self.calculate_complexity_score(functions);
        
        QualityMetrics {
            overall_quality_score,
            maintainability_score,
            readability_score,
            testability_score,
            code_duplication_ratio,
            comment_coverage_ratio,
            function_size_score,
            complexity_score,
        }
    }
    
    /// Estimate code duplication for the entire project
    fn estimate_project_code_duplication(&self, code_stats: &CodeStats) -> f64 {
        let total_lines = code_stats.total_lines;
        let ratio = if total_lines > 10000 {
            0.20 // Assume 20% duplication in very large projects
        } else if total_lines > 5000 {
            0.15 // Assume 15% duplication in large projects
        } else if total_lines > 1000 {
            0.10 // Assume 10% duplication in medium projects
        } else {
            0.05 // Assume 5% duplication in small projects
        };
        
        ratio * 100.0 // Return as percentage
    }
}

impl Default for ComplexityStatsCalculator {
    fn default() -> Self {
        Self::new()
    }
} 