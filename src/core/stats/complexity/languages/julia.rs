use crate::utils::errors::Result;
use super::super::types::{FunctionInfo, StructureInfo, StructureType, Visibility};
use super::LanguageAnalyzer;

/// Julia language complexity analyzer
pub struct JuliaAnalyzer;

impl JuliaAnalyzer {
    pub fn new() -> Self {
        Self
    }
    
    /// Extract function name from Julia function definition
    fn extract_function_name(&self, line: &str) -> Option<String> {
        let trimmed = line.trim();
        
        // Skip comments and empty lines
        if trimmed.starts_with("#") || trimmed.is_empty() {
            return None;
        }
        
        // Look for "function " pattern
        if let Some(start) = trimmed.find("function ") {
            let after_function = &trimmed[start + 9..];
            
            // Find function name (before parentheses or generic parameters)
            let end_pos = after_function.find('(')
                .or_else(|| after_function.find('{'))
                .or_else(|| after_function.find(' '))
                .unwrap_or(after_function.len());
            
            let func_name = after_function[..end_pos].trim();
            
            if !func_name.is_empty() && 
               func_name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '!' || c == '?') {
                return Some(func_name.to_string());
            }
        }
        
        // Look for short function syntax: name(args) = body
        if let Some(equals_pos) = trimmed.find('=') {
            let before_equals = &trimmed[..equals_pos].trim();
            
            if let Some(paren_pos) = before_equals.find('(') {
                let func_name = before_equals[..paren_pos].trim();
                
                if !func_name.is_empty() && 
                   func_name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '!' || c == '?') {
                    return Some(func_name.to_string());
                }
            }
        }
        
        // Look for anonymous functions
        if trimmed.contains("->") || trimmed.contains("function(") {
            return Some("anonymous".to_string());
        }
        
        None
    }
    
    /// Extract structure name from Julia declaration
    fn extract_structure_name(&self, line: &str) -> Option<String> {
        let trimmed = line.trim();
        
        let structure_keywords = ["struct", "mutable struct", "abstract type", "primitive type", "module"];
        
        for keyword in &structure_keywords {
            if let Some(start) = trimmed.find(keyword) {
                let after_keyword = &trimmed[start + keyword.len()..];
                let parts: Vec<&str> = after_keyword.split_whitespace().collect();
                
                if let Some(first_part) = parts.first() {
                    // Handle generic parameters
                    let name = if let Some(generic_pos) = first_part.find('{') {
                        &first_part[..generic_pos]
                    } else if let Some(subtype_pos) = first_part.find('<') {
                        &first_part[..subtype_pos]
                    } else {
                        first_part
                    };
                    
                    if !name.is_empty() && name.chars().all(|c| c.is_alphanumeric() || c == '_') {
                        return Some(name.to_string());
                    }
                }
            }
        }
        
        None
    }
    
    /// Count complexity keywords in Julia code
    fn count_complexity_keywords(&self, line: &str) -> usize {
        let keywords = [
            "if", "elseif", "else", "for", "while", "try", "catch", "finally",
            "&&", "||", "!", "?", ":", "begin", "let", "do", "break", "continue",
            "return", "throw", "rethrow"
        ];
        keywords.iter().map(|&keyword| line.matches(keyword).count()).sum()
    }
    
    /// Calculate cyclomatic complexity for a function
    fn calculate_cyclomatic_complexity(&self, lines: &[String], start_line: usize, end_line: usize) -> usize {
        let mut complexity = 1; // Base complexity
        
        for i in start_line..=end_line.min(lines.len().saturating_sub(1)) {
            complexity += self.count_complexity_keywords(&lines[i]);
        }
        
        complexity
    }
    
    /// Find the end of a function definition
    fn find_function_end(&self, lines: &[String], start_line: usize) -> usize {
        let mut depth = 0;
        let mut in_function = false;
        
        for (i, line) in lines.iter().enumerate().skip(start_line) {
            let trimmed = line.trim();
            
            if trimmed.contains("function ") {
                in_function = true;
                depth += 1;
            }
            
            if in_function {
                // Count begin/end blocks
                if trimmed.contains("begin") || trimmed.contains("let") || trimmed.contains("for") || 
                   trimmed.contains("while") || trimmed.contains("if") || trimmed.contains("try") {
                    depth += 1;
                }
                if trimmed == "end" || trimmed.starts_with("end ") {
                    depth -= 1;
                    if depth == 0 {
                        return i;
                    }
                }
                
                // Handle single-line functions
                if trimmed.contains("=") && !trimmed.contains("function") && depth == 1 {
                    return i;
                }
            }
        }
        
        lines.len().saturating_sub(1)
    }
    
    /// Determine visibility of a function
    fn determine_visibility(&self, _line: &str) -> Visibility {
        // Julia functions are public by default
        // Private functions are typically indicated by naming convention (underscore prefix)
        Visibility::Public
    }
    
    /// Determine structure type
    fn determine_structure_type(&self, line: &str) -> StructureType {
        let trimmed = line.trim();
        
        if trimmed.contains("module ") {
            StructureType::Module
        } else if trimmed.contains("struct ") || trimmed.contains("mutable struct ") {
            StructureType::Struct
        } else if trimmed.contains("abstract type ") {
            StructureType::Interface
        } else if trimmed.contains("primitive type ") {
            StructureType::Struct
        } else {
            StructureType::Class // Default
        }
    }
}

