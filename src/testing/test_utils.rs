use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;
use crate::utils::errors::Result;

pub struct TestProject {
    pub temp_dir: TempDir,
    pub root: PathBuf,
}

impl TestProject {
    pub fn new(name: &str) -> Result<Self> {
        let temp_dir = TempDir::new()?;
        let root = temp_dir.path().join(name);
        fs::create_dir_all(&root)?;
        
        Ok(Self { temp_dir, root })
    }
    
    pub fn path(&self) -> &std::path::Path {
        &self.root
    }
    
    pub fn create_file(&self, path: &str, content: &str) -> Result<PathBuf> {
        let file_path = self.root.join(path);
        
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        fs::write(&file_path, content)?;
        Ok(file_path)
    }
    
    pub fn create_file_binary(&self, path: &str, content: &[u8]) -> Result<PathBuf> {
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
        content.push_str("use std::collections::HashMap;\n");
        content.push_str("use std::io::Result;\n\n");
        
        for i in 0..functions {
            content.push_str(&format!("/// Documentation for function {}\n", i));
            content.push_str(&format!("/// This function demonstrates complexity level {}\n", i % 3 + 1));
            content.push_str(&format!("pub fn function_{}(param: i32) -> Result<String> {{\n", i));
            content.push_str("    // Implementation comment\n");
            
            // Add some complexity based on function index
            match i % 3 {
                0 => {
                    content.push_str("    if param > 0 {\n");
                    content.push_str("        Ok(format!(\"Positive: {}\", param))\n");
                    content.push_str("    } else {\n");
                    content.push_str("        Ok(\"Zero or negative\".to_string())\n");
                    content.push_str("    }\n");
                }
                1 => {
                    content.push_str("    match param {\n");
                    content.push_str("        0 => Ok(\"Zero\".to_string()),\n");
                    content.push_str("        1..=10 => Ok(\"Small\".to_string()),\n");
                    content.push_str("        11..=100 => Ok(\"Medium\".to_string()),\n");
                    content.push_str("        _ => Ok(\"Large\".to_string()),\n");
                    content.push_str("    }\n");
                }
                _ => {
                    content.push_str("    for i in 0..param {\n");
                    content.push_str("        if i % 2 == 0 {\n");
                    content.push_str("            println!(\"Even: {}\", i);\n");
                    content.push_str("        } else {\n");
                    content.push_str("            println!(\"Odd: {}\", i);\n");
                    content.push_str("        }\n");
                    content.push_str("    }\n");
                    content.push_str("    Ok(\"Processed\".to_string())\n");
                }
            }
            
            content.push_str("}\n\n");
        }
        
        for i in 0..comments {
            content.push_str(&format!("// Additional comment {}\n", i));
        }
        
        // Add some module-level documentation
        content.push_str("/// Module-level documentation\n");
        content.push_str("/// This module contains utility functions\n");
        
        self.create_file(path, &content)
    }
    
