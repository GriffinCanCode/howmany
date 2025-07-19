use crate::utils::errors::Result;
use super::super::types::{FunctionInfo, StructureInfo, StructureType, Visibility};
use super::LanguageAnalyzer;

/// C/C++ language complexity analyzer
pub struct CppAnalyzer;

impl CppAnalyzer {
    pub fn new() -> Self {
        Self
    }
    
    /// Extract function name from C/C++ function declaration
    fn extract_function_name(&self, line: &str) -> Option<String> {
        let trimmed = line.trim();
        
        // Skip preprocessor directives, comments, and declarations
        if trimmed.starts_with('#') || trimmed.starts_with("//") || 
           trimmed.starts_with("/*") || trimmed.ends_with(';') {
            return None;
        }
        
        // Look for function patterns with parentheses
        if let Some(paren_pos) = trimmed.find('(') {
            // Work backwards from the parenthesis to find the function name
            let before_paren = &trimmed[..paren_pos];
            let parts: Vec<&str> = before_paren.split_whitespace().collect();
            
            if parts.len() >= 1 {
                let potential_name = parts.last().unwrap();
                
                // Remove pointer/reference indicators
                let clean_name = potential_name.trim_start_matches('*').trim_start_matches('&');
                
                // Skip operators and constructors/destructors
                if !clean_name.is_empty() && 
                   !clean_name.starts_with("operator") &&
                   !clean_name.starts_with('~') &&
                   !clean_name.contains("::") &&
                   clean_name.chars().all(|c| c.is_alphanumeric() || c == '_') {
                    return Some(clean_name.to_string());
                }
            }
        }
        None
    }
    
    /// Extract class/struct name from C/C++ declaration
    fn extract_class_name(&self, line: &str) -> Option<String> {
        let trimmed = line.trim();
        
        // Look for class/struct/union/enum declarations
        let keywords = ["class ", "struct ", "union ", "enum "];
        
        for keyword in keywords {
            if let Some(start) = trimmed.find(keyword) {
                let after_keyword = &trimmed[start + keyword.len()..];
                let parts: Vec<&str> = after_keyword.split_whitespace().collect();
                
                if let Some(name) = parts.first() {
                    // Remove template parameters and inheritance
                    let clean_name = name.split('<').next().unwrap_or(name)
                                        .split(':').next().unwrap_or(name)
                                        .split('{').next().unwrap_or(name);
                    
                    if !clean_name.is_empty() {
                        return Some(clean_name.to_string());
                    }
                }
            }
        }
        None
    }
    
    /// Count complexity keywords in C/C++ code
    fn count_complexity_keywords(&self, line: &str) -> usize {
        let keywords = ["if", "else if", "while", "for", "do", "switch", "case", "catch", "&&", "||", "?"];
        keywords.iter().map(|&keyword| line.matches(keyword).count()).sum()
    }
    
    /// Count cognitive complexity for C/C++ code
    fn count_cognitive_complexity(&self, line: &str, nesting_level: usize) -> usize {
        let mut complexity = 0;
        let nesting_multiplier = nesting_level.max(1);
        
        // Basic control structures
        if line.contains("if ") || line.contains("if(") { complexity += 1 * nesting_multiplier; }
        if line.contains("else if") { complexity += 1; }
        if line.contains("else") && !line.contains("else if") { complexity += 1; }
        if line.contains("while ") || line.contains("while(") { complexity += 1 * nesting_multiplier; }
        if line.contains("for ") || line.contains("for(") { complexity += 1 * nesting_multiplier; }
        if line.contains("do ") { complexity += 1 * nesting_multiplier; }
        if line.contains("switch ") || line.contains("switch(") { complexity += 1 * nesting_multiplier; }
        if line.contains("case ") { complexity += 1; }
        if line.contains("catch ") || line.contains("catch(") { complexity += 1 * nesting_multiplier; }
        
        // Logical operators
        complexity += line.matches("&&").count() * nesting_multiplier;
        complexity += line.matches("||").count() * nesting_multiplier;
        
        // Ternary operator
        complexity += line.matches("?").count() * nesting_multiplier;
        
        // Goto statements (discouraged but add complexity)
        if line.contains("goto ") { complexity += 2 * nesting_multiplier; }
        
        complexity
    }
    
