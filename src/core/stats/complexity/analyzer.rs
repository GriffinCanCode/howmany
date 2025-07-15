use crate::utils::errors::Result;
use super::types::{FunctionInfo, StructureInfo, StructureType, Visibility};
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;

/// Language-specific code analyzer
pub struct CodeAnalyzer;

impl CodeAnalyzer {
    pub fn new() -> Self {
        Self
    }

    /// Analyze structures in a file (classes, interfaces, etc.)
    pub fn analyze_file_structures(&self, file_path: &str) -> Result<Vec<StructureInfo>> {
        let file = fs::File::open(file_path)?;
        let reader = BufReader::new(file);
        let lines: Vec<String> = reader.lines().collect::<std::io::Result<Vec<_>>>()?;
        
        let extension = Path::new(file_path)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("unknown")
            .to_lowercase();
        
        match extension.as_str() {
            "rs" => self.analyze_rust_structures(&lines),
            "py" => self.analyze_python_structures(&lines),
            "js" | "jsx" | "ts" | "tsx" => self.analyze_javascript_structures(&lines),
            "java" => self.analyze_java_structures(&lines),
            "cpp" | "cc" | "cxx" | "c" | "h" | "hpp" => self.analyze_cpp_structures(&lines),
            "go" => self.analyze_go_structures(&lines),
            "cs" => self.analyze_csharp_structures(&lines),
            "php" => self.analyze_php_structures(&lines),
            "rb" => self.analyze_ruby_structures(&lines),
            "swift" => self.analyze_swift_structures(&lines),
            "kt" => self.analyze_kotlin_structures(&lines),
            _ => Ok(Vec::new()), // Unsupported language
        }
    }
    
    /// Analyze functions in a file for complexity metrics
    pub fn analyze_file_functions(&self, file_path: &str) -> Result<Vec<FunctionInfo>> {
        let file = fs::File::open(file_path)?;
        let reader = BufReader::new(file);
        let lines: Vec<String> = reader.lines().collect::<std::io::Result<Vec<_>>>()?;
        
        let extension = Path::new(file_path)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("unknown")
            .to_lowercase();
        
        match extension.as_str() {
            "rs" => self.analyze_rust_functions(&lines),
            "py" => self.analyze_python_functions(&lines),
            "js" | "jsx" | "ts" | "tsx" => self.analyze_javascript_functions(&lines),
            "java" => self.analyze_java_functions(&lines),
            "cpp" | "cc" | "cxx" | "c" => self.analyze_cpp_functions(&lines),
            "go" => self.analyze_go_functions(&lines),
            _ => Ok(Vec::new()), // Unsupported language
        }
    }
    
    /// Analyze Rust functions
    fn analyze_rust_functions(&self, lines: &[String]) -> Result<Vec<FunctionInfo>> {
        let mut functions = Vec::new();
        let mut current_function: Option<FunctionInfo> = None;
        let mut brace_count = 0;
        let mut in_function = false;
        
        for (line_num, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            
            // Skip comments and empty lines
            if trimmed.starts_with("//") || trimmed.is_empty() {
                continue;
            }
            
            // Function declaration detection
            if trimmed.starts_with("fn ") || trimmed.contains(" fn ") {
                if let Some(func_name) = self.extract_rust_function_name(trimmed) {
                    current_function = Some(FunctionInfo {
                        name: func_name,
                        line_count: 0,
                        cyclomatic_complexity: 1, // Base complexity
                        cognitive_complexity: 1, // Base cognitive complexity
                        nesting_depth: 0,
                        parameter_count: 0,
                        return_path_count: 0,
                        start_line: line_num + 1,
                        end_line: line_num + 1,
                        is_method: false,
                        parent_class: None,
                        local_variable_count: 0,
                        has_recursion: false,
                        has_exception_handling: false,
                    });
                    in_function = true;
                    brace_count = 0;
                }
            }
            
            if in_function {
                if let Some(ref mut func) = current_function {
                    func.line_count += 1;
                    func.end_line = line_num + 1;
                    
                    // Count braces for nesting depth
                    let open_braces = trimmed.matches('{').count();
                    let close_braces = trimmed.matches('}').count();
                    brace_count += open_braces as i32 - close_braces as i32;
                    func.nesting_depth = func.nesting_depth.max(brace_count.max(0) as usize);
                    
                    // Calculate cyclomatic complexity
                    func.cyclomatic_complexity += self.count_rust_complexity_keywords(trimmed);
                    
                    // Calculate cognitive complexity
                    func.cognitive_complexity += self.count_rust_cognitive_complexity(trimmed, brace_count);
                    
                    // Count parameters
                    if trimmed.contains('(') && func.parameter_count == 0 {
                        func.parameter_count = self.count_function_parameters(trimmed);
                    }
                    
                    // Count return paths
                    if trimmed.contains("return") {
                        func.return_path_count += 1;
                    }
                    
                    // Check for recursion
                    if trimmed.contains(&func.name) && !trimmed.starts_with("fn ") {
                        func.has_recursion = true;
                    }
                    
                    // Check for exception handling
                    if trimmed.contains("try") || trimmed.contains("catch") || trimmed.contains("?") {
                        func.has_exception_handling = true;
                    }
                    
                    // Function end detection
                    if brace_count <= 0 && close_braces > 0 {
                        functions.push(func.clone());
                        current_function = None;
                        in_function = false;
                    }
                }
            }
        }
        
        Ok(functions)
    }
    
