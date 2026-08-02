use crate::core::counter::CodeCounter;
use crate::core::types::{CodeStats, FileStats};
use crate::testing::test_utils::TestProject;
use std::collections::BTreeMap;

#[cfg(test)]
mod counter {
    use super::*;

    /// A fresh counter must already know every language it claims to support.
    ///
    /// This used to be `assert!(true)`, so a counter constructed with an empty
    /// pattern table would have passed it.
    #[test]
    fn a_new_counter_knows_the_languages_it_claims() {
        let counter = CodeCounter::new();
        for extension in ["rs", "py", "ts", "go", "java", "c", "rb", "sql", "md"] {
            assert!(
                counter.supports_extension(extension),
                "a new counter did not recognize .{extension}"
            );
            assert!(
                counter.comment_pattern(extension).is_some(),
                "no comment syntax registered for .{extension}"
            );
        }
        assert!(
            counter.comment_pattern("definitelynotalanguage").is_none(),
            "an unknown extension should not resolve to a pattern"
        );
    }

    #[test]
    fn test_rust_file_counting_basic() {
        let project = TestProject::new("test_project").unwrap();
        let content = r#"
// File header comment
use std::collections::BTreeMap;

/// Documentation comment
pub fn main() {
    // Inline comment
    println!("Hello, world!");
}

/* Multi-line comment
   spanning multiple lines */
fn helper() {
    /* Another comment */
}
"#;
        let file_path = project.create_file("test.rs", content).unwrap();

        let counter = CodeCounter::new();
        let stats = counter.count_file(&file_path).unwrap();

        assert!(stats.total_lines > 0);
        assert!(stats.code_lines > 0);
        assert!(stats.comment_lines > 0);
        assert!(stats.doc_lines > 0);
        assert!(stats.blank_lines > 0);
        assert!(stats.file_size > 0);

        // Lines 1, 4 and 10 are blank; 2, 7, 11, 12 and 14 are comments; 5 is
        // the only doc comment; the remaining six are code.
        assert_eq!(stats.total_lines, 15);
        assert_eq!(stats.code_lines, 6);
        assert_eq!(stats.comment_lines, 5);
        assert_eq!(stats.doc_lines, 1);
        assert_eq!(stats.blank_lines, 3);
    }

    #[test]
    fn test_rust_complex_comments() {
        let project = TestProject::new("test_project").unwrap();
        let content = r#"
//! Crate-level documentation
//! Multiple lines of crate docs

/// Struct documentation
/// Multiple lines
pub struct TestStruct {
    /// Field documentation
    pub field: i32,
}

impl TestStruct {
    /// Method documentation
    /// With multiple lines
    pub fn method(&self) -> i32 {
        // Regular comment
        /* Block comment */
        self.field
    }
}

/**
 * Multi-line documentation comment
 * in JavaDoc style
 */
pub fn documented_function() {
    /*
     * Multi-line regular comment
     * Not documentation
     */
}
"#;
        let file_path = project.create_file("complex.rs", content).unwrap();

        let counter = CodeCounter::new();
        let stats = counter.count_file(&file_path).unwrap();

        // Should distinguish between doc comments and regular comments
        assert!(stats.doc_lines > 0);
        assert!(stats.comment_lines > 0);
        assert!(stats.doc_lines > stats.comment_lines);
    }

    #[test]
    fn test_python_file_counting() {
        let project = TestProject::new("test_project").unwrap();
        let content = r#"#!/usr/bin/env python3
"""
Module docstring
Multiple lines
"""

import os
import sys

def main():
    """Function docstring"""
    # Regular comment
    print("Hello, Python!")
    
    '''
    Multi-line string that could be mistaken for docstring
    but it's not at the beginning of function
    '''
    
def helper():
    '''Another docstring'''
    pass

# End of file comment
"#;
        let file_path = project.create_file("test.py", content).unwrap();

        let counter = CodeCounter::new();
        let stats = counter.count_file(&file_path).unwrap();

        assert!(stats.total_lines > 0);
        assert!(stats.code_lines > 0);
        assert!(stats.comment_lines > 0);
        assert!(stats.doc_lines > 0);
        assert!(stats.blank_lines > 0);

        // Python should have high doc_lines due to docstrings
        assert!(stats.doc_lines >= 3);
    }

