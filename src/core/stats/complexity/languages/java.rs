use crate::utils::errors::Result;
use super::super::types::{FunctionInfo, StructureInfo, StructureType, Visibility};
use super::LanguageAnalyzer;

/// Java language complexity analyzer
pub struct JavaAnalyzer;

impl JavaAnalyzer {
    pub fn new() -> Self {
        Self
    }
    
    /// Extract function name from Java method declaration
    fn extract_function_name(&self, line: &str) -> Option<String> {
        // Handle various Java method patterns
        let trimmed = line.trim();
        
        // Look for method patterns: public/private/protected/static/final void/int/String methodName(
        if let Some(paren_pos) = trimmed.find('(') {
            // Find the method name by working backwards from the parenthesis
            let before_paren = &trimmed[..paren_pos];
            let parts: Vec<&str> = before_paren.split_whitespace().collect();
            
            if parts.len() >= 2 {
                // Last part should be the method name
                let method_name = parts.last().unwrap();
                // Skip constructors and operators
                if !method_name.starts_with(char::is_uppercase) && !method_name.contains('<') {
                    return Some(method_name.to_string());
                }
            }
        }
        None
    }
    
    /// Extract class name from Java class declaration
    fn extract_class_name(&self, line: &str) -> Option<String> {
        let trimmed = line.trim();
        
        // Look for class/interface/enum declarations
        if let Some(start) = trimmed.find("class ").or_else(|| trimmed.find("interface ")).or_else(|| trimmed.find("enum ")) {
            let keyword_end = if trimmed[start..].starts_with("class") { start + 6 } 
                            else if trimmed[start..].starts_with("interface") { start + 10 }
                            else { start + 5 }; // enum
            
            let after_keyword = &trimmed[keyword_end..];
            let parts: Vec<&str> = after_keyword.split_whitespace().collect();
            
            if let Some(name) = parts.first() {
                // Remove generic type parameters if present
                if let Some(generic_start) = name.find('<') {
                    return Some(name[..generic_start].to_string());
                } else {
                    return Some(name.to_string());
                }
            }
        }
        None
    }
    
    /// Count complexity keywords in Java code
    fn count_complexity_keywords(&self, line: &str) -> usize {
        let keywords = ["if", "else if", "while", "for", "do", "switch", "case", "catch", "&&", "||", "?"];
        keywords.iter().map(|&keyword| line.matches(keyword).count()).sum()
    }
    
    /// Count cognitive complexity for Java code
    fn count_cognitive_complexity(&self, line: &str, nesting_level: usize) -> usize {
        let mut complexity = 0;
        let nesting_multiplier = nesting_level.max(1);
        
        // Basic control structures
        if line.contains("if ") { complexity += 1 * nesting_multiplier; }
        if line.contains("else if") { complexity += 1; }
        if line.contains("else") && !line.contains("else if") { complexity += 1; }
        if line.contains("while ") { complexity += 1 * nesting_multiplier; }
        if line.contains("for ") { complexity += 1 * nesting_multiplier; }
        if line.contains("do ") { complexity += 1 * nesting_multiplier; }
        if line.contains("switch ") { complexity += 1 * nesting_multiplier; }
        if line.contains("case ") { complexity += 1; }
        if line.contains("catch ") { complexity += 1 * nesting_multiplier; }
        if line.contains("finally") { complexity += 1; }
        
        // Logical operators
        complexity += line.matches("&&").count() * nesting_multiplier;
        complexity += line.matches("||").count() * nesting_multiplier;
        
        // Ternary operator
        complexity += line.matches("?").count() * nesting_multiplier;
        
        // Lambda expressions add complexity
        if line.contains("->") { complexity += 1; }
        
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
        
        // Common method patterns
        let method_keywords = ["public", "private", "protected", "static", "final", "abstract", "synchronized"];
        let has_modifier = method_keywords.iter().any(|&keyword| trimmed.contains(keyword));
        
        // Check for return types
        let return_types = ["void", "int", "String", "boolean", "double", "float", "long", "char", "byte", "short"];
        let has_return_type = return_types.iter().any(|&rtype| trimmed.contains(rtype));
        
        // Generic return types or custom types
        let has_generic_or_custom = trimmed.contains('<') || 
                                   (trimmed.contains('(') && 
                                    trimmed.split_whitespace().count() >= 2);
        
        has_modifier || has_return_type || has_generic_or_custom
    }
    
