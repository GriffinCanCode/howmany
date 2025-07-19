use crate::utils::errors::Result;
use super::super::types::{FunctionInfo, StructureInfo, StructureType, Visibility};
use super::LanguageAnalyzer;

/// PHP language complexity analyzer
pub struct PhpAnalyzer;

impl PhpAnalyzer {
    pub fn new() -> Self {
        Self
    }
    
    /// Extract function name from PHP function declaration
    fn extract_function_name(&self, line: &str) -> Option<String> {
        let trimmed = line.trim();
        
        // Skip comments and empty lines
        if trimmed.starts_with("//") || trimmed.starts_with("/*") || 
           trimmed.starts_with('#') || trimmed.is_empty() {
            return None;
        }
        
        // Look for "function " pattern
        if let Some(start) = trimmed.find("function ") {
            let after_function = &trimmed[start + 9..];
            
            // Handle method declarations with visibility
            let function_part = if after_function.starts_with("&") {
                &after_function[1..] // Skip reference return
            } else {
                after_function
            };
            
            if let Some(paren_pos) = function_part.find('(') {
                let func_name = function_part[..paren_pos].trim();
                if !func_name.is_empty() && func_name.chars().all(|c| c.is_alphanumeric() || c == '_') {
                    return Some(func_name.to_string());
                }
            }
        }
        None
    }
    
