use crate::utils::errors::Result;
use super::super::types::{FunctionInfo, StructureInfo, StructureType, Visibility};
use super::LanguageAnalyzer;

/// Ruby language complexity analyzer
pub struct RubyAnalyzer;

impl RubyAnalyzer {
    pub fn new() -> Self {
        Self
    }
    
    /// Extract method name from Ruby method definition
    fn extract_method_name(&self, line: &str) -> Option<String> {
        let trimmed = line.trim();
        
        // Skip comments and empty lines
        if trimmed.starts_with('#') || trimmed.is_empty() {
            return None;
        }
        
        // Look for "def " pattern
        if let Some(start) = trimmed.find("def ") {
            let after_def = &trimmed[start + 4..];
            
            // Handle method names with parameters
            let method_part = after_def.trim();
            
            // Find method name (before parentheses or parameters)
            let end_pos = method_part.find('(')
                .or_else(|| method_part.find(' '))
                .unwrap_or(method_part.len());
            
            let method_name = &method_part[..end_pos];
            
            if !method_name.is_empty() && 
               method_name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '?' || c == '!') {
                return Some(method_name.to_string());
            }
        }
        None
    }
    
    /// Extract class/module name from Ruby declaration
    fn extract_structure_name(&self, line: &str) -> Option<String> {
        let trimmed = line.trim();
        
        let structure_keywords = ["class", "module"];
        
        for keyword in &structure_keywords {
            if let Some(start) = trimmed.find(keyword) {
                let after_keyword = &trimmed[start + keyword.len()..];
                let parts: Vec<&str> = after_keyword.split_whitespace().collect();
                
                if let Some(first_part) = parts.first() {
                    // Handle inheritance
                    let name = if let Some(inheritance_pos) = first_part.find('<') {
                        &first_part[..inheritance_pos]
                    } else {
                        first_part
                    };
                    
                    if !name.is_empty() && name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == ':') {
                        return Some(name.to_string());
                    }
                }
            }
        }
        None
    }
    
    /// Count complexity keywords in Ruby code
    fn count_complexity_keywords(&self, line: &str) -> usize {
        let keywords = [
            "if", "elsif", "else", "unless", "for", "while", "until", "case", "when",
            "&&", "||", "and", "or", "not", "?", "rescue", "ensure", "raise", "begin"
        ];
        keywords.iter().map(|&keyword| line.matches(keyword).count()).sum()
    }
    
    /// Count cognitive complexity for Ruby code
    fn count_cognitive_complexity(&self, line: &str, nesting_level: usize) -> usize {
        let mut complexity = 0;
        let nesting_multiplier = nesting_level.max(1);
        
        // Basic control structures
        if line.contains("if ") || line.contains("if(") { complexity += 1 * nesting_multiplier; }
        if line.contains("elsif ") { complexity += 1; }
        if line.contains("else") && !line.contains("elsif") { complexity += 1; }
        if line.contains("unless ") { complexity += 1 * nesting_multiplier; }
        if line.contains("for ") { complexity += 1 * nesting_multiplier; }
        if line.contains("while ") { complexity += 1 * nesting_multiplier; }
        if line.contains("until ") { complexity += 1 * nesting_multiplier; }
        if line.contains("case ") { complexity += 1 * nesting_multiplier; }
        if line.contains("when ") { complexity += 1; }
        
        // Logical operators
        complexity += line.matches("&&").count() * nesting_multiplier;
        complexity += line.matches("||").count() * nesting_multiplier;
        complexity += line.matches(" and ").count() * nesting_multiplier;
        complexity += line.matches(" or ").count() * nesting_multiplier;
        complexity += line.matches(" not ").count() * nesting_multiplier;
        
        // Ternary operator
        complexity += line.matches('?').count() * nesting_multiplier;
        
        // Exception handling
        if line.contains("begin ") { complexity += 1 * nesting_multiplier; }
        if line.contains("rescue ") { complexity += 1 * nesting_multiplier; }
        if line.contains("ensure ") { complexity += 1; }
        if line.contains("raise ") { complexity += 1; }
        
        // Ruby-specific complexity
        if line.contains("yield") { complexity += 1; } // Block yielding
        if line.contains(".each") || line.contains(".map") || line.contains(".select") { complexity += 1; }
        if line.contains("lambda") || line.contains("proc") || line.contains("->") { complexity += 1; }
        if line.contains("eval") { complexity += 3; } // eval is very complex
        if line.contains("define_method") || line.contains("method_missing") { complexity += 2; }
        
        // Metaprogramming adds complexity
        if line.contains("class_eval") || line.contains("instance_eval") || 
           line.contains("module_eval") { complexity += 2; }
        
        complexity
    }
    
    /// Check if line contains a method definition
    fn is_method_definition(&self, line: &str) -> bool {
        let trimmed = line.trim();
        
        // Skip comments and empty lines
        if trimmed.starts_with('#') || trimmed.is_empty() {
            return false;
        }
        
        // Must start with "def "
        trimmed.starts_with("def ")
    }
    
    /// Check if line contains a structure declaration
    fn is_structure_declaration(&self, line: &str) -> bool {
        let trimmed = line.trim();
        let structure_keywords = ["class", "module"];
        
        structure_keywords.iter().any(|&keyword| {
            trimmed.starts_with(keyword) && 
            !trimmed.starts_with('#')
        })
    }
    
    /// Count parameters in method signature
    fn count_parameters(&self, line: &str) -> usize {
        // Ruby methods can have parameters without parentheses
        if let Some(def_pos) = line.find("def ") {
            let after_def = &line[def_pos + 4..];
            
            // Check if there are parentheses
            if let Some(paren_start) = after_def.find('(') {
                if let Some(paren_end) = after_def.find(')') {
                    let params = &after_def[paren_start + 1..paren_end];
                    if params.trim().is_empty() {
                        return 0;
                    }
                    return params.split(',').count();
                }
            } else {
                // No parentheses, check for parameters after method name
                let parts: Vec<&str> = after_def.split_whitespace().collect();
                if parts.len() > 1 {
                    // Simple heuristic: count space-separated parts after method name
                    return parts.len() - 1;
                }
            }
        }
        0
    }
    
    /// Determine structure type from declaration
    fn get_structure_type(&self, line: &str) -> StructureType {
        if line.contains("class") {
            StructureType::Class
        } else if line.contains("module") {
            StructureType::Module
        } else {
            StructureType::Class
        }
    }
    
    /// Determine visibility (Ruby uses method calls for visibility)
    fn get_visibility(&self, _line: &str) -> Visibility {
        // Ruby visibility is usually determined by method calls like private, protected, public
        // For structure declarations, everything is public by default
        Visibility::Public
    }
    
    /// Check if method is a class method (self.method_name)
    fn is_class_method(&self, line: &str) -> bool {
        line.contains("def self.") || line.contains("def ") && line.contains("self.")
    }
    
    /// Check if it's a special Ruby method
    fn is_special_method(&self, name: &str) -> bool {
        let special_methods = [
            "initialize", "new", "to_s", "to_str", "inspect", "hash", "eql?", "equal?",
            "method_missing", "respond_to?", "send", "class", "instance_of?", "kind_of?"
        ];
        special_methods.contains(&name)
    }
    
    /// Count inheritance depth from class declaration
    fn count_inheritance_depth(&self, line: &str) -> usize {
        // Ruby single inheritance with < symbol
        if line.contains('<') {
            return 1;
        }
        0
    }
    
    /// Check if line ends a block (contains 'end')
    fn is_block_end(&self, line: &str) -> bool {
        let trimmed = line.trim();
        trimmed == "end" || trimmed.starts_with("end ")
    }
}