    /// Analyze Python functions
    fn analyze_python_functions(&self, lines: &[String]) -> Result<Vec<FunctionInfo>> {
        let mut functions = Vec::new();
        let mut current_function: Option<FunctionInfo> = None;
        let mut function_indent = 0;
        
        for (line_num, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            
            // Skip comments and empty lines
            if trimmed.starts_with("#") || trimmed.is_empty() {
                continue;
            }
            
            // Calculate indentation
            let current_indent = line.len() - line.trim_start().len();
            
            // Function declaration detection
            if trimmed.starts_with("def ") {
                if let Some(func_name) = self.extract_python_function_name(trimmed) {
                    // Save previous function if exists
                    if let Some(func) = current_function.take() {
                        functions.push(func);
                    }
                    
                    current_function = Some(FunctionInfo {
                        name: func_name,
                        line_count: 0,
                        cyclomatic_complexity: 1, // Base complexity
                        cognitive_complexity: 1, // Base cognitive complexity
                        nesting_depth: 0,
                        parameter_count: 0,
                        return_path_count: 0,
                        start_line: line_num + 1,
                        end_line: line_num + 1,
                        is_method: trimmed.contains("self"),
                        parent_class: None,
                        local_variable_count: 0,
                        has_recursion: false,
                        has_exception_handling: false,
                    });
                    function_indent = current_indent;
                }
            }
            
            if let Some(ref mut func) = current_function {
                // Check if we're still in the function
                if current_indent <= function_indent && line_num > func.start_line - 1 && !trimmed.is_empty() {
                    // Function ended
                    functions.push(func.clone());
                    current_function = None;
                    continue;
                }
                
                if current_indent > function_indent {
                    func.line_count += 1;
                    func.end_line = line_num + 1;
                    
                    // Calculate nesting depth
                    let relative_indent = (current_indent - function_indent) / 4; // Assuming 4-space indentation
                    func.nesting_depth = func.nesting_depth.max(relative_indent);
                    
                    // Calculate cyclomatic complexity
                    func.cyclomatic_complexity += self.count_python_complexity_keywords(trimmed);
                    
                    // Calculate cognitive complexity
                    func.cognitive_complexity += self.count_python_cognitive_complexity(trimmed, relative_indent);
                    
                    // Count parameters
                    if trimmed.contains('(') && func.parameter_count == 0 {
                        func.parameter_count = self.count_function_parameters(trimmed);
                    }
                    
                    // Count return paths
                    if trimmed.contains("return") {
                        func.return_path_count += 1;
                    }
                    
                    // Check for recursion
                    if trimmed.contains(&func.name) && !trimmed.starts_with("def ") {
                        func.has_recursion = true;
                    }
                    
                    // Check for exception handling
                    if trimmed.contains("try:") || trimmed.contains("except") || trimmed.contains("finally") {
                        func.has_exception_handling = true;
                    }
                }
            }
        }
        
        // Add the last function if exists
        if let Some(func) = current_function {
            functions.push(func);
        }
        
        Ok(functions)
    }
    