    /// Check if line contains a class/interface/enum declaration
    fn is_structure_declaration(&self, line: &str) -> bool {
        let trimmed = line.trim();
        trimmed.contains("class ") || trimmed.contains("interface ") || trimmed.contains("enum ")
    }
    
    /// Count parameters in method signature
    fn count_parameters(&self, line: &str) -> usize {
        if let Some(start) = line.find('(') {
            if let Some(end) = line.find(')') {
                let params = &line[start + 1..end];
                if params.trim().is_empty() {
                    return 0;
                }
                // Count commas + 1, but be careful of generics
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
        } else if line.contains("interface ") {
            StructureType::Interface
        } else if line.contains("enum ") {
            StructureType::Enum
        } else {
            StructureType::Struct // Default fallback
        }
    }
    
    /// Determine visibility from modifiers
    fn get_visibility(&self, line: &str) -> Visibility {
        if line.contains("public") {
            Visibility::Public
        } else if line.contains("private") {
            Visibility::Private
        } else if line.contains("protected") {
            Visibility::Protected
        } else {
            Visibility::Internal // Java package-private
        }
    }
}

impl LanguageAnalyzer for JavaAnalyzer {
    fn analyze_functions(&self, lines: &[String]) -> Result<Vec<FunctionInfo>> {
        let mut functions = Vec::new();
        let mut current_function: Option<FunctionInfo> = None;
        let mut brace_count = 0;
        let mut in_function = false;
        let mut nesting_level: usize = 0;
        let mut current_class: Option<String> = None;
        
        for (line_num, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            
            // Skip comments and empty lines
            if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.is_empty() {
                continue;
            }
            
            // Track current class for method context
            if self.is_structure_declaration(trimmed) {
                current_class = self.extract_class_name(trimmed);
            }
            
            // Method declaration detection
            if self.is_method_declaration(trimmed) {
                if let Some(func_name) = self.extract_function_name(trimmed) {
                    let param_count = self.count_parameters(trimmed);
                    let is_method = current_class.is_some();
                    
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
                    if trimmed.contains("try") || trimmed.contains("catch") || trimmed.contains("throw") {
                        func.has_exception_handling = true;
                    }
                    
                    // Rough estimate of local variables
                    if trimmed.contains("int ") || trimmed.contains("String ") || 
                       trimmed.contains("boolean ") || trimmed.contains("double ") ||
                       trimmed.contains("float ") || trimmed.contains("long ") {
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
        
        for (line_num, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            
            // Skip comments and empty lines
            if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.is_empty() {
                continue;
            }
            
            // Structure declaration detection
            if self.is_structure_declaration(trimmed) {
                if let Some(struct_name) = self.extract_class_name(trimmed) {
                    let structure_type = self.get_structure_type(trimmed);
                    let visibility = self.get_visibility(trimmed);
                    
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
                }
            }
            
            if in_structure {
                // Count braces to track structure scope
                brace_count += line.matches('{').count() as i32;
                brace_count -= line.matches('}').count() as i32;
                
                if let Some(ref mut structure) = current_structure {
                    structure.line_count += 1;
                    structure.end_line = line_num + 1;
                    
                    // Count properties (field declarations)
                    if (trimmed.contains("private ") || trimmed.contains("public ") || 
                        trimmed.contains("protected ")) && 
                       !trimmed.contains('(') && trimmed.contains(';') {
                        structure.properties += 1;
                    }
                    
                    // Count method declarations within the structure
                    if self.is_method_declaration(trimmed) {
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
                        visibility: Visibility::Public,};
                            structure.methods.push(method_info);
                        }
                    }
                    
                    // Count implemented interfaces
                    if trimmed.contains("implements ") {
                        let after_implements = trimmed.split("implements ").nth(1).unwrap_or("");
                        let interfaces = after_implements.split(',').count();
                        structure.interface_count += interfaces;
                    }
                    
                    // Estimate inheritance depth
                    if trimmed.contains("extends ") {
                        structure.inheritance_depth = 1; // Simplified
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
        
        Ok(structures)
    }
    
    fn language_name(&self) -> &'static str {
        "Java"
    }
    
    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["java"]
    }
}

impl Default for JavaAnalyzer {
    fn default() -> Self {
        Self::new()
    }
} 