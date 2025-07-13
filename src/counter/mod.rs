use std::collections::HashMap;
use std::fs;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct FileStats {
    pub total_lines: usize,
    pub code_lines: usize,
    pub comment_lines: usize,
    pub blank_lines: usize,
    pub file_size: u64,
    pub doc_lines: usize, // New field for documentation content
}

#[derive(Debug, Clone)]
pub struct CodeStats {
    pub total_files: usize,
    pub total_lines: usize,
    pub total_code_lines: usize,
    pub total_comment_lines: usize,
    pub total_blank_lines: usize,
    pub total_size: u64,
    pub total_doc_lines: usize, // New field for documentation content
    pub stats_by_extension: HashMap<String, (usize, FileStats)>, // (file_count, aggregated_stats)
}

#[derive(Debug, Clone)]
struct CommentPattern {
    single_line: Vec<String>,
    multi_line_start: Vec<String>,
    multi_line_end: Vec<String>,
    doc_patterns: Vec<String>, // JSDoc, rustdoc, etc.
}

pub struct CodeCounter {
    comment_patterns: HashMap<String, CommentPattern>,
}

impl CodeCounter {
    pub fn new() -> Self {
        let mut comment_patterns = HashMap::new();
        
        // Rust patterns
        comment_patterns.insert("rs".to_string(), CommentPattern {
            single_line: vec!["//".to_string()],
            multi_line_start: vec!["/*".to_string()],
            multi_line_end: vec!["*/".to_string()],
            doc_patterns: vec!["///".to_string(), "//!".to_string(), "/**".to_string()],
        });
        
        // JavaScript/TypeScript patterns
        let js_pattern = CommentPattern {
            single_line: vec!["//".to_string()],
            multi_line_start: vec!["/*".to_string()],
            multi_line_end: vec!["*/".to_string()],
            doc_patterns: vec!["/**".to_string(), "//!".to_string()],
        };
        comment_patterns.insert("js".to_string(), js_pattern.clone());
        comment_patterns.insert("ts".to_string(), js_pattern.clone());
        comment_patterns.insert("jsx".to_string(), js_pattern.clone());
        comment_patterns.insert("tsx".to_string(), js_pattern.clone());
        
        // Python patterns
        comment_patterns.insert("py".to_string(), CommentPattern {
            single_line: vec!["#".to_string()],
            multi_line_start: vec!["\"\"\"".to_string(), "'''".to_string()],
            multi_line_end: vec!["\"\"\"".to_string(), "'''".to_string()],
            doc_patterns: vec!["\"\"\"".to_string(), "'''".to_string()],
        });
        
        // Java patterns
        comment_patterns.insert("java".to_string(), CommentPattern {
            single_line: vec!["//".to_string()],
            multi_line_start: vec!["/*".to_string()],
            multi_line_end: vec!["*/".to_string()],
            doc_patterns: vec!["/**".to_string()],
        });
        
        // C/C++ patterns
        let c_pattern = CommentPattern {
            single_line: vec!["//".to_string()],
            multi_line_start: vec!["/*".to_string()],
            multi_line_end: vec!["*/".to_string()],
            doc_patterns: vec!["/**".to_string(), "/*!".to_string()],
        };
        comment_patterns.insert("c".to_string(), c_pattern.clone());
        comment_patterns.insert("cpp".to_string(), c_pattern.clone());
        comment_patterns.insert("cc".to_string(), c_pattern.clone());
        comment_patterns.insert("cxx".to_string(), c_pattern.clone());
        comment_patterns.insert("h".to_string(), c_pattern.clone());
        comment_patterns.insert("hpp".to_string(), c_pattern.clone());
        
        // C# patterns
        comment_patterns.insert("cs".to_string(), CommentPattern {
            single_line: vec!["//".to_string()],
            multi_line_start: vec!["/*".to_string()],
            multi_line_end: vec!["*/".to_string()],
            doc_patterns: vec!["///".to_string(), "/**".to_string()],
        });
        
        // PHP patterns
        comment_patterns.insert("php".to_string(), CommentPattern {
            single_line: vec!["//".to_string(), "#".to_string()],
            multi_line_start: vec!["/*".to_string()],
            multi_line_end: vec!["*/".to_string()],
            doc_patterns: vec!["/**".to_string()],
        });
        
        // Ruby patterns
        comment_patterns.insert("rb".to_string(), CommentPattern {
            single_line: vec!["#".to_string()],
            multi_line_start: vec!["=begin".to_string()],
            multi_line_end: vec!["=end".to_string()],
            doc_patterns: vec!["##".to_string()],
        });
        
        // Go patterns
        comment_patterns.insert("go".to_string(), CommentPattern {
            single_line: vec!["//".to_string()],
            multi_line_start: vec!["/*".to_string()],
            multi_line_end: vec!["*/".to_string()],
            doc_patterns: vec!["//".to_string()], // Go uses // for docs
        });
        
        // Swift patterns
        comment_patterns.insert("swift".to_string(), CommentPattern {
            single_line: vec!["//".to_string()],
            multi_line_start: vec!["/*".to_string()],
            multi_line_end: vec!["*/".to_string()],
            doc_patterns: vec!["///".to_string(), "/**".to_string()],
        });
        
        // Kotlin patterns
        comment_patterns.insert("kt".to_string(), CommentPattern {
            single_line: vec!["//".to_string()],
            multi_line_start: vec!["/*".to_string()],
            multi_line_end: vec!["*/".to_string()],
            doc_patterns: vec!["/**".to_string()],
        });
        
        // Scala patterns
        comment_patterns.insert("scala".to_string(), CommentPattern {
            single_line: vec!["//".to_string()],
            multi_line_start: vec!["/*".to_string()],
            multi_line_end: vec!["*/".to_string()],
            doc_patterns: vec!["/**".to_string()],
        });
        
        // Shell script patterns
        let shell_pattern = CommentPattern {
            single_line: vec!["#".to_string()],
            multi_line_start: vec![],
            multi_line_end: vec![],
            doc_patterns: vec!["##".to_string()],
        };
        comment_patterns.insert("sh".to_string(), shell_pattern.clone());
        comment_patterns.insert("bash".to_string(), shell_pattern.clone());
        comment_patterns.insert("zsh".to_string(), shell_pattern.clone());
        comment_patterns.insert("fish".to_string(), shell_pattern.clone());
        
        // R patterns
        comment_patterns.insert("r".to_string(), CommentPattern {
            single_line: vec!["#".to_string()],
            multi_line_start: vec![],
            multi_line_end: vec![],
            doc_patterns: vec!["#'".to_string()],
        });
        
        // Lua patterns
        comment_patterns.insert("lua".to_string(), CommentPattern {
            single_line: vec!["--".to_string()],
            multi_line_start: vec!["--[[".to_string()],
            multi_line_end: vec!["]]".to_string()],
            doc_patterns: vec!["---".to_string()],
        });
        
        // Haskell patterns
        comment_patterns.insert("hs".to_string(), CommentPattern {
            single_line: vec!["--".to_string()],
            multi_line_start: vec!["{-".to_string()],
            multi_line_end: vec!["-}".to_string()],
            doc_patterns: vec!["-- |".to_string(), "-- ^".to_string()],
        });
        
        // OCaml patterns
        comment_patterns.insert("ml".to_string(), CommentPattern {
            single_line: vec![],
            multi_line_start: vec!["(*".to_string()],
            multi_line_end: vec!["*)".to_string()],
            doc_patterns: vec!["(**".to_string()],
        });
        
        // HTML patterns
        comment_patterns.insert("html".to_string(), CommentPattern {
            single_line: vec![],
            multi_line_start: vec!["<!--".to_string()],
            multi_line_end: vec!["-->".to_string()],
            doc_patterns: vec!["<!--".to_string()],
        });
        
        // CSS patterns
        comment_patterns.insert("css".to_string(), CommentPattern {
            single_line: vec![],
            multi_line_start: vec!["/*".to_string()],
            multi_line_end: vec!["*/".to_string()],
            doc_patterns: vec!["/**".to_string()],
        });
        
        // SCSS patterns
        comment_patterns.insert("scss".to_string(), CommentPattern {
            single_line: vec!["//".to_string()],
            multi_line_start: vec!["/*".to_string()],
            multi_line_end: vec!["*/".to_string()],
            doc_patterns: vec!["/**".to_string(), "///".to_string()],
        });
        
        // Sass patterns
        comment_patterns.insert("sass".to_string(), CommentPattern {
            single_line: vec!["//".to_string()],
            multi_line_start: vec![],
            multi_line_end: vec![],
            doc_patterns: vec!["///".to_string()],
        });
        
        // Markdown patterns (special handling)
        comment_patterns.insert("md".to_string(), CommentPattern {
            single_line: vec![],
            multi_line_start: vec!["<!--".to_string()],
            multi_line_end: vec!["-->".to_string()],
            doc_patterns: vec![], // Markdown content is documentation by nature
        });
        
        Self { comment_patterns }
    }

