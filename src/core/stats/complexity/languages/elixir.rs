use crate::utils::errors::Result;
use super::super::types::{FunctionInfo, StructureInfo, StructureType, Visibility};
use super::LanguageAnalyzer;

/// Elixir language complexity analyzer
pub struct ElixirAnalyzer;

impl ElixirAnalyzer {
    pub fn new() -> Self {
        Self
    }
    
    /// Extract function name from Elixir function definition
    fn extract_function_name(&self, line: &str) -> Option<String> {
        let trimmed = line.trim();
        
        // Skip comments and empty lines
        if trimmed.starts_with("#") || trimmed.is_empty() {
            return None;
        }
        
        // Look for "def " or "defp " patterns
        if let Some(start) = trimmed.find("def ").or_else(|| trimmed.find("defp ")) {
            let offset = if trimmed[start..].starts_with("defp ") { 5 } else { 4 };
            let after_def = &trimmed[start + offset..];
            
            // Handle function names with parameters
            let func_part = after_def.trim();
            
            // Find function name (before parentheses, comma, or do)
            let end_pos = func_part.find('(')
                .or_else(|| func_part.find(','))
                .or_else(|| func_part.find(" do"))
                .or_else(|| func_part.find(" when"))
                .unwrap_or(func_part.len());
            
            let func_name = &func_part[..end_pos].trim();
            
            if !func_name.is_empty() && 
               func_name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '?' || c == '!') {
                return Some(func_name.to_string());
            }
        }
        
        // Look for anonymous functions
        if trimmed.contains("fn ") {
            // This is an anonymous function, we'll count it but not extract a name
            return Some("anonymous".to_string());
        }
        
        None
    }
    
    /// Extract module/struct name from Elixir declaration
    fn extract_structure_name(&self, line: &str) -> Option<String> {
        let trimmed = line.trim();
        
        // Look for defmodule
        if let Some(start) = trimmed.find("defmodule ") {
            let after_defmodule = &trimmed[start + 10..];
            let parts: Vec<&str> = after_defmodule.split_whitespace().collect();
            
            if let Some(first_part) = parts.first() {
                // Handle module names with "do" at the end
                let name = if let Some(do_pos) = first_part.find(" do") {
                    &first_part[..do_pos]
                } else {
                    first_part
                };
                
                if !name.is_empty() && name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '.') {
                    return Some(name.to_string());
                }
            }
        }
        
        // Look for defstruct
        if trimmed.contains("defstruct ") {
            // Extract the module name from context (this is typically inside a module)
            return Some("struct".to_string());
        }
        
        // Look for defprotocol
        if let Some(start) = trimmed.find("defprotocol ") {
            let after_defprotocol = &trimmed[start + 12..];
            let parts: Vec<&str> = after_defprotocol.split_whitespace().collect();
            
            if let Some(first_part) = parts.first() {
                let name = if let Some(do_pos) = first_part.find(" do") {
                    &first_part[..do_pos]
                } else {
                    first_part
                };
                
                if !name.is_empty() && name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '.') {
                    return Some(name.to_string());
                }
            }
        }
        
        None
    }
    
    /// Count complexity keywords in Elixir code
    fn count_complexity_keywords(&self, line: &str) -> usize {
        let keywords = [
            "if", "unless", "cond", "case", "when", "for", "while", "until",
            "&&", "||", "and", "or", "not", "try", "rescue", "catch", "after",
            "receive", "with", "else", "->", "|>", "spawn", "send"
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
            
            if trimmed.contains("def ") || trimmed.contains("defp ") {
                in_function = true;
            }
            
            if in_function {
                // Count do/end blocks
                if trimmed.contains(" do") || trimmed.ends_with(" do") {
                    depth += 1;
                }
                if trimmed == "end" || trimmed.starts_with("end ") {
                    depth -= 1;
                    if depth == 0 {
                        return i;
                    }
                }
                
                // Handle single-line functions
                if trimmed.contains(", do:") && depth == 0 {
                    return i;
                }
            }
        }
        
        lines.len().saturating_sub(1)
    }
    
    /// Determine visibility of a function
    fn determine_visibility(&self, line: &str) -> Visibility {
        if line.trim().contains("defp ") {
            Visibility::Private
        } else if line.trim().contains("def ") {
            Visibility::Public
        } else {
            Visibility::Public // Default for Elixir
        }
    }
    
    /// Determine structure type
    fn determine_structure_type(&self, line: &str) -> StructureType {
        let trimmed = line.trim();
        
        if trimmed.contains("defmodule ") {
            StructureType::Module
        } else if trimmed.contains("defprotocol ") {
            StructureType::Interface
        } else if trimmed.contains("defstruct ") {
            StructureType::Struct
        } else {
            StructureType::Class // Default
        }
    }
}

