use howmany::{FileDetector, CodeCounter, FileFilter, Config, InteractiveDisplay, Result};
use howmany::ui::cli::{Commands, OutputFormat, SortBy};
use howmany::core::counter::{CodeStats, FileStats};
use std::env;
use std::path::Path;
use std::process;
use rayon::prelude::*;

fn main() {
    let config = Config::parse_args();
    
    match run(config) {
        Ok(()) => {}
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    }
}

fn run(config: Config) -> Result<()> {
    match config.command.unwrap_or(Commands::Interactive {
        path: config.path,
        max_depth: config.max_depth,
        files: config.files,
        include_hidden: config.include_hidden,
        ignore_gitignore: config.ignore_gitignore,
        custom_ignores: config.custom_ignores,
        extensions: config.extensions,
    }) {
        Commands::Count {
            path,
            max_depth,
            verbose,
            files,
            include_hidden,
            ignore_gitignore,
            custom_ignores,
            extensions,
            format,
            sort_by,
            descending,
        } => {
            let target_path = path.unwrap_or_else(|| env::current_dir().unwrap());
            
            println!("Analyzing directory: {}", target_path.display());
            println!("Scanning for user-created code files...\n");
            
            let stats = count_code(
                &target_path,
                max_depth,
                include_hidden,
                ignore_gitignore,
                custom_ignores,
                extensions,
                files,
            )?;
            
            output_results(&stats, format, sort_by, descending, verbose)?;
        }
        
        Commands::List {
            path,
            max_depth,
            include_hidden,
            ignore_gitignore,
            custom_ignores,
            extensions,
        } => {
            let target_path = path.unwrap_or_else(|| env::current_dir().unwrap());
            
            list_files(
                &target_path,
                max_depth,
                include_hidden,
                ignore_gitignore,
                custom_ignores,
                extensions,
            )?;
        }
        
        Commands::Interactive {
            path,
            max_depth,
            files,
            include_hidden,
            ignore_gitignore,
            custom_ignores,
            extensions,
        } => {
            let target_path = path.unwrap_or_else(|| env::current_dir().unwrap());
            
            let display = InteractiveDisplay::new();
            display.show_welcome()?;
            
            let pb = display.show_scanning_progress(&target_path.display().to_string());
            
            let stats = count_code(
                &target_path,
                max_depth,
                include_hidden,
                ignore_gitignore,
                custom_ignores,
                extensions,
                files,
            )?;
            
            pb.finish_and_clear();
            
            display.show_results(&stats.0, &stats.1)?;
        }
    }
    
    Ok(())
}

fn count_code(
    path: &Path,
    max_depth: Option<usize>,
    include_hidden: bool,
    ignore_gitignore: bool,
    custom_ignores: Vec<String>,
    extensions: Vec<String>,
    show_files: bool,
) -> Result<(CodeStats, Vec<(String, FileStats)>)> {
    let detector = FileDetector::new();
    let counter = CodeCounter::new();
    let mut filter = FileFilter::new()
        .respect_hidden(!include_hidden)
        .respect_gitignore(!ignore_gitignore);
    
    if let Some(depth) = max_depth {
        filter = filter.with_max_depth(depth);
    }
    
    if !custom_ignores.is_empty() {
        filter = filter.with_custom_ignores(custom_ignores);
    }
    
    // Collect all file paths first
    let file_paths: Vec<_> = filter.walk_directory(path)
        .filter_map(|entry| {
            let entry_path = entry.path();
            
            if !entry_path.is_file() {
                return None;
            }
            
            // Check if it's a user-created file
            if !detector.is_user_created_file(entry_path) {
                return None;
            }
            
            // Check if it passes additional filters
            if !filter.should_include_file(entry_path) {
                return None;
            }
            
            // Check extension filter if specified
            if !extensions.is_empty() {
                if let Some(ext) = entry_path.extension() {
                    let ext_str = ext.to_string_lossy().to_lowercase();
                    if !extensions.iter().any(|e| e.to_lowercase() == ext_str) {
                        return None;
                    }
                } else {
                    return None;
                }
            }
            
            Some(entry_path.to_path_buf())
        })
        .collect();
    
    // Process files in parallel
    let results: Vec<_> = file_paths
        .par_iter()
        .filter_map(|path| {
            match counter.count_file(path) {
                Ok(stats) => {
                    let extension = path.extension()
                        .and_then(|ext| ext.to_str())
                        .unwrap_or("no_extension")
                        .to_lowercase();
                    
                    Some((extension, stats, path.display().to_string()))
                }
                Err(e) => {
                    eprintln!("Warning: Could not read file {}: {}", path.display(), e);
                    None
                }
            }
        })
        .collect();
    
    // Separate into file_stats and individual_files
    let mut file_stats = Vec::new();
    let mut individual_files = Vec::new();
    
    for (extension, stats, path_str) in results {
        file_stats.push((extension, stats.clone()));
        
        if show_files {
            individual_files.push((path_str, stats));
        }
    }
    
    let aggregated_stats = counter.aggregate_stats(file_stats);
    
    Ok((aggregated_stats, individual_files))
}