    /// Analyze JavaScript/TypeScript functions
    fn analyze_javascript_functions(&self, lines: &[String]) -> Result<Vec<FunctionInfo>> {
        let mut functions = Vec::new();
        let mut current_function: Option<FunctionInfo> = None;
        let mut brace_count = 0;
        let mut in_function = false;
        
        for (line_num, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            
            // Skip comments and empty lines
            if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.is_empty() {
                continue;
            }
            
            // Function declaration detection
            if self.is_javascript_function_declaration(trimmed) {
                if let Some(func_name) = self.extract_javascript_function_name(trimmed) {
                    current_function = Some(FunctionInfo {
                        name: func_name,
                        line_count: 0,
                        cyclomatic_complexity: 1, // Base complexity
                        cognitive_complexity: 1, // Base cognitive complexity
                        nesting_depth: 0,
                        parameter_count: 0,
                        return_path_count: 0,
                        start_line: line_num + 1,
                        end_line: line_num + 1,
                        is_method: false,
                        parent_class: None,
                        local_variable_count: 0,
                        has_recursion: false,
                        has_exception_handling: false,
                    });
                    in_function = true;
                    brace_count = 0;
                }
            }
            
            if in_function {
                if let Some(ref mut func) = current_function {
                    func.line_count += 1;
                    func.end_line = line_num + 1;
                    
                    // Count braces for nesting depth
                    let open_braces = trimmed.matches('{').count();
                    let close_braces = trimmed.matches('}').count();
                    brace_count += open_braces as i32 - close_braces as i32;
                    func.nesting_depth = func.nesting_depth.max(brace_count.max(0) as usize);
                    
                    // Calculate cyclomatic complexity
                    func.cyclomatic_complexity += self.count_javascript_complexity_keywords(trimmed);
                    
                    // Calculate cognitive complexity
                    func.cognitive_complexity += self.count_javascript_cognitive_complexity(trimmed, brace_count);
                    
                    // Count parameters
                    if trimmed.contains('(') && func.parameter_count == 0 {
                        func.parameter_count = self.count_function_parameters(trimmed);
                    }
                    
                    // Count return paths
                    if trimmed.contains("return") {
                        func.return_path_count += 1;
                    }
                    
                    // Check for recursion
                    if trimmed.contains(&func.name) && !trimmed.starts_with("function ") {
                        func.has_recursion = true;
                    }
                    
                    // Check for exception handling
                    if trimmed.contains("try") || trimmed.contains("catch") || trimmed.contains("throw") {
                        func.has_exception_handling = true;
                    }
                    
                    // Function end detection
                    if brace_count <= 0 && close_braces > 0 {
                        functions.push(func.clone());
                        current_function = None;
                        in_function = false;
                    }
                }
            }
        }
        
        Ok(functions)
    }
    
    /// Analyze Java functions
    fn analyze_java_functions(&self, lines: &[String]) -> Result<Vec<FunctionInfo>> {
        // Similar to JavaScript but with Java-specific patterns
        self.analyze_javascript_functions(lines) // Simplified for now
    }
    
    /// Analyze C++ functions
    fn analyze_cpp_functions(&self, lines: &[String]) -> Result<Vec<FunctionInfo>> {
        // Similar to JavaScript but with C++-specific patterns
        self.analyze_javascript_functions(lines) // Simplified for now
    }
    
    /// Analyze Go functions
    fn analyze_go_functions(&self, lines: &[String]) -> Result<Vec<FunctionInfo>> {
        // Similar to JavaScript but with Go-specific patterns
        self.analyze_javascript_functions(lines) // Simplified for now
    }
    
    /// Extract function name from Rust function declaration
    fn extract_rust_function_name(&self, line: &str) -> Option<String> {
        if let Some(start) = line.find("fn ") {
            let after_fn = &line[start + 3..];
            if let Some(end) = after_fn.find('(') {
                Some(after_fn[..end].trim().to_string())
            } else {
                None
            }
        } else {
            None
        }
    }
    
