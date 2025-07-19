use crate::utils::errors::Result;
use super::super::types::{FunctionInfo, StructureInfo, StructureType, Visibility};
use super::LanguageAnalyzer;

/// Dart language complexity analyzer
pub struct DartAnalyzer;

impl DartAnalyzer {
    pub fn new() -> Self {
        Self
    }
    
    /// Extract function name from Dart function declaration
    fn extract_function_name(&self, line: &str) -> Option<String> {
        let trimmed = line.trim();
        
        // Skip comments and empty lines
        if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.is_empty() {
            return None;
        }
        
        // Handle different function declaration patterns
        let patterns = [
            ("void ", 5),
            ("Future<", 0), // Will be handled separately
            ("Stream<", 0), // Will be handled separately
            ("int ", 4),
            ("String ", 7),
            ("bool ", 5),
            ("double ", 7),
            ("List<", 0), // Will be handled separately
            ("Map<", 0), // Will be handled separately
            ("dynamic ", 8),
            ("var ", 4),
            ("final ", 6),
            ("const ", 6),
            ("static ", 7),
            ("async ", 6),
            ("sync ", 5),
        ];
        
        // Look for function patterns
        for (pattern, offset) in &patterns {
            if let Some(start) = trimmed.find(pattern) {
                let after_pattern = &trimmed[start + offset..];
                
                // For generic types, find the closing bracket
                if pattern.ends_with('<') {
                    if let Some(close_bracket) = after_pattern.find('>') {
                        let after_generic = &after_pattern[close_bracket + 1..].trim();
                        if let Some(func_name) = self.extract_name_before_paren(after_generic) {
                            return Some(func_name);
                        }
                    }
                } else if let Some(func_name) = self.extract_name_before_paren(after_pattern) {
                    return Some(func_name);
                }
            }
        }
        
        // Handle constructor patterns
        if trimmed.contains("(") && !trimmed.contains("=") {
            // Look for class constructor pattern
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            for (_i, part) in parts.iter().enumerate() {
                if part.contains("(") {
                    let name = part.split('(').next().unwrap_or("");
                    if !name.is_empty() && name.chars().all(|c| c.is_alphanumeric() || c == '_') {
                        return Some(name.to_string());
                    }
                }
            }
        }
        
        None
    }
    
    /// Extract name before parentheses
    fn extract_name_before_paren(&self, text: &str) -> Option<String> {
        let trimmed = text.trim();
        if let Some(paren_pos) = trimmed.find('(') {
            let name = trimmed[..paren_pos].trim();
            if !name.is_empty() && name.chars().all(|c| c.is_alphanumeric() || c == '_') {
                return Some(name.to_string());
            }
        }
        None
    }
    
