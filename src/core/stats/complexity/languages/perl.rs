use crate::utils::errors::Result;
use super::super::types::{FunctionInfo, StructureInfo, StructureType, Visibility};
use super::LanguageAnalyzer;

/// Perl language complexity analyzer
pub struct PerlAnalyzer;

impl PerlAnalyzer {
    pub fn new() -> Self {
        Self
    }
    
    /// Extract function name from Perl function declaration
    fn extract_function_name(&self, line: &str) -> Option<String> {
        let trimmed = line.trim();
        
        // Skip comments and empty lines
        if trimmed.starts_with("#") || trimmed.is_empty() {
            return None;
        }
        
        // Look for sub declarations: sub function_name
        if trimmed.starts_with("sub ") {
            let after_sub = &trimmed[4..];
            let parts: Vec<&str> = after_sub.split_whitespace().collect();
            
            if let Some(first_part) = parts.first() {
                // Handle function name before parentheses or braces
                let name = if let Some(paren_pos) = first_part.find('(') {
                    &first_part[..paren_pos]
                } else if let Some(brace_pos) = first_part.find('{') {
                    &first_part[..brace_pos]
                } else {
                    first_part
                };
                
                if !name.is_empty() && name.chars().all(|c| c.is_alphanumeric() || c == '_') {
                    return Some(name.to_string());
                }
            }
        }
        
        None
    }
    
    /// Extract package name from Perl package declaration
    fn extract_package_name(&self, line: &str) -> Option<String> {
        let trimmed = line.trim();
        
        // Look for package declarations: package PackageName;
        if trimmed.starts_with("package ") && trimmed.ends_with(';') {
            let package_part = &trimmed[8..trimmed.len()-1];
            let parts: Vec<&str> = package_part.split_whitespace().collect();
            
            if let Some(first_part) = parts.first() {
                if !first_part.is_empty() && 
                   first_part.chars().all(|c| c.is_alphanumeric() || c == '_' || c == ':') {
                    return Some(first_part.to_string());
                }
            }
        }
        
        None
    }
    
    /// Count complexity keywords in Perl code
    fn count_complexity_keywords(&self, line: &str) -> usize {
        let keywords = [
            "if", "elsif", "else", "unless", "for", "foreach", "while", "until", "do",
            "given", "when", "default", "&&", "||", "and", "or", "not", "xor",
            "eval", "die", "warn", "next", "last", "redo", "goto"
        ];
        keywords.iter().map(|&keyword| line.matches(keyword).count()).sum()
    }
    
    /// Count cognitive complexity for Perl code
    fn count_cognitive_complexity(&self, line: &str, nesting_level: usize) -> usize {
        let mut complexity = 0;
        let nesting_multiplier = nesting_level.max(1);
        
        // Basic control structures
        if line.contains("if ") || line.contains("if(") { complexity += 1 * nesting_multiplier; }
        if line.contains("elsif ") { complexity += 1; }
        if line.contains("else") && !line.contains("elsif") { complexity += 1; }
        if line.contains("unless ") { complexity += 1 * nesting_multiplier; }
        if line.contains("for ") || line.contains("for(") { complexity += 1 * nesting_multiplier; }
        if line.contains("foreach ") { complexity += 1 * nesting_multiplier; }
        if line.contains("while ") || line.contains("while(") { complexity += 1 * nesting_multiplier; }
        if line.contains("until ") { complexity += 1 * nesting_multiplier; }
        if line.contains("do ") { complexity += 1 * nesting_multiplier; }
        
        // Switch-like structures
        if line.contains("given ") { complexity += 1 * nesting_multiplier; }
        if line.contains("when ") { complexity += 1; }
        if line.contains("default") { complexity += 1; }
        
        // Logical operators
        complexity += line.matches("&&").count() * nesting_multiplier;
        complexity += line.matches("||").count() * nesting_multiplier;
        complexity += line.matches(" and ").count() * nesting_multiplier;
        complexity += line.matches(" or ").count() * nesting_multiplier;
        complexity += line.matches(" xor ").count() * nesting_multiplier;
        
        // Exception handling
        if line.contains("eval ") { complexity += 1; }
        if line.contains("die ") { complexity += 1; }
        if line.contains("warn ") { complexity += 1; }
        
        // Control flow
        if line.contains("next") { complexity += 1; }
        if line.contains("last") { complexity += 1; }
        if line.contains("redo") { complexity += 1; }
        if line.contains("goto") { complexity += 1; }
        
        // Regular expressions add complexity
        if line.contains("=~") || line.contains("!~") { complexity += 1; }
        if line.contains("m/") || line.contains("s/") || line.contains("tr/") { complexity += 1; }
        
        // References and dereferencing
        if line.contains("\\") && !line.contains("\\n") && !line.contains("\\t") { complexity += 1; }
        if line.contains("$") && line.contains("->") { complexity += 1; }
        
        // Complex data structures
        if line.contains("@{") || line.contains("%{") || line.contains("${") { complexity += 1; }
        
        // Perl-specific constructs
        if line.contains("map ") || line.contains("grep ") { complexity += 1; }
        if line.contains("sort ") { complexity += 1; }
        
        complexity
    }
    
    /// Check if line contains a function declaration
    fn is_function_declaration(&self, line: &str) -> bool {
        let trimmed = line.trim();
        
        // Skip comments and empty lines
        if trimmed.starts_with("#") || trimmed.is_empty() {
            return false;
        }
        
        // Must start with "sub "
        trimmed.starts_with("sub ")
    }
    