    /// Extract function name from Python function declaration
    fn extract_python_function_name(&self, line: &str) -> Option<String> {
        if let Some(start) = line.find("def ") {
            let after_def = &line[start + 4..];
            if let Some(end) = after_def.find('(') {
                Some(after_def[..end].trim().to_string())
            } else {
                None
            }
        } else {
            None
        }
    }
    
    /// Check if line is a JavaScript function declaration
    fn is_javascript_function_declaration(&self, line: &str) -> bool {
        line.contains("function ") || 
        line.contains("=> ") || 
        (line.contains("(") && line.contains(")") && line.contains("{"))
    }
    
    /// Extract function name from JavaScript function declaration
    fn extract_javascript_function_name(&self, line: &str) -> Option<String> {
        if line.contains("function ") {
            if let Some(start) = line.find("function ") {
                let after_function = &line[start + 9..];
                if let Some(end) = after_function.find('(') {
                    return Some(after_function[..end].trim().to_string());
                }
            }
        }
        
        // Handle arrow functions and method declarations
        if let Some(arrow_pos) = line.find("=>") {
            let before_arrow = &line[..arrow_pos];
            if let Some(equals_pos) = before_arrow.rfind('=') {
                let name_part = &before_arrow[..equals_pos];
                if let Some(name) = name_part.split_whitespace().last() {
                    return Some(name.to_string());
                }
            }
        }
        
        Some("anonymous".to_string())
    }
    
    /// Count complexity keywords in Rust code
    fn count_rust_complexity_keywords(&self, line: &str) -> usize {
        let keywords = ["if", "else if", "match", "while", "for", "loop", "&&", "||", "?"];
        keywords.iter().map(|&keyword| line.matches(keyword).count()).sum()
    }
    
    /// Count complexity keywords in Python code
    fn count_python_complexity_keywords(&self, line: &str) -> usize {
        let keywords = ["if", "elif", "while", "for", "and", "or", "except", "finally"];
        keywords.iter().map(|&keyword| line.matches(keyword).count()).sum()
    }
    
    /// Count complexity keywords in JavaScript code
    fn count_javascript_complexity_keywords(&self, line: &str) -> usize {
        let keywords = ["if", "else if", "while", "for", "switch", "case", "catch", "finally", "&&", "||", "?"];
        keywords.iter().map(|&keyword| line.matches(keyword).count()).sum()
    }
    
    /// Count cognitive complexity for Rust code
    fn count_rust_cognitive_complexity(&self, line: &str, nesting_level: i32) -> usize {
        let mut complexity = 0;
        let nesting_multiplier = (nesting_level as usize).max(1);
        
        // Basic control structures
        if line.contains("if") { complexity += 1 * nesting_multiplier; }
        if line.contains("else") { complexity += 1; }
        if line.contains("match") { complexity += 1 * nesting_multiplier; }
        if line.contains("while") { complexity += 1 * nesting_multiplier; }
        if line.contains("for") { complexity += 1 * nesting_multiplier; }
        if line.contains("loop") { complexity += 1 * nesting_multiplier; }
        
        // Logical operators
        complexity += line.matches("&&").count() * nesting_multiplier;
        complexity += line.matches("||").count() * nesting_multiplier;
        
        // Recursion penalty
        if line.contains("self.") && line.contains("(") { complexity += 1; }
        
        complexity
    }
    
    /// Count cognitive complexity for Python code
    fn count_python_cognitive_complexity(&self, line: &str, nesting_level: usize) -> usize {
        let mut complexity = 0;
        let nesting_multiplier = nesting_level.max(1);
        
        // Basic control structures
        if line.contains("if ") { complexity += 1 * nesting_multiplier; }
        if line.contains("elif ") { complexity += 1; }
        if line.contains("else:") { complexity += 1; }
        if line.contains("while ") { complexity += 1 * nesting_multiplier; }
        if line.contains("for ") { complexity += 1 * nesting_multiplier; }
        if line.contains("try:") { complexity += 1 * nesting_multiplier; }
        if line.contains("except") { complexity += 1; }
        if line.contains("finally") { complexity += 1; }
        
        // Logical operators
        complexity += line.matches(" and ").count() * nesting_multiplier;
        complexity += line.matches(" or ").count() * nesting_multiplier;
        
        // Comprehensions add complexity
        if line.contains(" for ") && (line.contains("[") || line.contains("{")) {
            complexity += 1;
        }
        
        complexity
    }
    