    /// Extract class/interface/trait name from PHP declaration
    fn extract_structure_name(&self, line: &str) -> Option<String> {
        let trimmed = line.trim();
        
        let structure_keywords = ["class", "interface", "trait"];
        
        for keyword in &structure_keywords {
            if let Some(start) = trimmed.find(keyword) {
                let after_keyword = &trimmed[start + keyword.len()..];
                let parts: Vec<&str> = after_keyword.split_whitespace().collect();
                
                if let Some(first_part) = parts.first() {
                    // Handle inheritance and implements
                    let name = if let Some(extends_pos) = first_part.find("extends") {
                        &first_part[..extends_pos]
                    } else if let Some(implements_pos) = first_part.find("implements") {
                        &first_part[..implements_pos]
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
    
    /// Count complexity keywords in PHP code
    fn count_complexity_keywords(&self, line: &str) -> usize {
        let keywords = [
            "if", "elseif", "else", "for", "foreach", "while", "do", "switch", "case",
            "&&", "||", "and", "or", "xor", "?", "catch", "finally", "try", "throw"
        ];
        keywords.iter().map(|&keyword| line.matches(keyword).count()).sum()
    }
    
    /// Count cognitive complexity for PHP code
    fn count_cognitive_complexity(&self, line: &str, nesting_level: usize) -> usize {
        let mut complexity = 0;
        let nesting_multiplier = nesting_level.max(1);
        
        // Basic control structures
        if line.contains("if ") { complexity += 1 * nesting_multiplier; }
        if line.contains("elseif ") { complexity += 1; }
        if line.contains("else") && !line.contains("elseif") { complexity += 1; }
        if line.contains("for ") { complexity += 1 * nesting_multiplier; }
        if line.contains("foreach ") { complexity += 1 * nesting_multiplier; }
        if line.contains("while ") { complexity += 1 * nesting_multiplier; }
        if line.contains("do ") { complexity += 1 * nesting_multiplier; }
        if line.contains("switch ") { complexity += 1 * nesting_multiplier; }
        if line.contains("case ") { complexity += 1; }
        
        // Logical operators
        complexity += line.matches("&&").count() * nesting_multiplier;
        complexity += line.matches("||").count() * nesting_multiplier;
        complexity += line.matches(" and ").count() * nesting_multiplier;
        complexity += line.matches(" or ").count() * nesting_multiplier;
        complexity += line.matches(" xor ").count() * nesting_multiplier;
        
        // Ternary operator
        complexity += line.matches('?').count() * nesting_multiplier;
        
        // Exception handling
        if line.contains("try ") { complexity += 1 * nesting_multiplier; }
        if line.contains("catch ") { complexity += 1 * nesting_multiplier; }
        if line.contains("finally ") { complexity += 1; }
        if line.contains("throw ") { complexity += 1; }
        
        // PHP-specific complexity
        if line.contains("include ") || line.contains("require ") { complexity += 1; }
        if line.contains("include_once ") || line.contains("require_once ") { complexity += 1; }
        if line.contains("eval(") { complexity += 3; } // eval is very complex
        if line.contains("goto ") { complexity += 2; } // goto adds significant complexity
        
        // Magic methods add complexity
        if line.contains("__construct") || line.contains("__destruct") ||
           line.contains("__call") || line.contains("__get") || line.contains("__set") {
            complexity += 1;
        }
        
        complexity
    }
    
    /// Check if line contains a function declaration
    fn is_function_declaration(&self, line: &str) -> bool {
        let trimmed = line.trim();
        
        // Skip comments and empty lines
        if trimmed.starts_with("//") || trimmed.starts_with("/*") || 
           trimmed.starts_with('#') || trimmed.is_empty() {
            return false;
        }
        
        // Must contain "function " and parentheses
        trimmed.contains("function ") && trimmed.contains('(')
    }
    
    /// Check if line contains a structure declaration
    fn is_structure_declaration(&self, line: &str) -> bool {
        let trimmed = line.trim();
        let structure_keywords = ["class", "interface", "trait"];
        
        structure_keywords.iter().any(|&keyword| {
            trimmed.contains(keyword) && 
            !trimmed.starts_with("//") && 
            !trimmed.starts_with("/*") &&
            !trimmed.starts_with('#')
        })
    }
    
    /// Count parameters in function signature
    fn count_parameters(&self, line: &str) -> usize {
        if let Some(start) = line.find('(') {
            if let Some(end) = line.rfind(')') {
                let params = &line[start + 1..end];
                if params.trim().is_empty() {
                    return 0;
                }
                
                // Count parameters by commas, handling default values
                let mut param_count = 1;
                let mut in_string = false;
                let mut string_char = '"';
                let mut paren_depth = 0;
                
                for ch in params.chars() {
                    match ch {
                        '"' | '\'' if !in_string => {
                            in_string = true;
                            string_char = ch;
                        }
                        c if c == string_char && in_string => {
                            in_string = false;
                        }
                        '(' if !in_string => paren_depth += 1,
                        ')' if !in_string => paren_depth -= 1,
                        ',' if !in_string && paren_depth == 0 => param_count += 1,
                        _ => {}
                    }
                }
                
                return param_count;
            }
        }
        0
    }
    
    /// Determine structure type from declaration
    fn get_structure_type(&self, line: &str) -> StructureType {
        if line.contains("class") {
            StructureType::Class
        } else if line.contains("interface") {
            StructureType::Interface
        } else if line.contains("trait") {
            StructureType::Trait
        } else {
            StructureType::Class
        }
    }
    
    /// Determine visibility from PHP modifiers
    fn get_visibility(&self, line: &str) -> Visibility {
        if line.contains("public") {
            Visibility::Public
        } else if line.contains("private") {
            Visibility::Private
        } else if line.contains("protected") {
            Visibility::Protected
        } else {
            Visibility::Public // Default in PHP
        }
    }
    
    /// Check if function is a method (inside a class)
    fn is_method(&self, line: &str) -> bool {
        line.contains("public") || line.contains("private") || line.contains("protected") ||
        line.contains("static") || line.contains("abstract") || line.contains("final")
    }
    
    /// Check if method is static
    fn is_static(&self, line: &str) -> bool {
        line.contains("static")
    }
    
    /// Check if it's a magic method
    fn is_magic_method(&self, name: &str) -> bool {
        name.starts_with("__")
    }
    
    /// Count inheritance depth from class declaration
    fn count_inheritance_depth(&self, line: &str) -> usize {
        let mut depth = 0;
        
        // Check for extends
        if line.contains("extends") {
            depth += 1;
        }
        
        // Check for implements (interfaces)
        if line.contains("implements") {
            if let Some(implements_pos) = line.find("implements") {
                let after_implements = &line[implements_pos + 10..];
                let interfaces: Vec<&str> = after_implements.split(',').collect();
                depth += interfaces.len();
            }
        }
        
        depth
    }
}

impl LanguageAnalyzer for PhpAnalyzer {
    fn analyze_functions(&self, lines: &[String]) -> Result<Vec<FunctionInfo>> {
        let mut functions = Vec::new();
        let mut current_function: Option<FunctionInfo> = None;
        let mut brace_count = 0;
        let mut in_function = false;
        let mut nesting_level: usize = 0;
        let mut in_comment_block = false;
        let mut in_php_block = false;
        
        for (line_num, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            
            // Track PHP opening/closing tags
            if trimmed.contains("<?php") || trimmed.contains("<?") {
                in_php_block = true;
            }
            if trimmed.contains("?>") {
                in_php_block = false;
                continue;
            }
            
            // Skip non-PHP content
            if !in_php_block {
                continue;
            }
            
            // Handle multi-line comments
            if trimmed.starts_with("/*") {
                in_comment_block = true;
            }
            if trimmed.ends_with("*/") {
                in_comment_block = false;
                continue;
            }
            if in_comment_block {
                continue;
            }
            
            // Skip single-line comments and empty lines
            if trimmed.starts_with("//") || trimmed.starts_with('#') || trimmed.is_empty() {
                continue;
            }
            
            // Function declaration detection
            if self.is_function_declaration(trimmed) {
                if let Some(func_name) = self.extract_function_name(trimmed) {
                    let param_count = self.count_parameters(trimmed);
                    let is_method = self.is_method(trimmed);
                    let _is_static = self.is_static(trimmed);
                    let _is_magic = self.is_magic_method(&func_name);
                    
                    current_function = Some(FunctionInfo {
                        name: func_name,
                        line_count: 0,
                        cyclomatic_complexity: 1, // Base complexity
                        cognitive_complexity: 1, // Base cognitive complexity
                        nesting_depth: 0,
                        parameter_count: param_count,
                        return_path_count: 0,
                        start_line: line_num + 1,
                        end_line: line_num + 1,
                        is_method,
                        parent_class: None, // Will be set later
                        local_variable_count: 0,
                        has_recursion: false,
                        has_exception_handling: false,
                        visibility: Visibility::Public,});
                    in_function = true;
                    brace_count = 0;
                    nesting_level = 0;
                }
            }
            
            if in_function {
                // Count braces to track function scope
                brace_count += line.matches('{').count() as i32;
                brace_count -= line.matches('}').count() as i32;
                
                // Track nesting level
                if line.contains('{') {
                    nesting_level += 1;
                }
                if line.contains('}') {
                    nesting_level = nesting_level.saturating_sub(1);
                }
                
                if let Some(ref mut func) = current_function {
                    func.line_count += 1;
                    func.end_line = line_num + 1;
                    func.nesting_depth = func.nesting_depth.max(nesting_level);
                    
                    // Add complexity from keywords
                    let keyword_complexity = self.count_complexity_keywords(trimmed);
                    func.cyclomatic_complexity += keyword_complexity;
                    
                    // Add cognitive complexity
                    let cognitive_complexity = self.count_cognitive_complexity(trimmed, nesting_level);
                    func.cognitive_complexity += cognitive_complexity;
                    
                    // Count return statements
                    if trimmed.contains("return") {
                        func.return_path_count += 1;
                    }
                    
                    // Check for recursion
                    if trimmed.contains(&func.name) && trimmed.contains('(') {
                        func.has_recursion = true;
                    }
                    
                    // Check for exception handling
                    if trimmed.contains("try") || trimmed.contains("catch") || 
                       trimmed.contains("finally") || trimmed.contains("throw") {
                        func.has_exception_handling = true;
                    }
                    
                    // Count local variables (rough estimate)
                    if trimmed.starts_with('$') || trimmed.contains(" $") {
                        func.local_variable_count += 1;
                    }
                }
                
                // End of function
                if brace_count <= 0 && in_function {
                    if let Some(func) = current_function.take() {
                        functions.push(func);
                    }
                    in_function = false;
                    nesting_level = 0;
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
        let mut in_comment_block = false;
        let mut in_php_block = false;
        
        for (line_num, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            
            // Track PHP opening/closing tags
            if trimmed.contains("<?php") || trimmed.contains("<?") {
                in_php_block = true;
            }
            if trimmed.contains("?>") {
                in_php_block = false;
                continue;
            }
            
            // Skip non-PHP content
            if !in_php_block {
                continue;
            }
            
            // Handle multi-line comments
            if trimmed.starts_with("/*") {
                in_comment_block = true;
            }
            if trimmed.ends_with("*/") {
                in_comment_block = false;
                continue;
            }
            if in_comment_block {
                continue;
            }
            
            // Skip single-line comments and empty lines
            if trimmed.starts_with("//") || trimmed.starts_with('#') || trimmed.is_empty() {
                continue;
            }
            
            // Structure declaration detection
            if self.is_structure_declaration(trimmed) {
                if let Some(struct_name) = self.extract_structure_name(trimmed) {
                    let structure_type = self.get_structure_type(trimmed);
                    let visibility = self.get_visibility(trimmed);
                    let inheritance_depth = self.count_inheritance_depth(trimmed);
                    
                    current_structure = Some(StructureInfo {
                        name: struct_name,
                        structure_type,
                        line_count: 0,
                        start_line: line_num + 1,
                        end_line: line_num + 1,
                        methods: Vec::new(),
                        properties: 0,
                        visibility,
                        inheritance_depth,
                        interface_count: 0,
                    });
                    in_structure = true;
                    brace_count = 0;
                }
            }
            
            if in_structure {
                // Count braces to track structure scope
                brace_count += line.matches('{').count() as i32;
                brace_count -= line.matches('}').count() as i32;
                
                if let Some(ref mut structure) = current_structure {
                    structure.line_count += 1;
                    structure.end_line = line_num + 1;
                    
                    // Count properties and constants
                    if (trimmed.contains("public") || trimmed.contains("private") || 
                        trimmed.contains("protected")) &&
                       !self.is_function_declaration(trimmed) &&
                       (trimmed.starts_with('$') || trimmed.contains(" $") || 
                        trimmed.contains("const ")) {
                        structure.properties += 1;
                    }
                    
                    // Count interface methods
                    if structure.structure_type == StructureType::Interface {
                        if self.is_function_declaration(trimmed) {
                            structure.interface_count += 1;
                        }
                    }
                }
                
                // End of structure
                if brace_count <= 0 && in_structure {
                    if let Some(structure) = current_structure.take() {
                        structures.push(structure);
                    }
                    in_structure = false;
                }
            }
        }
        
        // Find methods that belong to structures
        for (line_num, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            
            if self.is_function_declaration(trimmed) && self.is_method(trimmed) {
                if let Some(func_name) = self.extract_function_name(trimmed) {
                    let param_count = self.count_parameters(trimmed);
                    let _is_static = self.is_static(trimmed);
                    
                    let method_info = FunctionInfo {
                        name: func_name,
                        line_count: 0, // Would need separate tracking
                        cyclomatic_complexity: 1,
                        cognitive_complexity: 1,
                        nesting_depth: 0,
                        parameter_count: param_count,
                        return_path_count: 0,
                        start_line: line_num + 1,
                        end_line: line_num + 1,
                        is_method: true,
                        parent_class: None, // Would need context tracking
                        local_variable_count: 0,
                        has_recursion: false,
                        has_exception_handling: false,
                        visibility: Visibility::Public,};
                    
                    // Add method to the most recent structure (simple heuristic)
                    if let Some(structure) = structures.last_mut() {
                        structure.methods.push(method_info);
                    }
                }
            }
        }
        
        Ok(structures)
    }
    
    fn language_name(&self) -> &'static str {
        "PHP"
    }
    
    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["php", "php3", "php4", "php5", "phtml"]
    }
}

impl Default for PhpAnalyzer {
    fn default() -> Self {
        Self::new()
    }
} 