    /// Check if line contains a function declaration/definition
    fn is_function_declaration(&self, line: &str) -> bool {
        let trimmed = line.trim();
        
        // Skip preprocessor directives, comments, and simple declarations
        if trimmed.starts_with('#') || trimmed.starts_with("//") || 
           trimmed.starts_with("/*") || trimmed.is_empty() {
            return false;
        }
        
        // Must contain parentheses
        if !trimmed.contains('(') || !trimmed.contains(')') {
            return false;
        }
        
        // Skip function calls (look for assignment or semicolon)
        if trimmed.contains('=') && !trimmed.contains("==") && !trimmed.contains("!=") {
            return false;
        }
        
        // Skip simple declarations ending with semicolon
        if trimmed.ends_with(';') {
            return false;
        }
        
        // Look for function patterns
        let has_return_type = trimmed.contains("void") || trimmed.contains("int") || 
                             trimmed.contains("char") || trimmed.contains("float") ||
                             trimmed.contains("double") || trimmed.contains("bool") ||
                             trimmed.contains("string") || trimmed.contains("std::");
        
        // Check for function definition patterns (opening brace or function body)
        let has_body_indicator = trimmed.contains('{') || 
                                (trimmed.contains('(') && trimmed.contains(')') && 
                                 !trimmed.ends_with(';'));
        
        has_return_type || has_body_indicator
    }
    
    /// Check if line contains a class/struct/union/enum declaration
    fn is_structure_declaration(&self, line: &str) -> bool {
        let trimmed = line.trim();
        trimmed.contains("class ") || trimmed.contains("struct ") || 
        trimmed.contains("union ") || trimmed.contains("enum ")
    }
    
