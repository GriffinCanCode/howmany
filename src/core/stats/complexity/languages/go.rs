use crate::utils::errors::Result;
use super::super::types::{FunctionInfo, StructureInfo, StructureType, Visibility};
use super::LanguageAnalyzer;

/// Go language complexity analyzer
pub struct GoAnalyzer;

impl GoAnalyzer {
    pub fn new() -> Self {
        Self
    }
    
    /// Extract function name from Go function declaration
    fn extract_function_name(&self, line: &str) -> Option<String> {
        let trimmed = line.trim();
        
        // Skip comments and empty lines
        if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.is_empty() {
            return None;
        }
        
        // Look for "func " pattern
        if let Some(start) = trimmed.find("func ") {
            let after_func = &trimmed[start + 5..];
            
            // Handle method receivers: func (r *Receiver) methodName(
            if after_func.starts_with('(') {
                if let Some(end_paren) = after_func.find(')') {
                    let after_receiver = &after_func[end_paren + 1..].trim();
                    if let Some(paren_pos) = after_receiver.find('(') {
                        let method_name = after_receiver[..paren_pos].trim();
                        if !method_name.is_empty() {
                            return Some(method_name.to_string());
                        }
                    }
                }
            } else {
                // Regular function: func functionName(
                if let Some(paren_pos) = after_func.find('(') {
                    let func_name = after_func[..paren_pos].trim();
                    if !func_name.is_empty() {
                        return Some(func_name.to_string());
                    }
                }
            }
        }
        None
    }
    
    /// Extract struct/interface name from Go declaration
    fn extract_struct_name(&self, line: &str) -> Option<String> {
        let trimmed = line.trim();
        
        // Look for "type Name struct" or "type Name interface"
        if trimmed.starts_with("type ") {
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.len() >= 3 && (parts[2] == "struct" || parts[2] == "interface") {
                return Some(parts[1].to_string());
            }
        }
        None
    }
    
    /// Count complexity keywords in Go code
    fn count_complexity_keywords(&self, line: &str) -> usize {
        let keywords = ["if", "else if", "for", "switch", "case", "select", "&&", "||", "go ", "defer"];
        keywords.iter().map(|&keyword| line.matches(keyword).count()).sum()
    }
    
    /// Count cognitive complexity for Go code
    fn count_cognitive_complexity(&self, line: &str, nesting_level: usize) -> usize {
        let mut complexity = 0;
        let nesting_multiplier = nesting_level.max(1);
        
        // Basic control structures
        if line.contains("if ") { complexity += 1 * nesting_multiplier; }
        if line.contains("else if") { complexity += 1; }
        if line.contains("else") && !line.contains("else if") { complexity += 1; }
        if line.contains("for ") { complexity += 1 * nesting_multiplier; }
        if line.contains("switch ") { complexity += 1 * nesting_multiplier; }
        if line.contains("case ") { complexity += 1; }
        if line.contains("select ") { complexity += 1 * nesting_multiplier; }
        
        // Logical operators
        complexity += line.matches("&&").count() * nesting_multiplier;
        complexity += line.matches("||").count() * nesting_multiplier;
        
        // Go-specific complexity
        if line.contains("go ") { complexity += 1; } // Goroutines add complexity
        if line.contains("defer ") { complexity += 1; } // Defer statements
        if line.contains("panic(") { complexity += 2; } // Panic adds significant complexity
        if line.contains("recover(") { complexity += 1; } // Recovery handling
        
        // Channel operations
        if line.contains("<-") { complexity += 1; } // Channel send/receive
        
        complexity
    }
    
    /// Check if line contains a function declaration
    fn is_function_declaration(&self, line: &str) -> bool {
        let trimmed = line.trim();
        
        // Skip comments and empty lines
        if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.is_empty() {
            return false;
        }
        
        // Must start with "func "
        trimmed.starts_with("func ") && trimmed.contains('(')
    }
    
    /// Check if line contains a struct/interface declaration
    fn is_structure_declaration(&self, line: &str) -> bool {
        let trimmed = line.trim();
        trimmed.starts_with("type ") && 
        (trimmed.contains(" struct") || trimmed.contains(" interface"))
    }
    
    /// Count parameters in function signature
    fn count_parameters(&self, line: &str) -> usize {
        if let Some(start) = line.find('(') {
            if let Some(end) = line.find(')') {
                let params = &line[start + 1..end];
                if params.trim().is_empty() {
                    return 0;
                }
                
                // Go parameters can be grouped: func(a, b int, c string)
                // Count parameter groups by looking for type keywords
                let param_str = params.trim();
                if param_str.is_empty() {
                    return 0;
                }
                
                // Simple heuristic: count commas + 1, but account for grouped params
                let comma_count = param_str.matches(',').count();
                if comma_count == 0 {
                    return 1;
                }
                
                // More sophisticated counting would require parsing Go syntax
                return comma_count + 1;
            }
        }
        0
    }
    
