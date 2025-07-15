use crate::core::types::{CodeStats, FileStats};
use super::types::{QualityMetrics, FunctionInfo, StructureInfo, ComplexityLevel, FunctionComplexityDetail};

/// Quality metrics calculator
pub struct QualityCalculator;

impl QualityCalculator {
    pub fn new() -> Self {
        Self
    }

    /// Calculate code health metrics for practical developer insights
    pub fn calculate_quality_metrics(&self, functions: &[FunctionInfo], file_stats: &FileStats, _structures: &[StructureInfo]) -> QualityMetrics {
        let code_health_score = self.calculate_code_health_score(functions, file_stats);
        let maintainability_index = self.calculate_maintainability_index(functions, file_stats);
        let documentation_coverage = self.calculate_documentation_coverage(file_stats);
        let avg_complexity = self.calculate_average_complexity(functions);
        let function_size_health = self.calculate_function_size_health(functions, file_stats);
        let nesting_depth_health = self.calculate_nesting_depth_health(functions, file_stats);
        let code_duplication_ratio = self.estimate_code_duplication(file_stats);
        let technical_debt_ratio = self.calculate_technical_debt_ratio(functions, file_stats);
        
        QualityMetrics {
            code_health_score,
            maintainability_index,
            documentation_coverage,
            avg_complexity,
            function_size_health,
            nesting_depth_health,
            code_duplication_ratio,
            technical_debt_ratio,
        }
    }
    
    /// Calculate overall code health score based on practical metrics
    fn calculate_code_health_score(&self, functions: &[FunctionInfo], file_stats: &FileStats) -> f64 {
        let maintainability = self.calculate_maintainability_index(functions, file_stats);
        let documentation = self.calculate_documentation_coverage(file_stats);
        let complexity = 100.0 - (self.calculate_average_complexity(functions) * 10.0).min(100.0); // Invert complexity for score
        let function_size = self.calculate_function_size_health(functions, file_stats);
        let nesting_depth = self.calculate_nesting_depth_health(functions, file_stats);
        
        // Weighted average focusing on maintainability and complexity
        (maintainability * 0.3 + documentation * 0.2 + complexity * 0.25 + function_size * 0.15 + nesting_depth * 0.1).min(100.0).max(0.0)
    }
    
