use crate::utils::errors::Result;
use super::super::types::{FunctionInfo, StructureInfo, StructureType, Visibility};
use super::LanguageAnalyzer;

/// R language complexity analyzer
pub struct RAnalyzer;

impl RAnalyzer {
    pub fn new() -> Self {
        Self
    }
    
    /// Extract function name from R function declaration
    fn extract_function_name(&self, line: &str) -> Option<String> {
        let trimmed = line.trim();
        
        // Skip comments and empty lines
        if trimmed.starts_with("#") || trimmed.is_empty() {
            return None;
        }
        
        // Look for function assignment patterns: name <- function(...)
        if trimmed.contains("<-") && trimmed.contains("function(") {
            if let Some(assign_pos) = trimmed.find("<-") {
                let func_name = trimmed[..assign_pos].trim();
                
                if !func_name.is_empty() && 
                   func_name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '.') {
                    return Some(func_name.to_string());
                }
            }
        }
        
        // Look for function assignment patterns: name = function(...)
        if trimmed.contains("=") && trimmed.contains("function(") {
            if let Some(assign_pos) = trimmed.find("=") {
                let func_name = trimmed[..assign_pos].trim();
                
                if !func_name.is_empty() && 
                   func_name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '.') {
                    return Some(func_name.to_string());
                }
            }
        }
        
        None
    }
    
    /// Count complexity keywords in R code
    fn count_complexity_keywords(&self, line: &str) -> usize {
        let keywords = [
            "if", "else", "for", "while", "repeat", "switch", "ifelse",
            "&", "|", "&&", "||", "!", "any", "all", "which",
            "try", "tryCatch", "stop", "warning", "next", "break"
        ];
        keywords.iter().map(|&keyword| line.matches(keyword).count()).sum()
    }
    
    /// Count cognitive complexity for R code
    fn count_cognitive_complexity(&self, line: &str, nesting_level: usize) -> usize {
        let mut complexity = 0;
        let nesting_multiplier = nesting_level.max(1);
        
        // Basic control structures
        if line.contains("if ") || line.contains("if(") { complexity += 1 * nesting_multiplier; }
        if line.contains("else") { complexity += 1; }
        if line.contains("for ") || line.contains("for(") { complexity += 1 * nesting_multiplier; }
        if line.contains("while ") || line.contains("while(") { complexity += 1 * nesting_multiplier; }
        if line.contains("repeat") { complexity += 1 * nesting_multiplier; }
        if line.contains("switch(") { complexity += 1 * nesting_multiplier; }
        
        // Vectorized conditional
        if line.contains("ifelse(") { complexity += 1; }
        
        // Logical operators
        complexity += line.matches("&&").count() * nesting_multiplier;
        complexity += line.matches("||").count() * nesting_multiplier;
        complexity += line.matches("&").count(); // Vectorized AND
        complexity += line.matches("|").count(); // Vectorized OR
        
        // Exception handling
        if line.contains("try(") { complexity += 1; }
        if line.contains("tryCatch(") { complexity += 1 * nesting_multiplier; }
        if line.contains("stop(") { complexity += 1; }
        if line.contains("warning(") { complexity += 1; }
        
        // Control flow
        if line.contains("next") { complexity += 1; }
        if line.contains("break") { complexity += 1; }
        
        // R-specific complexity patterns
        if line.contains("apply(") || line.contains("lapply(") || 
           line.contains("sapply(") || line.contains("mapply(") { complexity += 1; }
        
        // Data manipulation complexity
        if line.contains("subset(") { complexity += 1; }
        if line.contains("merge(") { complexity += 1; }
        if line.contains("aggregate(") { complexity += 1; }
        
        // Statistical model complexity
        if line.contains("lm(") || line.contains("glm(") || 
           line.contains("aov(") || line.contains("t.test(") { complexity += 1; }
        
        // Complex data access patterns
        if line.contains("[[") || line.contains("$") { complexity += 1; }
        
        complexity
    }
    
    /// Check if line contains a function declaration
    fn is_function_declaration(&self, line: &str) -> bool {
        let trimmed = line.trim();
        
        // Skip comments and empty lines
        if trimmed.starts_with("#") || trimmed.is_empty() {
            return false;
        }
        
        // Must contain assignment and function keyword
        (trimmed.contains("<-") || trimmed.contains("=")) && trimmed.contains("function(")
    }
    
    /// Count parameters in function signature
    fn count_parameters(&self, line: &str) -> usize {
        if let Some(func_pos) = line.find("function(") {
            let after_func = &line[func_pos + 9..];
            if let Some(end) = after_func.find(')') {
                let params = &after_func[..end];
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
    
    /// Determine visibility (R doesn't have formal visibility)
    fn get_visibility(&self, line: &str) -> Visibility {
        // In R, functions starting with . are conventionally private
        if line.contains("<- .") || line.contains("= .") {
            Visibility::Private
        } else {
            Visibility::Public
        }
    }
    
    /// Check if function uses data frames
    fn uses_data_frames(&self, line: &str) -> bool {
        line.contains("data.frame(") || line.contains("$") || 
        line.contains("[[") || line.contains("subset(") ||
        line.contains("merge(") || line.contains("aggregate(")
    }
    
    /// Check if function uses vectors
    fn uses_vectors(&self, line: &str) -> bool {
        line.contains("c(") || line.contains("rep(") || 
        line.contains("seq(") || line.contains("length(") ||
        line.contains("which(") || line.contains("match(")
    }
    
    /// Check if function uses statistical functions
    fn uses_statistical_functions(&self, line: &str) -> bool {
        line.contains("mean(") || line.contains("median(") || 
        line.contains("sd(") || line.contains("var(") ||
        line.contains("cor(") || line.contains("lm(") ||
        line.contains("glm(") || line.contains("t.test(") ||
        line.contains("aov(") || line.contains("summary(")
    }
    
    /// Check if function uses apply family
    fn uses_apply_family(&self, line: &str) -> bool {
        line.contains("apply(") || line.contains("lapply(") || 
        line.contains("sapply(") || line.contains("mapply(") ||
        line.contains("tapply(")
    }
    
    /// Check if function uses plotting
    fn uses_plotting(&self, line: &str) -> bool {
        line.contains("plot(") || line.contains("ggplot(") || 
        line.contains("hist(") || line.contains("boxplot(") ||
        line.contains("barplot(") || line.contains("lines(") ||
        line.contains("points(") || line.contains("abline(")
    }
    
    /// Check if function uses matrix operations
    fn uses_matrix_operations(&self, line: &str) -> bool {
        line.contains("matrix(") || line.contains("%*%") || 
        line.contains("t(") || line.contains("solve(") ||
        line.contains("eigen(") || line.contains("svd(")
    }
}

impl LanguageAnalyzer for RAnalyzer {
    fn analyze_functions(&self, lines: &[String]) -> Result<Vec<FunctionInfo>> {
        let mut functions = Vec::new();
        let mut current_function: Option<FunctionInfo> = None;
        let mut brace_count = 0;
        let mut in_function = false;
        let mut nesting_level: usize = 0;
        
        for (line_num, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            
            // Skip comments and empty lines
            if trimmed.starts_with("#") || trimmed.is_empty() {
                continue;
            }
            
            // Function declaration detection
            if self.is_function_declaration(trimmed) {
                if let Some(func_name) = self.extract_function_name(trimmed) {
                    let param_count = self.count_parameters(trimmed);
                    let _visibility = self.get_visibility(trimmed);
                    
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
                        is_method: false, // R functions are not methods by default
                        parent_class: None,
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
                    if trimmed.contains("return(") {
                        func.return_path_count += 1;
                    }
                    
                    // Check for recursion
                    if trimmed.contains(&func.name) && trimmed.contains('(') {
                        func.has_recursion = true;
                    }
                    
                    // Check for exception handling
                    if trimmed.contains("try(") || trimmed.contains("tryCatch(") || 
                       trimmed.contains("stop(") || trimmed.contains("warning(") {
                        func.has_exception_handling = true;
                    }
                    
                    // Count local variables (assignments within function)
                    if (trimmed.contains("<-") || trimmed.contains("=")) &&
                       !trimmed.contains("function(") && !trimmed.contains("==") &&
                       !trimmed.contains("!=") && !trimmed.contains("<=") && 
                       !trimmed.contains(">=") {
                        func.local_variable_count += 1;
                    }
                }
                
                // End of function (R functions end with closing brace)
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
        
        // R doesn't have formal classes in the traditional sense, but we can analyze:
        // 1. S3 classes (informal)
        // 2. S4 classes (formal)
        // 3. R6 classes (reference classes)
        // 4. Scripts as modules
        
        // Create a default "script" structure to hold all functions
        let mut script_structure = StructureInfo {
            name: "R_Script".to_string(),
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
        
        for (line_num, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            
            // Skip comments and empty lines
            if trimmed.starts_with("#") || trimmed.is_empty() {
                continue;
            }
            
            // Count global variable assignments
            if (trimmed.contains("<-") || trimmed.contains("=")) &&
               !trimmed.contains("function(") && !trimmed.contains("==") &&
               !trimmed.contains("!=") && !trimmed.contains("<=") && 
               !trimmed.contains(">=") {
                global_vars += 1;
            }
            
            // Add functions to the script structure
            if self.is_function_declaration(trimmed) {
                if let Some(func_name) = self.extract_function_name(trimmed) {
                    let param_count = self.count_parameters(trimmed);
                    let _visibility = self.get_visibility(trimmed);
                    
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
                        parent_class: Some("R_Script".to_string()),
                        local_variable_count: 0,
                        has_recursion: false,
                        has_exception_handling: false,
                        visibility: Visibility::Public,};
                    
                    script_structure.methods.push(method_info);
                }
            }
        }
        
        script_structure.properties = global_vars;
        structures.push(script_structure);
        
        Ok(structures)
    }
    
    fn language_name(&self) -> &'static str {
        "R"
    }
    
    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["r", "R"]
    }
}

impl Default for RAnalyzer {
    fn default() -> Self {
        Self::new()
    }
} 