fn list_files(
    path: &Path,
    max_depth: Option<usize>,
    include_hidden: bool,
    ignore_gitignore: bool,
    custom_ignores: Vec<String>,
    extensions: Vec<String>,
) -> Result<()> {
    let detector = FileDetector::new();
    let mut filter = FileFilter::new()
        .respect_hidden(!include_hidden)
        .respect_gitignore(!ignore_gitignore);
    
    if let Some(depth) = max_depth {
        filter = filter.with_max_depth(depth);
    }
    
    if !custom_ignores.is_empty() {
        filter = filter.with_custom_ignores(custom_ignores);
    }
    
    println!("Files that would be counted in: {}\n", path.display());
    
    for entry in filter.walk_directory(path) {
        let entry_path = entry.path();
        
        if entry_path.is_file() {
            // Check if it's a user-created file
            if !detector.is_user_created_file(entry_path) {
                continue;
            }
            
            // Check if it passes additional filters
            if !filter.should_include_file(entry_path) {
                continue;
            }
            
            // Check extension filter if specified
            if !extensions.is_empty() {
                if let Some(ext) = entry_path.extension() {
                    let ext_str = ext.to_string_lossy().to_lowercase();
                    if !extensions.iter().any(|e| e.to_lowercase() == ext_str) {
                        continue;
                    }
                } else {
                    continue;
                }
            }
            
            println!("{}", entry_path.display());
        }
    }
    
    Ok(())
}

fn output_results(
    stats: &(CodeStats, Vec<(String, FileStats)>),
    format: OutputFormat,
    sort_by: SortBy,
    descending: bool,
    verbose: bool,
) -> Result<()> {
    let (code_stats, individual_files) = stats;
    
    match format {
        OutputFormat::Text => output_text(code_stats, individual_files, sort_by, descending, verbose),
        OutputFormat::Json => output_json(code_stats, individual_files),
        OutputFormat::Csv => output_csv(code_stats, individual_files),
    }
}

fn output_text(
    stats: &CodeStats,
    individual_files: &[(String, FileStats)],
    sort_by: SortBy,
    descending: bool,
    verbose: bool,
) -> Result<()> {
    println!("=== Code Statistics ===");
    println!("Total files: {}", stats.total_files);
    println!("Total lines: {}", stats.total_lines);
    println!("Code lines: {}", stats.total_code_lines);
    println!("Comment lines: {}", stats.total_comment_lines);
    println!("Documentation lines: {}", stats.total_doc_lines);
    println!("Blank lines: {}", stats.total_blank_lines);
    println!("Total size: {} bytes ({:.2} KB)", stats.total_size, stats.total_size as f64 / 1024.0);
    
    if verbose {
        println!("\n=== Breakdown by File Type ===");
        
        let mut ext_stats: Vec<_> = stats.stats_by_extension.iter().collect();
        
        // Sort by the specified criteria
        ext_stats.sort_by(|a, b| {
            let comparison = match sort_by {
                SortBy::Files => a.1.0.cmp(&b.1.0),
                SortBy::Lines => a.1.1.total_lines.cmp(&b.1.1.total_lines),
                SortBy::Code => a.1.1.code_lines.cmp(&b.1.1.code_lines),
                SortBy::Comments => a.1.1.comment_lines.cmp(&b.1.1.comment_lines),
                SortBy::Size => a.1.1.file_size.cmp(&b.1.1.file_size),
            };
            
            if descending {
                comparison.reverse()
            } else {
                comparison
            }
        });
        
        println!("{:<12} {:<8} {:<10} {:<10} {:<12} {:<10} {:<10} {:<12}", 
                 "Extension", "Files", "Total", "Code", "Comments", "Docs", "Blank", "Size (KB)");
        println!("{}", "-".repeat(88));
        
        for (ext, (file_count, file_stats)) in ext_stats {
            println!("{:<12} {:<8} {:<10} {:<10} {:<12} {:<10} {:<10} {:<12.2}", 
                     ext,
                     file_count,
                     file_stats.total_lines,
                     file_stats.code_lines,
                     file_stats.comment_lines,
                     file_stats.doc_lines,
                     file_stats.blank_lines,
                     file_stats.file_size as f64 / 1024.0);
        }
    }
    
    if !individual_files.is_empty() {
        println!("\n=== Individual Files ===");
        println!("{:<50} {:<10} {:<10} {:<12} {:<10} {:<10}", 
                 "File", "Total", "Code", "Comments", "Docs", "Blank");
        println!("{}", "-".repeat(102));
        
        for (file_path, file_stats) in individual_files {
            println!("{:<50} {:<10} {:<10} {:<12} {:<10} {:<10}", 
                     file_path,
                     file_stats.total_lines,
                     file_stats.code_lines,
                     file_stats.comment_lines,
                     file_stats.doc_lines,
                     file_stats.blank_lines);
        }
    }
    
    Ok(())
}

