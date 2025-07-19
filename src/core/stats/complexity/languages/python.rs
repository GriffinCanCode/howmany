use crate::utils::errors::Result;
use super::super::types::{FunctionInfo, StructureInfo, StructureType, Visibility};
use super::LanguageAnalyzer;

/// Python language complexity analyzer
pub struct PythonAnalyzer;

impl PythonAnalyzer {
    pub fn new() -> Self {
        Self
    }
    
    /// Extract function name from Python function declaration
    fn extract_function_name(&self, line: &str) -> Option<String> {
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
    
    /// Count complexity keywords in Python code
    fn count_complexity_keywords(&self, line: &str) -> usize {
        let keywords = ["if", "elif", "while", "for", "and", "or", "except", "finally"];
        keywords.iter().map(|&keyword| line.matches(keyword).count()).sum()
    }
    
    /// Count cognitive complexity for Python code
    fn count_cognitive_complexity(&self, line: &str, nesting_level: usize) -> usize {
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
                    if params_str.contains("self") {
                        return param_count.saturating_sub(1);
                    }
                    
                    return param_count;
                }
            }
        }
        0
    }
    
    /// Detect Python structure type and name
    fn detect_structure(&self, line: &str) -> Option<(StructureType, String)> {
        if line.starts_with("class ") {
            if let Some(name) = self.extract_class_name(line) {
                return Some((StructureType::Class, name));
            }
        }
        
        // Python doesn't have interfaces, but we can detect ABC classes
        if line.contains("ABC") && line.starts_with("class ") {
            if let Some(name) = self.extract_class_name(line) {
                return Some((StructureType::Interface, name));
            }
        }
        
        None
    }
    
    /// Extract Python class name
    fn extract_class_name(&self, line: &str) -> Option<String> {
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
}

impl LanguageAnalyzer for PythonAnalyzer {
    fn analyze_functions(&self, lines: &[String]) -> Result<Vec<FunctionInfo>> {
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
                if let Some(func_name) = self.extract_function_name(trimmed) {
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
                        visibility: Visibility::Public,});
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
                    func.cyclomatic_complexity += self.count_complexity_keywords(trimmed);
                    
                    // Calculate cognitive complexity
                    func.cognitive_complexity += self.count_cognitive_complexity(trimmed, relative_indent);
                    
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
    
    fn analyze_structures(&self, lines: &[String]) -> Result<Vec<StructureInfo>> {
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
            if let Some((structure_type, name)) = self.detect_structure(trimmed) {
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
    
    fn language_name(&self) -> &'static str {
        "Python"
    }
    
    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["py"]
    }
}

impl Default for PythonAnalyzer {
    fn default() -> Self {
        Self::new()
    }
} 