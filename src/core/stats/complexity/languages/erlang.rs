use crate::utils::errors::Result;
use super::super::types::{FunctionInfo, StructureInfo, StructureType, Visibility};
use super::LanguageAnalyzer;

/// Erlang language complexity analyzer
pub struct ErlangAnalyzer;

impl ErlangAnalyzer {
    pub fn new() -> Self {
        Self
    }
    
    /// Extract function name from Erlang function declaration
    fn extract_function_name(&self, line: &str) -> Option<String> {
        let trimmed = line.trim();
        
        // Skip comments and empty lines
        if trimmed.starts_with("%") || trimmed.is_empty() {
            return None;
        }
        
        // Look for function patterns: function_name(Args) ->
        if let Some(arrow_pos) = trimmed.find("->") {
            let before_arrow = &trimmed[..arrow_pos].trim();
            
            // Find function name before parentheses
            if let Some(paren_pos) = before_arrow.find('(') {
                let func_name = before_arrow[..paren_pos].trim();
                
                // Remove any guards or when clauses
                let clean_name = if let Some(when_pos) = func_name.find(" when ") {
                    func_name[..when_pos].trim()
                } else {
                    func_name
                };
                
                if !clean_name.is_empty() && 
                   clean_name.chars().all(|c| c.is_alphanumeric() || c == '_') {
                    return Some(clean_name.to_string());
                }
            }
        }
        
        None
    }
    
    /// Extract module name from Erlang module declaration
    fn extract_module_name(&self, line: &str) -> Option<String> {
        let trimmed = line.trim();
        
        // Look for -module(module_name).
        if trimmed.starts_with("-module(") && trimmed.ends_with(").") {
            let module_part = &trimmed[8..trimmed.len()-2];
            if !module_part.is_empty() && 
               module_part.chars().all(|c| c.is_alphanumeric() || c == '_') {
                return Some(module_part.to_string());
            }
        }
        
        None
    }
    
    /// Extract behavior name from Erlang behavior declaration
    fn extract_behavior_name(&self, line: &str) -> Option<String> {
        let trimmed = line.trim();
        
        // Look for -behaviour(behavior_name). or -behavior(behavior_name).
        if (trimmed.starts_with("-behaviour(") || trimmed.starts_with("-behavior(")) 
           && trimmed.ends_with(").") {
            let start_pos = if trimmed.starts_with("-behaviour(") { 11 } else { 10 };
            let behavior_part = &trimmed[start_pos..trimmed.len()-2];
            if !behavior_part.is_empty() && 
               behavior_part.chars().all(|c| c.is_alphanumeric() || c == '_') {
                return Some(behavior_part.to_string());
            }
        }
        
        None
    }
    
    /// Count complexity keywords in Erlang code
    fn count_complexity_keywords(&self, line: &str) -> usize {
        let keywords = [
            "if", "case", "receive", "try", "catch", "after", "when",
            "andalso", "orelse", "not", "and", "or", "xor",
            "throw", "exit", "error", "spawn", "spawn_link", "spawn_monitor"
        ];
        keywords.iter().map(|&keyword| line.matches(keyword).count()).sum()
    }
    
    /// Count cognitive complexity for Erlang code
    fn count_cognitive_complexity(&self, line: &str, nesting_level: usize) -> usize {
        let mut complexity = 0;
        let nesting_multiplier = nesting_level.max(1);
        
        // Basic control structures
        if line.contains("if ") { complexity += 1 * nesting_multiplier; }
        if line.contains("case ") { complexity += 1 * nesting_multiplier; }
        if line.contains("receive") { complexity += 1 * nesting_multiplier; }
        if line.contains("try ") { complexity += 1 * nesting_multiplier; }
        if line.contains("catch ") { complexity += 1 * nesting_multiplier; }
        if line.contains("after ") { complexity += 1; }
        
        // Pattern matching in function heads
        if line.contains("->") && !line.contains("fun") { complexity += 1; }
        
        // Guards add complexity
        if line.contains(" when ") { complexity += 1 * nesting_multiplier; }
        
        // Logical operators
        complexity += line.matches("andalso").count() * nesting_multiplier;
        complexity += line.matches("orelse").count() * nesting_multiplier;
        complexity += line.matches(" and ").count() * nesting_multiplier;
        complexity += line.matches(" or ").count() * nesting_multiplier;
        complexity += line.matches(" xor ").count() * nesting_multiplier;
        
        // Exception handling
        if line.contains("throw ") { complexity += 1; }
        if line.contains("exit(") { complexity += 1; }
        if line.contains("error(") { complexity += 1; }
        
        // Process spawning adds complexity
        if line.contains("spawn(") { complexity += 1; }
        if line.contains("spawn_link(") { complexity += 1; }
        if line.contains("spawn_monitor(") { complexity += 1; }
        
        // Message passing
        if line.contains("!") { complexity += 1; } // Send operator
        
        // List comprehensions
        if line.contains("||") && line.contains("[") { complexity += 1; }
        
        // Binary comprehensions
        if line.contains("<=") && line.contains("<<") { complexity += 1; }
        
        // Fun expressions
        if line.contains("fun ") { complexity += 1; }
        
        complexity
    }
    