    /// Calculate industry-standard maintainability index
    fn calculate_maintainability_index(&self, functions: &[FunctionInfo], file_stats: &FileStats) -> f64 {
        // If no functions detected, estimate based on file characteristics
        if functions.is_empty() {
            let mut score = 85.0; // Start with good baseline
            
            // Penalize very large files
            if file_stats.total_lines > 1000 {
                score -= ((file_stats.total_lines - 1000) as f64 / 100.0).min(30.0);
            }
            
            // Reward good documentation
            let doc_ratio = (file_stats.comment_lines + file_stats.doc_lines) as f64 / file_stats.code_lines.max(1) as f64;
            if doc_ratio > 0.2 {
                score += 10.0;
            } else if doc_ratio < 0.05 {
                score -= 15.0;
            }
            
            // Penalize files with very little code (likely config files)
            if file_stats.code_lines < 10 {
                score -= 20.0;
            }
            
            return score.min(100.0).max(0.0);
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
        // More realistic estimation based on file characteristics
        let mut duplication_score: f64 = 0.0;
        
        // Base duplication estimate based on file size
        if file_stats.total_lines > 2000 {
            duplication_score += 12.0; // Large files tend to have more duplication
        } else if file_stats.total_lines > 1000 {
            duplication_score += 8.0;
        } else if file_stats.total_lines > 500 {
            duplication_score += 5.0;
        } else {
            duplication_score += 2.0; // Small files have minimal duplication
        }
        
        // Adjust based on code density
        let code_density = file_stats.code_lines as f64 / file_stats.total_lines.max(1) as f64;
        if code_density > 0.8 {
            duplication_score += 3.0; // Dense code files may have more duplication
        }
        
        // Adjust based on comment ratio (well-documented code tends to have less duplication)
        let comment_ratio = (file_stats.comment_lines + file_stats.doc_lines) as f64 / file_stats.code_lines.max(1) as f64;
        if comment_ratio > 0.2 {
            duplication_score -= 2.0;
        } else if comment_ratio < 0.05 {
            duplication_score += 2.0;
        }
        
        duplication_score.min(25.0).max(0.0) // Cap at 25% max duplication
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
    
    /// Calculate code health metrics for the entire project
    pub fn calculate_project_quality_metrics(&self, functions: &[FunctionInfo], code_stats: &CodeStats, _structures: &[StructureInfo]) -> QualityMetrics {
        // Create a synthetic FileStats for project-level calculations
        let project_file_stats = FileStats {
            total_lines: code_stats.total_lines,
            code_lines: code_stats.total_code_lines,
            comment_lines: code_stats.total_comment_lines,
            doc_lines: code_stats.total_doc_lines,
            blank_lines: code_stats.total_blank_lines,
            file_size: code_stats.total_size,
        };
        
        let code_health_score = self.calculate_code_health_score(functions, &project_file_stats);
        let maintainability_index = self.calculate_maintainability_index(functions, &project_file_stats);
        let documentation_coverage = self.calculate_documentation_coverage(&project_file_stats);
        let avg_complexity = self.calculate_average_complexity(functions);
        let function_size_health = self.calculate_function_size_health(functions, &project_file_stats);
        let nesting_depth_health = self.calculate_nesting_depth_health(functions, &project_file_stats);
        let code_duplication_ratio = self.estimate_project_code_duplication(code_stats);
        let technical_debt_ratio = self.calculate_technical_debt_ratio(functions, &project_file_stats);
        
        QualityMetrics {
            code_health_score,
            maintainability_index,
            documentation_coverage,
            avg_complexity,
            function_size_health,
            nesting_depth_health,
            code_duplication_ratio,
            technical_debt_ratio,
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

    /// Classify complexity level based on cyclomatic complexity
    pub fn classify_complexity_level(&self, complexity: usize) -> ComplexityLevel {
        match complexity {
            1..=5 => ComplexityLevel::VeryLow,
            6..=10 => ComplexityLevel::Low,
            11..=20 => ComplexityLevel::Medium,
            21..=50 => ComplexityLevel::High,
            _ => ComplexityLevel::VeryHigh,
        }
    }
    
    /// Identify maintainability concerns for a function
    pub fn identify_maintainability_concerns(&self, func: &FunctionInfo) -> Vec<String> {
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

    /// Create detailed complexity information for functions
    pub fn create_function_complexity_details(&self, functions: &[FunctionInfo], file_path: &str) -> Vec<FunctionComplexityDetail> {
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
    
    /// Calculate documentation coverage percentage
    fn calculate_documentation_coverage(&self, file_stats: &FileStats) -> f64 {
        if file_stats.code_lines == 0 {
            return 0.0;
        }
        
        let documentation_lines = file_stats.comment_lines + file_stats.doc_lines;
        let coverage = (documentation_lines as f64 / file_stats.code_lines as f64) * 100.0;
        
        // More realistic documentation coverage scoring
        // 20% documentation coverage = 100 score (excellent)
        // 10% documentation coverage = 50 score (good)
        // 5% documentation coverage = 25 score (poor)
        // 0% documentation coverage = 0 score (very poor)
        (coverage * 5.0).min(100.0)
    }
    
    /// Calculate average cyclomatic complexity
    fn calculate_average_complexity(&self, functions: &[FunctionInfo]) -> f64 {
        if functions.is_empty() {
            return 0.0;
        }
        
        let total_complexity: usize = functions.iter().map(|f| f.cyclomatic_complexity).sum();
        total_complexity as f64 / functions.len() as f64
    }
    
    /// Calculate function size health score
    fn calculate_function_size_health(&self, functions: &[FunctionInfo], file_stats: &FileStats) -> f64 {
        // If no functions detected, estimate based on file characteristics
        if functions.is_empty() {
            let mut score = 75.0; // Start with decent baseline
            
            // Estimate based on lines of code per "logical unit"
            let avg_lines_per_unit = file_stats.code_lines as f64 / (file_stats.code_lines / 20).max(1) as f64;
            
            if avg_lines_per_unit > 50.0 {
                score -= (avg_lines_per_unit - 50.0) * 0.5;
            } else if avg_lines_per_unit < 5.0 {
                score -= (5.0 - avg_lines_per_unit) * 2.0;
            }
            
            // Penalize very large files
            if file_stats.total_lines > 500 {
                score -= ((file_stats.total_lines - 500) as f64 / 100.0).min(25.0);
            }
            
            return score.min(100.0).max(0.0);
        }
        
        let mut score = 100.0;
        let avg_length = functions.iter().map(|f| f.line_count).sum::<usize>() as f64 / functions.len() as f64;
        
        // Penalty for functions that are too long
        if avg_length > 20.0 {
            score -= (avg_length - 20.0) * 2.0;
        }
        
        // Additional penalty for any extremely long functions
        for func in functions {
            if func.line_count > 100 {
                score -= 10.0;
            } else if func.line_count > 50 {
                score -= 5.0;
            }
        }
        
        score.min(100.0).max(0.0)
    }
    
    /// Calculate nesting depth health score
    fn calculate_nesting_depth_health(&self, functions: &[FunctionInfo], file_stats: &FileStats) -> f64 {
        // If no functions detected, estimate based on file characteristics
        if functions.is_empty() {
            let mut score: f64 = 80.0; // Start with good baseline
            
            // Estimate nesting based on brace density
            let brace_density = file_stats.code_lines as f64 / (file_stats.total_lines.max(1) as f64);
            
            if brace_density > 0.8 {
                score -= 20.0; // Likely highly nested
            } else if brace_density > 0.6 {
                score -= 10.0; // Moderately nested
            }
            
            // Very large files tend to have more nesting
            if file_stats.total_lines > 1000 {
                score -= 15.0;
            }
            
            return score.min(100.0).max(0.0);
        }
        
        let mut score = 100.0;
        let avg_nesting = functions.iter().map(|f| f.nesting_depth).sum::<usize>() as f64 / functions.len() as f64;
        
        // Penalty for deep nesting
        if avg_nesting > 3.0 {
            score -= (avg_nesting - 3.0) * 15.0;
        }
        
        // Additional penalty for extremely nested functions
        for func in functions {
            if func.nesting_depth > 8 {
                score -= 15.0;
            } else if func.nesting_depth > 5 {
                score -= 10.0;
            }
        }
        
        score.min(100.0).max(0.0)
    }
    
    /// Calculate technical debt ratio
    fn calculate_technical_debt_ratio(&self, functions: &[FunctionInfo], file_stats: &FileStats) -> f64 {
        if functions.is_empty() {
            return 0.0;
        }
        
        let mut debt_score = 0.0;
        
        // High complexity functions contribute to technical debt
        for func in functions {
            if func.cyclomatic_complexity > 20 {
                debt_score += 20.0;
            } else if func.cyclomatic_complexity > 10 {
                debt_score += 10.0;
            } else if func.cyclomatic_complexity > 5 {
                debt_score += 5.0;
            }
        }
        
        // Long functions contribute to technical debt
        for func in functions {
            if func.line_count > 100 {
                debt_score += 15.0;
            } else if func.line_count > 50 {
                debt_score += 10.0;
            }
        }
        
        // Lack of documentation contributes to technical debt
        let doc_coverage = self.calculate_documentation_coverage(file_stats);
        if doc_coverage < 20.0 {
            debt_score += 30.0 - doc_coverage;
        }
        
        // High nesting depth contributes to technical debt
        for func in functions {
            if func.nesting_depth > 5 {
                debt_score += (func.nesting_depth - 5) as f64 * 5.0;
            }
        }
        
        // Normalize to 0-100 scale
        let max_possible_debt = functions.len() as f64 * 50.0; // Rough estimate
        (debt_score / max_possible_debt.max(1.0) * 100.0).min(100.0)
    }
}

impl Default for QualityCalculator {
    fn default() -> Self {
        Self::new()
    }
} 