    pub fn create_python_file(&self, path: &str, functions: usize) -> Result<PathBuf> {
        let mut content = String::new();
        content.push_str("#!/usr/bin/env python3\n");
        content.push_str("\"\"\"\nModule docstring\nThis module demonstrates Python code analysis.\n\"\"\"\n\n");
        content.push_str("import os\nimport sys\nimport json\nfrom typing import Optional, List, Dict\n\n");
        
        for i in 0..functions {
            content.push_str(&format!("def function_{}(param: int) -> Optional[str]:\n", i));
            content.push_str(&format!("    \"\"\"\n    Function {} docstring\n", i));
            content.push_str("    \n");
            content.push_str("    Args:\n");
            content.push_str("        param (int): Input parameter\n");
            content.push_str("    \n");
            content.push_str("    Returns:\n");
            content.push_str("        Optional[str]: Result string or None\n");
            content.push_str("    \"\"\"\n");
            content.push_str("    # Implementation comment\n");
            
            // Add complexity based on function index
            match i % 3 {
                0 => {
                    content.push_str("    if param > 0:\n");
                    content.push_str("        return f\"Positive: {param}\"\n");
                    content.push_str("    else:\n");
                    content.push_str("        return \"Zero or negative\"\n");
                }
                1 => {
                    content.push_str("    try:\n");
                    content.push_str("        result = param * 2\n");
                    content.push_str("        if result > 100:\n");
                    content.push_str("            raise ValueError(\"Result too large\")\n");
                    content.push_str("        return str(result)\n");
                    content.push_str("    except ValueError as e:\n");
                    content.push_str("        print(f\"Error: {e}\")\n");
                    content.push_str("        return None\n");
                }
                _ => {
                    content.push_str("    results = []\n");
                    content.push_str("    for i in range(param):\n");
                    content.push_str("        if i % 2 == 0:\n");
                    content.push_str("            results.append(f\"Even: {i}\")\n");
                    content.push_str("        else:\n");
                    content.push_str("            results.append(f\"Odd: {i}\")\n");
                    content.push_str("    return \"\\n\".join(results)\n");
                }
            }
            
            content.push_str("\n\n");
        }
        
        // Add a class for more complexity
        content.push_str("class DataProcessor:\n");
        content.push_str("    \"\"\"\n    A class for processing data.\n    \"\"\"\n");
        content.push_str("    \n");
        content.push_str("    def __init__(self, data: List[Dict]):\n");
        content.push_str("        \"\"\"Initialize with data.\"\"\"\n");
        content.push_str("        self.data = data\n");
        content.push_str("    \n");
        content.push_str("    def process(self) -> Dict:\n");
        content.push_str("        \"\"\"Process the data.\"\"\"\n");
        content.push_str("        # Processing logic\n");
        content.push_str("        return {\"processed\": len(self.data)}\n");
        
        self.create_file(path, &content)
    }
    
    pub fn create_javascript_file(&self, path: &str, functions: usize) -> Result<PathBuf> {
        let mut content = String::new();
        content.push_str("/**\n * Module description\n * This module demonstrates JavaScript code analysis.\n */\n\n");
        content.push_str("const express = require('express');\n");
        content.push_str("const { promisify } = require('util');\n\n");
        
        for i in 0..functions {
            content.push_str(&format!("/**\n * Function {} description\n", i));
            content.push_str(" * @param {{number}} param - Input parameter\n");
            content.push_str(" * @returns {{Promise<string>}} Promise resolving to result\n");
            content.push_str(" */\n");
            content.push_str(&format!("async function function_{}(param) {{\n", i));
            content.push_str("    // Implementation comment\n");
            
            // Add complexity based on function index
            match i % 3 {
                0 => {
                    content.push_str("    if (param > 0) {\n");
                    content.push_str("        return `Positive: ${param}`;\n");
                    content.push_str("    } else {\n");
                    content.push_str("        return 'Zero or negative';\n");
                    content.push_str("    }\n");
                }
                1 => {
                    content.push_str("    try {\n");
                    content.push_str("        const result = await processAsync(param);\n");
                    content.push_str("        if (result > 100) {\n");
                    content.push_str("            throw new Error('Result too large');\n");
                    content.push_str("        }\n");
                    content.push_str("        return result.toString();\n");
                    content.push_str("    } catch (error) {\n");
                    content.push_str("        console.error('Error:', error.message);\n");
                    content.push_str("        return null;\n");
                    content.push_str("    }\n");
                }
                _ => {
                    content.push_str("    const results = [];\n");
                    content.push_str("    for (let i = 0; i < param; i++) {\n");
                    content.push_str("        if (i % 2 === 0) {\n");
                    content.push_str("            results.push(`Even: ${i}`);\n");
                    content.push_str("        } else {\n");
                    content.push_str("            results.push(`Odd: ${i}`);\n");
                    content.push_str("        }\n");
                    content.push_str("    }\n");
                    content.push_str("    return results.join('\\n');\n");
                }
            }
            
            content.push_str("}\n\n");
        }
        
        // Add helper function
        content.push_str("/**\n * Helper function for async processing\n */\n");
        content.push_str("function processAsync(value) {\n");
        content.push_str("    return new Promise((resolve) => {\n");
        content.push_str("        setTimeout(() => resolve(value * 2), 10);\n");
        content.push_str("    });\n");
        content.push_str("}\n\n");
        
        // Add export
        content.push_str("module.exports = {\n");
        for i in 0..functions {
            content.push_str(&format!("    function_{},\n", i));
        }
        content.push_str("    processAsync\n");
        content.push_str("};\n");
        
        self.create_file(path, &content)
    }
    