    pub fn count_file(&self, path: &Path) -> io::Result<FileStats> {
        let file = fs::File::open(path)?;
        let reader = BufReader::new(file);
        
        let extension = path.extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_lowercase();
        
        // Special handling for Markdown files
        if extension == "md" {
            let metadata = fs::metadata(path)?;
            let file_size = metadata.len();
            return self.count_markdown_file(reader, file_size);
        }
        
        let mut total_lines = 0;
        let mut code_lines = 0;
        let mut comment_lines = 0;
        let mut blank_lines = 0;
        let mut doc_lines = 0;
        
        let comment_pattern = self.comment_patterns.get(&extension).cloned().unwrap_or_else(|| {
            CommentPattern {
                single_line: vec![],
                multi_line_start: vec![],
                multi_line_end: vec![],
                doc_patterns: vec![],
            }
        });
        
        let mut in_multi_line_comment = false;
        let mut in_doc_comment = false;
        let mut multi_line_end_pattern = String::new();
        
        for line in reader.lines() {
            let line = line?;
            total_lines += 1;
            
            let trimmed = line.trim();
            
            if trimmed.is_empty() {
                blank_lines += 1;
                continue;
            }
            
            // Check for multi-line comment start/end
            if !in_multi_line_comment {
                for start_pattern in &comment_pattern.multi_line_start {
                    if trimmed.contains(start_pattern) {
                        in_multi_line_comment = true;
                        // Find corresponding end pattern
                        let start_index = comment_pattern.multi_line_start.iter()
                            .position(|p| p == start_pattern)
                            .unwrap_or(0);
                        multi_line_end_pattern = comment_pattern.multi_line_end
                            .get(start_index)
                            .cloned()
                            .unwrap_or_else(|| start_pattern.clone());
                        
                        // Check if it's a documentation comment
                        in_doc_comment = comment_pattern.doc_patterns.iter()
                            .any(|doc_pattern| trimmed.contains(doc_pattern));
                        
                        break;
                    }
                }
            }
            
            if in_multi_line_comment {
                if trimmed.contains(&multi_line_end_pattern) {
                    in_multi_line_comment = false;
                    in_doc_comment = false;
                }
                
                if in_doc_comment {
                    doc_lines += 1;
                } else {
                    comment_lines += 1;
                }
            } else if self.is_single_line_comment(trimmed, &comment_pattern) {
                // Check if it's a documentation comment
                if self.is_doc_comment(trimmed, &comment_pattern) {
                    doc_lines += 1;
                } else {
                    comment_lines += 1;
                }
            } else {
                code_lines += 1;
            }
        }
        
        let metadata = fs::metadata(path)?;
        let file_size = metadata.len();
        
        Ok(FileStats {
            total_lines,
            code_lines,
            comment_lines,
            blank_lines,
            file_size,
            doc_lines,
        })
    }
    