    /// Check if line contains a package declaration
    fn is_package_declaration(&self, line: &str) -> bool {
        let trimmed = line.trim();
        trimmed.starts_with("package ") && trimmed.ends_with(';')
    }
    
    /// Count parameters in function signature
    fn count_parameters(&self, line: &str) -> usize {
        // Perl functions often use @_ for parameters, but we can try to count declared params
        if let Some(start) = line.find('(') {
            if let Some(end) = line.find(')') {
                let params = &line[start + 1..end];
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
        
        // If no parentheses, check for @_ usage (common in Perl)
        if line.contains("@_") {
            return 1; // Assume at least one parameter
        }
        
        0
    }
    
    /// Determine visibility (Perl doesn't have formal visibility)
    fn get_visibility(&self, line: &str) -> Visibility {
        // In Perl, functions starting with _ are conventionally private
        if line.contains("sub _") {
            Visibility::Private
        } else {
            Visibility::Public
        }
    }
    
    /// Check if function uses references
    fn uses_references(&self, line: &str) -> bool {
        line.contains("\\") && !line.contains("\\n") && !line.contains("\\t")
    }
    
    /// Check if function uses regular expressions
    fn uses_regex(&self, line: &str) -> bool {
        line.contains("=~") || line.contains("!~") || 
        line.contains("m/") || line.contains("s/") || line.contains("tr/")
    }
    
    /// Check if function uses complex data structures
    fn uses_complex_data(&self, line: &str) -> bool {
        line.contains("@{") || line.contains("%{") || line.contains("${")
    }
    
    /// Check if function uses map/grep/sort
    fn uses_functional_constructs(&self, line: &str) -> bool {
        line.contains("map ") || line.contains("grep ") || line.contains("sort ")
    }
    
    /// Check if line contains object-oriented constructs
    fn uses_oo_constructs(&self, line: &str) -> bool {
        line.contains("->") || line.contains("bless") || line.contains("SUPER::")
    }
}

impl LanguageAnalyzer for PerlAnalyzer {
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
                        is_method: false, // Perl functions are not methods by default
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
                    if trimmed.contains("return") {
                        func.return_path_count += 1;
                    }
                    
                    // Check for recursion
                    if trimmed.contains(&func.name) && trimmed.contains('(') {
                        func.has_recursion = true;
                    }
                    
                    // Check for exception handling
                    if trimmed.contains("eval") || trimmed.contains("die") || 
                       trimmed.contains("warn") {
                        func.has_exception_handling = true;
                    }
                    
                    // Count local variables (my, our, local declarations)
                    if trimmed.contains("my ") || trimmed.contains("our ") || 
                       trimmed.contains("local ") {
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
        let mut current_package: Option<StructureInfo> = None;
        
        for (line_num, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            
            // Skip comments and empty lines
            if trimmed.starts_with("#") || trimmed.is_empty() {
                continue;
            }
            
            // Package declaration detection
            if self.is_package_declaration(trimmed) {
                if let Some(package_name) = self.extract_package_name(trimmed) {
                    // Finalize previous package if exists
                    if let Some(package) = current_package.take() {
                        structures.push(package);
                    }
                    
                    current_package = Some(StructureInfo {
                        name: package_name,
                        structure_type: StructureType::Class, // Packages are like classes
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
            
            // Count global variables as properties
            if (trimmed.contains("our ") || trimmed.contains("use vars")) &&
               !self.is_function_declaration(trimmed) {
                if let Some(ref mut package) = current_package {
                    package.properties += 1;
                }
            }
        }
        
        // Finalize last package
        if let Some(mut package) = current_package {
            package.end_line = lines.len();
            package.line_count = lines.len() - package.start_line + 1;
            
            // Find functions that belong to this package
            for (line_num, line) in lines.iter().enumerate() {
                let trimmed = line.trim();
                
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
                            parent_class: Some(package.name.clone()),
                            local_variable_count: 0,
                            has_recursion: false,
                            has_exception_handling: false,
                        visibility: Visibility::Public,};
                        
                        package.methods.push(method_info);
                    }
                }
            }
            
            structures.push(package);
        }
        
        // If no packages found, create a default one
        if structures.is_empty() {
            let mut default_package = StructureInfo {
                name: "main".to_string(),
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
            
            // Add all functions to default package
            for (line_num, line) in lines.iter().enumerate() {
                let trimmed = line.trim();
                
                if self.is_function_declaration(trimmed) {
                    if let Some(func_name) = self.extract_function_name(trimmed) {
                        let param_count = self.count_parameters(trimmed);
                        let _visibility = self.get_visibility(trimmed);
                        
                        let method_info = FunctionInfo {
                            name: func_name,
                            line_count: 0,
                            cyclomatic_complexity: 1,
                            cognitive_complexity: 1,
                            nesting_depth: 0,
                            parameter_count: param_count,
                            return_path_count: 0,
                            start_line: line_num + 1,
                            end_line: line_num + 1,
                            is_method: false,
                            parent_class: Some("main".to_string()),
                            local_variable_count: 0,
                            has_recursion: false,
                            has_exception_handling: false,
                        visibility: Visibility::Public,};
                        
                        default_package.methods.push(method_info);
                    }
                }
            }
            
            structures.push(default_package);
        }
        
        Ok(structures)
    }
    
    fn language_name(&self) -> &'static str {
        "Perl"
    }
    
    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["pl", "pm", "perl"]
    }
}

impl Default for PerlAnalyzer {
    fn default() -> Self {
        Self::new()
    }
} 