    /// Determine structure type from declaration
    fn get_structure_type(&self, line: &str) -> StructureType {
        if line.contains(" struct") {
            StructureType::Struct
        } else if line.contains(" interface") {
            StructureType::Interface
        } else {
            StructureType::Struct
        }
    }
    
    /// Determine visibility from Go naming conventions
    fn get_visibility(&self, name: &str) -> Visibility {
        // In Go, exported (public) names start with uppercase
        if name.chars().next().map_or(false, |c| c.is_uppercase()) {
            Visibility::Public
        } else {
            Visibility::Private
        }
    }
    
    /// Check if function is a method (has receiver)
    fn is_method(&self, line: &str) -> bool {
        if let Some(start) = line.find("func ") {
            let after_func = &line[start + 5..];
            after_func.starts_with('(') && after_func.contains(')')
        } else {
            false
        }
    }
    
    /// Extract receiver type from method declaration
    fn extract_receiver_type(&self, line: &str) -> Option<String> {
        if let Some(start) = line.find("func (") {
            let after_func = &line[start + 6..];
            if let Some(end) = after_func.find(')') {
                let receiver = &after_func[..end];
                // Parse receiver like "r *Receiver" or "r Receiver"
                let parts: Vec<&str> = receiver.split_whitespace().collect();
                if parts.len() >= 2 {
                    let receiver_type = parts[1].trim_start_matches('*');
                    return Some(receiver_type.to_string());
                }
            }
        }
        None
    }
}

impl LanguageAnalyzer for GoAnalyzer {
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
                    let is_method = self.is_method(trimmed);
                    let parent_class = if is_method {
                        self.extract_receiver_type(trimmed)
                    } else {
                        None
                    };
                    
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
                        parent_class,
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
                    
                    // Check for error handling (Go's way of exception handling)
                    if trimmed.contains("panic(") || trimmed.contains("recover(") || 
                       trimmed.contains("if err != nil") {
                        func.has_exception_handling = true;
                    }
                    
                    // Rough estimate of local variables
                    if trimmed.contains(":=") || 
                       (trimmed.contains("var ") && !trimmed.contains("func")) {
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
                if let Some(struct_name) = self.extract_struct_name(trimmed) {
                    let structure_type = self.get_structure_type(trimmed);
                    let visibility = self.get_visibility(&struct_name);
                    
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
                    
                    // Count fields in struct or methods in interface
                    if structure.structure_type == StructureType::Struct {
                        // Count struct fields (simple heuristic)
                        if !trimmed.is_empty() && 
                           !trimmed.starts_with("//") && 
                           !trimmed.starts_with("/*") &&
                           !trimmed.starts_with("type ") &&
                           !trimmed.starts_with("func ") &&
                           !trimmed.starts_with("}") &&
                           !trimmed.starts_with("{") &&
                           trimmed.contains(' ') {
                            structure.properties += 1;
                        }
                    } else if structure.structure_type == StructureType::Interface {
                        // Count interface methods
                        if trimmed.contains('(') && trimmed.contains(')') &&
                           !trimmed.starts_with("//") && 
                           !trimmed.starts_with("/*") {
                            structure.interface_count += 1;
                        }
                    }
                    
                    // Go doesn't have traditional inheritance, but interfaces can embed other interfaces
                    if structure.structure_type == StructureType::Interface && 
                       !trimmed.contains('(') && !trimmed.contains(')') &&
                       !trimmed.is_empty() && !trimmed.starts_with("//") {
                        structure.inheritance_depth += 1;
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
                    if let Some(receiver_type) = self.extract_receiver_type(trimmed) {
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
                            parent_class: Some(receiver_type.clone()),
                            local_variable_count: 0,
                            has_recursion: false,
                            has_exception_handling: false,
                        visibility: Visibility::Public,};
                        
                        // Add method to corresponding structure
                        for structure in &mut structures {
                            if structure.name == receiver_type {
                                structure.methods.push(method_info.clone());
                                break;
                            }
                        }
                    }
                }
            }
        }
        
        Ok(structures)
    }
    
    fn language_name(&self) -> &'static str {
        "Go"
    }
    
    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["go"]
    }
}

impl Default for GoAnalyzer {
    fn default() -> Self {
        Self::new()
    }
} 