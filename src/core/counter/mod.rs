use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;
use crate::utils::errors::Result;
use crate::core::types::{CodeStats, FileStats};
use crate::core::stats::{StatsCalculator, AggregatedStats};

#[derive(Debug, Clone)]
struct CommentPattern {
    single_line: Vec<String>,
    multi_line_start: Vec<String>,
    multi_line_end: Vec<String>,
    doc_patterns: Vec<String>, // JSDoc, rustdoc, etc.
}

pub struct CodeCounter {
    comment_patterns: HashMap<String, CommentPattern>,
    stats_calculator: StatsCalculator,
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
        
        // PowerShell patterns
        comment_patterns.insert("ps1".to_string(), CommentPattern {
            single_line: vec!["#".to_string()],
            multi_line_start: vec!["<#".to_string()],
            multi_line_end: vec!["#>".to_string()],
            doc_patterns: vec!["<#".to_string()],
        });
        
        // Elm patterns
        comment_patterns.insert("elm".to_string(), CommentPattern {
            single_line: vec!["--".to_string()],
            multi_line_start: vec!["{-".to_string()],
            multi_line_end: vec!["-}".to_string()],
            doc_patterns: vec!["{-|".to_string()],
        });
        
        // Erlang patterns
        comment_patterns.insert("erl".to_string(), CommentPattern {
            single_line: vec!["%".to_string()],
            multi_line_start: vec![],
            multi_line_end: vec![],
            doc_patterns: vec!["%%".to_string()],
        });
        
        // Elixir patterns
        comment_patterns.insert("ex".to_string(), CommentPattern {
            single_line: vec!["#".to_string()],
            multi_line_start: vec![],
            multi_line_end: vec![],
            doc_patterns: vec!["@doc".to_string(), "@moduledoc".to_string()],
        });
        comment_patterns.insert("exs".to_string(), CommentPattern {
            single_line: vec!["#".to_string()],
            multi_line_start: vec![],
            multi_line_end: vec![],
            doc_patterns: vec!["@doc".to_string(), "@moduledoc".to_string()],
        });
        
        // Julia patterns
        comment_patterns.insert("jl".to_string(), CommentPattern {
            single_line: vec!["#".to_string()],
            multi_line_start: vec!["#=".to_string()],
            multi_line_end: vec!["=#".to_string()],
            doc_patterns: vec!["\"\"\"".to_string()],
        });
        
        // MATLAB patterns
        comment_patterns.insert("m".to_string(), CommentPattern {
            single_line: vec!["%".to_string()],
            multi_line_start: vec!["%{".to_string()],
            multi_line_end: vec!["%}".to_string()],
            doc_patterns: vec!["%%".to_string()],
        });
        
        // SQL patterns
        comment_patterns.insert("sql".to_string(), CommentPattern {
            single_line: vec!["--".to_string()],
            multi_line_start: vec!["/*".to_string()],
            multi_line_end: vec!["*/".to_string()],
            doc_patterns: vec!["--".to_string()],
        });
        
        // Objective-C patterns
        comment_patterns.insert("mm".to_string(), CommentPattern {
            single_line: vec!["//".to_string()],
            multi_line_start: vec!["/*".to_string()],
            multi_line_end: vec!["*/".to_string()],
            doc_patterns: vec!["/**".to_string()],
        });
        
        // Dart patterns
        comment_patterns.insert("dart".to_string(), CommentPattern {
            single_line: vec!["//".to_string()],
            multi_line_start: vec!["/*".to_string()],
            multi_line_end: vec!["*/".to_string()],
            doc_patterns: vec!["///".to_string(), "/**".to_string()],
        });
        
        // Perl patterns
        comment_patterns.insert("pl".to_string(), CommentPattern {
            single_line: vec!["#".to_string()],
            multi_line_start: vec!["=pod".to_string()],
            multi_line_end: vec!["=cut".to_string()],
            doc_patterns: vec!["=pod".to_string()],
        });
        
