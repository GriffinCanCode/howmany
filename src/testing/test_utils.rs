use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;
use crate::utils::errors::Result;

pub struct TestProject {
    pub temp_dir: TempDir,
    pub root: PathBuf,
}

impl TestProject {
    pub fn new() -> Result<Self> {
        let temp_dir = TempDir::new()?;
        let root = temp_dir.path().to_path_buf();
        
        Ok(Self { temp_dir, root })
    }
    
    pub fn create_file(&self, path: &str, content: &str) -> Result<PathBuf> {
        let file_path = self.root.join(path);
        
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        fs::write(&file_path, content)?;
        Ok(file_path)
    }
    
    pub fn create_rust_file(&self, path: &str, functions: usize, comments: usize) -> Result<PathBuf> {
        let mut content = String::new();
        content.push_str("// File header comment\n");
        content.push_str("use std::collections::HashMap;\n\n");
        
        for i in 0..functions {
            content.push_str(&format!("/// Documentation for function {}\n", i));
            content.push_str(&format!("pub fn function_{}() {{\n", i));
            content.push_str("    // Implementation comment\n");
            content.push_str("    println!(\"Hello from function\");\n");
            content.push_str("}\n\n");
        }
        
        for i in 0..comments {
            content.push_str(&format!("// Additional comment {}\n", i));
        }
        
        self.create_file(path, &content)
    }
    
    pub fn create_python_file(&self, path: &str, functions: usize) -> Result<PathBuf> {
        let mut content = String::new();
        content.push_str("#!/usr/bin/env python3\n");
        content.push_str("\"\"\"Module docstring\"\"\"\n\n");
        content.push_str("import os\nimport sys\n\n");
        
        for i in 0..functions {
            content.push_str(&format!("def function_{}():\n", i));
            content.push_str(&format!("    \"\"\"Function {} docstring\"\"\"\n", i));
            content.push_str("    # Implementation comment\n");
            content.push_str("    print(\"Hello from Python\")\n\n");
        }
        
        self.create_file(path, &content)
    }
    
    pub fn create_javascript_file(&self, path: &str, functions: usize) -> Result<PathBuf> {
        let mut content = String::new();
        content.push_str("/**\n * Module description\n */\n\n");
        content.push_str("const express = require('express');\n\n");
        
        for i in 0..functions {
            content.push_str(&format!("/**\n * Function {} description\n */\n", i));
            content.push_str(&format!("function function_{}() {{\n", i));
            content.push_str("    // Implementation comment\n");
            content.push_str("    console.log('Hello from JavaScript');\n");
            content.push_str("}\n\n");
        }
        
        self.create_file(path, &content)
    }
    
    pub fn create_directory(&self, path: &str) -> Result<PathBuf> {
        let dir_path = self.root.join(path);
        fs::create_dir_all(&dir_path)?;
        Ok(dir_path)
    }
    
    pub fn create_node_modules(&self) -> Result<()> {
        self.create_directory("node_modules")?;
        self.create_file("node_modules/express/package.json", r#"{"name": "express"}"#)?;
        self.create_file("node_modules/express/index.js", "module.exports = {};")?;
        Ok(())
    }
    
    pub fn create_target_dir(&self) -> Result<()> {
        self.create_directory("target")?;
        self.create_directory("target/debug")?;
        self.create_file("target/debug/myapp", "binary content")?;
        Ok(())
    }
    
    pub fn create_gitignore(&self, patterns: &[&str]) -> Result<PathBuf> {
        let content = patterns.join("\n");
        self.create_file(".gitignore", &content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::counter::CodeCounter;
    use crate::core::detector::FileDetector;
    
    #[test]
    fn test_project_creation() {
        let project = TestProject::new().unwrap();
        assert!(project.root.exists());
    }
    
    #[test]
    fn test_rust_file_creation() {
        let project = TestProject::new().unwrap();
        let file_path = project.create_rust_file("src/main.rs", 3, 2).unwrap();
        
        assert!(file_path.exists());
        
        let counter = CodeCounter::new();
        let stats = counter.count_file(&file_path).unwrap();
        
        assert!(stats.total_lines > 0);
        assert!(stats.code_lines > 0);
        assert!(stats.comment_lines > 0);
        assert!(stats.doc_lines > 0);
    }
    
    #[test]
    fn test_file_detection() {
        let project = TestProject::new().unwrap();
        
        // Create user files
        project.create_rust_file("src/main.rs", 1, 1).unwrap();
        project.create_python_file("script.py", 1).unwrap();
        
        // Create dependency files
        project.create_node_modules().unwrap();
        project.create_target_dir().unwrap();
        
        let detector = FileDetector::new();
        
        // User files should be detected
        assert!(detector.is_user_created_file(&project.root.join("src/main.rs")));
        assert!(detector.is_user_created_file(&project.root.join("script.py")));
        
        // Dependency files should not be detected
        assert!(!detector.is_user_created_file(&project.root.join("node_modules/express/index.js")));
        assert!(!detector.is_user_created_file(&project.root.join("target/debug/myapp")));
    }
} 