impl LanguageAnalyzer for ElixirAnalyzer {
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
                    cognitive_complexity: complexity, // Use same value for now
                    nesting_depth: 0, // Calculate if needed
                    parameter_count: self.count_parameters(line),
                    return_path_count: 1, // Default value
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
                let _visibility = Visibility::Public; // Elixir modules are typically public
                
                structures.push(StructureInfo {
                    name: struct_name,
                    structure_type,
                    line_count: end_line.saturating_sub(i).max(1),
                    start_line: i + 1,
                    end_line: end_line + 1,
                    methods: self.collect_methods_in_structure(lines, i, end_line),
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
        "Elixir"
    }
    
    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["ex", "exs"]
    }
}

impl ElixirAnalyzer {
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
        
        // Handle functions without parentheses
        if line.contains("def ") || line.contains("defp ") {
            let after_def = if let Some(pos) = line.find("def ") {
                &line[pos + 4..]
            } else if let Some(pos) = line.find("defp ") {
                &line[pos + 5..]
            } else {
                return 0;
            };
            
            // Count arguments separated by commas before "do" or "when"
            let args_part = if let Some(do_pos) = after_def.find(" do") {
                &after_def[..do_pos]
            } else if let Some(when_pos) = after_def.find(" when") {
                &after_def[..when_pos]
            } else {
                after_def
            };
            
            if args_part.trim().is_empty() {
                return 0;
            }
            
            return args_part.split(',').count();
        }
        
        0
    }
    
    /// Find the end of a structure definition
    fn find_structure_end(&self, lines: &[String], start_line: usize) -> usize {
        let mut depth = 0;
        let mut in_structure = false;
        
        for (i, line) in lines.iter().enumerate().skip(start_line) {
            let trimmed = line.trim();
            
            if trimmed.contains("defmodule ") || trimmed.contains("defprotocol ") {
                in_structure = true;
            }
            
            if in_structure {
                if trimmed.contains(" do") || trimmed.ends_with(" do") {
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
    
    /// Collect methods within a structure
    fn collect_methods_in_structure(&self, lines: &[String], start_line: usize, end_line: usize) -> Vec<FunctionInfo> {
        let mut methods = Vec::new();
        
        for i in start_line..=end_line.min(lines.len().saturating_sub(1)) {
            if let Some(func_name) = self.extract_function_name(&lines[i]) {
                let func_end_line = self.find_function_end(lines, i);
                let complexity = self.calculate_cyclomatic_complexity(lines, i, func_end_line);
                
                methods.push(FunctionInfo {
                    name: func_name,
                    line_count: func_end_line.saturating_sub(i).max(1),
                    cyclomatic_complexity: complexity,
                    cognitive_complexity: complexity,
                    nesting_depth: 0,
                    parameter_count: self.count_parameters(&lines[i]),
                    return_path_count: 1,
                    start_line: i + 1,
                    end_line: func_end_line + 1,
                    is_method: true,
                    parent_class: None,
                    local_variable_count: 0,
                    has_recursion: false,
                    has_exception_handling: false,
                        visibility: Visibility::Public,});
            }
        }
        
        methods
    }
    
    /// Count fields within a structure
    fn count_fields_in_structure(&self, lines: &[String], start_line: usize, end_line: usize) -> usize {
        let mut count = 0;
        
        for i in start_line..=end_line.min(lines.len().saturating_sub(1)) {
            let line = &lines[i];
            // Count defstruct fields
            if line.trim().contains("defstruct ") {
                if let Some(start) = line.find("[") {
                    if let Some(end) = line.find("]") {
                        let fields = &line[start + 1..end];
                        count += fields.split(',').count();
                    }
                }
            }
            
            // Count @spec and @type definitions
            if line.trim().starts_with("@") {
                count += 1;
            }
        }
        
        count
    }
}

impl Default for ElixirAnalyzer {
    fn default() -> Self {
        Self::new()
    }
} 