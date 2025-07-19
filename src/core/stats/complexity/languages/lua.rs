use crate::utils::errors::Result;
use super::super::types::{FunctionInfo, StructureInfo, StructureType, Visibility};
use super::LanguageAnalyzer;

/// Lua language complexity analyzer
pub struct LuaAnalyzer;

impl LuaAnalyzer {
    pub fn new() -> Self {
        Self
    }
    
    /// Extract function name from Lua function definition
    fn extract_function_name(&self, line: &str) -> Option<String> {
        let trimmed = line.trim();
        
        // Skip comments and empty lines
        if trimmed.starts_with("--") || trimmed.is_empty() {
            return None;
        }
        
        // Look for "function " pattern
        if let Some(start) = trimmed.find("function ") {
            let after_function = &trimmed[start + 9..];
            
            // Find function name (before parentheses)
            let end_pos = after_function.find('(').unwrap_or(after_function.len());
            let func_name = after_function[..end_pos].trim();
            
            if !func_name.is_empty() && 
               func_name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '.' || c == ':') {
                return Some(func_name.to_string());
            }
        }
        
        // Look for local function
        if let Some(start) = trimmed.find("local function ") {
            let after_function = &trimmed[start + 15..];
            
            let end_pos = after_function.find('(').unwrap_or(after_function.len());
            let func_name = after_function[..end_pos].trim();
            
            if !func_name.is_empty() && 
               func_name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '.' || c == ':') {
                return Some(func_name.to_string());
            }
        }
        
        // Look for function assignment: name = function(...)
        if let Some(equals_pos) = trimmed.find('=') {
            let before_equals = &trimmed[..equals_pos].trim();
            let after_equals = &trimmed[equals_pos + 1..].trim();
            
            if after_equals.starts_with("function") {
                // Extract variable name
                let parts: Vec<&str> = before_equals.split_whitespace().collect();
                if let Some(last_part) = parts.last() {
                    if last_part.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '.' || c == ':') {
                        return Some(last_part.to_string());
                    }
                }
            }
        }
        
        None
    }
    
    /// Extract table/module name from Lua declaration
    fn extract_structure_name(&self, line: &str) -> Option<String> {
        let trimmed = line.trim();
        
        // Look for table creation: name = {}
        if let Some(equals_pos) = trimmed.find('=') {
            let before_equals = &trimmed[..equals_pos].trim();
            let after_equals = &trimmed[equals_pos + 1..].trim();
            
            if after_equals.starts_with('{') {
                let parts: Vec<&str> = before_equals.split_whitespace().collect();
                if let Some(last_part) = parts.last() {
                    if last_part.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '.') {
                        return Some(last_part.to_string());
                    }
                }
            }
        }
        
        // Look for module pattern
        if trimmed.contains("module(") {
            if let Some(start) = trimmed.find("module(") {
                let after_module = &trimmed[start + 7..];
                if let Some(end) = after_module.find(')') {
                    let module_name = after_module[..end].trim().trim_matches('"').trim_matches('\'');
                    if !module_name.is_empty() {
                        return Some(module_name.to_string());
                    }
                }
            }
        }
        
        None
    }
    
    /// Count complexity keywords in Lua code
    fn count_complexity_keywords(&self, line: &str) -> usize {
        let keywords = [
            "if", "elseif", "else", "for", "while", "repeat", "until", "and", "or", "not",
            "break", "return", "goto", "pcall", "xpcall", "error", "assert"
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
            
            if trimmed.contains("function") {
                in_function = true;
                depth += 1;
            }
            
            if in_function {
                // Count function/do/if/for/while/repeat blocks
                if trimmed.contains("function") || trimmed.contains("do") || 
                   trimmed.contains("if") || trimmed.contains("for") || 
                   trimmed.contains("while") || trimmed.contains("repeat") {
                    depth += 1;
                }
                if trimmed == "end" || trimmed.starts_with("end ") {
                    depth -= 1;
                    if depth == 0 {
                        return i;
                    }
                }
                if trimmed.contains("until") {
                    depth -= 1;
                    if depth == 0 {
                        return i;
                    }
                }
            }
        }
        
        lines.len().saturating_sub(1)
    }
    
    /// Determine visibility of a function
    fn determine_visibility(&self, line: &str) -> Visibility {
        if line.trim().contains("local function") || line.trim().contains("local ") {
            Visibility::Private
        } else {
            Visibility::Public
        }
    }
    
    /// Determine structure type
    fn determine_structure_type(&self, line: &str) -> StructureType {
        let trimmed = line.trim();
        
        if trimmed.contains("module(") {
            StructureType::Module
        } else if trimmed.contains("= {}") || trimmed.contains("={}") {
            StructureType::Class // Lua tables used as objects
        } else {
            StructureType::Class // Default
        }
    }
}

impl LanguageAnalyzer for LuaAnalyzer {
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
                let _visibility = Visibility::Public; // Lua structures are typically public
                
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
        "Lua"
    }
    
    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["lua"]
    }
}

impl LuaAnalyzer {
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
            
            if trimmed.contains("= {") {
                in_structure = true;
                depth += 1;
            }
            
            if in_structure {
                if trimmed.contains('{') {
                    depth += trimmed.matches('{').count();
                }
                if trimmed.contains('}') {
                    depth -= trimmed.matches('}').count();
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
            
            // Count field assignments in tables
            if trimmed.contains('=') && !trimmed.contains("function") && 
               !trimmed.starts_with("--") && !trimmed.is_empty() {
                count += 1;
            }
        }
        
        count
    }
}

impl Default for LuaAnalyzer {
    fn default() -> Self {
        Self::new()
    }
} 