use crate::utils::errors::Result;
use super::super::types::{FunctionInfo, StructureInfo, StructureType, Visibility};
use super::LanguageAnalyzer;

/// Kotlin language complexity analyzer
pub struct KotlinAnalyzer;

impl KotlinAnalyzer {
    pub fn new() -> Self {
        Self
    }
    
    /// Extract function name from Kotlin function declaration
    fn extract_function_name(&self, line: &str) -> Option<String> {
        let trimmed = line.trim();
        
        // Skip comments and empty lines
        if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.is_empty() {
            return None;
        }
        
        // Look for "fun " pattern
        if let Some(start) = trimmed.find("fun ") {
            let after_fun = &trimmed[start + 4..];
            
            // Find function name before parentheses or generic parameters
            let end_pos = after_fun.find('(')
                .or_else(|| after_fun.find('<'))
                .unwrap_or(after_fun.len());
            
            let func_name = after_fun[..end_pos].trim();
            
            if !func_name.is_empty() && func_name.chars().all(|c| c.is_alphanumeric() || c == '_') {
                return Some(func_name.to_string());
            }
        }
        None
    }
    
    /// Extract structure name from Kotlin declaration
    fn extract_structure_name(&self, line: &str) -> Option<String> {
        let trimmed = line.trim();
        
        let structure_keywords = ["class", "interface", "object", "enum", "sealed", "data"];
        
        for keyword in &structure_keywords {
            if let Some(start) = trimmed.find(keyword) {
                let after_keyword = &trimmed[start + keyword.len()..];
                let parts: Vec<&str> = after_keyword.split_whitespace().collect();
                
                if let Some(first_part) = parts.first() {
                    // Handle generic parameters and inheritance
                    let name = if let Some(generic_pos) = first_part.find('<') {
                        &first_part[..generic_pos]
                    } else if let Some(colon_pos) = first_part.find(':') {
                        &first_part[..colon_pos]
                    } else if let Some(paren_pos) = first_part.find('(') {
                        &first_part[..paren_pos]
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
    
    /// Count complexity keywords in Kotlin code
    fn count_complexity_keywords(&self, line: &str) -> usize {
        let keywords = [
            "if", "else", "when", "for", "while", "do", "try", "catch", "finally",
            "&&", "||", "?:", "?.", "!!", "elvis", "safe", "throw", "suspend"
        ];
        keywords.iter().map(|&keyword| line.matches(keyword).count()).sum()
    }
    
    /// Count cognitive complexity for Kotlin code
    fn count_cognitive_complexity(&self, line: &str, nesting_level: usize) -> usize {
        let mut complexity = 0;
        let nesting_multiplier = nesting_level.max(1);
        
        // Basic control structures
        if line.contains("if ") || line.contains("if(") { complexity += 1 * nesting_multiplier; }
        if line.contains("else if") { complexity += 1; }
        if line.contains("else") && !line.contains("else if") { complexity += 1; }
        if line.contains("when ") || line.contains("when(") { complexity += 1 * nesting_multiplier; }
        if line.contains("for ") || line.contains("for(") { complexity += 1 * nesting_multiplier; }
        if line.contains("while ") || line.contains("while(") { complexity += 1 * nesting_multiplier; }
        if line.contains("do ") { complexity += 1 * nesting_multiplier; }
        
        // When expression branches
        if line.contains("->") && !line.contains("lambda") { complexity += 1; }
        
        // Logical operators
        complexity += line.matches("&&").count() * nesting_multiplier;
        complexity += line.matches("||").count() * nesting_multiplier;
        
        // Kotlin-specific operators
        complexity += line.matches("?:").count(); // Elvis operator
        complexity += line.matches("?.").count(); // Safe call operator
        complexity += line.matches("!!").count(); // Not-null assertion
        
        // Exception handling
        if line.contains("try ") { complexity += 1; }
        if line.contains("catch ") { complexity += 1 * nesting_multiplier; }
        if line.contains("finally ") { complexity += 1; }
        if line.contains("throw ") { complexity += 1; }
        
        // Coroutines add complexity
        if line.contains("suspend ") { complexity += 1; }
        if line.contains("async ") { complexity += 1; }
        if line.contains("await ") { complexity += 1; }
        if line.contains("launch ") { complexity += 1; }
        if line.contains("runBlocking ") { complexity += 1; }
        
        // Lambda expressions and higher-order functions
        if line.contains("{ ") || line.contains(" -> ") { complexity += 1; }
        if line.contains(".let ") || line.contains(".apply ") || 
           line.contains(".run ") || line.contains(".with ") || 
           line.contains(".also ") { complexity += 1; }
        
        // Collection operations that can be complex
        if line.contains(".map ") || line.contains(".filter ") || 
           line.contains(".reduce ") || line.contains(".fold ") { complexity += 1; }
        
        complexity
    }
    
    /// Check if line contains a function declaration
    fn is_function_declaration(&self, line: &str) -> bool {
        let trimmed = line.trim();
        
        // Skip comments and empty lines
        if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.is_empty() {
            return false;
        }
        
        // Must contain "fun " and parentheses
        trimmed.contains("fun ") && trimmed.contains('(')
    }
    
    /// Check if line contains a structure declaration
    fn is_structure_declaration(&self, line: &str) -> bool {
        let trimmed = line.trim();
        let structure_keywords = ["class", "interface", "object", "enum", "sealed", "data"];
        
        structure_keywords.iter().any(|&keyword| {
            trimmed.contains(keyword) && 
            !trimmed.starts_with("//") && 
            !trimmed.starts_with("/*")
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
                
                // Count parameters by commas, handling lambdas and generics
                let mut param_count = 1;
                let mut paren_depth = 0;
                let mut angle_depth = 0;
                let mut brace_depth = 0;
                
                for ch in params.chars() {
                    match ch {
                        '(' => paren_depth += 1,
                        ')' => paren_depth -= 1,
                        '<' => angle_depth += 1,
                        '>' => angle_depth -= 1,
                        '{' => brace_depth += 1,
                        '}' => brace_depth -= 1,
                        ',' if paren_depth == 0 && angle_depth == 0 && brace_depth == 0 => {
                            param_count += 1;
                        }
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
        if line.contains("data class") || line.contains("class") {
            StructureType::Class
        } else if line.contains("interface") {
            StructureType::Interface
        } else if line.contains("enum") {
            StructureType::Enum
        } else if line.contains("object") || line.contains("sealed") {
            StructureType::Class
        } else {
            StructureType::Class
        }
    }
    
    /// Determine visibility from Kotlin access modifiers
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
            Visibility::Public // Default in Kotlin
        }
    }
    
    /// Check if function is suspend (coroutine)
    fn is_suspend(&self, line: &str) -> bool {
        line.contains("suspend")
    }
    
    /// Check if function is inline
    fn is_inline(&self, line: &str) -> bool {
        line.contains("inline")
    }
    
    /// Check if function is extension function
    fn is_extension_function(&self, line: &str) -> bool {
        if let Some(fun_pos) = line.find("fun ") {
            let after_fun = &line[fun_pos + 4..];
            after_fun.contains('.') && after_fun.find('.').unwrap() < after_fun.find('(').unwrap_or(after_fun.len())
        } else {
            false
        }
    }
    
    /// Check if it's a data class
    fn is_data_class(&self, line: &str) -> bool {
        line.contains("data class")
    }
    
    /// Check if it's a sealed class
    fn is_sealed_class(&self, line: &str) -> bool {
        line.contains("sealed class") || line.contains("sealed interface")
    }
    
    /// Count inheritance/interface implementations from declaration
    fn count_inheritance_depth(&self, line: &str) -> usize {
        if line.contains(':') {
            if let Some(colon_pos) = line.find(':') {
                let after_colon = &line[colon_pos + 1..];
                // Remove where clauses and body
                let inheritance_part = if let Some(where_pos) = after_colon.find(" where ") {
                    &after_colon[..where_pos]
                } else if let Some(brace_pos) = after_colon.find('{') {
                    &after_colon[..brace_pos]
                } else {
                    after_colon
                };
                
                let parts: Vec<&str> = inheritance_part.split(',').collect();
                return parts.len();
            }
        }
        0
    }
}

impl LanguageAnalyzer for KotlinAnalyzer {
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
            
            // Function declaration detection
            if self.is_function_declaration(trimmed) {
                if let Some(func_name) = self.extract_function_name(trimmed) {
                    let param_count = self.count_parameters(trimmed);
                    let _is_suspend = self.is_suspend(trimmed);
                    let _is_inline = self.is_inline(trimmed);
                    let is_extension = self.is_extension_function(trimmed);
                    
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
                        is_method: !is_extension, // Extension functions are not methods
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
                    
                    // Count local variables (val and var)
                    if (trimmed.contains("val ") || trimmed.contains("var ")) &&
                       !trimmed.contains("fun ") && !trimmed.contains("class ") &&
                       !trimmed.contains("interface ") && !trimmed.contains("object ") {
                        func.local_variable_count += 1;
                    }
                }
                
                // End of function (handle single-expression functions)
                if (brace_count <= 0 && in_function) || 
                   (trimmed.contains("=") && !trimmed.contains("{") && trimmed.ends_with(")")) {
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
                    
                    // Count properties (val and var declarations)
                    if (trimmed.contains("val ") || trimmed.contains("var ")) &&
                       !self.is_function_declaration(trimmed) &&
                       (trimmed.contains("public") || trimmed.contains("private") || 
                        trimmed.contains("protected") || trimmed.contains("internal") ||
                        (!trimmed.contains("fun ") && !trimmed.contains("class "))) {
                        structure.properties += 1;
                    }
                    
                    // Count interface methods for interfaces
                    if structure.structure_type == StructureType::Interface {
                        if self.is_function_declaration(trimmed) || 
                           (trimmed.contains("val ") || trimmed.contains("var ")) {
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
            
            if self.is_function_declaration(trimmed) {
                if let Some(func_name) = self.extract_function_name(trimmed) {
                    let param_count = self.count_parameters(trimmed);
                    let _is_suspend = self.is_suspend(trimmed);
                    let is_extension = self.is_extension_function(trimmed);
                    
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
                        is_method: !is_extension,
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
        "Kotlin"
    }
    
    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["kt", "kts"]
    }
}

impl Default for KotlinAnalyzer {
    fn default() -> Self {
        Self::new()
    }
} 