use crate::utils::errors::Result;
use super::super::types::{FunctionInfo, StructureInfo, StructureType, Visibility};
use super::LanguageAnalyzer;

/// Clojure language complexity analyzer
pub struct ClojureAnalyzer;

impl ClojureAnalyzer {
    pub fn new() -> Self {
        Self
    }
    
    /// Extract function name from Clojure function definition
    fn extract_function_name(&self, line: &str) -> Option<String> {
        let trimmed = line.trim();
        
        // Skip comments and empty lines
        if trimmed.starts_with(";") || trimmed.is_empty() {
            return None;
        }
        
        // Look for (defn function-name or (defn- function-name
        if let Some(start) = trimmed.find("(defn ").or_else(|| trimmed.find("(defn- ")) {
            let offset = if trimmed[start..].starts_with("(defn- ") { 7 } else { 6 };
            let after_defn = &trimmed[start + offset..];
            
            // Find function name (before parameters or docstring)
            let parts: Vec<&str> = after_defn.split_whitespace().collect();
            if let Some(first_part) = parts.first() {
                let func_name = first_part.trim();
                
                if !func_name.is_empty() && 
                   func_name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '?' || c == '!' || c == '*' || c == '+') {
                    return Some(func_name.to_string());
                }
            }
        }
        
        // Look for (def function-name (fn ...))
        if let Some(start) = trimmed.find("(def ") {
            let after_def = &trimmed[start + 5..];
            let parts: Vec<&str> = after_def.split_whitespace().collect();
            if let Some(first_part) = parts.first() {
                // Check if it's a function definition
                if after_def.contains("(fn") {
                    let func_name = first_part.trim();
                    if !func_name.is_empty() && 
                       func_name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '?' || c == '!' || c == '*' || c == '+') {
                        return Some(func_name.to_string());
                    }
                }
            }
        }
        
        // Look for anonymous functions (fn ...)
        if trimmed.contains("(fn ") {
            return Some("anonymous".to_string());
        }
        
        None
    }
    
    /// Extract namespace/protocol name from Clojure declaration
    fn extract_structure_name(&self, line: &str) -> Option<String> {
        let trimmed = line.trim();
        
        // Look for (ns namespace-name)
        if let Some(start) = trimmed.find("(ns ") {
            let after_ns = &trimmed[start + 4..];
            let parts: Vec<&str> = after_ns.split_whitespace().collect();
            if let Some(first_part) = parts.first() {
                let ns_name = first_part.trim();
                if !ns_name.is_empty() && 
                   ns_name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.') {
                    return Some(ns_name.to_string());
                }
            }
        }
        
        // Look for (defprotocol protocol-name)
        if let Some(start) = trimmed.find("(defprotocol ") {
            let after_defprotocol = &trimmed[start + 13..];
            let parts: Vec<&str> = after_defprotocol.split_whitespace().collect();
            if let Some(first_part) = parts.first() {
                let protocol_name = first_part.trim();
                if !protocol_name.is_empty() && 
                   protocol_name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
                    return Some(protocol_name.to_string());
                }
            }
        }
        
        // Look for (defrecord record-name)
        if let Some(start) = trimmed.find("(defrecord ") {
            let after_defrecord = &trimmed[start + 11..];
            let parts: Vec<&str> = after_defrecord.split_whitespace().collect();
            if let Some(first_part) = parts.first() {
                let record_name = first_part.trim();
                if !record_name.is_empty() && 
                   record_name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
                    return Some(record_name.to_string());
                }
            }
        }
        
        // Look for (deftype type-name)
        if let Some(start) = trimmed.find("(deftype ") {
            let after_deftype = &trimmed[start + 9..];
            let parts: Vec<&str> = after_deftype.split_whitespace().collect();
            if let Some(first_part) = parts.first() {
                let type_name = first_part.trim();
                if !type_name.is_empty() && 
                   type_name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
                    return Some(type_name.to_string());
                }
            }
        }
        
        None
    }
    
    /// Count complexity keywords in Clojure code
    fn count_complexity_keywords(&self, line: &str) -> usize {
        let keywords = [
            "if", "if-not", "when", "when-not", "cond", "condp", "case", "and", "or", "not",
            "try", "catch", "finally", "throw", "loop", "recur", "for", "doseq", "dotimes",
            "while", "when-let", "if-let", "some->", "some->>", "->", "->>"
        ];
        keywords.iter().map(|&keyword| {
            // Count occurrences but be careful of partial matches
            let pattern = format!("({}", keyword);
            line.matches(&pattern).count() + line.matches(&format!(" {}", keyword)).count()
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
        let mut paren_depth = 0;
        let mut in_function = false;
        
        for (i, line) in lines.iter().enumerate().skip(start_line) {
            let trimmed = line.trim();
            
            if trimmed.contains("(defn ") || trimmed.contains("(defn- ") || trimmed.contains("(def ") {
                in_function = true;
            }
            
            if in_function {
                // Count parentheses
                paren_depth += trimmed.matches('(').count();
                paren_depth = paren_depth.saturating_sub(trimmed.matches(')').count());
                
                if paren_depth == 0 {
                    return i;
                }
            }
        }
        
        lines.len().saturating_sub(1)
    }
    
    /// Determine visibility of a function
    fn determine_visibility(&self, line: &str) -> Visibility {
        if line.trim().contains("(defn- ") {
            Visibility::Private
        } else {
            Visibility::Public
        }
    }
    
    /// Determine structure type
    fn determine_structure_type(&self, line: &str) -> StructureType {
        let trimmed = line.trim();
        
        if trimmed.contains("(ns ") {
            StructureType::Module
        } else if trimmed.contains("(defprotocol ") {
            StructureType::Interface
        } else if trimmed.contains("(defrecord ") || trimmed.contains("(deftype ") {
            StructureType::Class
        } else {
            StructureType::Class // Default
        }
    }
}

impl LanguageAnalyzer for ClojureAnalyzer {
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
                let _visibility = Visibility::Public; // Clojure structures are typically public
                
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
        "Clojure"
    }
    
    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["clj", "cljs", "cljc", "edn"]
    }
}

