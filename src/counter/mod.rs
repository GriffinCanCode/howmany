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
}

#[derive(Debug, Clone)]
pub struct CodeStats {
    pub total_files: usize,
    pub total_lines: usize,
    pub total_code_lines: usize,
    pub total_comment_lines: usize,
    pub total_blank_lines: usize,
    pub total_size: u64,
    pub stats_by_extension: HashMap<String, (usize, FileStats)>, // (file_count, aggregated_stats)
}

pub struct CodeCounter {
    comment_patterns: HashMap<String, Vec<String>>,
}

impl CodeCounter {
    pub fn new() -> Self {
        let mut comment_patterns = HashMap::new();
        
        // Single-line comment patterns by file extension
        comment_patterns.insert("rs".to_string(), vec!["//".to_string()]);
        comment_patterns.insert("py".to_string(), vec!["#".to_string()]);
        comment_patterns.insert("js".to_string(), vec!["//".to_string()]);
        comment_patterns.insert("ts".to_string(), vec!["//".to_string()]);
        comment_patterns.insert("jsx".to_string(), vec!["//".to_string()]);
        comment_patterns.insert("tsx".to_string(), vec!["//".to_string()]);
        comment_patterns.insert("java".to_string(), vec!["//".to_string()]);
        comment_patterns.insert("c".to_string(), vec!["//".to_string()]);
        comment_patterns.insert("cpp".to_string(), vec!["//".to_string()]);
        comment_patterns.insert("cc".to_string(), vec!["//".to_string()]);
        comment_patterns.insert("cxx".to_string(), vec!["//".to_string()]);
        comment_patterns.insert("h".to_string(), vec!["//".to_string()]);
        comment_patterns.insert("hpp".to_string(), vec!["//".to_string()]);
        comment_patterns.insert("cs".to_string(), vec!["//".to_string()]);
        comment_patterns.insert("php".to_string(), vec!["//".to_string(), "#".to_string()]);
        comment_patterns.insert("rb".to_string(), vec!["#".to_string()]);
        comment_patterns.insert("go".to_string(), vec!["//".to_string()]);
        comment_patterns.insert("swift".to_string(), vec!["//".to_string()]);
        comment_patterns.insert("kt".to_string(), vec!["//".to_string()]);
        comment_patterns.insert("scala".to_string(), vec!["//".to_string()]);
        comment_patterns.insert("sh".to_string(), vec!["#".to_string()]);
        comment_patterns.insert("bash".to_string(), vec!["#".to_string()]);
        comment_patterns.insert("zsh".to_string(), vec!["#".to_string()]);
        comment_patterns.insert("fish".to_string(), vec!["#".to_string()]);
        comment_patterns.insert("r".to_string(), vec!["#".to_string()]);
        comment_patterns.insert("lua".to_string(), vec!["--".to_string()]);
        comment_patterns.insert("hs".to_string(), vec!["--".to_string()]);
        comment_patterns.insert("ml".to_string(), vec!["(*".to_string()]);
        comment_patterns.insert("html".to_string(), vec!["<!--".to_string()]);
        comment_patterns.insert("css".to_string(), vec!["/*".to_string()]);
        comment_patterns.insert("scss".to_string(), vec!["//".to_string(), "/*".to_string()]);
        comment_patterns.insert("sass".to_string(), vec!["//".to_string()]);
        
        Self { comment_patterns }
    }

    pub fn count_file(&self, path: &Path) -> io::Result<FileStats> {
        let file = fs::File::open(path)?;
        let reader = BufReader::new(file);
        
        let mut total_lines = 0;
        let mut code_lines = 0;
        let mut comment_lines = 0;
        let mut blank_lines = 0;
        
        let extension = path.extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_lowercase();
        
        let comment_prefixes = self.comment_patterns.get(&extension).cloned().unwrap_or_default();
        
        for line in reader.lines() {
            let line = line?;
            total_lines += 1;
            
            let trimmed = line.trim();
            
            if trimmed.is_empty() {
                blank_lines += 1;
            } else if self.is_comment_line(trimmed, &comment_prefixes) {
                comment_lines += 1;
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
        })
    }
    
    fn is_comment_line(&self, line: &str, comment_prefixes: &[String]) -> bool {
        for prefix in comment_prefixes {
            if line.starts_with(prefix) {
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
        let mut stats_by_extension: HashMap<String, (usize, FileStats)> = HashMap::new();
        
        for (extension, stats) in file_stats {
            total_files += 1;
            total_lines += stats.total_lines;
            total_code_lines += stats.code_lines;
            total_comment_lines += stats.comment_lines;
            total_blank_lines += stats.blank_lines;
            total_size += stats.file_size;
            
            let entry = stats_by_extension.entry(extension).or_insert((0, FileStats {
                total_lines: 0,
                code_lines: 0,
                comment_lines: 0,
                blank_lines: 0,
                file_size: 0,
            }));
            
            entry.0 += 1; // file count
            entry.1.total_lines += stats.total_lines;
            entry.1.code_lines += stats.code_lines;
            entry.1.comment_lines += stats.comment_lines;
            entry.1.blank_lines += stats.blank_lines;
            entry.1.file_size += stats.file_size;
        }
        
        CodeStats {
            total_files,
            total_lines,
            total_code_lines,
            total_comment_lines,
            total_blank_lines,
            total_size,
            stats_by_extension,
        }
    }
} 