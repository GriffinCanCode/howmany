use crate::utils::errors::Result;
use super::super::types::{FunctionInfo, StructureInfo, StructureType, Visibility};
use super::LanguageAnalyzer;

/// Zig language complexity analyzer
pub struct ZigAnalyzer;

impl ZigAnalyzer {
    pub fn new() -> Self {
        Self
    }
    
    /// Extract function name from Zig function definition
    fn extract_function_name(&self, line: &str) -> Option<String> {
        let trimmed = line.trim();
        
        // Skip comments and empty lines
        if trimmed.starts_with("//") || trimmed.is_empty() {
            return None;
        }
        
        // Look for "fn " pattern
        if let Some(start) = trimmed.find("fn ") {
            let after_fn = &trimmed[start + 3..];
            
            // Find function name (before parentheses)
            let end_pos = after_fn.find('(').unwrap_or(after_fn.len());
            let func_name = after_fn[..end_pos].trim();
            
            if !func_name.is_empty() && 
               func_name.chars().all(|c| c.is_alphanumeric() || c == '_') {
                return Some(func_name.to_string());
            }
        }
        
        // Look for pub fn
        if let Some(start) = trimmed.find("pub fn ") {
            let after_fn = &trimmed[start + 7..];
            
            let end_pos = after_fn.find('(').unwrap_or(after_fn.len());
            let func_name = after_fn[..end_pos].trim();
            
            if !func_name.is_empty() && 
               func_name.chars().all(|c| c.is_alphanumeric() || c == '_') {
                return Some(func_name.to_string());
            }
        }
        
        None
    }
    
    /// Extract structure name from Zig declaration
    fn extract_structure_name(&self, line: &str) -> Option<String> {
        let trimmed = line.trim();
        
        // Look for const name = struct
        if let Some(equals_pos) = trimmed.find('=') {
            let before_equals = &trimmed[..equals_pos].trim();
            let after_equals = &trimmed[equals_pos + 1..].trim();
            
            if after_equals.starts_with("struct") || after_equals.starts_with("union") || 
               after_equals.starts_with("enum") || after_equals.starts_with("packed struct") {
                let parts: Vec<&str> = before_equals.split_whitespace().collect();
                if let Some(last_part) = parts.last() {
                    if last_part.chars().all(|c| c.is_alphanumeric() || c == '_') {
                        return Some(last_part.to_string());
                    }
                }
            }
        }
        
        // Look for pub const name = struct
        if trimmed.contains("pub const") && (trimmed.contains("= struct") || 
           trimmed.contains("= union") || trimmed.contains("= enum")) {
            if let Some(const_pos) = trimmed.find("const ") {
                let after_const = &trimmed[const_pos + 6..];
                if let Some(equals_pos) = after_const.find('=') {
                    let struct_name = after_const[..equals_pos].trim();
                    if !struct_name.is_empty() && 
                       struct_name.chars().all(|c| c.is_alphanumeric() || c == '_') {
                        return Some(struct_name.to_string());
                    }
                }
            }
        }
        
        None
    }
    
    /// Count complexity keywords in Zig code
    fn count_complexity_keywords(&self, line: &str) -> usize {
        let keywords = [
            "if", "else", "for", "while", "switch", "and", "or", "orelse", "catch",
            "try", "defer", "errdefer", "break", "continue", "return", "unreachable"
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
            
            if trimmed.contains("fn ") {
                in_function = true;
            }
            
            if in_function {
                // Count braces
                depth += trimmed.matches('{').count();
                depth = depth.saturating_sub(trimmed.matches('}').count());
                
                if depth == 0 && trimmed.contains('}') {
                    return i;
                }
            }
        }
        
        lines.len().saturating_sub(1)
    }
    
    /// Determine visibility of a function
    fn determine_visibility(&self, line: &str) -> Visibility {
        if line.trim().contains("pub fn") {
            Visibility::Public
        } else {
            Visibility::Private
        }
    }
    
    /// Determine structure type
    fn determine_structure_type(&self, line: &str) -> StructureType {
        let trimmed = line.trim();
        
        if trimmed.contains("= struct") || trimmed.contains("= packed struct") {
            StructureType::Struct
        } else if trimmed.contains("= union") {
            StructureType::Struct  // Use Struct instead of Union
        } else if trimmed.contains("= enum") {
            StructureType::Enum
        } else {
            StructureType::Class // Default
        }
    }
}

impl LanguageAnalyzer for ZigAnalyzer {
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
                let _visibility = if line.contains("pub") { Visibility::Public } else { Visibility::Private };
                
                structures.push(StructureInfo {
                    name: struct_name,
                    structure_type,
                    line_count: end_line.saturating_sub(i).max(1),
                    start_line: i + 1,
                    end_line: end_line + 1,
                    methods: Vec::new(),
                    properties: self.count_fields_in_structure(lines, i, end_line),
                    visibility: if line.contains("pub") { Visibility::Public } else { Visibility::Private },
                    inheritance_depth: 0,
                    interface_count: 0,
                });
            }
        }
        
        Ok(structures)
    }
    
    fn language_name(&self) -> &'static str {
        "Zig"
    }
    
    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["zig"]
    }
}

impl ZigAnalyzer {
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
    
    /// Extract return type from function definition
    fn extract_return_type(&self, line: &str) -> Option<String> {
        // Look for return type after closing parenthesis
        if let Some(paren_pos) = line.find(')') {
            let after_paren = &line[paren_pos + 1..];
            if let Some(arrow_pos) = after_paren.find("->") {
                let return_type = after_paren[arrow_pos + 2..].trim();
                if let Some(brace_pos) = return_type.find('{') {
                    let clean_type = return_type[..brace_pos].trim();
                    if !clean_type.is_empty() {
                        return Some(clean_type.to_string());
                    }
                }
            }
        }
        
        None
    }
    
    /// Find the end of a structure definition
    fn find_structure_end(&self, lines: &[String], start_line: usize) -> usize {
        let mut depth = 0;
        let mut in_structure = false;
        
        for (i, line) in lines.iter().enumerate().skip(start_line) {
            let trimmed = line.trim();
            
            if trimmed.contains("= struct") || trimmed.contains("= union") || trimmed.contains("= enum") {
                in_structure = true;
            }
            
            if in_structure {
                depth += trimmed.matches('{').count();
                depth = depth.saturating_sub(trimmed.matches('}').count());
                
                if depth == 0 && trimmed.contains('}') {
                    return i;
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
            if trimmed.contains(':') && !trimmed.contains("fn") && 
               !trimmed.starts_with("//") && !trimmed.is_empty() && 
               !trimmed.contains("if") && !trimmed.contains("for") {
                count += 1;
            }
        }
        
        count
    }
}

impl Default for ZigAnalyzer {
    fn default() -> Self {
        Self::new()
    }
} 