    /// Extract structure name from Dart declaration
    fn extract_structure_name(&self, line: &str) -> Option<String> {
        let trimmed = line.trim();
        
        let structure_keywords = ["class", "abstract", "mixin", "enum", "extension"];
        
        for keyword in &structure_keywords {
            if let Some(start) = trimmed.find(keyword) {
                let after_keyword = &trimmed[start + keyword.len()..];
                let parts: Vec<&str> = after_keyword.split_whitespace().collect();
                
                if let Some(first_part) = parts.first() {
                    // Handle generic parameters and inheritance
                    let name = if let Some(generic_pos) = first_part.find('<') {
                        &first_part[..generic_pos]
                    } else if let Some(extends_pos) = first_part.find(' ') {
                        &first_part[..extends_pos]
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
    
    /// Count complexity keywords in Dart code
    fn count_complexity_keywords(&self, line: &str) -> usize {
        let keywords = [
            "if", "else", "for", "while", "do", "switch", "case", "default",
            "try", "catch", "finally", "throw", "rethrow", "assert",
            "&&", "||", "?", "??", "?..", "await", "async", "sync*", "async*"
        ];
        keywords.iter().map(|&keyword| line.matches(keyword).count()).sum()
    }
    
    /// Count cognitive complexity for Dart code
    fn count_cognitive_complexity(&self, line: &str, nesting_level: usize) -> usize {
        let mut complexity = 0;
        let nesting_multiplier = nesting_level.max(1);
        
        // Basic control structures
        if line.contains("if ") || line.contains("if(") { complexity += 1 * nesting_multiplier; }
        if line.contains("else if") { complexity += 1; }
        if line.contains("else") && !line.contains("else if") { complexity += 1; }
        if line.contains("for ") || line.contains("for(") { complexity += 1 * nesting_multiplier; }
        if line.contains("while ") || line.contains("while(") { complexity += 1 * nesting_multiplier; }
        if line.contains("do ") { complexity += 1 * nesting_multiplier; }
        if line.contains("switch ") || line.contains("switch(") { complexity += 1 * nesting_multiplier; }
        if line.contains("case ") { complexity += 1; }
        if line.contains("default:") { complexity += 1; }
        
        // Logical operators
        complexity += line.matches("&&").count() * nesting_multiplier;
        complexity += line.matches("||").count() * nesting_multiplier;
        
        // Dart-specific operators
        complexity += line.matches("??").count(); // Null coalescing
        complexity += line.matches("?.").count(); // Null-aware operator
        complexity += line.matches("?..").count(); // Null-aware cascade
        complexity += line.matches("?").count(); // Ternary operator
        
        // Exception handling
        if line.contains("try ") { complexity += 1; }
        if line.contains("catch ") { complexity += 1 * nesting_multiplier; }
        if line.contains("finally ") { complexity += 1; }
        if line.contains("throw ") { complexity += 1; }
        if line.contains("rethrow") { complexity += 1; }
        
        // Async/await complexity
        if line.contains("async ") { complexity += 1; }
        if line.contains("await ") { complexity += 1; }
        if line.contains("sync*") { complexity += 1; }
        if line.contains("async*") { complexity += 1; }
        
        // Stream and Future complexity
        if line.contains("Stream<") { complexity += 1; }
        if line.contains("Future<") { complexity += 1; }
        if line.contains(".listen(") { complexity += 1; }
        if line.contains(".then(") { complexity += 1; }
        if line.contains(".catchError(") { complexity += 1; }
        
        // Widget complexity (Flutter specific)
        if line.contains("Widget") || line.contains("StatefulWidget") || 
           line.contains("StatelessWidget") { complexity += 1; }
        if line.contains("build(") { complexity += 1; }
        if line.contains("setState(") { complexity += 1; }
        
        // Collection operations
        if line.contains(".map(") || line.contains(".where(") || 
           line.contains(".reduce(") || line.contains(".fold(") { complexity += 1; }
        
        complexity
    }
    
    /// Check if line contains a function declaration
    fn is_function_declaration(&self, line: &str) -> bool {
        let trimmed = line.trim();
        
        // Skip comments and empty lines
        if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.is_empty() {
            return false;
        }
        
        // Must contain parentheses and not be a call
        if !trimmed.contains('(') {
            return false;
        }
        
        // Check for function patterns
        let function_indicators = [
            "void ", "Future<", "Stream<", "int ", "String ", "bool ", "double ",
            "List<", "Map<", "dynamic ", "var ", "final ", "const ", "static ",
            "async ", "sync ", "get ", "set "
        ];
        
        function_indicators.iter().any(|&indicator| trimmed.contains(indicator)) ||
        // Constructor pattern
        (trimmed.contains("(") && !trimmed.contains("=") && !trimmed.contains("if") && 
         !trimmed.contains("for") && !trimmed.contains("while") && !trimmed.contains("switch"))
    }
    
    /// Check if line contains a structure declaration
    fn is_structure_declaration(&self, line: &str) -> bool {
        let trimmed = line.trim();
        let structure_keywords = ["class", "abstract", "mixin", "enum", "extension"];
        
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
                
                // Count parameters by commas, handling generics and function types
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
        if line.contains("class") || line.contains("abstract") {
            StructureType::Class
        } else if line.contains("mixin") {
            StructureType::Interface // Mixins are similar to interfaces
        } else if line.contains("enum") {
            StructureType::Enum
        } else if line.contains("extension") {
            StructureType::Class // Extensions extend existing types
        } else {
            StructureType::Class
        }
    }
    
    /// Determine visibility from Dart access patterns
    fn get_visibility(&self, line: &str) -> Visibility {
        if line.starts_with('_') || line.contains(" _") {
            Visibility::Private // Dart uses _ for private
        } else {
            Visibility::Public // Default in Dart
        }
    }
    
    /// Check if function is async
    fn is_async(&self, line: &str) -> bool {
        line.contains("async") || line.contains("Future<") || line.contains("Stream<")
    }
    
    /// Check if function is static
    fn is_static(&self, line: &str) -> bool {
        line.contains("static")
    }
    
    /// Check if it's a getter
    fn is_getter(&self, line: &str) -> bool {
        line.contains("get ")
    }
    
    /// Check if it's a setter
    fn is_setter(&self, line: &str) -> bool {
        line.contains("set ")
    }
    
    /// Check if it's a constructor
    fn is_constructor(&self, line: &str, class_name: &str) -> bool {
        line.contains(class_name) && line.contains("(") && !line.contains("=")
    }
    
    /// Check if it's a Widget (Flutter specific)
    fn is_widget(&self, line: &str) -> bool {
        line.contains("Widget") || line.contains("StatefulWidget") || 
        line.contains("StatelessWidget")
    }
    
    /// Count inheritance/mixin usage from declaration
    fn count_inheritance_depth(&self, line: &str) -> usize {
        let mut depth = 0;
        
        // Count extends
        if line.contains(" extends ") {
            depth += 1;
        }
        
        // Count implements
        if line.contains(" implements ") {
            if let Some(implements_pos) = line.find(" implements ") {
                let after_implements = &line[implements_pos + 12..];
                let interfaces: Vec<&str> = after_implements.split(',').collect();
                depth += interfaces.len();
            }
        }
        
        // Count with (mixins)
        if line.contains(" with ") {
            if let Some(with_pos) = line.find(" with ") {
                let after_with = &line[with_pos + 6..];
                let mixins: Vec<&str> = after_with.split(',').collect();
                depth += mixins.len();
            }
        }
        
        depth
    }
}

impl LanguageAnalyzer for DartAnalyzer {
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
                    let _is_async = self.is_async(trimmed);
                    let is_static = self.is_static(trimmed);
                    let _is_getter = self.is_getter(trimmed);
                    let _is_setter = self.is_setter(trimmed);
                    
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
                       trimmed.contains("finally") || trimmed.contains("throw") ||
                       trimmed.contains("rethrow") {
                        func.has_exception_handling = true;
                    }
                    
                    // Count local variables
                    if (trimmed.contains("var ") || trimmed.contains("final ") || 
                        trimmed.contains("const ") || trimmed.contains("late ")) &&
                       !trimmed.contains("class ") && !trimmed.contains("mixin ") {
                        func.local_variable_count += 1;
                    }
                }
                
                // End of function (handle arrow functions and single-line functions)
                if (brace_count <= 0 && in_function) || 
                   (trimmed.contains("=>") && !trimmed.contains("{")) {
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
                    
                    // Count properties (field declarations)
                    if (trimmed.contains("final ") || trimmed.contains("var ") || 
                        trimmed.contains("const ") || trimmed.contains("late ") ||
                        trimmed.contains("static ")) &&
                       !self.is_function_declaration(trimmed) &&
                       !trimmed.contains("class ") && !trimmed.contains("mixin ") {
                        structure.properties += 1;
                    }
                    
                    // Count mixin methods for mixins
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
            
            if self.is_function_declaration(trimmed) {
                if let Some(func_name) = self.extract_function_name(trimmed) {
                    let param_count = self.count_parameters(trimmed);
                    let _is_async = self.is_async(trimmed);
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
        "Dart"
    }
    
    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["dart"]
    }
}

impl Default for DartAnalyzer {
    fn default() -> Self {
        Self::new()
    }
} 