    fn count_markdown_file(&self, reader: BufReader<fs::File>, file_size: u64) -> io::Result<FileStats> {
        let mut total_lines = 0;
        let mut code_lines = 0; // Code blocks
        let mut comment_lines = 0; // HTML comments
        let mut blank_lines = 0;
        let mut doc_lines = 0; // Markdown content
        
        let mut in_code_block = false;
        let mut in_html_comment = false;
        
        for line in reader.lines() {
            let line = line?;
            total_lines += 1;
            
            let trimmed = line.trim();
            
            if trimmed.is_empty() {
                blank_lines += 1;
                continue;
            }
            
            // Check for HTML comments
            if trimmed.starts_with("<!--") {
                in_html_comment = true;
            }
            
            if in_html_comment {
                comment_lines += 1;
                if trimmed.ends_with("-->") {
                    in_html_comment = false;
                }
                continue;
            }
            
            // Check for code blocks (fenced with ``` or indented)
            if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
                in_code_block = !in_code_block;
                code_lines += 1;
                continue;
            }
            
            if in_code_block || trimmed.starts_with("    ") || trimmed.starts_with("\t") {
                code_lines += 1;
            } else {
                // Regular markdown content is documentation
                doc_lines += 1;
            }
        }
        
        // File size is passed as parameter from metadata
        
