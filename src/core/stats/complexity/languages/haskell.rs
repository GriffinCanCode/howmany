use crate::utils::errors::Result;
use super::super::types::{FunctionInfo, StructureInfo, StructureType, Visibility};
use super::LanguageAnalyzer;

/// Haskell language complexity analyzer
pub struct HaskellAnalyzer;

impl HaskellAnalyzer {
    pub fn new() -> Self {
        Self
    }
    
    /// Extract function name from Haskell function definition
    fn extract_function_name(&self, line: &str) -> Option<String> {
        let trimmed = line.trim();
        
        // Skip comments and empty lines
        if trimmed.starts_with("--") || trimmed.is_empty() {
            return None;
        }
        
        // Look for function definitions: functionName :: Type -> Type
        if trimmed.contains("::") {
            let parts: Vec<&str> = trimmed.split("::").collect();
            if let Some(first_part) = parts.first() {
                let func_name = first_part.trim();
                if !func_name.is_empty() && 
                   func_name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '\'') &&
                   func_name.chars().next().unwrap_or('A').is_lowercase() {
                    return Some(func_name.to_string());
                }
            }
        }
        
        // Look for function implementations: functionName args = body
        if trimmed.contains('=') && !trimmed.contains("::") {
            let parts: Vec<&str> = trimmed.split('=').collect();
            if let Some(first_part) = parts.first() {
                let func_part = first_part.trim();
                let words: Vec<&str> = func_part.split_whitespace().collect();
                if let Some(first_word) = words.first() {
                    if !first_word.is_empty() && 
                       first_word.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '\'') &&
                       first_word.chars().next().unwrap_or('A').is_lowercase() {
                        return Some(first_word.to_string());
                    }
                }
            }
        }
        
        // Look for lambda expressions: \args -> body
        if trimmed.contains("\\") && trimmed.contains("->") {
            return Some("lambda".to_string());
        }
        
        None
    }
    
    /// Extract data type/module name from Haskell declaration
    fn extract_structure_name(&self, line: &str) -> Option<String> {
        let trimmed = line.trim();
        
        // Look for module declarations: module ModuleName where
        if let Some(start) = trimmed.find("module ") {
            let after_module = &trimmed[start + 7..];
            let parts: Vec<&str> = after_module.split_whitespace().collect();
            if let Some(first_part) = parts.first() {
                let module_name = first_part.trim();
                if !module_name.is_empty() && 
                   module_name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '.') &&
                   module_name.chars().next().unwrap_or('a').is_uppercase() {
                    return Some(module_name.to_string());
                }
            }
        }
        
        // Look for data type declarations: data TypeName = ...
        if let Some(start) = trimmed.find("data ") {
            let after_data = &trimmed[start + 5..];
            let parts: Vec<&str> = after_data.split_whitespace().collect();
            if let Some(first_part) = parts.first() {
                let type_name = first_part.trim();
                if !type_name.is_empty() && 
                   type_name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '\'') &&
                   type_name.chars().next().unwrap_or('a').is_uppercase() {
                    return Some(type_name.to_string());
                }
            }
        }
        
        // Look for newtype declarations: newtype TypeName = ...
        if let Some(start) = trimmed.find("newtype ") {
            let after_newtype = &trimmed[start + 8..];
            let parts: Vec<&str> = after_newtype.split_whitespace().collect();
            if let Some(first_part) = parts.first() {
                let type_name = first_part.trim();
                if !type_name.is_empty() && 
                   type_name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '\'') &&
                   type_name.chars().next().unwrap_or('a').is_uppercase() {
                    return Some(type_name.to_string());
                }
            }
        }
        
        // Look for type aliases: type TypeName = ...
        if let Some(start) = trimmed.find("type ") {
            let after_type = &trimmed[start + 5..];
            let parts: Vec<&str> = after_type.split_whitespace().collect();
            if let Some(first_part) = parts.first() {
                let type_name = first_part.trim();
                if !type_name.is_empty() && 
                   type_name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '\'') &&
                   type_name.chars().next().unwrap_or('a').is_uppercase() {
                    return Some(type_name.to_string());
                }
            }
        }
        
        // Look for class declarations: class ClassName where
        if let Some(start) = trimmed.find("class ") {
            let after_class = &trimmed[start + 6..];
            let parts: Vec<&str> = after_class.split_whitespace().collect();
            if let Some(first_part) = parts.first() {
                let class_name = first_part.trim();
                if !class_name.is_empty() && 
                   class_name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '\'') &&
                   class_name.chars().next().unwrap_or('a').is_uppercase() {
                    return Some(class_name.to_string());
                }
            }
        }
        
        None
    }
    
    /// Count complexity keywords in Haskell code
    fn count_complexity_keywords(&self, line: &str) -> usize {
        let keywords = [
            "if", "then", "else", "case", "of", "let", "in", "where", "do",
            "guard", "|", "&&", "||", "not", "otherwise", "maybe", "either",
            "catch", "try", "throw", "error"
        ];
        keywords.iter().map(|&keyword| {
            // Be careful with partial matches, especially for short keywords like "|"
            if keyword == "|" {
                line.matches(" | ").count() + line.matches("| ").count()
            } else {
                line.matches(keyword).count()
            }
        }).sum()
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
        let mut in_function = false;
        let mut base_indentation = None;
        
        for (i, line) in lines.iter().enumerate().skip(start_line) {
            let trimmed = line.trim();
            
            if !trimmed.is_empty() {
                let current_indentation = line.len() - line.trim_start().len();
                
                if !in_function {
                    // First non-empty line defines the function
                    in_function = true;
                    base_indentation = Some(current_indentation);
                } else {
                    // Check if we've moved to a new top-level definition
                    if current_indentation <= base_indentation.unwrap_or(0) && 
                       (trimmed.contains("::") || trimmed.contains("=")) &&
                       !trimmed.starts_with("--") {
                        return i.saturating_sub(1);
                    }
                }
            }
        }
        
        lines.len().saturating_sub(1)
    }
    
    /// Determine visibility of a function
    fn determine_visibility(&self, _line: &str) -> Visibility {
        // Haskell functions are public by default unless not exported
        // We can't easily determine export status from a single line
        Visibility::Public
    }
    
    /// Determine structure type
    fn determine_structure_type(&self, line: &str) -> StructureType {
        let trimmed = line.trim();
        
        if trimmed.contains("module ") {
            StructureType::Module
        } else if trimmed.contains("data ") {
            StructureType::Class
        } else if trimmed.contains("newtype ") {
            StructureType::Struct
        } else if trimmed.contains("type ") {
            StructureType::Struct
        } else if trimmed.contains("class ") {
            StructureType::Interface
        } else {
            StructureType::Class // Default
        }
    }
}