impl ClojureAnalyzer {
    /// Count parameters in a function definition
    fn count_parameters(&self, line: &str) -> usize {
        // Look for parameter vector [param1 param2 ...]
        if let Some(start) = line.find('[') {
            if let Some(end) = line.find(']') {
                let params = &line[start + 1..end];
                if params.trim().is_empty() {
                    return 0;
                }
                // Count parameters separated by whitespace
                return params.split_whitespace().count();
            }
        }
        
        0
    }
    
    /// Find the end of a structure definition
    fn find_structure_end(&self, lines: &[String], start_line: usize) -> usize {
        let mut paren_depth = 0;
        let mut in_structure = false;
        
        for (i, line) in lines.iter().enumerate().skip(start_line) {
            let trimmed = line.trim();
            
            if trimmed.contains("(ns ") || trimmed.contains("(defprotocol ") || 
               trimmed.contains("(defrecord ") || trimmed.contains("(deftype ") {
                in_structure = true;
            }
            
            if in_structure {
                paren_depth += trimmed.matches('(').count();
                paren_depth = paren_depth.saturating_sub(trimmed.matches(')').count());
                
                if paren_depth == 0 {
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
            
            // Count field declarations in defrecord/deftype
            if line.contains("defrecord") || line.contains("deftype") {
                // Look for field vector [field1 field2 ...]
                if let Some(start) = line.find('[') {
                    if let Some(end) = line.find(']') {
                        let fields = &line[start + 1..end];
                        if !fields.trim().is_empty() {
                            count += fields.split_whitespace().count();
                        }
                    }
                }
            }
        }
        
        count
    }
}

impl Default for ClojureAnalyzer {
    fn default() -> Self {
        Self::new()
    }
} 