    /// Check if line contains a function declaration
    fn is_function_declaration(&self, line: &str) -> bool {
        let trimmed = line.trim();
        
        // Skip comments and empty lines
        if trimmed.starts_with("%") || trimmed.is_empty() {
            return false;
        }
        
        // Must contain -> and parentheses
        trimmed.contains("->") && trimmed.contains('(') && !trimmed.starts_with("-")
    }
    
    /// Check if line contains a module declaration
    fn is_module_declaration(&self, line: &str) -> bool {
        let trimmed = line.trim();
        trimmed.starts_with("-module(") && trimmed.ends_with(").")
    }
    
    /// Check if line contains a behavior declaration
    fn is_behavior_declaration(&self, line: &str) -> bool {
        let trimmed = line.trim();
        (trimmed.starts_with("-behaviour(") || trimmed.starts_with("-behavior(")) 
        && trimmed.ends_with(").")
    }
    
    /// Count parameters in function signature
    fn count_parameters(&self, line: &str) -> usize {
        if let Some(start) = line.find('(') {
            if let Some(end) = line.find(')') {
                let params = &line[start + 1..end];
                if params.trim().is_empty() {
                    return 0;
                }
                
                // Count parameters by commas, handling nested structures
                let mut param_count = 1;
                let mut paren_depth = 0;
                let mut bracket_depth = 0;
                let mut brace_depth = 0;
                
                for ch in params.chars() {
                    match ch {
                        '(' => paren_depth += 1,
                        ')' => paren_depth -= 1,
                        '[' => bracket_depth += 1,
                        ']' => bracket_depth -= 1,
                        '{' => brace_depth += 1,
                        '}' => brace_depth -= 1,
                        ',' if paren_depth == 0 && bracket_depth == 0 && brace_depth == 0 => {
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
    
    /// Determine visibility (all functions are public in Erlang unless unexported)
    fn get_visibility(&self, _line: &str) -> Visibility {
        Visibility::Public // Default in Erlang
    }
    
    /// Check if function is exported
    fn is_exported(&self, _line: &str) -> bool {
        // Would need to check export list, simplified for now
        true
    }
    
    /// Check if line contains a process spawn
    fn is_process_spawn(&self, line: &str) -> bool {
        line.contains("spawn(") || line.contains("spawn_link(") || line.contains("spawn_monitor(")
    }
    
    /// Check if line contains message passing
    fn has_message_passing(&self, line: &str) -> bool {
        line.contains("!") || line.contains("receive")
    }
    
    /// Check if line contains pattern matching
    fn has_pattern_matching(&self, line: &str) -> bool {
        line.contains("=") && !line.contains("==") && !line.contains("=/=")
    }
    
    /// Check if line contains guards
    fn has_guards(&self, line: &str) -> bool {
        line.contains(" when ")
    }
    
    /// Check if line contains list comprehension
    fn has_list_comprehension(&self, line: &str) -> bool {
        line.contains("||") && line.contains("[")
    }
    
    /// Check if line contains binary comprehension
    fn has_binary_comprehension(&self, line: &str) -> bool {
        line.contains("<=") && line.contains("<<")
    }
    
    /// Check if line contains fun expression
    fn has_fun_expression(&self, line: &str) -> bool {
        line.contains("fun ")
    }
    
    /// Count clauses in function (multiple function heads)
    fn count_function_clauses(&self, lines: &[String], start_line: usize, func_name: &str) -> usize {
        let mut clause_count = 0;
        let mut i = start_line;
        
        while i < lines.len() {
            let line = &lines[i];
            if line.trim().starts_with(&format!("{}(", func_name)) {
                clause_count += 1;
            } else if !line.trim().is_empty() && 
                      !line.trim().starts_with("%") && 
                      !line.trim().starts_with(&format!("{}(", func_name)) {
                break;
            }
            i += 1;
        }
        
        clause_count.max(1)
    }
}

impl LanguageAnalyzer for ErlangAnalyzer {
    fn analyze_functions(&self, lines: &[String]) -> Result<Vec<FunctionInfo>> {
        let mut functions = Vec::new();
        let mut current_function: Option<FunctionInfo> = None;
        let mut in_function = false;
        let mut nesting_level: usize = 0;
        let mut function_end_patterns = 0;
        
        for (line_num, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            
            // Skip comments and empty lines
            if trimmed.starts_with("%") || trimmed.is_empty() {
                continue;
            }
            
            // Function declaration detection
            if self.is_function_declaration(trimmed) {
                if let Some(func_name) = self.extract_function_name(trimmed) {
                    let param_count = self.count_parameters(trimmed);
                    let _clause_count = self.count_function_clauses(lines, line_num, &func_name);
                    
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
                        is_method: false, // Erlang doesn't have methods
                        parent_class: None,
                        local_variable_count: 0,
                        has_recursion: false,
                        has_exception_handling: false,
                        visibility: Visibility::Public,});
                    in_function = true;
                    nesting_level = 0;
                    function_end_patterns = 0;
                }
            }
            
            if in_function {
                if let Some(ref mut func) = current_function {
                    func.line_count += 1;
                    func.end_line = line_num + 1;
                    
                    // Track nesting level
                    if trimmed.contains("case ") || trimmed.contains("if ") || 
                       trimmed.contains("receive") || trimmed.contains("try ") {
                        nesting_level += 1;
                    }
                    if trimmed.contains("end") {
                        nesting_level = nesting_level.saturating_sub(1);
                    }
                    
                    func.nesting_depth = func.nesting_depth.max(nesting_level);
                    
                    // Add complexity from keywords
                    let keyword_complexity = self.count_complexity_keywords(trimmed);
                    func.cyclomatic_complexity += keyword_complexity;
                    
                    // Add cognitive complexity
                    let cognitive_complexity = self.count_cognitive_complexity(trimmed, nesting_level);
                    func.cognitive_complexity += cognitive_complexity;
                    
                    // Count pattern matching as return paths
                    if trimmed.contains("->") {
                        func.return_path_count += 1;
                    }
                    
                    // Check for recursion
                    if trimmed.contains(&func.name) && trimmed.contains('(') {
                        func.has_recursion = true;
                    }
                    
                    // Check for exception handling
                    if trimmed.contains("try") || trimmed.contains("catch") || 
                       trimmed.contains("throw") || trimmed.contains("exit") ||
                       trimmed.contains("error") {
                        func.has_exception_handling = true;
                    }
                    
                    // Count local variables (pattern matching assignments)
                    if self.has_pattern_matching(trimmed) && !trimmed.contains("->") {
                        func.local_variable_count += 1;
                    }
                    
                    // Check for function end (period followed by newline or another function)
                    if trimmed.ends_with('.') {
                        function_end_patterns += 1;
                    }
                }
                
                // End of function detection
                if function_end_patterns > 0 && 
                   (line_num + 1 >= lines.len() || 
                    lines[line_num + 1].trim().is_empty() ||
                    lines[line_num + 1].trim().starts_with("%") ||
                    self.is_function_declaration(&lines[line_num + 1])) {
                    
                    if let Some(func) = current_function.take() {
                        functions.push(func);
                    }
                    in_function = false;
                    nesting_level = 0;
                    function_end_patterns = 0;
                }
            }
        }
        
        Ok(functions)
    }
    
    fn analyze_structures(&self, lines: &[String]) -> Result<Vec<StructureInfo>> {
        let mut structures = Vec::new();
        let mut current_module: Option<StructureInfo> = None;
        let mut behaviors = Vec::new();
        
        for (line_num, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            
            // Skip comments and empty lines
            if trimmed.starts_with("%") || trimmed.is_empty() {
                continue;
            }
            
            // Module declaration detection
            if self.is_module_declaration(trimmed) {
                if let Some(module_name) = self.extract_module_name(trimmed) {
                    current_module = Some(StructureInfo {
                        name: module_name,
                        structure_type: StructureType::Class, // Modules are like classes
                        line_count: 0,
                        start_line: line_num + 1,
                        end_line: line_num + 1,
                        methods: Vec::new(),
                        properties: 0,
                        visibility: Visibility::Public,
                        inheritance_depth: 0,
                        interface_count: 0,
                    });
                }
            }
            
            // Behavior declaration detection
            if self.is_behavior_declaration(trimmed) {
                if let Some(behavior_name) = self.extract_behavior_name(trimmed) {
                    behaviors.push(behavior_name);
                }
            }
            
            // Count module attributes as properties
            if trimmed.starts_with("-") && 
               !trimmed.starts_with("-module(") && 
               !trimmed.starts_with("-behaviour(") &&
               !trimmed.starts_with("-behavior(") {
                if let Some(ref mut module) = current_module {
                    module.properties += 1;
                }
            }
        }
        
        // Finalize module structure
        if let Some(mut module) = current_module {
            module.end_line = lines.len();
            module.line_count = lines.len();
            module.inheritance_depth = behaviors.len();
            
            // Find functions that belong to this module
            for (line_num, line) in lines.iter().enumerate() {
                let trimmed = line.trim();
                
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
                            is_method: false,
                            parent_class: Some(module.name.clone()),
                            local_variable_count: 0,
                            has_recursion: false,
                            has_exception_handling: false,
                        visibility: Visibility::Public,};
                        
                        module.methods.push(method_info);
                    }
                }
            }
            
            structures.push(module);
        }
        
        Ok(structures)
    }
    
    fn language_name(&self) -> &'static str {
        "Erlang"
    }
    
    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["erl", "hrl"]
    }
}

impl Default for ErlangAnalyzer {
    fn default() -> Self {
        Self::new()
    }
} 