    #[test]
    fn test_javascript_file_counting() {
        let project = TestProject::new("test_project").unwrap();
        let content = r#"/**
 * Module description
 * @author Test Author
 */

const express = require('express');

/**
 * Function description
 * @param {string} name - The name parameter
 * @returns {string} Greeting message
 */
function greet(name) {
    // Regular comment
    return `Hello, ${name}!`;
}

/* 
 * Multi-line comment
 * Not documentation
 */
function helper() {
    /* Another comment */
    console.log('Helper function');
}

// End comment
"#;
        let file_path = project.create_file("test.js", content).unwrap();

        let counter = CodeCounter::new();
        let stats = counter.count_file(&file_path).unwrap();

        assert!(stats.total_lines > 0);
        assert!(stats.code_lines > 0);
        assert!(stats.comment_lines > 0);
        assert!(stats.doc_lines > 0);

        // JSDoc comments should be counted as documentation
        assert!(stats.doc_lines >= 2);
    }

    #[test]
    fn test_java_file_counting() {
        let project = TestProject::new("test_project").unwrap();
        let content = r#"package com.example;

/**
 * Class documentation
 * @author Test Author
 * @version 1.0
 */
public class TestClass {
    
    /**
     * Method documentation
     * @param name The name parameter
     * @return Greeting message
     */
    public String greet(String name) {
        // Regular comment
        return "Hello, " + name + "!";
    }
    
    /*
     * Multi-line comment
     * Not documentation
     */
    private void helper() {
        /* Another comment */
        System.out.println("Helper");
    }
}
"#;
        let file_path = project.create_file("TestClass.java", content).unwrap();

        let counter = CodeCounter::new();
        let stats = counter.count_file(&file_path).unwrap();

        assert!(stats.total_lines > 0);
        assert!(stats.code_lines > 0);
        assert!(stats.comment_lines > 0);
        assert!(stats.doc_lines > 0);

        // JavaDoc comments should be counted as documentation
        assert!(stats.doc_lines >= 2);
    }

    #[test]
    fn test_cpp_file_counting() {
        let project = TestProject::new("test_project").unwrap();
        let content = r#"#include <iostream>
#include <string>

/**
 * @brief Class description
 * @details Detailed description
 */
class TestClass {
private:
    int value;
    
public:
    /**
     * @brief Constructor
     * @param val Initial value
     */
    TestClass(int val) : value(val) {
        // Constructor implementation
    }
    
    /*!
     * @brief Get value
     * @return The current value
     */
    int getValue() const {
        /* Return the value */
        return value;
    }
};

// End of file
"#;
        let file_path = project.create_file("test.cpp", content).unwrap();

        let counter = CodeCounter::new();
        let stats = counter.count_file(&file_path).unwrap();

        assert!(stats.total_lines > 0);
        assert!(stats.code_lines > 0);
        assert!(stats.comment_lines > 0);
        assert!(stats.doc_lines > 0);

        // Doxygen comments should be counted as documentation
        assert!(stats.doc_lines >= 3);
    }

    #[test]
    fn test_markdown_file_counting() {
        let project = TestProject::new("test_project").unwrap();
        let content = r#"# Project Title

This is a markdown file with various content types.

## Code Examples

Here's some Rust code:

```rust
fn main() {
    println!("Hello, world!");
}
```

And some Python:

```python
def hello():
    print("Hello, Python!")
```

## Regular Content

This is regular markdown content that should be counted as documentation.

<!-- HTML comment -->

More content here.

    Indented code block
    Another line of code

Final paragraph.
"#;
        let file_path = project.create_file("README.md", content).unwrap();

        let counter = CodeCounter::new();
        let stats = counter.count_file(&file_path).unwrap();

        // Nine prose lines, one HTML comment, and eleven lines of code: four
        // fence markers, five lines inside the two fenced blocks and two
        // indented ones.
        assert_eq!(stats.total_lines, 33);
        assert_eq!(stats.doc_lines, 9);
        assert_eq!(stats.comment_lines, 1);
        assert_eq!(stats.code_lines, 11);
        assert_eq!(stats.blank_lines, 12);
        assert_eq!(
            stats.code_lines + stats.comment_lines + stats.doc_lines + stats.blank_lines,
            stats.total_lines
        );
    }