    pub fn create_typescript_file(&self, path: &str, functions: usize) -> Result<PathBuf> {
        let mut content = String::new();
        content.push_str("/**\n * TypeScript module for code analysis demonstration\n */\n\n");
        content.push_str("interface DataItem {\n");
        content.push_str("    id: number;\n");
        content.push_str("    name: string;\n");
        content.push_str("    value: number;\n");
        content.push_str("}\n\n");
        
        content.push_str("type ProcessResult = {\n");
        content.push_str("    success: boolean;\n");
        content.push_str("    data?: string;\n");
        content.push_str("    error?: string;\n");
        content.push_str("};\n\n");
        
        for i in 0..functions {
            content.push_str(&format!("/**\n * Function {} with TypeScript types\n", i));
            content.push_str(" * @param param Input parameter\n");
            content.push_str(" * @returns Promise with processing result\n");
            content.push_str(" */\n");
            content.push_str(&format!("export async function function_{}(param: number): Promise<ProcessResult> {{\n", i));
            content.push_str("    // Implementation with type safety\n");
            
            match i % 2 {
                0 => {
                    content.push_str("    if (param > 0) {\n");
                    content.push_str("        return {\n");
                    content.push_str("            success: true,\n");
                    content.push_str("            data: `Processed: ${param}`\n");
                    content.push_str("        };\n");
                    content.push_str("    } else {\n");
                    content.push_str("        return {\n");
                    content.push_str("            success: false,\n");
                    content.push_str("            error: 'Invalid parameter'\n");
                    content.push_str("        };\n");
                    content.push_str("    }\n");
                }
                _ => {
                    content.push_str("    try {\n");
                    content.push_str("        const items: DataItem[] = [];\n");
                    content.push_str("        for (let i = 0; i < param; i++) {\n");
                    content.push_str("            items.push({ id: i, name: `Item ${i}`, value: i * 2 });\n");
                    content.push_str("        }\n");
                    content.push_str("        return {\n");
                    content.push_str("            success: true,\n");
                    content.push_str("            data: JSON.stringify(items)\n");
                    content.push_str("        };\n");
                    content.push_str("    } catch (error) {\n");
                    content.push_str("        return {\n");
                    content.push_str("            success: false,\n");
                    content.push_str("            error: error instanceof Error ? error.message : 'Unknown error'\n");
                    content.push_str("        };\n");
                    content.push_str("    }\n");
                }
            }
            
            content.push_str("}\n\n");
        }
        
        // Add a class
        content.push_str("export class DataManager {\n");
        content.push_str("    private items: DataItem[] = [];\n");
        content.push_str("    \n");
        content.push_str("    /**\n");
        content.push_str("     * Add an item to the collection\n");
        content.push_str("     */\n");
        content.push_str("    addItem(item: DataItem): void {\n");
        content.push_str("        this.items.push(item);\n");
        content.push_str("    }\n");
        content.push_str("    \n");
        content.push_str("    /**\n");
        content.push_str("     * Get all items\n");
        content.push_str("     */\n");
        content.push_str("    getItems(): DataItem[] {\n");
        content.push_str("        return [...this.items];\n");
        content.push_str("    }\n");
        content.push_str("}\n");
        
        self.create_file(path, &content)
    }
    
