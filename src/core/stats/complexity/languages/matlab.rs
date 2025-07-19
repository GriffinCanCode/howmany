use crate::utils::errors::Result;
use super::super::types::{FunctionInfo, StructureInfo, StructureType, Visibility};
use super::LanguageAnalyzer;

/// MATLAB language complexity analyzer
pub struct MatlabAnalyzer;

impl MatlabAnalyzer {
    pub fn new() -> Self {
        Self
    }
    
    /// Extract function name from MATLAB function declaration
    fn extract_function_name(&self, line: &str) -> Option<String> {
        let trimmed = line.trim();
        
        // Skip comments and empty lines
        if trimmed.starts_with("%") || trimmed.is_empty() {
            return None;
        }
        
        // Look for function declarations: function [output] = function_name(input)
        if trimmed.starts_with("function ") {
            let after_function = &trimmed[9..];
            
            // Handle different function patterns
            if let Some(equals_pos) = after_function.find('=') {
                // Pattern: function [output] = function_name(input)
                let after_equals = &after_function[equals_pos + 1..].trim();
                if let Some(paren_pos) = after_equals.find('(') {
                    let func_name = after_equals[..paren_pos].trim();
                    if !func_name.is_empty() && 
                       func_name.chars().all(|c| c.is_alphanumeric() || c == '_') {
                        return Some(func_name.to_string());
                    }
                }
            } else {
                // Pattern: function function_name(input)
                if let Some(paren_pos) = after_function.find('(') {
                    let func_name = after_function[..paren_pos].trim();
                    if !func_name.is_empty() && 
                       func_name.chars().all(|c| c.is_alphanumeric() || c == '_') {
                        return Some(func_name.to_string());
                    }
                } else {
                    // Pattern: function function_name
                    let parts: Vec<&str> = after_function.split_whitespace().collect();
                    if let Some(first_part) = parts.first() {
                        if !first_part.is_empty() && 
                           first_part.chars().all(|c| c.is_alphanumeric() || c == '_') {
                            return Some(first_part.to_string());
                        }
                    }
                }
            }
        }
        
        None
    }
    
    /// Count complexity keywords in MATLAB code
    fn count_complexity_keywords(&self, line: &str) -> usize {
        let keywords = [
            "if", "elseif", "else", "for", "while", "switch", "case", "otherwise",
            "&", "|", "&&", "||", "~", "any", "all", "find",
            "try", "catch", "error", "warning", "break", "continue", "return"
        ];
        keywords.iter().map(|&keyword| line.matches(keyword).count()).sum()
    }
    
    /// Count cognitive complexity for MATLAB code
    fn count_cognitive_complexity(&self, line: &str, nesting_level: usize) -> usize {
        let mut complexity = 0;
        let nesting_multiplier = nesting_level.max(1);
        
        // Basic control structures
        if line.contains("if ") || line.contains("if(") { complexity += 1 * nesting_multiplier; }
        if line.contains("elseif ") { complexity += 1; }
        if line.contains("else") && !line.contains("elseif") { complexity += 1; }
        if line.contains("for ") || line.contains("for(") { complexity += 1 * nesting_multiplier; }
        if line.contains("while ") || line.contains("while(") { complexity += 1 * nesting_multiplier; }
        if line.contains("switch ") { complexity += 1 * nesting_multiplier; }
        if line.contains("case ") { complexity += 1; }
        if line.contains("otherwise") { complexity += 1; }
        
        // Logical operators
        complexity += line.matches("&&").count() * nesting_multiplier;
        complexity += line.matches("||").count() * nesting_multiplier;
        complexity += line.matches("&").count(); // Element-wise AND
        complexity += line.matches("|").count(); // Element-wise OR
        
        // Exception handling
        if line.contains("try") { complexity += 1; }
        if line.contains("catch") { complexity += 1 * nesting_multiplier; }
        if line.contains("error(") { complexity += 1; }
        if line.contains("warning(") { complexity += 1; }
        
        // Control flow
        if line.contains("break") { complexity += 1; }
        if line.contains("continue") { complexity += 1; }
        if line.contains("return") { complexity += 1; }
        
        // MATLAB-specific complexity patterns
        if line.contains("cellfun(") || line.contains("arrayfun(") || 
           line.contains("structfun(") { complexity += 1; }
        
        // Matrix operations complexity
        if line.contains("*") || line.contains(".*") || 
           line.contains("\\") || line.contains("./") { complexity += 1; }
        
        // Complex indexing
        if line.contains("(:") || line.contains(":,") || 
           line.contains("end") { complexity += 1; }
        
        // Function handles and anonymous functions
        if line.contains("@") { complexity += 1; }
        
        complexity
    }
    