fn output_json(
    stats: &CodeStats,
    individual_files: &[(String, FileStats)],
) -> Result<()> {
    let mut json_stats = serde_json::Map::new();
    json_stats.insert("total_files".to_string(), serde_json::Value::Number(stats.total_files.into()));
    json_stats.insert("total_lines".to_string(), serde_json::Value::Number(stats.total_lines.into()));
    json_stats.insert("total_code_lines".to_string(), serde_json::Value::Number(stats.total_code_lines.into()));
    json_stats.insert("total_comment_lines".to_string(), serde_json::Value::Number(stats.total_comment_lines.into()));
    json_stats.insert("total_doc_lines".to_string(), serde_json::Value::Number(stats.total_doc_lines.into()));
    json_stats.insert("total_blank_lines".to_string(), serde_json::Value::Number(stats.total_blank_lines.into()));
    json_stats.insert("total_size".to_string(), serde_json::Value::Number(stats.total_size.into()));
    
    let mut by_extension = serde_json::Map::new();
    for (ext, (file_count, file_stats)) in &stats.stats_by_extension {
        let mut ext_data = serde_json::Map::new();
        ext_data.insert("files".to_string(), serde_json::Value::Number((*file_count).into()));
        ext_data.insert("total_lines".to_string(), serde_json::Value::Number(file_stats.total_lines.into()));
        ext_data.insert("code_lines".to_string(), serde_json::Value::Number(file_stats.code_lines.into()));
        ext_data.insert("comment_lines".to_string(), serde_json::Value::Number(file_stats.comment_lines.into()));
        ext_data.insert("doc_lines".to_string(), serde_json::Value::Number(file_stats.doc_lines.into()));
        ext_data.insert("blank_lines".to_string(), serde_json::Value::Number(file_stats.blank_lines.into()));
        ext_data.insert("file_size".to_string(), serde_json::Value::Number(file_stats.file_size.into()));
        
        by_extension.insert(ext.clone(), serde_json::Value::Object(ext_data));
    }
    json_stats.insert("by_extension".to_string(), serde_json::Value::Object(by_extension));
    
    if !individual_files.is_empty() {
        let mut files_data = serde_json::Map::new();
        for (file_path, file_stats) in individual_files {
            let mut file_data = serde_json::Map::new();
            file_data.insert("total_lines".to_string(), serde_json::Value::Number(file_stats.total_lines.into()));
            file_data.insert("code_lines".to_string(), serde_json::Value::Number(file_stats.code_lines.into()));
            file_data.insert("comment_lines".to_string(), serde_json::Value::Number(file_stats.comment_lines.into()));
            file_data.insert("doc_lines".to_string(), serde_json::Value::Number(file_stats.doc_lines.into()));
            file_data.insert("blank_lines".to_string(), serde_json::Value::Number(file_stats.blank_lines.into()));
            file_data.insert("file_size".to_string(), serde_json::Value::Number(file_stats.file_size.into()));
            
            files_data.insert(file_path.clone(), serde_json::Value::Object(file_data));
        }
        json_stats.insert("individual_files".to_string(), serde_json::Value::Object(files_data));
    }
    
    let json_output = serde_json::Value::Object(json_stats);
    println!("{}", serde_json::to_string_pretty(&json_output)?);
    
    Ok(())
}

fn output_csv(
    stats: &CodeStats,
    _individual_files: &[(String, FileStats)],
) -> Result<()> {
    println!("Extension,Files,Total Lines,Code Lines,Comment Lines,Doc Lines,Blank Lines,Size (bytes)");
    
    for (ext, (file_count, file_stats)) in &stats.stats_by_extension {
        println!("{},{},{},{},{},{},{},{}", 
                 ext,
                 file_count,
                 file_stats.total_lines,
                 file_stats.code_lines,
                 file_stats.comment_lines,
                 file_stats.doc_lines,
                 file_stats.blank_lines,
                 file_stats.file_size);
    }
    
    Ok(())
} 