    pub fn create_html_file(&self, path: &str) -> Result<PathBuf> {
        let content = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Test Application</title>
    <style>
        body {
            font-family: Arial, sans-serif;
            margin: 0;
            padding: 20px;
            background-color: #f5f5f5;
        }
        
        .container {
            max-width: 800px;
            margin: 0 auto;
            background: white;
            padding: 20px;
            border-radius: 8px;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }
        
        .header {
            text-align: center;
            margin-bottom: 30px;
        }
        
        .form-group {
            margin-bottom: 15px;
        }
        
        label {
            display: block;
            margin-bottom: 5px;
            font-weight: bold;
        }
        
        input, textarea, select {
            width: 100%;
            padding: 8px;
            border: 1px solid #ddd;
            border-radius: 4px;
        }
        
        button {
            background-color: #007bff;
            color: white;
            padding: 10px 20px;
            border: none;
            border-radius: 4px;
            cursor: pointer;
        }
        
        button:hover {
            background-color: #0056b3;
        }
        
        .results {
            margin-top: 20px;
            padding: 15px;
            background-color: #f8f9fa;
            border-radius: 4px;
        }
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>Code Analysis Test Application</h1>
            <p>This is a test HTML file for code analysis</p>
        </div>
        
        <form id="analysisForm">
            <div class="form-group">
                <label for="projectPath">Project Path:</label>
                <input type="text" id="projectPath" name="projectPath" required>
            </div>
            
            <div class="form-group">
                <label for="language">Primary Language:</label>
                <select id="language" name="language">
                    <option value="rust">Rust</option>
                    <option value="python">Python</option>
                    <option value="javascript">JavaScript</option>
                    <option value="typescript">TypeScript</option>
                </select>
            </div>
            
            <div class="form-group">
                <label for="options">Analysis Options:</label>
                <textarea id="options" name="options" rows="3" placeholder="Enter analysis options..."></textarea>
            </div>
            
            <button type="submit">Run Analysis</button>
        </form>
        
        <div id="results" class="results" style="display: none;">
            <h3>Analysis Results</h3>
            <div id="resultContent"></div>
        </div>
    </div>
    
    <script>
        document.getElementById('analysisForm').addEventListener('submit', function(e) {
            e.preventDefault();
            
            // Simulate analysis
            const results = document.getElementById('results');
            const content = document.getElementById('resultContent');
            
            content.innerHTML = `
                <p><strong>Files analyzed:</strong> 42</p>
                <p><strong>Lines of code:</strong> 1,337</p>
                <p><strong>Functions found:</strong> 23</p>
                <p><strong>Complexity score:</strong> 7.5/10</p>
            `;
            
            results.style.display = 'block';
        });
    </script>
</body>
</html>"#;
        
        self.create_file(path, content)
    }
    