    /// Check if line contains a function declaration
    fn is_function_declaration(&self, line: &str) -> bool {
        let trimmed = line.trim();
        
        // Skip comments and empty lines
        if trimmed.starts_with("%") || trimmed.is_empty() {
            return false;
        }
        
        // Must start with "function "
        trimmed.starts_with("function ")
    }
    
    /// Count parameters in function signature
    fn count_parameters(&self, line: &str) -> usize {
        if let Some(paren_start) = line.find('(') {
            if let Some(paren_end) = line.find(')') {
                let params = &line[paren_start + 1..paren_end];
                if params.trim().is_empty() {
                    return 0;
                }
                
                // Count parameters by commas
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
    
    /// Determine visibility (MATLAB doesn't have formal visibility)
    fn get_visibility(&self, _line: &str) -> Visibility {
        // All MATLAB functions are public by default
        Visibility::Public
    }
    
    /// Check if function uses matrix operations
    fn uses_matrix_operations(&self, line: &str) -> bool {
        line.contains("*") || line.contains(".*") || 
        line.contains("\\") || line.contains("./") ||
        line.contains("'") || line.contains(".'") ||
        line.contains("inv(") || line.contains("pinv(") ||
        line.contains("eig(") || line.contains("svd(")
    }
    
    /// Check if function uses vectorized operations
    fn uses_vectorized_operations(&self, line: &str) -> bool {
        line.contains(".*") || line.contains("./") || 
        line.contains(".^") || line.contains("sum(") ||
        line.contains("mean(") || line.contains("std(") ||
        line.contains("max(") || line.contains("min(")
    }
    
    /// Check if function uses cell arrays
    fn uses_cell_arrays(&self, line: &str) -> bool {
        line.contains("{") || line.contains("}") ||
        line.contains("cell(") || line.contains("cellfun(") ||
        line.contains("cellstr(")
    }
    
    /// Check if function uses structures
    fn uses_structures(&self, line: &str) -> bool {
        line.contains("struct(") || line.contains("fieldnames(") ||
        line.contains("isfield(") || line.contains("rmfield(") ||
        line.contains(".")
    }
    
    /// Check if function uses plotting
    fn uses_plotting(&self, line: &str) -> bool {
        line.contains("plot(") || line.contains("figure(") || 
        line.contains("subplot(") || line.contains("hold ") ||
        line.contains("xlabel(") || line.contains("ylabel(") ||
        line.contains("title(") || line.contains("legend(")
    }
    
    /// Check if function uses signal processing
    fn uses_signal_processing(&self, line: &str) -> bool {
        line.contains("fft(") || line.contains("ifft(") ||
        line.contains("filter(") || line.contains("conv(") ||
        line.contains("xcorr(") || line.contains("pwelch(")
    }
    
    /// Check if function uses image processing
    fn uses_image_processing(&self, line: &str) -> bool {
        line.contains("imread(") || line.contains("imwrite(") ||
        line.contains("imshow(") || line.contains("imresize(") ||
        line.contains("rgb2gray(") || line.contains("imadjust(")
    }
}

impl LanguageAnalyzer for MatlabAnalyzer {
    fn analyze_functions(&self, lines: &[String]) -> Result<Vec<FunctionInfo>> {
        let mut functions = Vec::new();
        let mut current_function: Option<FunctionInfo> = None;
        let mut in_function = false;
        let mut nesting_level: usize = 0;
        let mut function_end_keywords = 0;
        
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
                        is_method: false, // MATLAB functions are not methods by default
                        parent_class: None,
                        local_variable_count: 0,
                        has_recursion: false,
                        has_exception_handling: false,
                        visibility: Visibility::Public,});
                    in_function = true;
                    nesting_level = 0;
                    function_end_keywords = 0;
                }
            }
            