impl LanguageAnalyzer for HaskellAnalyzer {
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
                let _visibility = Visibility::Public; // Haskell structures are typically public
                
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
        "Haskell"
    }
    
    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["hs", "lhs"]
    }
}

impl HaskellAnalyzer {
    /// Count parameters in a function definition
    fn count_parameters(&self, line: &str) -> usize {
        // For type signatures, count arrows to estimate parameters
        if line.contains("::") {
            let parts: Vec<&str> = line.split("::").collect();
            if let Some(type_part) = parts.get(1) {
                return type_part.matches("->").count();
            }
        }
        
        // For function definitions, count arguments before =
        if line.contains('=') && !line.contains("::") {
            let parts: Vec<&str> = line.split('=').collect();
            if let Some(first_part) = parts.first() {
                let args: Vec<&str> = first_part.trim().split_whitespace().collect();
                if args.len() > 1 {
                    return args.len() - 1; // Subtract 1 for the function name
                }
            }
        }
        
        0
    }
    
    /// Extract return type from function signature
    fn extract_return_type(&self, line: &str) -> Option<String> {
        if line.contains("::") {
            let parts: Vec<&str> = line.split("::").collect();
            if let Some(type_part) = parts.get(1) {
                // Get the last type after the final arrow
                let type_parts: Vec<&str> = type_part.split("->").collect();
                if let Some(return_type) = type_parts.last() {
                    return Some(return_type.trim().to_string());
                }
            }
        }
        
        None
    }
    
    /// Find the end of a structure definition
    fn find_structure_end(&self, lines: &[String], start_line: usize) -> usize {
        let mut in_structure = false;
        let mut base_indentation = None;
        
        for (i, line) in lines.iter().enumerate().skip(start_line) {
            let trimmed = line.trim();
            
            if !trimmed.is_empty() {
                let current_indentation = line.len() - line.trim_start().len();
                
                if !in_structure {
                    in_structure = true;
                    base_indentation = Some(current_indentation);
                } else {
                    // Check if we've moved to a new top-level definition
                    if current_indentation <= base_indentation.unwrap_or(0) && 
                       (trimmed.contains("data ") || trimmed.contains("newtype ") || 
                        trimmed.contains("type ") || trimmed.contains("class ") || 
                        trimmed.contains("module ")) &&
                       !trimmed.starts_with("--") {
                        return i.saturating_sub(1);
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
            
            // Count constructor fields in data declarations
            if line.contains("data ") {
                // Simple heuristic: count type annotations in constructor
                count += line.matches("::").count();
            }
            
            // Count record fields
            if line.contains("{") && line.contains("}") {
                // Count commas in record syntax as field separators
                if let Some(start) = line.find('{') {
                    if let Some(end) = line.find('}') {
                        let record_part = &line[start + 1..end];
                        count += record_part.matches(',').count() + 1; // +1 for the last field
                    }
                }
            }
        }
        
        count
    }
}

impl Default for HaskellAnalyzer {
    fn default() -> Self {
        Self::new()
    }
} 