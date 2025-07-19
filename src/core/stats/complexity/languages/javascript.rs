use crate::utils::errors::Result;
use super::super::types::{FunctionInfo, StructureInfo, StructureType, Visibility};
use super::LanguageAnalyzer;

/// JavaScript/TypeScript language complexity analyzer
pub struct JavaScriptAnalyzer;

impl JavaScriptAnalyzer {
    pub fn new() -> Self {
        Self
    }
    
    /// Check if line is a JavaScript function declaration
    fn is_function_declaration(&self, line: &str) -> bool {
        line.contains("function ") || 
        line.contains("=> ") || 
        (line.contains("(") && line.contains(")") && line.contains("{"))
    }
    
    /// Extract function name from JavaScript function declaration
    fn extract_function_name(&self, line: &str) -> Option<String> {
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
    
    /// Count complexity keywords in JavaScript code
    fn count_complexity_keywords(&self, line: &str) -> usize {
        let keywords = ["if", "else if", "while", "for", "switch", "case", "catch", "finally", "&&", "||", "?"];
        keywords.iter().map(|&keyword| line.matches(keyword).count()).sum()
    }
    
    /// Count cognitive complexity for JavaScript code
    fn count_cognitive_complexity(&self, line: &str, nesting_level: i32) -> usize {
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
            if let Some(end) = line.rfind(')') {
                if end > start {
                    let params_str = &line[start + 1..end];
                    if params_str.trim().is_empty() {
                        return 0;
                    }
                    
                    // Simple parameter counting (split by comma)
                    let param_count = params_str.split(',').count();
                    
                    // Adjust for common patterns
                    if params_str.contains("this") {
                        return param_count.saturating_sub(1);
                    }
                    
                    return param_count;
                }
            }
        }
        0
    }
    
    /// Detect JavaScript/TypeScript structure type and name
    fn detect_structure(&self, line: &str) -> Option<(StructureType, String, Visibility)> {
        let visibility = if line.contains("export ") {
            Visibility::Public
        } else {
            Visibility::Private
        };
        
        if line.contains("class ") {
            if let Some(name) = self.extract_class_name(line) {
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
    fn extract_class_name(&self, line: &str) -> Option<String> {
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
}

impl LanguageAnalyzer for JavaScriptAnalyzer {
    fn analyze_functions(&self, lines: &[String]) -> Result<Vec<FunctionInfo>> {
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
            if self.is_function_declaration(trimmed) {
                if let Some(func_name) = self.extract_function_name(trimmed) {
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
                        visibility: Visibility::Public,});
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
                    func.cyclomatic_complexity += self.count_complexity_keywords(trimmed);
                    
                    // Calculate cognitive complexity
                    func.cognitive_complexity += self.count_cognitive_complexity(trimmed, brace_count);
                    
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
    
    fn analyze_structures(&self, lines: &[String]) -> Result<Vec<StructureInfo>> {
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
            if let Some((structure_type, name, visibility)) = self.detect_structure(trimmed) {
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
    
    fn language_name(&self) -> &'static str {
        "JavaScript/TypeScript"
    }
    
    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["js", "jsx", "ts", "tsx"]
    }
}

impl Default for JavaScriptAnalyzer {
    fn default() -> Self {
        Self::new()
    }
} 