    /// Count cognitive complexity for JavaScript code
    fn count_javascript_cognitive_complexity(&self, line: &str, nesting_level: i32) -> usize {
        let mut complexity = 0;
        let nesting_multiplier = (nesting_level as usize).max(1);
        
        // Basic control structures
        if line.contains("if") { complexity += 1 * nesting_multiplier; }
        if line.contains("else") { complexity += 1; }
        if line.contains("while") { complexity += 1 * nesting_multiplier; }
        if line.contains("for") { complexity += 1 * nesting_multiplier; }
        if line.contains("switch") { complexity += 1 * nesting_multiplier; }
        if line.contains("case") { complexity += 1; }
        if line.contains("catch") { complexity += 1; }
        if line.contains("finally") { complexity += 1; }
        
        // Logical operators
        complexity += line.matches("&&").count() * nesting_multiplier;
        complexity += line.matches("||").count() * nesting_multiplier;
        
        // Ternary operator
        complexity += line.matches("?").count() * nesting_multiplier;
        
        // Recursion penalty
        if line.contains("(") && line.contains(")") { 
            // Simple heuristic for recursive calls
            complexity += line.matches("()").count().min(1);
        }
        
        complexity
    }
    
    /// Count function parameters
    fn count_function_parameters(&self, line: &str) -> usize {
        if let Some(start) = line.find('(') {
            if let Some(end) = line.rfind(')') {
                if end > start {
                    let params_str = &line[start + 1..end];
                    if params_str.trim().is_empty() {
                        return 0;
                    }
                    
                    // Simple parameter counting (split by comma)
                    let param_count = params_str.split(',').count();
                
                                    // Adjust for common patterns
                    if params_str.contains("self") || params_str.contains("this") {
                        return param_count.saturating_sub(1);
                    }
                    
                    return param_count;
                }
            }
        }
        0
    }