impl LanguageAnalyzer for RubyAnalyzer {
    fn analyze_functions(&self, lines: &[String]) -> Result<Vec<FunctionInfo>> {
        let mut functions = Vec::new();
        let mut current_function: Option<FunctionInfo> = None;
        let mut block_depth = 0;
        let mut in_function = false;
        let mut nesting_level: usize = 0;
        let mut in_comment_block = false;
        
        for (line_num, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            
            // Handle multi-line comments (=begin/=end)
            if trimmed.starts_with("=begin") {
                in_comment_block = true;
                continue;
            }
            if trimmed.starts_with("=end") {
                in_comment_block = false;
                continue;
            }
            if in_comment_block {
                continue;
            }
            
            // Skip single-line comments and empty lines
            if trimmed.starts_with('#') || trimmed.is_empty() {
                continue;
            }
            
            // Method definition detection
            if self.is_method_definition(trimmed) {
                if let Some(method_name) = self.extract_method_name(trimmed) {
                    let param_count = self.count_parameters(trimmed);
                    let is_class_method = self.is_class_method(trimmed);
                    let _is_special = self.is_special_method(&method_name);
                    
                    current_function = Some(FunctionInfo {
                        name: method_name,
                        line_count: 0,
                        cyclomatic_complexity: 1, // Base complexity
                        cognitive_complexity: 1, // Base cognitive complexity
                        nesting_depth: 0,
                        parameter_count: param_count,
                        return_path_count: 0,
                        start_line: line_num + 1,
                        end_line: line_num + 1,
                        is_method: !is_class_method,
                        parent_class: None, // Will be set later
                        local_variable_count: 0,
                        has_recursion: false,
                        has_exception_handling: false,
                        visibility: Visibility::Public,});
                    in_function = true;
                    block_depth = 0;
                    nesting_level = 0;
                }
            }
            
            if in_function {
                // Track block depth (def/class/module/if/while/etc. increase, end decreases)
                if trimmed.starts_with("def ") || trimmed.starts_with("class ") || 
                   trimmed.starts_with("module ") || trimmed.contains("if ") || 
                   trimmed.contains("while ") || trimmed.contains("for ") ||
                   trimmed.contains("case ") || trimmed.contains("begin ") {
                    block_depth += 1;
                    nesting_level += 1;
                }
                
                if self.is_block_end(trimmed) {
                    block_depth -= 1;
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
                    
                    // Count return statements (Ruby has implicit returns)
                    if trimmed.contains("return") {
                        func.return_path_count += 1;
                    }
                    
                    // Check for recursion
                    if trimmed.contains(&func.name) && 
                       (trimmed.contains('(') || trimmed.contains(' ')) {
                        func.has_recursion = true;
                    }
                    
                    // Check for exception handling
                    if trimmed.contains("begin") || trimmed.contains("rescue") || 
                       trimmed.contains("ensure") || trimmed.contains("raise") {
                        func.has_exception_handling = true;
                    }
                    
                    // Count local variables (rough estimate)
                    if trimmed.contains(" = ") && !trimmed.contains("def ") &&
                       !trimmed.contains("class ") && !trimmed.contains("module ") {
                        func.local_variable_count += 1;
                    }
                }
                
                // End of method
                if block_depth <= 0 && in_function && self.is_block_end(trimmed) {
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
        let mut block_depth = 0;
        let mut in_structure = false;
        let mut in_comment_block = false;
        
        for (line_num, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            
            // Handle multi-line comments (=begin/=end)
            if trimmed.starts_with("=begin") {
                in_comment_block = true;
                continue;
            }
            if trimmed.starts_with("=end") {
                in_comment_block = false;
                continue;
            }
            if in_comment_block {
                continue;
            }
            
            // Skip single-line comments and empty lines
            if trimmed.starts_with('#') || trimmed.is_empty() {
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
                    block_depth = 0;
                }
            }
            
            if in_structure {
                // Track block depth
                if trimmed.starts_with("def ") || trimmed.starts_with("class ") || 
                   trimmed.starts_with("module ") || trimmed.contains("if ") || 
                   trimmed.contains("while ") || trimmed.contains("for ") ||
                   trimmed.contains("case ") || trimmed.contains("begin ") {
                    block_depth += 1;
                }
                
                if self.is_block_end(trimmed) {
                    block_depth -= 1;
                }
                
                if let Some(ref mut structure) = current_structure {
                    structure.line_count += 1;
                    structure.end_line = line_num + 1;
                    
                    // Count instance variables and constants
                    if trimmed.starts_with('@') || trimmed.starts_with("@@") ||
                       (trimmed.chars().next().map_or(false, |c| c.is_uppercase()) &&
                        trimmed.contains(" = ")) {
                        structure.properties += 1;
                    }
                    
                    // Count attribute accessors
                    if trimmed.contains("attr_reader") || trimmed.contains("attr_writer") ||
                       trimmed.contains("attr_accessor") {
                        // Count symbols in attr_* declarations
                        let symbols = trimmed.matches(':').count();
                        structure.properties += symbols;
                    }
                }
                
                // End of structure
                if block_depth <= 0 && in_structure && self.is_block_end(trimmed) {
                    if let Some(structure) = current_structure.take() {
                        structures.push(structure);
                    }
                    in_structure = false;
                }
            }
        }
        
        // Find methods that belong to structures
        let mut current_class: Option<String> = None;
        let mut class_depth = 0;
        
        for (line_num, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            
            // Track current class context
            if self.is_structure_declaration(trimmed) {
                if let Some(class_name) = self.extract_structure_name(trimmed) {
                    current_class = Some(class_name);
                    class_depth = 0;
                }
            }
            
            // Track depth to know when we exit a class
            if trimmed.starts_with("class ") || trimmed.starts_with("module ") {
                class_depth += 1;
            }
            if self.is_block_end(trimmed) {
                class_depth -= 1;
                if class_depth <= 0 {
                    current_class = None;
                }
            }
            
            if self.is_method_definition(trimmed) {
                if let Some(method_name) = self.extract_method_name(trimmed) {
                    let param_count = self.count_parameters(trimmed);
                    let is_class_method = self.is_class_method(trimmed);
                    
                    let method_info = FunctionInfo {
                        name: method_name,
                        line_count: 0, // Would need separate tracking
                        cyclomatic_complexity: 1,
                        cognitive_complexity: 1,
                        nesting_depth: 0,
                        parameter_count: param_count,
                        return_path_count: 0,
                        start_line: line_num + 1,
                        end_line: line_num + 1,
                        is_method: !is_class_method,
                        parent_class: current_class.clone(),
                        local_variable_count: 0,
                        has_recursion: false,
                        has_exception_handling: false,
                        visibility: Visibility::Public,};
                    
                    // Add method to corresponding structure
                    if let Some(ref class_name) = current_class {
                        for structure in &mut structures {
                            if structure.name == *class_name {
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
        "Ruby"
    }
    
    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["rb", "rbw", "rake", "gemspec"]
    }
}

impl Default for RubyAnalyzer {
    fn default() -> Self {
        Self::new()
    }
} 