impl LanguageAnalyzer for JuliaAnalyzer {
    fn analyze_functions(&self, lines: &[String]) -> Result<Vec<FunctionInfo>> {
        let mut functions = Vec::new();
        
        for (i, line) in lines.iter().enumerate() {
            if let Some(func_name) = self.extract_function_name(line) {
                let end_line = self.find_function_end(lines, i);
                let complexity = self.calculate_cyclomatic_complexity(lines, i, end_line);
                let _visibility = self.determine_visibility(line);
                
                functions.push(FunctionInfo {
                    name: func_name,
                    line_count: end_line.saturating_sub(i).max(1),
                    cyclomatic_complexity: complexity,
                    cognitive_complexity: complexity,
                    nesting_depth: 0,
                    parameter_count: self.count_parameters(line),
                    return_path_count: 1,
                    start_line: i + 1,
                    end_line: end_line + 1,
                    is_method: false,
                    parent_class: None,
                    local_variable_count: 0,
                    has_recursion: false,
                    has_exception_handling: false,
                        visibility: Visibility::Public,});
            }
        }
        
        Ok(functions)
    }
    
    fn analyze_structures(&self, lines: &[String]) -> Result<Vec<StructureInfo>> {
        let mut structures = Vec::new();
        
        for (i, line) in lines.iter().enumerate() {
            if let Some(struct_name) = self.extract_structure_name(line) {
                let end_line = self.find_structure_end(lines, i);
                let structure_type = self.determine_structure_type(line);
                let _visibility = Visibility::Public; // Julia structures are typically public
                
                structures.push(StructureInfo {
                    name: struct_name,
                    structure_type,
                    line_count: end_line.saturating_sub(i).max(1),
                    start_line: i + 1,
                    end_line: end_line + 1,
                    methods: Vec::new(),
                    properties: self.count_fields_in_structure(lines, i, end_line),
                    visibility: Visibility::Public,
                    inheritance_depth: 0,
                    interface_count: 0,
                });
            }
        }
        
        Ok(structures)
    }
    
    fn language_name(&self) -> &'static str {
        "Julia"
    }
    
    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["jl"]
    }
}

impl JuliaAnalyzer {
    /// Count parameters in a function definition
    fn count_parameters(&self, line: &str) -> usize {
        if let Some(start) = line.find('(') {
            if let Some(end) = line.find(')') {
                let params = &line[start + 1..end];
                if params.trim().is_empty() {
                    return 0;
                }
                return params.split(',').count();
            }
        }
        
        0
    }
    
    /// Find the end of a structure definition
    fn find_structure_end(&self, lines: &[String], start_line: usize) -> usize {
        let mut depth = 0;
        let mut in_structure = false;
        
        for (i, line) in lines.iter().enumerate().skip(start_line) {
            let trimmed = line.trim();
            
            if trimmed.contains("struct ") || trimmed.contains("module ") || 
               trimmed.contains("abstract type ") || trimmed.contains("primitive type ") {
                in_structure = true;
                depth += 1;
            }
            
            if in_structure {
                if trimmed.contains("begin") || trimmed.contains("let") || trimmed.contains("for") || 
                   trimmed.contains("while") || trimmed.contains("if") || trimmed.contains("try") {
                    depth += 1;
                }
                if trimmed == "end" || trimmed.starts_with("end ") {
                    depth -= 1;
                    if depth == 0 {
                        return i;
                    }
                }
            }
        }
        
        lines.len().saturating_sub(1)
    }
    
    /// Count methods within a structure
    fn count_methods_in_structure(&self, lines: &[String], start_line: usize, end_line: usize) -> usize {
        let mut count = 0;
        
        for i in start_line..=end_line.min(lines.len().saturating_sub(1)) {
            if self.extract_function_name(&lines[i]).is_some() {
                count += 1;
            }
        }
        
        count
    }
    
    /// Count fields within a structure
    fn count_fields_in_structure(&self, lines: &[String], start_line: usize, end_line: usize) -> usize {
        let mut count = 0;
        
        for i in start_line..=end_line.min(lines.len().saturating_sub(1)) {
            let line = &lines[i];
            let trimmed = line.trim();
            
            // Count field declarations in structs
            if !trimmed.starts_with("function") && !trimmed.starts_with("#") && 
               !trimmed.is_empty() && !trimmed.contains("struct") && !trimmed.contains("end") {
                // Simple heuristic: lines that look like field declarations
                if trimmed.contains("::") || (trimmed.contains(":") && !trimmed.contains("if")) {
                    count += 1;
                }
            }
        }
        
        count
    }
}

impl Default for JuliaAnalyzer {
    fn default() -> Self {
        Self::new()
    }
} 