        // Clojure patterns
        comment_patterns.insert("clj".to_string(), CommentPattern {
            single_line: vec![";".to_string()],
            multi_line_start: vec!["#_".to_string()],
            multi_line_end: vec![], // #_ is single-form comment
            doc_patterns: vec![";;".to_string()],
        });
        comment_patterns.insert("cljs".to_string(), CommentPattern {
            single_line: vec![";".to_string()],
            multi_line_start: vec!["#_".to_string()],
            multi_line_end: vec![],
            doc_patterns: vec![";;".to_string()],
        });
        
        // F# patterns
        comment_patterns.insert("fs".to_string(), CommentPattern {
            single_line: vec!["//".to_string()],
            multi_line_start: vec!["(*".to_string()],
            multi_line_end: vec!["*)".to_string()],
            doc_patterns: vec!["///".to_string(), "(**".to_string()],
        });
        
        // Zig patterns
        comment_patterns.insert("zig".to_string(), CommentPattern {
            single_line: vec!["//".to_string()],
            multi_line_start: vec![],
            multi_line_end: vec![],
            doc_patterns: vec!["///".to_string(), "//!".to_string()],
        });
        
        // YAML patterns (comments only)
        comment_patterns.insert("yaml".to_string(), CommentPattern {
            single_line: vec!["#".to_string()],
            multi_line_start: vec![],
            multi_line_end: vec![],
            doc_patterns: vec!["##".to_string()],
        });
        comment_patterns.insert("yml".to_string(), CommentPattern {
            single_line: vec!["#".to_string()],
            multi_line_start: vec![],
            multi_line_end: vec![],
            doc_patterns: vec!["##".to_string()],
        });
        
        // TOML patterns
        comment_patterns.insert("toml".to_string(), CommentPattern {
            single_line: vec!["#".to_string()],
            multi_line_start: vec![],
            multi_line_end: vec![],
            doc_patterns: vec!["##".to_string()],
        });
        
        // INI patterns
        comment_patterns.insert("ini".to_string(), CommentPattern {
            single_line: vec![";".to_string(), "#".to_string()],
            multi_line_start: vec![],
            multi_line_end: vec![],
            doc_patterns: vec![";;".to_string()],
        });
        
        // XML patterns
        comment_patterns.insert("xml".to_string(), CommentPattern {
            single_line: vec![],
            multi_line_start: vec!["<!--".to_string()],
            multi_line_end: vec!["-->".to_string()],
            doc_patterns: vec!["<!--".to_string()],
        });
        
        // reStructuredText patterns
        comment_patterns.insert("rst".to_string(), CommentPattern {
            single_line: vec!["..".to_string()],
            multi_line_start: vec![],
            multi_line_end: vec![],
            doc_patterns: vec![], // RST content is documentation by nature
        });
        
        // AsciiDoc patterns
        comment_patterns.insert("adoc".to_string(), CommentPattern {
            single_line: vec!["//".to_string()],
            multi_line_start: vec!["////".to_string()],
            multi_line_end: vec!["////".to_string()],
            doc_patterns: vec![], // AsciiDoc content is documentation by nature
        });
        comment_patterns.insert("asciidoc".to_string(), CommentPattern {
            single_line: vec!["//".to_string()],
            multi_line_start: vec!["////".to_string()],
            multi_line_end: vec!["////".to_string()],
            doc_patterns: vec![], // AsciiDoc content is documentation by nature
        });
        
        // Batch file patterns
        comment_patterns.insert("bat".to_string(), CommentPattern {
            single_line: vec!["REM".to_string(), "rem".to_string(), "::".to_string()],
            multi_line_start: vec![],
            multi_line_end: vec![],
            doc_patterns: vec!["REM".to_string()],
        });
        comment_patterns.insert("cmd".to_string(), CommentPattern {
            single_line: vec!["REM".to_string(), "rem".to_string(), "::".to_string()],
            multi_line_start: vec![],
            multi_line_end: vec![],
            doc_patterns: vec!["REM".to_string()],
        });
        
