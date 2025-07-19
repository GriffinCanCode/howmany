use crate::utils::errors::Result;
use super::super::types::{FunctionInfo, StructureInfo, StructureType, Visibility};
use super::LanguageAnalyzer;

/// C# language complexity analyzer
pub struct CSharpAnalyzer;

impl CSharpAnalyzer {
    pub fn new() -> Self {
        Self
    }
    
    /// Extract function/method name from C# declaration
    fn extract_function_name(&self, line: &str) -> Option<String> {
        let trimmed = line.trim();
        
        // Skip comments and empty lines
        if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.is_empty() {
            return None;
        }
        
        // Look for method patterns
        let method_keywords = ["public", "private", "protected", "internal", "static", "virtual", "override", "abstract", "async"];
        let mut has_method_keyword = false;
        
        for keyword in &method_keywords {
            if trimmed.contains(keyword) {
                has_method_keyword = true;
                break;
            }
        }
        
        if !has_method_keyword {
            return None;
        }
        
        // Find the method name before the opening parenthesis
        if let Some(paren_pos) = trimmed.find('(') {
            let before_paren = &trimmed[..paren_pos];
            let parts: Vec<&str> = before_paren.split_whitespace().collect();
            
            // Method name is typically the last part before the parenthesis
            if let Some(last_part) = parts.last() {
                // Skip generic type parameters
                let name = if let Some(generic_pos) = last_part.find('<') {
                    &last_part[..generic_pos]
                } else {
                    last_part
                };
                
                if !name.is_empty() && name.chars().all(|c| c.is_alphanumeric() || c == '_') {
                    return Some(name.to_string());
                }
            }
        }
        None
    }
    