    #[test]
    fn test_html_file_counting() {
        let project = TestProject::new("test_project").unwrap();
        let content = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>Test Page</title>
</head>
<body>
    <!-- Main content -->
    <div class="container">
        <h1>Welcome</h1>
        <p>This is a test page.</p>
    </div>
    
    <!-- 
    Multi-line HTML comment
    spanning multiple lines
    -->
    
    <script>
        // JavaScript comment
        console.log("Hello");
    </script>
</body>
</html>
"#;
        let file_path = project.create_file("test.html", content).unwrap();

        let counter = CodeCounter::new();
        let stats = counter.count_file(&file_path).unwrap();

        assert!(stats.total_lines > 0);
        assert!(stats.code_lines > 0);
        assert!(stats.comment_lines > 0);
        assert!(stats.blank_lines > 0);
    }

    #[test]
    fn test_css_file_counting() {
        let project = TestProject::new("test_project").unwrap();
        let content = r#"/* Main stylesheet */

body {
    font-family: Arial, sans-serif;
    margin: 0;
    padding: 0;
}

/**
 * Container styles
 * Multi-line documentation comment
 */
.container {
    max-width: 1200px;
    margin: 0 auto;
    /* Center the container */
}

/* Responsive design */
@media (max-width: 768px) {
    .container {
        padding: 10px;
    }
}
"#;
        let file_path = project.create_file("styles.css", content).unwrap();

        let counter = CodeCounter::new();
        let stats = counter.count_file(&file_path).unwrap();

        assert!(stats.total_lines > 0);
        assert!(stats.code_lines > 0);
        assert!(stats.comment_lines > 0);
        assert!(stats.blank_lines > 0);
    }

    #[test]
    fn test_json_file_counting() {
        let project = TestProject::new("test_project").unwrap();
        let content = r#"{
  "name": "test-project",
  "version": "1.0.0",
  "description": "A test project",
  "main": "index.js",
  "scripts": {
    "test": "jest",
    "build": "webpack"
  },
  "dependencies": {
    "express": "^4.18.0",
    "lodash": "^4.17.21"
  },
  "devDependencies": {
    "jest": "^28.0.0",
    "webpack": "^5.0.0"
  }
}
"#;
        let file_path = project.create_file("package.json", content).unwrap();

        let counter = CodeCounter::new();
        let stats = counter.count_file(&file_path).unwrap();

