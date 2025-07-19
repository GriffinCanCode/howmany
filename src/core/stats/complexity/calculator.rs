use crate::core::types::{CodeStats, FileStats};
use crate::utils::errors::Result;
use super::types::{ComplexityStats, ComplexityDistribution, StructureDistribution, ExtensionComplexity, FunctionInfo, StructureInfo, StructureType};
use super::analyzer::CodeAnalyzer;
use super::quality::QualityCalculator;
use std::collections::HashMap;
use std::path::Path;

/// Main complexity statistics calculator
pub struct ComplexityCalculator {
    analyzer: CodeAnalyzer,
    quality_calculator: QualityCalculator,
}

impl ComplexityCalculator {
    pub fn new() -> Self {
        Self {
            analyzer: CodeAnalyzer::new(),
            quality_calculator: QualityCalculator::new(),
        }
    }

    /// Calculate complexity statistics for a single file
    pub fn calculate_complexity_stats(&self, file_stats: &FileStats, file_path: &str) -> Result<ComplexityStats> {
        let functions = self.analyzer.analyze_file_functions(file_path)?;
        let structures = self.analyzer.analyze_file_structures(file_path)?;
        
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
        
        let function_complexity_details = self.quality_calculator.create_function_complexity_details(&functions, file_path);
        let quality_metrics = self.quality_calculator.calculate_quality_metrics(&functions, file_stats, &structures);
        
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
            if let Ok(functions) = self.analyzer.analyze_file_functions(file_path) {
                all_functions.extend(functions.clone());
            }
            
            if let Ok(structures) = self.analyzer.analyze_file_structures(file_path) {
                all_structures.extend(structures.clone());
                
                total_classes += structures.iter().filter(|s| s.structure_type == StructureType::Class).count();
                total_interfaces += structures.iter().filter(|s| s.structure_type == StructureType::Interface).count();
                total_traits += structures.iter().filter(|s| s.structure_type == StructureType::Trait).count();
                total_enums += structures.iter().filter(|s| s.structure_type == StructureType::Enum).count();
                total_structs += structures.iter().filter(|s| s.structure_type == StructureType::Struct).count();
                total_modules += structures.iter().filter(|s| s.structure_type == StructureType::Module || s.structure_type == StructureType::Namespace).count();
            }
            
            if let Ok(functions) = self.analyzer.analyze_file_functions(file_path) {
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
        let total_functions = all_functions.len();
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
        let quality_metrics = self.quality_calculator.calculate_project_quality_metrics(&all_functions, code_stats, &all_structures);
        
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

    /// Calculate maintainability index (simplified version)
    fn calculate_maintainability_index(&self, functions: &[FunctionInfo], _file_stats: &FileStats) -> f64 {
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

    /// Get complexity level description
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
}

impl Default for ComplexityCalculator {
    fn default() -> Self {
        Self::new()
    }
} 