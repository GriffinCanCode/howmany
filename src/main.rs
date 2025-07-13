use howmany::{FileDetector, CodeCounter, FileFilter, Config, InteractiveDisplay, Result};
use howmany::ui::cli::{OutputFormat, SortBy};
use howmany::core::types::{CodeStats, FileStats};
use howmany::core::stats::{StatsCalculator, AggregatedStats, FormattingOptions};
use howmany::core::stats::techstack::TechStackAnalyzer;
use std::path::Path;
use std::process;
use rayon::prelude::*;

fn main() {
    let config = Config::parse_args();
    
    if let Err(e) = run(config) {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

fn run(config: Config) -> Result<()> {
    let path = config.path.as_deref().unwrap_or_else(|| Path::new("."));
    
    // Interactive mode (default unless --no-interactive is passed or specific output format is requested)
    if config.interactive() && matches!(config.format, OutputFormat::Text) {
        let (aggregated_stats, individual_files) = analyze_code_comprehensive(
            path,
            config.max_depth,
            config.include_hidden,
            config.get_ignore_patterns(),
            config.get_extensions(),
            config.show_files,
        )?;
        
        let mut display = InteractiveDisplay::new();
        display.show_welcome()?;
        let pb = display.show_scanning_progress(&path.display().to_string())?;
        pb.finish_and_clear();
        return display.show_comprehensive_results(&aggregated_stats, &individual_files).map_err(|e| {
            howmany::utils::errors::HowManyError::display(format!("Interactive display error: {}", e))
        });
    }
    
    // List files mode
    if config.list_files {
        return list_files(
            path,
            config.max_depth,
            config.include_hidden,
            config.get_ignore_patterns(),
            config.get_extensions(),
        );
    }
    
    // Regular counting mode with comprehensive analysis
    let (aggregated_stats, individual_files) = analyze_code_comprehensive(
        path,
        config.max_depth,
        config.include_hidden,
        config.get_ignore_patterns(),
        config.get_extensions(),
        config.show_files,
    )?;
    
    output_comprehensive_results(
        &aggregated_stats,
        &individual_files,
        config.format,
        config.sort_by,
        config.descending,
        config.verbose,
    )
}

/// Comprehensive code analysis using the full stats pipeline
fn analyze_code_comprehensive(
    path: &Path,
    max_depth: Option<usize>,
    include_hidden: bool,
    ignore_patterns: Vec<String>,
    extensions: Vec<String>,
    show_files: bool,
) -> Result<(AggregatedStats, Vec<(String, FileStats)>)> {
    println!("Analyzing directory: {}", path.display());
    
    let detector = FileDetector::new();
    let mut filter = FileFilter::new()
        .respect_hidden(!include_hidden)
        .respect_gitignore(true);
    
    if let Some(depth) = max_depth {
        filter = filter.with_max_depth(depth);
    }
    
    // Add custom ignore patterns
    if !ignore_patterns.is_empty() {
        filter = filter.with_custom_ignores(ignore_patterns);
    }
    
    println!("Scanning for user-created code files...");
    
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
    
    if file_paths.is_empty() {
        println!("No files found matching the criteria.");
        let empty_stats = StatsCalculator::new().calculate_project_stats(
            &CodeStats {
                total_files: 0,
                total_lines: 0,
                total_code_lines: 0,
                total_comment_lines: 0,
                total_blank_lines: 0,
                total_size: 0,
                total_doc_lines: 0,
                stats_by_extension: std::collections::HashMap::new(),
            },
            &[],
        )?;
        return Ok((empty_stats, Vec::new()));
    }
    
    let counter = CodeCounter::new();
    
    // Process files in parallel for better performance
    let file_stats: Vec<(String, FileStats)> = file_paths
        .par_iter()
        .filter_map(|file_path| {
            match counter.count_file(file_path) {
                Ok(stats) => {
                    let extension = file_path
                        .extension()
                        .and_then(|ext| ext.to_str())
                        .unwrap_or("no_ext")
                        .to_string();
                    Some((extension, stats))
                }
                Err(e) => {
                    if show_files {
                        eprintln!("Warning: Failed to process {}: {}", file_path.display(), e);
                    }
                    None
                }
            }
        })
        .collect();
    
    let individual_files: Vec<(String, FileStats)> = if show_files {
        file_paths
            .par_iter()
            .filter_map(|file_path| {
                match counter.count_file(file_path) {
                    Ok(stats) => Some((file_path.to_string_lossy().to_string(), stats)),
                    Err(_) => None,
                }
            })
            .collect()
    } else {
        Vec::new()
    };
    
    // Create basic aggregated stats
    let basic_code_stats = counter.aggregate_stats(file_stats);
    
    // Use comprehensive stats calculator
    let stats_calculator = StatsCalculator::new();
    let mut aggregated_stats = stats_calculator.calculate_project_stats(&basic_code_stats, &individual_files)?;
    
    // Optional: Add techstack analysis if enabled
    if std::env::var("HOWMANY_ENABLE_TECHSTACK").is_ok() {
        println!("🔍 Analyzing technology stack...");
        if let Ok(techstack_analyzer) = TechStackAnalyzer::new().analyze_project(&path.to_string_lossy()) {
            // Store techstack analysis in metadata for future use
            println!("✅ Detected {} technologies", techstack_analyzer.inventory.technologies.len());
        }
    }
    
    Ok((aggregated_stats, individual_files))
}

fn list_files(
    path: &Path,
    max_depth: Option<usize>,
    include_hidden: bool,
    ignore_patterns: Vec<String>,
    extensions: Vec<String>,
) -> Result<()> {
    let detector = FileDetector::new();
    let mut filter = FileFilter::new()
        .respect_hidden(!include_hidden)
        .respect_gitignore(true);
    
    if let Some(depth) = max_depth {
        filter = filter.with_max_depth(depth);
    }
    
    // Add custom ignore patterns
    if !ignore_patterns.is_empty() {
        filter = filter.with_custom_ignores(ignore_patterns);
    }
    
    println!("Files that would be counted:");
    
    for entry in filter.walk_directory(path) {
        let entry_path = entry.path();
        
        if entry_path.is_file() {
            // Check if it's a user-created file
            if !detector.is_user_created_file(entry_path) {
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
            
            println!("  {}", entry_path.display());
        }
    }
    
    Ok(())
}

fn output_comprehensive_results(
    aggregated_stats: &AggregatedStats,
    individual_files: &[(String, FileStats)],
    format: OutputFormat,
    sort_by: SortBy,
    descending: bool,
    verbose: bool,
) -> Result<()> {
    match format {
        OutputFormat::Text => output_text(aggregated_stats, individual_files, sort_by, descending, verbose),
        OutputFormat::Json => output_json(aggregated_stats, individual_files),
        OutputFormat::Csv => output_csv(aggregated_stats, individual_files),
        OutputFormat::Html => output_html(aggregated_stats, individual_files),
        OutputFormat::TimeWasted => output_time_wasted(aggregated_stats, individual_files),
    }
}

fn output_text(
    aggregated_stats: &AggregatedStats,
    individual_files: &[(String, FileStats)],
    sort_by: SortBy,
    descending: bool,
    verbose: bool,
) -> Result<()> {
    println!();
    println!("=== Code Statistics ===");
    println!("Total files: {}", aggregated_stats.basic.total_files);
    println!("Total lines: {}", aggregated_stats.basic.total_lines);
    println!("Code lines: {}", aggregated_stats.basic.code_lines);
    println!("Comment lines: {}", aggregated_stats.basic.comment_lines);
    println!("Documentation lines: {}", aggregated_stats.basic.doc_lines);
    println!("Blank lines: {}", aggregated_stats.basic.blank_lines);
    println!("Total size: {} bytes ({:.2} KB)", aggregated_stats.basic.total_size, aggregated_stats.basic.total_size as f64 / 1024.0);
    
    // Enhanced stats from comprehensive analysis
    if aggregated_stats.complexity.function_count > 0 {
        println!();
        println!("=== Complexity Analysis ===");
        println!("Functions: {}", aggregated_stats.complexity.function_count);
        println!("Average complexity: {:.1}", aggregated_stats.complexity.cyclomatic_complexity);
        println!("Max nesting depth: {}", aggregated_stats.complexity.max_nesting_depth);
    }
    
    // Time estimates
    println!();
    println!("=== Time Estimates ===");
    println!("Total development time: {}", aggregated_stats.time.total_time_formatted);
    println!("Code writing time: {}", aggregated_stats.time.code_time_formatted);
    println!("Documentation time: {}", aggregated_stats.time.doc_time_formatted);
    
    // Quality metrics
    println!();
    println!("=== Quality Metrics ===");
    println!("Overall quality score: {:.1}/100", aggregated_stats.ratios.quality_metrics.overall_quality_score);
    println!("Documentation score: {:.1}/100", aggregated_stats.ratios.quality_metrics.documentation_score);
    println!("Maintainability score: {:.1}/100", aggregated_stats.ratios.quality_metrics.maintainability_score);
    
    if verbose || !aggregated_stats.basic.stats_by_extension.is_empty() {
        println!();
        println!("=== Breakdown by Extension ===");
        
        let mut extensions: Vec<_> = aggregated_stats.basic.stats_by_extension.iter().collect();
        
        // Sort based on the selected criteria
        match sort_by {
            SortBy::Files => extensions.sort_by_key(|(_, ext_stats)| ext_stats.file_count),
            SortBy::Lines => extensions.sort_by_key(|(_, ext_stats)| ext_stats.total_lines),
            SortBy::Code => extensions.sort_by_key(|(_, ext_stats)| ext_stats.code_lines),
            SortBy::Comments => extensions.sort_by_key(|(_, ext_stats)| ext_stats.comment_lines),
            SortBy::Size => extensions.sort_by_key(|(_, ext_stats)| ext_stats.total_size),
        }
        
        if descending {
            extensions.reverse();
        }
        
        for (ext, ext_stats) in extensions {
            println!("  {}: {} files, {} lines ({} code, {} docs, {} comments)",
                ext, ext_stats.file_count, ext_stats.total_lines, ext_stats.code_lines,
                ext_stats.doc_lines, ext_stats.comment_lines);
        }
    }
    
    if !individual_files.is_empty() {
        println!();
        println!("=== Individual Files ===");
        for (file_path, file_stats) in individual_files {
            println!("  {}: {} lines ({} code)", file_path, file_stats.total_lines, file_stats.code_lines);
        }
    }
    
    Ok(())
}

fn output_json(
    aggregated_stats: &AggregatedStats,
    individual_files: &[(String, FileStats)],
) -> Result<()> {
    // Use the comprehensive stats serialization
    let json_output = serde_json::to_string_pretty(aggregated_stats)?;
    println!("{}", json_output);
    Ok(())
}

fn output_csv(
    aggregated_stats: &AggregatedStats,
    _individual_files: &[(String, FileStats)],
) -> Result<()> {
    println!("Extension,Files,Total Lines,Code Lines,Comment Lines,Doc Lines,Blank Lines,Size (bytes)");
    
    for (ext, ext_stats) in &aggregated_stats.basic.stats_by_extension {
        println!("{},{},{},{},{},{},{},{}",
            ext,
            ext_stats.file_count,
            ext_stats.total_lines,
            ext_stats.code_lines,
            ext_stats.comment_lines,
            ext_stats.doc_lines,
            ext_stats.blank_lines,
            ext_stats.total_size);
    }
    
    Ok(())
}

fn output_html(
    aggregated_stats: &AggregatedStats,
    individual_files: &[(String, FileStats)],
) -> Result<()> {
    use howmany::ui::html::HtmlReporter;
    
    let reporter = HtmlReporter::new();
    let output_path = Path::new("howmany-report.html");
    
    // Convert AggregatedStats back to CodeStats for compatibility
    let code_stats = CodeStats {
        total_files: aggregated_stats.basic.total_files,
        total_lines: aggregated_stats.basic.total_lines,
        total_code_lines: aggregated_stats.basic.code_lines,
        total_comment_lines: aggregated_stats.basic.comment_lines,
        total_blank_lines: aggregated_stats.basic.blank_lines,
        total_size: aggregated_stats.basic.total_size,
        total_doc_lines: aggregated_stats.basic.doc_lines,
        stats_by_extension: aggregated_stats.basic.stats_by_extension.iter()
            .map(|(ext, ext_stats)| {
                (ext.clone(), (ext_stats.file_count, FileStats {
                    total_lines: ext_stats.total_lines,
                    code_lines: ext_stats.code_lines,
                    comment_lines: ext_stats.comment_lines,
                    blank_lines: ext_stats.blank_lines,
                    file_size: ext_stats.total_size,
                    doc_lines: ext_stats.doc_lines,
                }))
            })
            .collect(),
    };
    
    reporter.generate_report(&code_stats, individual_files, output_path)?;
    println!("HTML report generated: {}", output_path.display());
    
    Ok(())
}

fn output_time_wasted(
    aggregated_stats: &AggregatedStats,
    individual_files: &[(String, FileStats)],
) -> Result<()> {
    use howmany::ui::html::HtmlReporter;
    
    let reporter = HtmlReporter::new();
    let output_path = Path::new("time-wasted-report.html");
    
    // Convert AggregatedStats back to CodeStats for compatibility
    let code_stats = CodeStats {
        total_files: aggregated_stats.basic.total_files,
        total_lines: aggregated_stats.basic.total_lines,
        total_code_lines: aggregated_stats.basic.code_lines,
        total_comment_lines: aggregated_stats.basic.comment_lines,
        total_blank_lines: aggregated_stats.basic.blank_lines,
        total_size: aggregated_stats.basic.total_size,
        total_doc_lines: aggregated_stats.basic.doc_lines,
        stats_by_extension: aggregated_stats.basic.stats_by_extension.iter()
            .map(|(ext, ext_stats)| {
                (ext.clone(), (ext_stats.file_count, FileStats {
                    total_lines: ext_stats.total_lines,
                    code_lines: ext_stats.code_lines,
                    comment_lines: ext_stats.comment_lines,
                    blank_lines: ext_stats.blank_lines,
                    file_size: ext_stats.total_size,
                    doc_lines: ext_stats.doc_lines,
                }))
            })
            .collect(),
    };
    
    reporter.generate_time_wasted_report(&code_stats, individual_files, output_path)?;
    println!("Time wasted report generated: {}", output_path.display());
    
    Ok(())
} 