        Ok(FileStats {
            total_lines,
            code_lines,
            comment_lines,
            blank_lines,
            file_size,
            doc_lines,
        })
    }
    
    fn is_single_line_comment(&self, line: &str, pattern: &CommentPattern) -> bool {
        for prefix in &pattern.single_line {
            if line.starts_with(prefix) {
                return true;
            }
        }
        false
    }
    
    fn is_doc_comment(&self, line: &str, pattern: &CommentPattern) -> bool {
        for doc_pattern in &pattern.doc_patterns {
            if line.starts_with(doc_pattern) {
                return true;
            }
        }
        false
    }
    
    pub fn aggregate_stats(&self, file_stats: Vec<(String, FileStats)>) -> CodeStats {
        let mut total_files = 0;
        let mut total_lines = 0;
        let mut total_code_lines = 0;
        let mut total_comment_lines = 0;
        let mut total_blank_lines = 0;
        let mut total_size = 0;
        let mut total_doc_lines = 0;
        let mut stats_by_extension: HashMap<String, (usize, FileStats)> = HashMap::new();
        
        for (extension, stats) in file_stats {
            total_files += 1;
            total_lines += stats.total_lines;
            total_code_lines += stats.code_lines;
            total_comment_lines += stats.comment_lines;
            total_blank_lines += stats.blank_lines;
            total_size += stats.file_size;
            total_doc_lines += stats.doc_lines;
            
            let entry = stats_by_extension.entry(extension).or_insert((0, FileStats {
                total_lines: 0,
                code_lines: 0,
                comment_lines: 0,
                blank_lines: 0,
                file_size: 0,
                doc_lines: 0,
            }));
            
            entry.0 += 1; // file count
            entry.1.total_lines += stats.total_lines;
            entry.1.code_lines += stats.code_lines;
            entry.1.comment_lines += stats.comment_lines;
            entry.1.blank_lines += stats.blank_lines;
            entry.1.file_size += stats.file_size;
            entry.1.doc_lines += stats.doc_lines;
        }
        
        CodeStats {
            total_files,
            total_lines,
            total_code_lines,
            total_comment_lines,
            total_blank_lines,
            total_size,
            total_doc_lines,
            stats_by_extension,
        }
    }
} 