    pub fn create_css_file(&self, path: &str) -> Result<PathBuf> {
        let content = r#"/* CSS file for code analysis testing */

:root {
    --primary-color: #007bff;
    --secondary-color: #6c757d;
    --success-color: #28a745;
    --danger-color: #dc3545;
    --warning-color: #ffc107;
    --info-color: #17a2b8;
    --light-color: #f8f9fa;
    --dark-color: #343a40;
}

* {
    box-sizing: border-box;
    margin: 0;
    padding: 0;
}

body {
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
    line-height: 1.6;
    color: var(--dark-color);
    background-color: var(--light-color);
}

.container {
    max-width: 1200px;
    margin: 0 auto;
    padding: 0 20px;
}

.header {
    background: linear-gradient(135deg, var(--primary-color), var(--info-color));
    color: white;
    padding: 2rem 0;
    text-align: center;
}

.header h1 {
    font-size: 2.5rem;
    margin-bottom: 0.5rem;
}

.header p {
    font-size: 1.2rem;
    opacity: 0.9;
}

.main-content {
    padding: 2rem 0;
}

.card {
    background: white;
    border-radius: 8px;
    box-shadow: 0 2px 10px rgba(0, 0, 0, 0.1);
    padding: 1.5rem;
    margin-bottom: 1.5rem;
}

.card-header {
    border-bottom: 1px solid #eee;
    padding-bottom: 1rem;
    margin-bottom: 1rem;
}

.card-title {
    font-size: 1.5rem;
    color: var(--primary-color);
}

.btn {
    display: inline-block;
    padding: 0.5rem 1rem;
    border: none;
    border-radius: 4px;
    text-decoration: none;
    text-align: center;
    cursor: pointer;
    transition: all 0.3s ease;
}

.btn-primary {
    background-color: var(--primary-color);
    color: white;
}

.btn-primary:hover {
    background-color: #0056b3;
}

.btn-secondary {
    background-color: var(--secondary-color);
    color: white;
}

.btn-secondary:hover {
    background-color: #545b62;
}

.grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
    gap: 1.5rem;
}

.stat-card {
    background: white;
    padding: 1.5rem;
    border-radius: 8px;
    text-align: center;
    box-shadow: 0 2px 5px rgba(0, 0, 0, 0.1);
}

.stat-number {
    font-size: 2.5rem;
    font-weight: bold;
    color: var(--primary-color);
}

.stat-label {
    color: var(--secondary-color);
    margin-top: 0.5rem;
}

.progress-bar {
    width: 100%;
    height: 20px;
    background-color: #e9ecef;
    border-radius: 10px;
    overflow: hidden;
    margin: 1rem 0;
}

.progress-fill {
    height: 100%;
    background: linear-gradient(90deg, var(--success-color), var(--info-color));
    transition: width 0.3s ease;
}

.table {
    width: 100%;
    border-collapse: collapse;
    margin-top: 1rem;
}

.table th,
.table td {
    padding: 0.75rem;
    text-align: left;
    border-bottom: 1px solid #dee2e6;
}

.table th {
    background-color: var(--light-color);
    font-weight: 600;
    color: var(--dark-color);
}

.table tbody tr:hover {
    background-color: #f8f9fa;
}

.footer {
    background-color: var(--dark-color);
    color: white;
    text-align: center;
    padding: 2rem 0;
    margin-top: 3rem;
}

@media (max-width: 768px) {
    .container {
        padding: 0 10px;
    }
    
    .header h1 {
        font-size: 2rem;
    }
    
    .grid {
        grid-template-columns: 1fr;
    }
    
    .stat-number {
        font-size: 2rem;
    }
}

/* Animation classes */
.fade-in {
    animation: fadeIn 0.5s ease-in;
}

@keyframes fadeIn {
    from {
        opacity: 0;
        transform: translateY(20px);
    }
    to {
        opacity: 1;
        transform: translateY(0);
    }
}

.slide-in {
    animation: slideIn 0.3s ease-out;
}

@keyframes slideIn {
    from {
        transform: translateX(-100%);
    }
    to {
        transform: translateX(0);
    }
}

/* Utility classes */
.text-center { text-align: center; }
.text-left { text-align: left; }
.text-right { text-align: right; }

.mt-1 { margin-top: 0.25rem; }
.mt-2 { margin-top: 0.5rem; }
.mt-3 { margin-top: 1rem; }
.mt-4 { margin-top: 1.5rem; }
.mt-5 { margin-top: 3rem; }

.mb-1 { margin-bottom: 0.25rem; }
.mb-2 { margin-bottom: 0.5rem; }
.mb-3 { margin-bottom: 1rem; }
.mb-4 { margin-bottom: 1.5rem; }
.mb-5 { margin-bottom: 3rem; }

.p-1 { padding: 0.25rem; }
.p-2 { padding: 0.5rem; }
.p-3 { padding: 1rem; }
.p-4 { padding: 1.5rem; }
.p-5 { padding: 3rem; }
"#;
        