            if in_function {
                if let Some(ref mut func) = current_function {
                    func.line_count += 1;
                    func.end_line = line_num + 1;
                    
                    // Track nesting level
                    if trimmed.contains("if ") || trimmed.contains("for ") || 
                       trimmed.contains("while ") || trimmed.contains("switch ") ||
                       trimmed.contains("try") {
                        nesting_level += 1;
                    }
                    if trimmed == "end" {
                        if nesting_level > 0 {
                            nesting_level -= 1;
                        } else {
                            function_end_keywords += 1;
                        }
                    }
                    
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
                       trimmed.contains("error(") || trimmed.contains("warning(") {
                        func.has_exception_handling = true;
                    }
                    
                    // Count local variables (assignments)
                    if trimmed.contains("=") && !trimmed.contains("==") &&
                       !trimmed.contains("~=") && !trimmed.contains("<=") && 
                       !trimmed.contains(">=") && !trimmed.contains("function ") {
                        func.local_variable_count += 1;
                    }
                }
                
                // End of function detection (MATLAB functions end with "end")
                if function_end_keywords > 0 && nesting_level == 0 {
                    if let Some(func) = current_function.take() {
                        functions.push(func);
                    }
                    in_function = false;
                    function_end_keywords = 0;
                }
            }
        }
        
        Ok(functions)
    }
    
    fn analyze_structures(&self, lines: &[String]) -> Result<Vec<StructureInfo>> {
        let mut structures = Vec::new();
        
        // MATLAB doesn't have traditional classes in older versions, but we can analyze:
        // 1. Scripts as modules
        // 2. Class definitions (newer MATLAB)
        // 3. Function files
        
        // Create a default "script" structure to hold all functions
        let mut script_structure = StructureInfo {
            name: "MATLAB_Script".to_string(),
            structure_type: StructureType::Class,
            line_count: lines.len(),
            start_line: 1,
            end_line: lines.len(),
            methods: Vec::new(),
            properties: 0,
            visibility: Visibility::Public,
            inheritance_depth: 0,
            interface_count: 0,
        };
        
        // Count global variables as properties
        let mut global_vars = 0;
        let mut has_class_definition = false;
        
        for (line_num, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            
            // Skip comments and empty lines
            if trimmed.starts_with("%") || trimmed.is_empty() {
                continue;
            }
            
            // Check for class definition
            if trimmed.starts_with("classdef ") {
                has_class_definition = true;
                // Extract class name
                if let Some(class_name) = trimmed.strip_prefix("classdef ") {
                    let name = class_name.split_whitespace().next().unwrap_or("UnknownClass");
                    script_structure.name = name.to_string();
                    script_structure.structure_type = StructureType::Class;
                }
            }
            
            // Count global variable assignments
            if trimmed.contains("=") && !trimmed.contains("==") &&
               !trimmed.contains("~=") && !trimmed.contains("<=") && 
               !trimmed.contains(">=") && !trimmed.contains("function ") &&
               !has_class_definition {
                global_vars += 1;
            }
            
            // Count properties in class definition
            if has_class_definition && trimmed.starts_with("properties") {
                // Properties section found
                script_structure.properties += 1;
            }
            
            // Add functions to the script structure
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
                        is_method: has_class_definition,
                        parent_class: Some(script_structure.name.clone()),
                        local_variable_count: 0,
                        has_recursion: false,
                        has_exception_handling: false,
                        visibility: Visibility::Public,};
                    
                    script_structure.methods.push(method_info);
                }
            }
        }
        
        if !has_class_definition {
            script_structure.properties = global_vars;
        }
        
        structures.push(script_structure);
        
        Ok(structures)
    }
    
    fn language_name(&self) -> &'static str {
        "MATLAB"
    }
    
    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["m"]
    }
}

impl Default for MatlabAnalyzer {
    fn default() -> Self {
        Self::new()
    }
} 