    /// Count parameters in function signature
    fn count_parameters(&self, line: &str) -> usize {
        if let Some(start) = line.find('(') {
            if let Some(end) = line.rfind(')') {
                let params = &line[start + 1..end];
                if params.trim().is_empty() || params.trim() == "void" {
                    return 0;
                }
                
                // Count commas, handling nested parentheses and templates
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
        if line.contains("class ") {
            StructureType::Class
        } else if line.contains("struct ") {
            StructureType::Struct
        } else if line.contains("union ") {
            StructureType::Struct // Treat union as struct variant
        } else if line.contains("enum ") {
            StructureType::Enum
        } else {
            StructureType::Struct
        }
    }
    
    /// Determine visibility from context (C++ specific)
    fn get_visibility(&self, line: &str) -> Visibility {
        if line.contains("public:") || line.contains("public ") {
            Visibility::Public
        } else if line.contains("private:") || line.contains("private ") {
            Visibility::Private
        } else if line.contains("protected:") || line.contains("protected ") {
            Visibility::Protected
        } else {
            Visibility::Public // C structs and functions are public by default
        }
    }
    
    /// Check if function is a method (inside a class/struct)
    fn is_method(&self, current_class: &Option<String>) -> bool {
        current_class.is_some()
    }
}

impl LanguageAnalyzer for CppAnalyzer {
    fn analyze_functions(&self, lines: &[String]) -> Result<Vec<FunctionInfo>> {
        let mut functions = Vec::new();
        let mut current_function: Option<FunctionInfo> = None;
        let mut brace_count = 0;
        let mut in_function = false;
        let mut nesting_level: usize = 0;
        let mut current_class: Option<String> = None;
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
            
            // Skip single-line comments, preprocessor directives, and empty lines
            if trimmed.starts_with("//") || trimmed.starts_with('#') || trimmed.is_empty() {
                continue;
            }
            
            // Track current class/struct for method context
            if self.is_structure_declaration(trimmed) {
                current_class = self.extract_class_name(trimmed);
            }
            
            // Function declaration detection
            if self.is_function_declaration(trimmed) {
                if let Some(func_name) = self.extract_function_name(trimmed) {
                    let param_count = self.count_parameters(trimmed);
                    let is_method = self.is_method(&current_class);
                    
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
                        parent_class: current_class.clone(),
                        local_variable_count: 0,
                        has_recursion: false,
                        has_exception_handling: false,
                        visibility: Visibility::Public, // Default visibility for standalone functions
                    });
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
                    
                    // Check for exception handling (C++)
                    if trimmed.contains("try") || trimmed.contains("catch") || 
                       trimmed.contains("throw") || trimmed.contains("except") {
                        func.has_exception_handling = true;
                    }
                    
                    // Rough estimate of local variables
                    if (trimmed.contains("int ") || trimmed.contains("char ") || 
                        trimmed.contains("float ") || trimmed.contains("double ") ||
                        trimmed.contains("bool ") || trimmed.contains("string ") ||
                        trimmed.contains("auto ")) && !trimmed.contains("(") {
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
        let mut current_visibility = Visibility::Public;
        
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
            
            // Skip single-line comments, preprocessor directives, and empty lines
            if trimmed.starts_with("//") || trimmed.starts_with('#') || trimmed.is_empty() {
                continue;
            }
            
            // Track visibility changes in C++
            if trimmed.starts_with("public:") {
                current_visibility = Visibility::Public;
            } else if trimmed.starts_with("private:") {
                current_visibility = Visibility::Private;
            } else if trimmed.starts_with("protected:") {
                current_visibility = Visibility::Protected;
            }
            
            // Structure declaration detection
            if self.is_structure_declaration(trimmed) {
                if let Some(struct_name) = self.extract_class_name(trimmed) {
                    let structure_type = self.get_structure_type(trimmed);
                    let visibility = self.get_visibility(trimmed);
                    let is_class = structure_type == StructureType::Class;
                    
                    current_structure = Some(StructureInfo {
                        name: struct_name,
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
                    current_visibility = if is_class {
                        Visibility::Private // C++ classes default to private
                    } else {
                        Visibility::Public // C structs default to public
                    };
                }
            }
            
            if in_structure {
                // Count braces to track structure scope
                brace_count += line.matches('{').count() as i32;
                brace_count -= line.matches('}').count() as i32;
                
                if let Some(ref mut structure) = current_structure {
                    structure.line_count += 1;
                    structure.end_line = line_num + 1;
                    
                    // Count member variables/properties
                    if (trimmed.contains("int ") || trimmed.contains("char ") || 
                        trimmed.contains("float ") || trimmed.contains("double ") ||
                        trimmed.contains("bool ") || trimmed.contains("string ")) &&
                       !trimmed.contains('(') && trimmed.contains(';') {
                        structure.properties += 1;
                    }
                    
                    // Count method declarations within the structure
                    if self.is_function_declaration(trimmed) {
                        if let Some(func_name) = self.extract_function_name(trimmed) {
                            let param_count = self.count_parameters(trimmed);
                            
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
                                parent_class: Some(structure.name.clone()),
                                local_variable_count: 0,
                                has_recursion: false,
                                has_exception_handling: false,
                                visibility: current_visibility,
                            };
                            structure.methods.push(method_info);
                        }
                    }
                    
                    // Count inheritance (simplified)
                    if trimmed.contains(": public") || trimmed.contains(": private") || 
                       trimmed.contains(": protected") {
                        structure.inheritance_depth = 1;
                    }
                    
                    // Count virtual functions (interface-like behavior)
                    if trimmed.contains("virtual ") {
                        structure.interface_count += 1;
                    }
                }
                
                // End of structure
                if brace_count <= 0 && in_structure {
                    if let Some(structure) = current_structure.take() {
                        structures.push(structure);
                    }
                    in_structure = false;
                    current_visibility = Visibility::Public;
                }
            }
        }
        
        Ok(structures)
    }
    
    fn language_name(&self) -> &'static str {
        "C/C++"
    }
    
    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["cpp", "cc", "cxx", "c", "h", "hpp"]
    }
}

impl Default for CppAnalyzer {
    fn default() -> Self {
        Self::new()
    }
} 