        self.create_file(path, content)
    }
    
    pub fn create_directory(&self, path: &str) -> Result<PathBuf> {
        let dir_path = self.root.join(path);
        fs::create_dir_all(&dir_path)?;
        Ok(dir_path)
    }
    
    pub fn create_dir(&self, path: &str) -> Result<PathBuf> {
        self.create_directory(path)
    }
    
    pub fn create_node_modules(&self) -> Result<()> {
        self.create_directory("node_modules")?;
        self.create_directory("node_modules/express")?;
        self.create_directory("node_modules/lodash")?;
        self.create_directory("node_modules/@types")?;
        
        self.create_file("node_modules/express/package.json", r#"{
  "name": "express",
  "version": "4.18.2",
  "main": "lib/express.js",
  "dependencies": {
    "accepts": "~1.3.8",
    "array-flatten": "1.1.1",
    "body-parser": "1.20.1"
  }
}"#)?;
        
        self.create_file("node_modules/express/lib/express.js", r#"
'use strict';

module.exports = require('./application');
module.exports.Router = require('./router');
module.exports.static = require('serve-static');

// Middleware exports
module.exports.json = require('body-parser').json;
module.exports.urlencoded = require('body-parser').urlencoded;
"#)?;
        
        self.create_file("node_modules/lodash/package.json", r#"{
  "name": "lodash",
  "version": "4.17.21",
  "main": "lodash.js",
  "description": "Lodash modular utilities."
}"#)?;
        
        self.create_file("node_modules/lodash/lodash.js", r#"
(function() {
  'use strict';
  
  var _ = {};
  
  _.isArray = Array.isArray || function(obj) {
    return Object.prototype.toString.call(obj) === '[object Array]';
  };
  
  _.map = function(collection, iteratee) {
    var result = [];
    for (var i = 0; i < collection.length; i++) {
      result.push(iteratee(collection[i], i, collection));
    }
    return result;
  };
  
  if (typeof module !== 'undefined' && module.exports) {
    module.exports = _;
  } else {
    window._ = _;
  }
})();
"#)?;
        
        Ok(())
    }
    
    pub fn create_target_dir(&self) -> Result<()> {
        self.create_directory("target")?;
        self.create_directory("target/debug")?;
        self.create_directory("target/release")?;
        self.create_directory("target/debug/deps")?;
        self.create_directory("target/release/deps")?;
        
        // Create some build artifacts
        self.create_file_binary("target/debug/myapp", &[0x7f, 0x45, 0x4c, 0x46, 0x02, 0x01, 0x01, 0x00])?;
        self.create_file_binary("target/release/myapp", &[0x7f, 0x45, 0x4c, 0x46, 0x02, 0x01, 0x01, 0x00])?;
        
        self.create_file("target/debug/myapp.d", "target/debug/myapp: src/main.rs src/lib.rs")?;
        self.create_file("target/release/myapp.d", "target/release/myapp: src/main.rs src/lib.rs")?;
        
        // Create some dependency artifacts
        self.create_file_binary("target/debug/deps/libserde-123.rlib", &[0x72, 0x6c, 0x69, 0x62])?;
        self.create_file_binary("target/release/deps/libserde-456.rlib", &[0x72, 0x6c, 0x69, 0x62])?;
        
        Ok(())
    }
    
    pub fn create_python_cache(&self) -> Result<()> {
        self.create_directory("__pycache__")?;
        self.create_directory("src/__pycache__")?;
        self.create_directory("tests/__pycache__")?;
        
        // Create some Python cache files
        self.create_file_binary("__pycache__/main.cpython-39.pyc", &[0x61, 0x0d, 0x0d, 0x0a])?;
        self.create_file_binary("src/__pycache__/utils.cpython-39.pyc", &[0x61, 0x0d, 0x0d, 0x0a])?;
        self.create_file_binary("tests/__pycache__/test_main.cpython-39.pyc", &[0x61, 0x0d, 0x0d, 0x0a])?;
        
        Ok(())
    }
    
    pub fn create_git_dir(&self) -> Result<()> {
        self.create_directory(".git")?;
        self.create_directory(".git/objects")?;
        self.create_directory(".git/refs")?;
        self.create_directory(".git/refs/heads")?;
        self.create_directory(".git/refs/tags")?;
        
        self.create_file(".git/HEAD", "ref: refs/heads/main")?;
        self.create_file(".git/config", r#"[core]
    repositoryformatversion = 0
    filemode = true
    bare = false
    logallrefupdates = true
[remote "origin"]
    url = https://github.com/user/repo.git
    fetch = +refs/heads/*:refs/remotes/origin/*
"#)?;
        
        self.create_file(".git/refs/heads/main", "a1b2c3d4e5f6789012345678901234567890abcd")?;
        
        Ok(())
    }
    
    pub fn create_realistic_gitignore(&self) -> Result<()> {
        let content = r#"# Compiled binaries
target/
*.exe
*.dll
*.so
*.dylib

# Node.js
node_modules/
npm-debug.log*
yarn-debug.log*
yarn-error.log*

# Python
__pycache__/
*.py[cod]
*$py.class
*.so
.Python
build/
develop-eggs/
dist/
downloads/
eggs/
.eggs/
lib/
lib64/
parts/
sdist/
var/
wheels/
*.egg-info/
.installed.cfg
*.egg

# IDE
.vscode/
.idea/
*.swp
*.swo
*~

# OS
.DS_Store
.DS_Store?
._*
.Spotlight-V100
.Trashes
ehthumbs.db
Thumbs.db

# Logs
*.log
logs/

# Temporary files
*.tmp
*.temp
*.bak
*.backup

# Environment
.env
.env.local
.env.development.local
.env.test.local
.env.production.local

# Coverage
coverage/
*.lcov
.nyc_output

# Build artifacts
dist/
build/
out/

# Package managers
package-lock.json
yarn.lock
Cargo.lock
"#;
        
        self.create_file(".gitignore", content)?;
        Ok(())
    }
    
    pub fn has_hidden_files(&self) -> bool {
        self.root.join(".gitignore").exists() || 
        self.root.join(".env").exists() ||
        self.root.join(".git").exists()
    }
    
    pub fn file_count(&self) -> usize {
        self.count_files_recursive(&self.root)
    }
    
    fn count_files_recursive(&self, dir: &std::path::Path) -> usize {
        let mut count = 0;
        
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    count += 1;
                } else if path.is_dir() {
                    count += self.count_files_recursive(&path);
                }
            }
        }
        
        count
    }
    
    pub fn total_size(&self) -> u64 {
        self.calculate_size_recursive(&self.root)
    }
    
    fn calculate_size_recursive(&self, dir: &std::path::Path) -> u64 {
        let mut size = 0;
        
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Ok(metadata) = fs::metadata(&path) {
                        size += metadata.len();
                    }
                } else if path.is_dir() {
                    size += self.calculate_size_recursive(&path);
                }
            }
        }
        
        size
    }
    
    /// Create a comprehensive test project with multiple languages and realistic structure
    pub fn create_comprehensive_project(&self) -> Result<()> {
        // Create main source directories
        self.create_directory("src")?;
        self.create_directory("tests")?;
        self.create_directory("docs")?;
        self.create_directory("scripts")?;
        self.create_directory("web")?;
        self.create_directory("config")?;
        
        // Create Rust files
        self.create_rust_file("src/main.rs", 5, 3)?;
        self.create_rust_file("src/lib.rs", 8, 5)?;
        self.create_rust_file("src/utils.rs", 6, 4)?;
        self.create_rust_file("src/config.rs", 4, 2)?;
        self.create_rust_file("tests/integration_test.rs", 10, 6)?;
        
        // Create Python files
        self.create_python_file("scripts/setup.py", 4)?;
        self.create_python_file("scripts/deploy.py", 6)?;
        self.create_python_file("scripts/migrate.py", 3)?;
        self.create_python_file("tests/test_utils.py", 8)?;
        
        // Create JavaScript/TypeScript files
        self.create_javascript_file("web/app.js", 7)?;
        self.create_javascript_file("web/utils.js", 4)?;
        self.create_typescript_file("web/types.ts", 3)?;
        self.create_typescript_file("web/api.ts", 5)?;
        
        // Create HTML/CSS files
        self.create_html_file("web/index.html")?;
        self.create_css_file("web/styles.css")?;
        
        // Create configuration files
        self.create_file("Cargo.toml", r#"[package]
name = "comprehensive-test"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { version = "1.0", features = ["derive"] }
tokio = { version = "1.0", features = ["full"] }
clap = { version = "4.0", features = ["derive"] }

[dev-dependencies]
tempfile = "3.0"
"#)?;
        
        self.create_file("package.json", r#"{
  "name": "comprehensive-test",
  "version": "1.0.0",
  "description": "A comprehensive test project",
  "main": "web/app.js",
  "scripts": {
    "start": "node web/app.js",
    "test": "jest",
    "build": "webpack --mode production",
    "dev": "webpack --mode development --watch"
  },
  "dependencies": {
    "express": "^4.18.2",
    "lodash": "^4.17.21",
    "axios": "^1.4.0"
  },
  "devDependencies": {
    "jest": "^29.5.0",
    "webpack": "^5.88.0",
    "typescript": "^5.1.0"
  }
}"#)?;
        
        self.create_file("requirements.txt", r#"flask==2.3.2
requests==2.31.0
pytest==7.4.0
black==23.3.0
flake8==6.0.0
"#)?;
        
        self.create_file("tsconfig.json", r#"{
  "compilerOptions": {
    "target": "es2020",
    "module": "commonjs",
    "lib": ["es2020"],
    "outDir": "./dist",
    "rootDir": "./web",
    "strict": true,
    "esModuleInterop": true,
    "skipLibCheck": true,
    "forceConsistentCasingInFileNames": true
  },
  "include": ["web/**/*"],
  "exclude": ["node_modules", "dist"]
}"#)?;
        
        // Create documentation
        self.create_file("README.md", r#"# Comprehensive Test Project

This is a comprehensive test project for the howmany code analysis tool.

## Features

- Multi-language support (Rust, Python, JavaScript, TypeScript)
- Realistic project structure
- Build configurations
- Documentation
- Tests

## Languages

### Rust
- Main application code
- Configuration management
- Integration tests

### Python
- Deployment scripts
- Data migration tools
- Unit tests

### JavaScript/TypeScript
- Web frontend
- API client
- Type definitions

## Building

```bash
# Rust
cargo build --release

# JavaScript
npm install
npm run build

# Python
pip install -r requirements.txt
```

## Testing

```bash
# Rust
cargo test

# JavaScript
npm test

# Python
pytest
```
"#)?;
        
        self.create_file("docs/api.md", r#"# API Documentation

## Overview

This document describes the API endpoints and usage.

## Endpoints

### GET /api/stats
Returns code analysis statistics.

**Response:**
```json
{
  "total_files": 42,
  "total_lines": 1337,
  "languages": ["rust", "python", "javascript"]
}
```

### POST /api/analyze
Analyzes a project directory.

**Request:**
```json
{
  "path": "/path/to/project",
  "options": {
    "include_hidden": false,
    "max_depth": 10
  }
}
```

**Response:**
```json
{
  "status": "success",
  "results": {
    "total_files": 42,
    "total_lines": 1337,
    "complexity": 7.5
  }
}
```
"#)?;
        
        self.create_file("docs/development.md", r#"# Development Guide

## Setup

1. Install dependencies
2. Configure environment
3. Run tests

## Code Style

- Rust: Use rustfmt
- Python: Use black
- JavaScript: Use prettier

## Testing

All code should have tests. Aim for >80% coverage.

## Deployment

Use the deployment scripts in the `scripts/` directory.
"#)?;
        
        // Create build artifacts and caches
        self.create_target_dir()?;
        self.create_node_modules()?;
        self.create_python_cache()?;
        self.create_git_dir()?;
        self.create_realistic_gitignore()?;
        
        // Create some additional files for complexity
        self.create_file("config/database.toml", r#"[database]
host = "localhost"
port = 5432
name = "testdb"
user = "testuser"
password = "testpass"

[redis]
host = "localhost"
port = 6379
db = 0
"#)?;
        
        self.create_file("config/app.json", r#"{
  "app_name": "comprehensive-test",
  "version": "1.0.0",
  "debug": true,
  "features": {
    "auth": true,
    "analytics": false,
    "caching": true
  },
  "limits": {
    "max_file_size": 10485760,
    "max_files": 10000,
    "timeout": 30
  }
}"#)?;
        
        self.create_file(".env.example", r#"DATABASE_URL=postgresql://user:pass@localhost/db
REDIS_URL=redis://localhost:6379
SECRET_KEY=your-secret-key-here
DEBUG=true
LOG_LEVEL=info
"#)?;
        
        Ok(())
    }
}

impl Default for TestProject {
    fn default() -> Self {
        Self::new("default_test").expect("Failed to create default test project")
    }
} 