        // Less patterns
        comment_patterns.insert("less".to_string(), CommentPattern {
            single_line: vec!["//".to_string()],
            multi_line_start: vec!["/*".to_string()],
            multi_line_end: vec!["*/".to_string()],
            doc_patterns: vec!["/**".to_string()],
        });
        
        // Vue patterns (similar to HTML but with JS-style comments in script sections)
        comment_patterns.insert("vue".to_string(), CommentPattern {
            single_line: vec!["//".to_string()],
            multi_line_start: vec!["<!--".to_string(), "/*".to_string()],
            multi_line_end: vec!["-->".to_string(), "*/".to_string()],
            doc_patterns: vec!["/**".to_string()],
        });
        
        // Svelte patterns
        comment_patterns.insert("svelte".to_string(), CommentPattern {
            single_line: vec!["//".to_string()],
            multi_line_start: vec!["<!--".to_string(), "/*".to_string()],
            multi_line_end: vec!["-->".to_string(), "*/".to_string()],
            doc_patterns: vec!["/**".to_string()],
        });
        
        Self { 
            comment_patterns,
            stats_calculator: StatsCalculator::new(),
        }
    }

    pub fn count_file(&self, path: &Path) -> Result<FileStats> {
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
                let is_doc_line = in_doc_comment;
                if trimmed.contains(&multi_line_end_pattern) {
                    in_multi_line_comment = false;
                    in_doc_comment = false;
                }
                
                if is_doc_line {
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
    
    fn count_markdown_file(&self, reader: BufReader<fs::File>, file_size: u64) -> Result<FileStats> {
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

    /// Calculate comprehensive statistics for a single file
    pub fn calculate_file_stats(&self, path: &Path) -> Result<AggregatedStats> {
        let file_stats = self.count_file(path)?;
        let path_str = path.to_string_lossy().to_string();
        
        let start_time = std::time::Instant::now();
        let mut aggregated_stats = self.stats_calculator.calculate_file_stats(&file_stats, &path_str)?;
        crate::core::stats::aggregation::StatsAggregator::update_timing(&mut aggregated_stats, start_time);
        
        Ok(aggregated_stats)
    }
    
    /// Calculate comprehensive statistics for a project
    pub fn calculate_project_stats(&self, code_stats: &CodeStats, individual_files: &[(String, FileStats)]) -> Result<AggregatedStats> {
        let start_time = std::time::Instant::now();
        let mut aggregated_stats = self.stats_calculator.calculate_project_stats(code_stats, individual_files)?;
        crate::core::stats::aggregation::StatsAggregator::update_timing(&mut aggregated_stats, start_time);
        
        Ok(aggregated_stats)
    }
    
    /// Get the stats calculator for direct access
    pub fn stats_calculator(&self) -> &StatsCalculator {
        &self.stats_calculator
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::test_utils::TestProject;
    
    #[test]
    fn test_rust_file_counting() {
        let project = TestProject::new().unwrap();
        let file_path = project.create_rust_file("test.rs", 2, 3).unwrap();
        
        let counter = CodeCounter::new();
        let stats = counter.count_file(&file_path).unwrap();
        
        assert!(stats.total_lines > 0);
        assert!(stats.code_lines > 0);
        assert!(stats.comment_lines > 0);
        assert!(stats.doc_lines > 0);
        assert!(stats.blank_lines > 0);
    }
    
    #[test]
    fn test_python_file_counting() {
        let project = TestProject::new().unwrap();
        let file_path = project.create_python_file("test.py", 2).unwrap();
        
        let counter = CodeCounter::new();
        let stats = counter.count_file(&file_path).unwrap();
        
        assert!(stats.total_lines > 0);
        assert!(stats.code_lines > 0);
        assert!(stats.comment_lines > 0);
        assert!(stats.doc_lines > 0); // Python docstrings
    }
    
    #[test]
    fn test_javascript_file_counting() {
        let project = TestProject::new().unwrap();
        let file_path = project.create_javascript_file("test.js", 2).unwrap();
        
        let counter = CodeCounter::new();
        let stats = counter.count_file(&file_path).unwrap();
        
        assert!(stats.total_lines > 0);
        assert!(stats.code_lines > 0);
        assert!(stats.comment_lines > 0);
        assert!(stats.doc_lines > 0); // JSDoc comments
    }
    
    #[test]
    fn test_markdown_file_counting() {
        let project = TestProject::new().unwrap();
        let content = r#"# Title

This is documentation content.

```rust
fn main() {
    println!("Hello, world!");
}
```

More documentation.

<!-- HTML comment -->
"#;
        let file_path = project.create_file("test.md", content).unwrap();
        
        let counter = CodeCounter::new();
        let stats = counter.count_file(&file_path).unwrap();
        
        assert!(stats.total_lines > 0);
        assert!(stats.code_lines > 0); // Code blocks
        assert!(stats.comment_lines > 0); // HTML comments
        assert!(stats.doc_lines > 0); // Markdown content
    }
    
    #[test]
    fn test_empty_file() {
        let project = TestProject::new().unwrap();
        let file_path = project.create_file("empty.rs", "").unwrap();
        
        let counter = CodeCounter::new();
        let stats = counter.count_file(&file_path).unwrap();
        
        assert_eq!(stats.total_lines, 0);
        assert_eq!(stats.code_lines, 0);
        assert_eq!(stats.comment_lines, 0);
        assert_eq!(stats.doc_lines, 0);
        assert_eq!(stats.blank_lines, 0);
    }
    
    #[test]
    fn test_only_blank_lines() {
        let project = TestProject::new().unwrap();
        let file_path = project.create_file("blank.rs", "\n\n\n\n").unwrap();
        
        let counter = CodeCounter::new();
        let stats = counter.count_file(&file_path).unwrap();
        
        assert_eq!(stats.total_lines, 4);
        assert_eq!(stats.code_lines, 0);
        assert_eq!(stats.comment_lines, 0);
        assert_eq!(stats.doc_lines, 0);
        assert_eq!(stats.blank_lines, 4);
    }
    
    #[test]
    fn test_aggregation() {
        let counter = CodeCounter::new();
        
        let file_stats = vec![
            ("rs".to_string(), FileStats {
                total_lines: 100,
                code_lines: 70,
                comment_lines: 20,
                blank_lines: 10,
                file_size: 1000,
                doc_lines: 15,
            }),
            ("rs".to_string(), FileStats {
                total_lines: 50,
                code_lines: 35,
                comment_lines: 10,
                blank_lines: 5,
                file_size: 500,
                doc_lines: 8,
            }),
            ("py".to_string(), FileStats {
                total_lines: 80,
                code_lines: 60,
                comment_lines: 15,
                blank_lines: 5,
                file_size: 800,
                doc_lines: 12,
            }),
        ];
        
        let aggregated = counter.aggregate_stats(file_stats);
        
        assert_eq!(aggregated.total_files, 3);
        assert_eq!(aggregated.total_lines, 230);
        assert_eq!(aggregated.total_code_lines, 165);
        assert_eq!(aggregated.total_comment_lines, 45);
        assert_eq!(aggregated.total_blank_lines, 20);
        assert_eq!(aggregated.total_size, 2300);
        assert_eq!(aggregated.total_doc_lines, 35);
        
        // Check per-extension stats
        assert_eq!(aggregated.stats_by_extension.len(), 2);
        
        let rust_stats = &aggregated.stats_by_extension["rs"];
        assert_eq!(rust_stats.0, 2); // 2 files
        assert_eq!(rust_stats.1.total_lines, 150);
        
        let python_stats = &aggregated.stats_by_extension["py"];
        assert_eq!(python_stats.0, 1); // 1 file
        assert_eq!(python_stats.1.total_lines, 80);
    }
    
    #[test]
    fn test_comment_patterns() {
        let counter = CodeCounter::new();
        
        // Test Rust patterns
        let rust_pattern = counter.comment_patterns.get("rs").unwrap();
        assert!(rust_pattern.single_line.contains(&"//".to_string()));
        assert!(rust_pattern.doc_patterns.contains(&"///".to_string()));
        assert!(rust_pattern.doc_patterns.contains(&"//!".to_string()));
        
        // Test Python patterns
        let python_pattern = counter.comment_patterns.get("py").unwrap();
        assert!(python_pattern.single_line.contains(&"#".to_string()));
        assert!(python_pattern.doc_patterns.contains(&"\"\"\"".to_string()));
        
        // Test JavaScript patterns
        let js_pattern = counter.comment_patterns.get("js").unwrap();
        assert!(js_pattern.single_line.contains(&"//".to_string()));
        assert!(js_pattern.doc_patterns.contains(&"/**".to_string()));
    }
    
    #[test]
    fn test_new_language_patterns() {
        let counter = CodeCounter::new();
        
        // Test PowerShell patterns
        assert!(counter.comment_patterns.contains_key("ps1"));
        let ps_pattern = counter.comment_patterns.get("ps1").unwrap();
        assert!(ps_pattern.single_line.contains(&"#".to_string()));
        assert!(ps_pattern.multi_line_start.contains(&"<#".to_string()));
        
        // Test Elm patterns
        assert!(counter.comment_patterns.contains_key("elm"));
        let elm_pattern = counter.comment_patterns.get("elm").unwrap();
        assert!(elm_pattern.single_line.contains(&"--".to_string()));
        assert!(elm_pattern.multi_line_start.contains(&"{-".to_string()));
        assert!(elm_pattern.doc_patterns.contains(&"{-|".to_string()));
        
        // Test Julia patterns
        assert!(counter.comment_patterns.contains_key("jl"));
        let julia_pattern = counter.comment_patterns.get("jl").unwrap();
        assert!(julia_pattern.single_line.contains(&"#".to_string()));
        assert!(julia_pattern.multi_line_start.contains(&"#=".to_string()));
        
        // Test SQL patterns
        assert!(counter.comment_patterns.contains_key("sql"));
        let sql_pattern = counter.comment_patterns.get("sql").unwrap();
        assert!(sql_pattern.single_line.contains(&"--".to_string()));
        assert!(sql_pattern.multi_line_start.contains(&"/*".to_string()));
        
        // Test Elixir patterns
        assert!(counter.comment_patterns.contains_key("ex"));
        let elixir_pattern = counter.comment_patterns.get("ex").unwrap();
        assert!(elixir_pattern.single_line.contains(&"#".to_string()));
        assert!(elixir_pattern.doc_patterns.contains(&"@doc".to_string()));
        
        // Test YAML patterns
        assert!(counter.comment_patterns.contains_key("yaml"));
        let yaml_pattern = counter.comment_patterns.get("yaml").unwrap();
        assert!(yaml_pattern.single_line.contains(&"#".to_string()));
        
        // Test Zig patterns
        assert!(counter.comment_patterns.contains_key("zig"));
        let zig_pattern = counter.comment_patterns.get("zig").unwrap();
        assert!(zig_pattern.single_line.contains(&"//".to_string()));
        assert!(zig_pattern.doc_patterns.contains(&"///".to_string()));
        
        // Test Clojure patterns
        assert!(counter.comment_patterns.contains_key("clj"));
        let clj_pattern = counter.comment_patterns.get("clj").unwrap();
        assert!(clj_pattern.single_line.contains(&";".to_string()));
        assert!(clj_pattern.doc_patterns.contains(&";;".to_string()));
        
        // Test F# patterns
        assert!(counter.comment_patterns.contains_key("fs"));
        let fs_pattern = counter.comment_patterns.get("fs").unwrap();
        assert!(fs_pattern.single_line.contains(&"//".to_string()));
        assert!(fs_pattern.multi_line_start.contains(&"(*".to_string()));
        assert!(fs_pattern.doc_patterns.contains(&"///".to_string()));
    }
} 