    /// Extract class/struct/interface name from C# declaration
    fn extract_structure_name(&self, line: &str) -> Option<String> {
        let trimmed = line.trim();
        
        let structure_keywords = ["class", "struct", "interface", "enum", "record"];
        
        for keyword in &structure_keywords {
            if let Some(start) = trimmed.find(keyword) {
                let after_keyword = &trimmed[start + keyword.len()..];
                let parts: Vec<&str> = after_keyword.split_whitespace().collect();
                
                if let Some(first_part) = parts.first() {
                    // Handle generic parameters
                    let name = if let Some(generic_pos) = first_part.find('<') {
                        &first_part[..generic_pos]
                    } else if let Some(colon_pos) = first_part.find(':') {
                        &first_part[..colon_pos]
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
    
    /// Count complexity keywords in C# code
    fn count_complexity_keywords(&self, line: &str) -> usize {
        let keywords = [
            "if", "else if", "for", "foreach", "while", "do", "switch", "case", 
            "&&", "||", "?", "catch", "finally", "try", "throw", "using", "lock"
        ];
        keywords.iter().map(|&keyword| line.matches(keyword).count()).sum()
    }
    
    /// Count cognitive complexity for C# code
    fn count_cognitive_complexity(&self, line: &str, nesting_level: usize) -> usize {
        let mut complexity = 0;
        let nesting_multiplier = nesting_level.max(1);
        
        // Basic control structures
        if line.contains("if ") { complexity += 1 * nesting_multiplier; }
        if line.contains("else if") { complexity += 1; }
        if line.contains("else") && !line.contains("else if") { complexity += 1; }
        if line.contains("for ") { complexity += 1 * nesting_multiplier; }
        if line.contains("foreach ") { complexity += 1 * nesting_multiplier; }
        if line.contains("while ") { complexity += 1 * nesting_multiplier; }
        if line.contains("do ") { complexity += 1 * nesting_multiplier; }
        if line.contains("switch ") { complexity += 1 * nesting_multiplier; }
        if line.contains("case ") { complexity += 1; }
        
        // Logical operators
        complexity += line.matches("&&").count() * nesting_multiplier;
        complexity += line.matches("||").count() * nesting_multiplier;
        
        // Ternary operator
        complexity += line.matches('?').count() * nesting_multiplier;
        
        // Exception handling
        if line.contains("try ") { complexity += 1 * nesting_multiplier; }
        if line.contains("catch ") { complexity += 1 * nesting_multiplier; }
        if line.contains("finally ") { complexity += 1; }
        if line.contains("throw ") { complexity += 1; }
        
        // C#-specific complexity
        if line.contains("async ") { complexity += 1; } // Async methods add complexity
        if line.contains("await ") { complexity += 1; } // Await calls
        if line.contains("yield ") { complexity += 2; } // Yield statements are complex
        if line.contains("lock ") { complexity += 1 * nesting_multiplier; } // Lock statements
        if line.contains("using ") && line.contains('(') { complexity += 1; } // Using statements
        
        // LINQ complexity
        if line.contains(".Where(") || line.contains(".Select(") || 
           line.contains(".Any(") || line.contains(".All(") ||
           line.contains(".First(") || line.contains(".Last(") {
            complexity += 1;
        }
        
        complexity
    }
    
    /// Check if line contains a method declaration
    fn is_method_declaration(&self, line: &str) -> bool {
        let trimmed = line.trim();
        
        // Skip comments and empty lines
        if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.is_empty() {
            return false;
        }
        
        // Must contain parentheses for parameters
        if !trimmed.contains('(') {
            return false;
        }
        
        // Check for method modifiers
        let method_keywords = ["public", "private", "protected", "internal", "static", "virtual", "override", "abstract", "async"];
        method_keywords.iter().any(|&keyword| trimmed.contains(keyword))
    }
    
    /// Check if line contains a structure declaration
    fn is_structure_declaration(&self, line: &str) -> bool {
        let trimmed = line.trim();
        let structure_keywords = ["class", "struct", "interface", "enum", "record"];
        
        structure_keywords.iter().any(|&keyword| {
            trimmed.contains(keyword) && 
            !trimmed.starts_with("//") && 
            !trimmed.starts_with("/*")
        })
    }
    
    /// Count parameters in method signature
    fn count_parameters(&self, line: &str) -> usize {
        if let Some(start) = line.find('(') {
            if let Some(end) = line.rfind(')') {
                let params = &line[start + 1..end];
                if params.trim().is_empty() {
                    return 0;
                }
                
                // Count parameters by commas, but be careful with generics
                let mut paren_depth = 0;
                let mut angle_depth = 0;
                let mut param_count = 1;
                
                for ch in params.chars() {
                    match ch {
                        '(' => paren_depth += 1,
                        ')' => paren_depth -= 1,
                        '<' => angle_depth += 1,
                        '>' => angle_depth -= 1,
                        ',' if paren_depth == 0 && angle_depth == 0 => param_count += 1,
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
        } else if line.contains("struct") {
            StructureType::Struct
        } else if line.contains("enum") {
            StructureType::Enum
        } else if line.contains("record") {
            StructureType::Class // Records are a special type of class in C#
        } else {
            StructureType::Class
        }
    }
    
    /// Determine visibility from C# modifiers
    fn get_visibility(&self, line: &str) -> Visibility {
        if line.contains("public") {
            Visibility::Public
        } else if line.contains("private") {
            Visibility::Private
        } else if line.contains("protected") {
            Visibility::Protected
        } else if line.contains("internal") {
            Visibility::Internal
        } else {
            Visibility::Private // Default in C#
        }
    }
    
    /// Check if method is static
    fn is_static(&self, line: &str) -> bool {
        line.contains("static")
    }
    
    /// Check if method is async
    fn is_async(&self, line: &str) -> bool {
        line.contains("async")
    }
    
    /// Count inheritance depth from class declaration
    fn count_inheritance_depth(&self, line: &str) -> usize {
        if line.contains(':') {
            // Count base classes/interfaces after colon
            if let Some(colon_pos) = line.find(':') {
                let after_colon = &line[colon_pos + 1..];
                let parts: Vec<&str> = after_colon.split(',').collect();
                return parts.len();
            }
        }
        0
    }
}

impl LanguageAnalyzer for CSharpAnalyzer {
    fn analyze_functions(&self, lines: &[String]) -> Result<Vec<FunctionInfo>> {
        let mut functions = Vec::new();
        let mut current_function: Option<FunctionInfo> = None;
        let mut brace_count = 0;
        let mut in_function = false;
        let mut nesting_level: usize = 0;
        let mut in_comment_block = false;
        
        for (line_num, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            
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
            if trimmed.starts_with("//") || trimmed.is_empty() {
                continue;
            }
            
            // Method declaration detection
            if self.is_method_declaration(trimmed) {
                if let Some(func_name) = self.extract_function_name(trimmed) {
                    let param_count = self.count_parameters(trimmed);
                    let is_static = self.is_static(trimmed);
                    let _is_async = self.is_async(trimmed);
                    
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
                        is_method: !is_static,
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
                    
                    // Count local variables
                    if (trimmed.contains("var ") || trimmed.contains("int ") || 
                        trimmed.contains("string ") || trimmed.contains("bool ") ||
                        trimmed.contains("double ") || trimmed.contains("float ") ||
                        trimmed.contains("decimal ") || trimmed.contains("object ")) &&
                       !trimmed.contains("public") && !trimmed.contains("private") &&
                       !trimmed.contains("protected") && !trimmed.contains("internal") {
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
        
        for (line_num, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            
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
            if trimmed.starts_with("//") || trimmed.is_empty() {
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
                    
                    // Count properties and fields
                    if (trimmed.contains("public") || trimmed.contains("private") || 
                        trimmed.contains("protected") || trimmed.contains("internal")) &&
                       !trimmed.contains("class") && !trimmed.contains("struct") &&
                       !trimmed.contains("interface") && !trimmed.contains("enum") &&
                       !self.is_method_declaration(trimmed) &&
                       (trimmed.contains(';') || trimmed.contains("{ get") || 
                        trimmed.contains("{ set") || trimmed.contains("=>")) {
                        structure.properties += 1;
                    }
                    
                    // Count interface implementations
                    if structure.structure_type == StructureType::Interface {
                        if trimmed.contains('(') && trimmed.contains(')') &&
                           !trimmed.starts_with("//") && 
                           !trimmed.starts_with("/*") {
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
            
            if self.is_method_declaration(trimmed) {
                if let Some(func_name) = self.extract_function_name(trimmed) {
                    let param_count = self.count_parameters(trimmed);
                    let is_static = self.is_static(trimmed);
                    
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
                        is_method: !is_static,
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
        "C#"
    }
    
    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["cs"]
    }
}

impl Default for CSharpAnalyzer {
    fn default() -> Self {
        Self::new()
    }
} 