        assert!(stats.total_lines > 0);
        assert!(stats.code_lines > 0);
        assert_eq!(stats.comment_lines, 0); // JSON doesn't support comments
        assert_eq!(stats.doc_lines, 0);
        // Every line of this fixture carries content, so the categories have to
        // account for all of them with nothing left over.
        assert_eq!(stats.blank_lines, 0);
        assert_eq!(stats.code_lines, stats.total_lines);
    }

    #[test]
    fn test_empty_file() {
        let project = TestProject::new("test_project").unwrap();
        let file_path = project.create_file("empty.rs", "").unwrap();

        let counter = CodeCounter::new();
        let stats = counter.count_file(&file_path).unwrap();

        assert_eq!(stats.total_lines, 0);
        assert_eq!(stats.code_lines, 0);
        assert_eq!(stats.comment_lines, 0);
        assert_eq!(stats.doc_lines, 0);
        assert_eq!(stats.blank_lines, 0);
        assert_eq!(stats.file_size, 0, "an empty file has no bytes");
    }

    #[test]
    fn test_only_blank_lines() {
        let project = TestProject::new("test_project").unwrap();
        let content = "\n\n\n\n\n";
        let file_path = project.create_file("blank.rs", content).unwrap();

        let counter = CodeCounter::new();
        let stats = counter.count_file(&file_path).unwrap();

        assert_eq!(stats.total_lines, 5);
        assert_eq!(stats.code_lines, 0);
        assert_eq!(stats.comment_lines, 0);
        assert_eq!(stats.doc_lines, 0);
        assert_eq!(stats.blank_lines, 5);
    }

    #[test]
    fn test_only_comments() {
        let project = TestProject::new("test_project").unwrap();
        let content = r#"// Comment 1
// Comment 2
/* Comment 3 */
/// Doc comment
//! Another doc comment
"#;
        let file_path = project.create_file("comments.rs", content).unwrap();

        let counter = CodeCounter::new();
        let stats = counter.count_file(&file_path).unwrap();

        assert_eq!(stats.total_lines, 5);
        assert_eq!(stats.code_lines, 0);
        assert!(stats.comment_lines > 0);
        assert!(stats.doc_lines > 0);
        assert_eq!(stats.blank_lines, 0);
    }

    #[test]
    fn test_mixed_content() {
        let project = TestProject::new("test_project").unwrap();
        let content = r#"// Header comment
use std::io;

/// Main function documentation
fn main() {
    // Print message
    println!("Hello");
    
    /* Multi-line comment
       with details */
    
    let x = 42; // Inline comment
}

// End comment
"#;
        let file_path = project.create_file("mixed.rs", content).unwrap();

        let counter = CodeCounter::new();
        let stats = counter.count_file(&file_path).unwrap();

        assert_eq!(stats.total_lines, 15);
        assert_eq!(stats.code_lines, 5);
        assert_eq!(stats.comment_lines, 5);
        assert_eq!(stats.doc_lines, 1);
        assert_eq!(stats.blank_lines, 4, "whitespace-only lines are blank");

        // Verify the sum
        assert_eq!(
            stats.total_lines,
            stats.code_lines + stats.comment_lines + stats.doc_lines + stats.blank_lines
        );
    }

    #[test]
    fn test_aggregation_single_language() {
        let counter = CodeCounter::new();

        let file_stats = vec![
            (
                "rs".to_string(),
                FileStats {
                    total_lines: 100,
                    code_lines: 70,
                    comment_lines: 20,
                    blank_lines: 10,
                    file_size: 1000,
                    doc_lines: 15,
                },
            ),
            (
                "rs".to_string(),
                FileStats {
                    total_lines: 50,
                    code_lines: 35,
                    comment_lines: 10,
                    blank_lines: 5,
                    file_size: 500,
                    doc_lines: 8,
                },
            ),
        ];

        let aggregated = counter.aggregate_stats(file_stats);

        assert_eq!(aggregated.total_files, 2);
        assert_eq!(aggregated.total_lines, 150);
        assert_eq!(aggregated.total_code_lines, 105);
        assert_eq!(aggregated.total_comment_lines, 30);
        assert_eq!(aggregated.total_blank_lines, 15);
        assert_eq!(aggregated.total_size, 1500);
        assert_eq!(aggregated.total_doc_lines, 23);

        // Check per-extension stats
        assert_eq!(aggregated.stats_by_extension.len(), 1);
        let rust_stats = &aggregated.stats_by_extension["rs"];
        assert_eq!(rust_stats.0, 2); // 2 files
        assert_eq!(rust_stats.1.total_lines, 150);
        assert_eq!(rust_stats.1.code_lines, 105);
    }

    #[test]
    fn test_aggregation_multiple_languages() {
        let counter = CodeCounter::new();

        let file_stats = vec![
            (
                "rs".to_string(),
                FileStats {
                    total_lines: 100,
                    code_lines: 70,
                    comment_lines: 20,
                    blank_lines: 10,
                    file_size: 1000,
                    doc_lines: 15,
                },
            ),
            (
                "py".to_string(),
                FileStats {
                    total_lines: 80,
                    code_lines: 60,
                    comment_lines: 15,
                    blank_lines: 5,
                    file_size: 800,
                    doc_lines: 12,
                },
            ),
            (
                "js".to_string(),
                FileStats {
                    total_lines: 60,
                    code_lines: 45,
                    comment_lines: 10,
                    blank_lines: 5,
                    file_size: 600,
                    doc_lines: 8,
                },
            ),
        ];

        let aggregated = counter.aggregate_stats(file_stats);

        assert_eq!(aggregated.total_files, 3);
        assert_eq!(aggregated.total_lines, 240);
        assert_eq!(aggregated.total_code_lines, 175);
        assert_eq!(aggregated.total_comment_lines, 45);
        assert_eq!(aggregated.total_blank_lines, 20);
        assert_eq!(aggregated.total_size, 2400);
        assert_eq!(aggregated.total_doc_lines, 35);

        // Check per-extension stats
        assert_eq!(aggregated.stats_by_extension.len(), 3);

        let rust_stats = &aggregated.stats_by_extension["rs"];
        assert_eq!(rust_stats.0, 1);
        assert_eq!(rust_stats.1.total_lines, 100);

        let python_stats = &aggregated.stats_by_extension["py"];
        assert_eq!(python_stats.0, 1);
        assert_eq!(python_stats.1.total_lines, 80);

        let js_stats = &aggregated.stats_by_extension["js"];
        assert_eq!(js_stats.0, 1);
        assert_eq!(js_stats.1.total_lines, 60);
    }

    #[test]
    fn test_comprehensive_file_stats() {
        let project = TestProject::new("test_project").unwrap();
        let file_path = project.create_rust_file("comprehensive.rs", 10, 5).unwrap();

        let counter = CodeCounter::new();
        let aggregated_stats = counter.calculate_file_stats(&file_path).unwrap();

        // Check that all stat types are calculated
        assert!(aggregated_stats.basic.total_lines > 0);
        assert_eq!(
            aggregated_stats.basic.code_lines
                + aggregated_stats.basic.comment_lines
                + aggregated_stats.basic.doc_lines
                + aggregated_stats.basic.blank_lines,
            aggregated_stats.basic.total_lines,
            "line categories must partition total_lines"
        );
        assert!(aggregated_stats.ratios.code_ratio > 0.0);
        assert!(aggregated_stats.ratios.code_ratio <= 1.0);

        // Check metadata
        assert!(!aggregated_stats.metadata.version.is_empty());
        assert!(!aggregated_stats.metadata.timestamp.is_empty());
        assert_eq!(aggregated_stats.metadata.file_count_analyzed, 1);
        assert!(aggregated_stats.metadata.total_bytes_analyzed > 0);
    }

    #[test]
    fn test_comprehensive_project_stats() {
        let counter = CodeCounter::new();

        let mut stats_by_extension = BTreeMap::new();
        stats_by_extension.insert(
            "rs".to_string(),
            (
                2,
                FileStats {
                    total_lines: 100,
                    code_lines: 70,
                    comment_lines: 20,
                    doc_lines: 5,
                    blank_lines: 10,
                    file_size: 2000,
                },
            ),
        );
        stats_by_extension.insert(
            "py".to_string(),
            (
                1,
                FileStats {
                    total_lines: 50,
                    code_lines: 35,
                    comment_lines: 10,
                    doc_lines: 2,
                    blank_lines: 5,
                    file_size: 1000,
                },
            ),
        );

        let code_stats = CodeStats {
            total_files: 3,
            total_lines: 150,
            total_code_lines: 105,
            total_comment_lines: 30,
            total_doc_lines: 7,
            total_blank_lines: 15,
            total_size: 3000,
            stats_by_extension,
        };

        let individual_files = vec![
            (
                "main.rs".to_string(),
                FileStats {
                    total_lines: 50,
                    code_lines: 35,
                    comment_lines: 10,
                    doc_lines: 2,
                    blank_lines: 5,
                    file_size: 1000,
                },
            ),
            (
                "lib.rs".to_string(),
                FileStats {
                    total_lines: 50,
                    code_lines: 35,
                    comment_lines: 10,
                    doc_lines: 3,
                    blank_lines: 5,
                    file_size: 1000,
                },
            ),
            (
                "script.py".to_string(),
                FileStats {
                    total_lines: 50,
                    code_lines: 35,
                    comment_lines: 10,
                    doc_lines: 2,
                    blank_lines: 5,
                    file_size: 1000,
                },
            ),
        ];

        let aggregated_stats = counter
            .calculate_project_stats(&code_stats, &individual_files)
            .unwrap();

        // Check comprehensive stats
        assert_eq!(aggregated_stats.basic.total_files, 3);
        assert_eq!(aggregated_stats.basic.total_lines, 150);
        assert_eq!(aggregated_stats.basic.code_lines, 105);
        assert!(aggregated_stats.ratios.code_ratio > 0.0);
        assert!(aggregated_stats.ratios.code_ratio <= 1.0);
        assert!(aggregated_stats.metadata.calculation_time_ms < 60_000);

        // Check metadata
        assert_eq!(aggregated_stats.metadata.file_count_analyzed, 3);
        assert!(aggregated_stats.metadata.languages_detected.len() >= 2);
        assert!(aggregated_stats.metadata.total_bytes_analyzed > 0);
    }

    #[test]
    fn test_very_long_lines() {
        let project = TestProject::new("test_project").unwrap();

        // Create a file with very long lines to test memory efficiency
        let long_line = "// ".to_string() + &"x".repeat(10000);
        let content = format!("{}\nfn main() {{}}\n{}", long_line, long_line);
        let file_path = project.create_file("long.rs", &content).unwrap();

        let counter = CodeCounter::new();
        let stats = counter.count_file(&file_path).unwrap();

        assert_eq!(stats.total_lines, 3);
        assert_eq!(stats.comment_lines, 2);
        assert_eq!(stats.code_lines, 1);
        assert_eq!(stats.blank_lines, 0);
    }

    #[test]
    fn test_nested_comments() {
        let project = TestProject::new("test_project").unwrap();
        let content = r#"
/* Outer comment
   /* This might be nested but Rust doesn't support nested comments */
   Still outer comment */
fn main() {
    // Regular comment
    println!("Hello");
}
"#;
        let file_path = project.create_file("nested.rs", content).unwrap();

        let counter = CodeCounter::new();
        let stats = counter.count_file(&file_path).unwrap();

        assert!(stats.comment_lines >= 3);
        assert!(stats.code_lines >= 2);
    }

    #[test]
    fn test_string_literals_with_comment_markers() {
        let project = TestProject::new("test_project").unwrap();
        let content = r#"
fn main() {
    let url = "https://example.com"; // Not a comment marker in string
    let comment = "// This is not a comment";
    // This IS a comment
    println!("/* Not a comment */");
    /* This IS a comment */
    let regex = r"//.*"; // Regex pattern, not comment
}
"#;
        let file_path = project.create_file("strings.rs", content).unwrap();

        let counter = CodeCounter::new();
        let stats = counter.count_file(&file_path).unwrap();

        // Should correctly identify comments vs code
        assert!(stats.comment_lines >= 2);
        assert!(stats.code_lines >= 4);
    }

    #[test]
    fn test_error_handling() {
        let counter = CodeCounter::new();

        // Test with non-existent file
        let non_existent = std::path::Path::new("/non/existent/file.rs");
        let result = counter.count_file(non_existent);
        assert!(result.is_err());

        // Test with directory instead of file
        let temp_dir = tempfile::tempdir().unwrap();
        let result = counter.count_file(temp_dir.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_binary_file_handling() {
        let project = TestProject::new("test_project").unwrap();

        // Create a binary-like file
        let binary_content = vec![0u8, 1, 2, 3, 255, 254, 253];
        let file_path = project.root.join("binary.bin");
        std::fs::write(&file_path, binary_content).unwrap();

        let counter = CodeCounter::new();
        let result = counter.count_file(&file_path);

        // Should handle binary files gracefully
        match result {
            Ok(stats) => {
                // If it succeeds, it should have minimal stats
                assert!(stats.total_lines <= 1);
            }
            Err(_) => {
                // It's OK if it errors on binary files
            }
        }
    }

    #[test]
    fn test_performance_with_large_file() {
        let project = TestProject::new("test_project").unwrap();

        // Create a large file
        let mut large_content = String::new();
        for i in 0..1000 {
            large_content.push_str(&format!("// Comment line {}\n", i));
            large_content.push_str(&format!("fn function_{}() {{\n", i));
            large_content.push_str("    println!(\"Hello\");\n");
            large_content.push_str("}\n\n");
        }

        let file_path = project.create_file("large.rs", &large_content).unwrap();

        let counter = CodeCounter::new();
        let start = std::time::Instant::now();
        let stats = counter.count_file(&file_path).unwrap();
        let duration = start.elapsed();

        // Should complete in reasonable time (less than 1 second)
        assert!(duration.as_secs() < 1);

        // Should have correct counts
        assert_eq!(stats.comment_lines, 1000);
        assert!(stats.code_lines >= 2000);
        assert!(stats.total_lines >= 4000);
    }

    #[test]
    fn test_all_supported_languages() {
        let project = TestProject::new("test_project").unwrap();

        let test_files = vec![
            ("test.rs", "fn main() {}", "rs"),
            ("test.py", "def main():", "py"),
            ("test.js", "function main() {}", "js"),
            ("test.ts", "function main(): void {}", "ts"),
            ("test.java", "public class Test {}", "java"),
            ("test.cpp", "int main() {}", "cpp"),
            ("test.c", "int main() {}", "c"),
            ("test.go", "func main() {}", "go"),
            ("test.rb", "def main", "rb"),
            ("test.php", "<?php function main() {}", "php"),
            ("test.cs", "public class Test {}", "cs"),
            ("test.swift", "func main() {}", "swift"),
            ("test.kt", "fun main() {}", "kt"),
            ("test.scala", "object Main {}", "scala"),
            ("test.md", "# Header", "md"),
            ("test.html", "<html></html>", "html"),
            ("test.css", "body { color: red; }", "css"),
            ("test.json", "{\"key\": \"value\"}", "json"),
            ("test.xml", "<root></root>", "xml"),
            ("test.yaml", "key: value", "yaml"),
            ("test.yml", "key: value", "yml"),
            ("test.toml", "key = \"value\"", "toml"),
        ];

        let counter = CodeCounter::new();

        for (filename, content, _expected_ext) in test_files {
            let file_path = project.create_file(filename, content).unwrap();
            let stats = counter.count_file(&file_path).unwrap();

            // All files should have at least some content
            assert!(
                stats.total_lines > 0,
                "File {} should have content",
                filename
            );
            assert!(stats.file_size > 0, "File {} should have size", filename);
        }
    }

    /// The borrowed calculator is the counter's own, not a fresh one per call.
    ///
    /// Handing back a new calculator each time would silently discard any state
    /// a caller configured on it.
    #[test]
    fn the_borrowed_stats_calculator_is_the_counters_own() {
        let counter = CodeCounter::new();
        assert!(std::ptr::eq(
            counter.stats_calculator(),
            counter.stats_calculator()
        ));
    }

    #[test]
    fn test_edge_cases() {
        let project = TestProject::new("test_project").unwrap();

        // Test file with only whitespace
        let whitespace_content = "   \t  \n  \t\t  \n\t\t\t\n";
        let file_path = project
            .create_file("whitespace.rs", whitespace_content)
            .unwrap();

        let counter = CodeCounter::new();
        let stats = counter.count_file(&file_path).unwrap();

        assert_eq!(stats.total_lines, 3);
        assert_eq!(stats.blank_lines, 3);
        assert_eq!(stats.code_lines, 0);
        assert_eq!(stats.comment_lines, 0);
    }

    #[test]
    fn test_unicode_content() {
        let project = TestProject::new("test_project").unwrap();
        let content = r#"
// Unicode comment: 你好世界
fn main() {
    println!("Hello, 世界! 🌍");
    // More unicode: αβγδε
    let emoji = "🚀🎉✨";
}
"#;
        let file_path = project.create_file("unicode.rs", content).unwrap();

        let counter = CodeCounter::new();
        let stats = counter.count_file(&file_path).unwrap();

        assert!(stats.total_lines > 0);
        assert!(stats.code_lines > 0);
        assert!(stats.comment_lines > 0);
        assert!(stats.file_size > 0);
    }

    #[test]
    fn test_concurrent_counting() {
        use std::sync::Arc;
        use std::thread;

        let project = TestProject::new("test_project").unwrap();
        let counter = Arc::new(CodeCounter::new());

        // Create multiple files
        let files = (0..10)
            .map(|i| {
                let content =
                    format!("// File {i}\nfn main_{i}() {{\n    println!(\"Hello {i}\");\n}}\n");
                project
                    .create_file(&format!("file_{}.rs", i), &content)
                    .unwrap()
            })
            .collect::<Vec<_>>();

        let handles: Vec<_> = files
            .into_iter()
            .map(|file_path| {
                let counter = Arc::clone(&counter);
                thread::spawn(move || counter.count_file(&file_path).unwrap())
            })
            .collect();

        let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();

        // All files should have been counted successfully
        assert_eq!(results.len(), 10);
        for stats in results {
            assert!(stats.total_lines > 0);
            assert!(stats.code_lines > 0);
        }
    }
}