    /// Analyze Rust structures
    fn analyze_rust_structures(&self, lines: &[String]) -> Result<Vec<StructureInfo>> {
        let mut structures = Vec::new();
        let mut current_structure: Option<StructureInfo> = None;
        let mut brace_count = 0;
        let mut in_structure = false;
        
        for (line_num, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            
            // Skip comments and empty lines
            if trimmed.starts_with("//") || trimmed.is_empty() {
                continue;
            }
            
            // Structure declaration detection
            if let Some((structure_type, name, visibility)) = self.detect_rust_structure(trimmed) {
                current_structure = Some(StructureInfo {
                    name,
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
            
            if in_structure {
                if let Some(ref mut structure) = current_structure {
                    structure.line_count += 1;
                    structure.end_line = line_num + 1;
                    
                    // Count braces for structure end detection
                    let open_braces = trimmed.matches('{').count();
                    let close_braces = trimmed.matches('}').count();
                    brace_count += open_braces as i32 - close_braces as i32;
                    
                    // Count properties (field declarations)
                    if trimmed.contains(':') && !trimmed.contains("fn ") && !trimmed.contains("//") {
                        structure.properties += 1;
                    }
                    
                    // Structure end detection
                    if brace_count <= 0 && close_braces > 0 {
                        structures.push(structure.clone());
                        current_structure = None;
                        in_structure = false;
                    }
                }
            }
        }
        
        Ok(structures)
    }
    
    /// Detect Rust structure type and name
    fn detect_rust_structure(&self, line: &str) -> Option<(StructureType, String, Visibility)> {
        let visibility = if line.starts_with("pub ") {
            Visibility::Public
        } else {
            Visibility::Private
        };
        
        if line.contains("struct ") {
            if let Some(name) = self.extract_structure_name(line, "struct ") {
                return Some((StructureType::Struct, name, visibility));
            }
        }
        
        if line.contains("enum ") {
            if let Some(name) = self.extract_structure_name(line, "enum ") {
                return Some((StructureType::Enum, name, visibility));
            }
        }
        
        if line.contains("trait ") {
            if let Some(name) = self.extract_structure_name(line, "trait ") {
                return Some((StructureType::Trait, name, visibility));
            }
        }
        
        if line.contains("impl ") {
            if let Some(name) = self.extract_structure_name(line, "impl ") {
                return Some((StructureType::Class, name, visibility)); // Treat impl as class-like
            }
        }
        
        if line.contains("mod ") {
            if let Some(name) = self.extract_structure_name(line, "mod ") {
                return Some((StructureType::Module, name, visibility));
            }
        }
        
        None
    }
    
    /// Extract structure name from declaration
    fn extract_structure_name(&self, line: &str, keyword: &str) -> Option<String> {
        if let Some(start) = line.find(keyword) {
            let after_keyword = &line[start + keyword.len()..];
            let name_part = after_keyword.split_whitespace().next()?;
            let name = name_part.split('<').next()?.split('{').next()?.trim();
            if !name.is_empty() {
                Some(name.to_string())
            } else {
                None
            }
        } else {
            None
        }
    }
    
    /// Analyze Python structures
    fn analyze_python_structures(&self, lines: &[String]) -> Result<Vec<StructureInfo>> {
        let mut structures = Vec::new();
        let mut current_structure: Option<StructureInfo> = None;
        let mut structure_indent = 0;
        
        for (line_num, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            
            // Skip comments and empty lines
            if trimmed.starts_with("#") || trimmed.is_empty() {
                continue;
            }
            
            // Calculate indentation
            let current_indent = line.len() - line.trim_start().len();
            
            // Structure declaration detection
            if let Some((structure_type, name)) = self.detect_python_structure(trimmed) {
                // Save previous structure if exists
                if let Some(structure) = current_structure.take() {
                    structures.push(structure);
                }
                
                current_structure = Some(StructureInfo {
                    name,
                    structure_type,
                    line_count: 0,
                    start_line: line_num + 1,
                    end_line: line_num + 1,
                    methods: Vec::new(),
                    properties: 0,
                    visibility: Visibility::Public, // Python doesn't have strict visibility
                    inheritance_depth: 0,
                    interface_count: 0,
                });
                structure_indent = current_indent;
            }
            
            if let Some(ref mut structure) = current_structure {
                // Check if we're still in the structure
                if current_indent <= structure_indent && line_num > structure.start_line - 1 && !trimmed.is_empty() {
                    // Structure ended
                    structures.push(structure.clone());
                    current_structure = None;
                    continue;
                }
                
                if current_indent > structure_indent {
                    structure.line_count += 1;
                    structure.end_line = line_num + 1;
                    
                    // Count properties (self.property assignments)
                    if trimmed.starts_with("self.") && trimmed.contains('=') {
                        structure.properties += 1;
                    }
                }
            }
        }
        
        // Add the last structure if exists
        if let Some(structure) = current_structure {
            structures.push(structure);
        }
        
        Ok(structures)
    }
    
    /// Detect Python structure type and name
    fn detect_python_structure(&self, line: &str) -> Option<(StructureType, String)> {
        if line.starts_with("class ") {
            if let Some(name) = self.extract_python_class_name(line) {
                return Some((StructureType::Class, name));
            }
        }
        
        // Python doesn't have interfaces, but we can detect ABC classes
        if line.contains("ABC") && line.starts_with("class ") {
            if let Some(name) = self.extract_python_class_name(line) {
                return Some((StructureType::Interface, name));
            }
        }
        
        None
    }
    
    /// Extract Python class name
    fn extract_python_class_name(&self, line: &str) -> Option<String> {
        if let Some(start) = line.find("class ") {
            let after_class = &line[start + 6..];
            let name_part = after_class.split('(').next()?.split(':').next()?.trim();
            if !name_part.is_empty() {
                Some(name_part.to_string())
            } else {
                None
            }
        } else {
            None
        }
    }
    
    /// Analyze JavaScript/TypeScript structures
    fn analyze_javascript_structures(&self, lines: &[String]) -> Result<Vec<StructureInfo>> {
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
            if let Some((structure_type, name, visibility)) = self.detect_javascript_structure(trimmed) {
                current_structure = Some(StructureInfo {
                    name,
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
            
            if in_structure {
                if let Some(ref mut structure) = current_structure {
                    structure.line_count += 1;
                    structure.end_line = line_num + 1;
                    
                    // Count braces for structure end detection
                    let open_braces = trimmed.matches('{').count();
                    let close_braces = trimmed.matches('}').count();
                    brace_count += open_braces as i32 - close_braces as i32;
                    
                    // Count properties (this.property or property:)
                    if (trimmed.contains("this.") && trimmed.contains('=')) || 
                       (trimmed.contains(':') && !trimmed.contains("function") && !trimmed.contains("=>")) {
                        structure.properties += 1;
                    }
                    
                    // Structure end detection
                    if brace_count <= 0 && close_braces > 0 {
                        structures.push(structure.clone());
                        current_structure = None;
                        in_structure = false;
                    }
                }
            }
        }
        
        Ok(structures)
    }
    
    /// Detect JavaScript/TypeScript structure type and name
    fn detect_javascript_structure(&self, line: &str) -> Option<(StructureType, String, Visibility)> {
        let visibility = if line.contains("export ") {
            Visibility::Public
        } else {
            Visibility::Private
        };
        
        if line.contains("class ") {
            if let Some(name) = self.extract_javascript_class_name(line) {
                return Some((StructureType::Class, name, visibility));
            }
        }
        
        if line.contains("interface ") {
            if let Some(name) = self.extract_structure_name(line, "interface ") {
                return Some((StructureType::Interface, name, visibility));
            }
        }
        
        if line.contains("enum ") {
            if let Some(name) = self.extract_structure_name(line, "enum ") {
                return Some((StructureType::Enum, name, visibility));
            }
        }
        
        None
    }
    
    /// Extract JavaScript/TypeScript class name
    fn extract_javascript_class_name(&self, line: &str) -> Option<String> {
        if let Some(start) = line.find("class ") {
            let after_class = &line[start + 6..];
            let name_part = after_class.split_whitespace().next()?;
            let name = name_part.split('{').next()?.split('(').next()?.trim();
            if !name.is_empty() {
                Some(name.to_string())
            } else {
                None
            }
        } else {
            None
        }
    }
    
    // Placeholder implementations for other languages
    fn analyze_java_structures(&self, _lines: &[String]) -> Result<Vec<StructureInfo>> {
        Ok(Vec::new()) // TODO: Implement Java structure analysis
    }
    
    fn analyze_cpp_structures(&self, _lines: &[String]) -> Result<Vec<StructureInfo>> {
        Ok(Vec::new()) // TODO: Implement C++ structure analysis
    }
    
    fn analyze_go_structures(&self, _lines: &[String]) -> Result<Vec<StructureInfo>> {
        Ok(Vec::new()) // TODO: Implement Go structure analysis
    }
    
    fn analyze_csharp_structures(&self, _lines: &[String]) -> Result<Vec<StructureInfo>> {
        Ok(Vec::new()) // TODO: Implement C# structure analysis
    }
    
    fn analyze_php_structures(&self, _lines: &[String]) -> Result<Vec<StructureInfo>> {
        Ok(Vec::new()) // TODO: Implement PHP structure analysis
    }
    
    fn analyze_ruby_structures(&self, _lines: &[String]) -> Result<Vec<StructureInfo>> {
        Ok(Vec::new()) // TODO: Implement Ruby structure analysis
    }
    
    fn analyze_swift_structures(&self, _lines: &[String]) -> Result<Vec<StructureInfo>> {
        Ok(Vec::new()) // TODO: Implement Swift structure analysis
    }
    
    fn analyze_kotlin_structures(&self, _lines: &[String]) -> Result<Vec<StructureInfo>> {
        Ok(Vec::new()) // TODO: Implement Kotlin structure analysis
    }
}

impl Default for CodeAnalyzer {
    fn default() -> Self {
        Self::new()
    }
} 