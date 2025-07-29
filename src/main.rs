use howmany::{FileDetector, FileFilter, Config, InteractiveDisplay, Result};
use howmany::ui::cli::{OutputFormat, SortBy};
use howmany::ui::filters::{FilterOptions, FileFilter as FileStatsFilter, FilteredOutputFormatter};
use howmany::core::types::{CodeStats, FileStats};
use howmany::core::stats::{StatsCalculator, AggregatedStats};
use howmany::core::counter::CachedCodeCounter;
use howmany::utils::metrics::MetricsCollector;
use std::path::Path;
use std::process;

fn main() {
    let mut config = Config::parse_args();
    
    // Apply presets and shortcuts before processing
    config.apply_output_preset();
    config.apply_advanced_filter_shortcuts();
    
    if let Err(e) = run(config) {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

fn run(config: Config) -> Result<()> {
    let path = config.path.as_deref().unwrap_or_else(|| Path::new("."));
    
    // Handle quiet mode - suppress most output except essential results
    if config.quiet && !config.cli_mode {
        return quiet_output(
            path,
            config.max_depth,
            config.include_hidden,
            config.get_ignore_patterns(),
            config.get_extensions(),
            config.get_filter_options(),
        );
    }
    
    // Simple CLI mode - just show basic counts
    if config.cli_mode {
        return simple_cli_output(
            path,
            config.max_depth,
            config.include_hidden,
            config.get_ignore_patterns(),
            config.get_extensions(),
            config.get_filter_options(),
        );
    }
    
    // Interactive mode (default unless --no-interactive is passed or specific output format is requested)
    if config.interactive() && matches!(config.format, OutputFormat::Text) && !config.quiet {
        let (aggregated_stats, individual_files) = analyze_code_comprehensive(
            path,
            config.max_depth,
            config.include_hidden,
            config.get_ignore_patterns(),
            config.get_extensions(),
            true, // Always collect individual files for interactive mode to enable real-time analysis
            &config.format,
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
            &config.format,
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
        &config.format,
    )?;
    
    output_comprehensive_results(
        &aggregated_stats,
        &individual_files,
        config.format.clone(),
        config.sort_by.clone(),
        config.descending,
        config.verbose,
        &config,
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
    output_format: &OutputFormat,
) -> Result<(AggregatedStats, Vec<(String, FileStats)>)> {
    // Only print messages for text output format
    let should_print = matches!(output_format, OutputFormat::Text);
    
    if should_print {
        println!("Analyzing directory: {}", path.display());
    }
    
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
    
    if should_print {
        println!("Scanning for user-created code files...");
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
        if should_print {
            println!("No files found matching the criteria.");
        }
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
    
    let mut counter = CachedCodeCounter::new();
    let mut metrics = MetricsCollector::new();
    
    if should_print {
        println!("Processing {} files...", file_paths.len());
    }
    
    // Process files sequentially to enable caching
    let mut file_stats = Vec::new();
    let mut individual_files = Vec::new();
    
    for file_path in &file_paths {
        match counter.count_file(file_path) {
            Ok(stats) => {
                // Record metrics
                metrics.record_file_processed(stats.total_lines, stats.file_size);
                
                let extension = file_path
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .unwrap_or("no_ext")
                    .to_string();
                file_stats.push((extension, stats.clone()));
                
                if show_files {
                    individual_files.push((file_path.to_string_lossy().to_string(), stats));
                }
            }
            Err(e) => {
                if show_files && should_print {
                    eprintln!("Warning: Failed to process {}: {}", file_path.display(), e);
                }
            }
        }
    }
    
    // Create basic aggregated stats
    let basic_code_stats = counter.aggregate_stats(file_stats);
    
    // Use comprehensive stats calculator
    let stats_calculator = StatsCalculator::new();
    let aggregated_stats = stats_calculator.calculate_project_stats(&basic_code_stats, &individual_files)?;
    
    // Save cache and cleanup
    counter.cleanup_cache();
    if let Err(e) = counter.save_cache() {
        if should_print {
            eprintln!("Warning: Failed to save cache: {}", e);
        }
    }
    
    // Show performance metrics only for text output
    let final_metrics = metrics.finish();
    let (cache_hits, cache_misses) = counter.cache_stats();
    
    if final_metrics.files_processed > 0 && should_print {
        println!("📊 Performance Summary:");
        println!("   • Files processed: {}", final_metrics.files_processed);
        println!("   • Processing time: {:.2}s", final_metrics.total_duration.as_secs_f64());
        
        if cache_hits + cache_misses > 0 {
            println!("   • Cache hit rate: {:.1}%", counter.cache_hit_rate() * 100.0);
            println!("   • Cache hits: {}", cache_hits);
            println!("   • Cache misses: {}", cache_misses);
            println!("   • Cache size: {} entries", counter.cache_size());
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
    output_format: &OutputFormat,
) -> Result<()> {
    let should_print = matches!(output_format, OutputFormat::Text);
    
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
    
    if should_print {
        println!("Files that would be counted:");
    }
    
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
    config: &Config,
) -> Result<()> {
    match format {
        OutputFormat::Text => output_text(aggregated_stats, individual_files, sort_by, descending, verbose, config),
        OutputFormat::Json => output_json(aggregated_stats, individual_files),
        OutputFormat::Csv => output_csv(aggregated_stats, individual_files),
        OutputFormat::Html => output_html(aggregated_stats, individual_files),
        OutputFormat::Sarif => output_sarif(aggregated_stats, individual_files),
    }
}

fn output_text(
    aggregated_stats: &AggregatedStats,
    individual_files: &[(String, FileStats)],
    sort_by: SortBy,
    descending: bool,
    verbose: bool,
    config: &Config,
) -> Result<()> {
    // Handle summary-only mode
    if config.summary_only {
        print_summary_only(aggregated_stats, config);
        return Ok(());
    }
    
    // Handle compact mode
    if config.compact_output {
        print_compact_output(aggregated_stats, config);
        return Ok(());
    }
    
    let use_color = !config.no_color && atty::is(atty::Stream::Stdout);
    
    // Header
    println!();
    println!("=== Code Statistics ===");
    
    // Basic stats
    println!("Total files: {}", format_number(aggregated_stats.basic.total_files, use_color));
    println!("Total lines: {}", format_number(aggregated_stats.basic.total_lines, use_color));
    println!("Code lines: {}", format_number(aggregated_stats.basic.code_lines, use_color));
    println!("Comment lines: {}", format_number(aggregated_stats.basic.comment_lines, use_color));
    println!("Documentation lines: {}", format_number(aggregated_stats.basic.doc_lines, use_color));
    println!("Blank lines: {}", format_number(aggregated_stats.basic.blank_lines, use_color));
    
    if config.show_size {
        let size_mb = aggregated_stats.basic.total_size as f64 / (1024.0 * 1024.0);
        println!("Total size: {} bytes ({:.2} MB)", 
            format_number(aggregated_stats.basic.total_size as usize, use_color), 
            size_mb
        );
    }
    
    // Time estimates
    if config.show_time_estimates {
        println!();
        println!("=== Time Estimates ===");
        
        // Simple time estimation based on lines of code
        let hours = (aggregated_stats.basic.code_lines as f64 * 0.5) / 60.0; // ~30 seconds per line
        let days = hours / 8.0;
        
        if days >= 1.0 {
            println!("Estimated development time: {:.1} days ({:.1} hours)", days, hours);
        } else {
            println!("Estimated development time: {:.1} hours", hours);
        }
    }
    
    // Enhanced stats from comprehensive analysis
    if config.show_complexity && aggregated_stats.complexity.function_count > 0 {
        println!();
        println!("=== Complexity Analysis ===");
        println!("Functions: {}", format_number(aggregated_stats.complexity.function_count, use_color));
        println!("Average complexity: {:.1}", aggregated_stats.complexity.cyclomatic_complexity);
        println!("Max nesting depth: {}", aggregated_stats.complexity.max_nesting_depth);
        
        if config.show_function_details {
            println!("Average function length: {:.1} lines", aggregated_stats.complexity.average_function_length);
            println!("Methods per class: {:.1}", aggregated_stats.complexity.methods_per_class);
        }
    }
    
    // Quality metrics
    if config.show_quality {
        println!();
        println!("=== Quality Metrics ===");
        
        let quality_score = aggregated_stats.ratios.quality_metrics.overall_quality_score;
        let quality_color = if use_color {
            if quality_score >= 80.0 { "\x1b[32m" } // Green
            else if quality_score >= 60.0 { "\x1b[33m" } // Yellow  
            else { "\x1b[31m" } // Red
        } else { "" };
        let reset = if use_color { "\x1b[0m" } else { "" };
        
        println!("Overall quality score: {}{:.1}/100{}", quality_color, quality_score, reset);
        println!("Documentation score: {:.1}/100", aggregated_stats.ratios.quality_metrics.documentation_score);
        println!("Maintainability score: {:.1}/100", aggregated_stats.ratios.quality_metrics.maintainability_score);
    }
    
    // Code ratios
    if config.show_ratios {
        println!();
        println!("=== Code Ratios ===");
        println!("Code ratio: {:.1}%", aggregated_stats.ratios.code_ratio * 100.0);
        println!("Comment ratio: {:.1}%", aggregated_stats.ratios.comment_ratio * 100.0);
        println!("Documentation ratio: {:.1}%", aggregated_stats.ratios.doc_ratio * 100.0);
    }
    
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
            SortBy::Complexity => extensions.sort_by(|(_, a), (_, b)| {
                // Sort by complexity if available, otherwise by lines
                let a_complexity = a.total_lines as f64;
                let b_complexity = b.total_lines as f64;
                a_complexity.partial_cmp(&b_complexity).unwrap_or(std::cmp::Ordering::Equal)
            }),
            SortBy::Quality => extensions.sort_by_key(|(_, ext_stats)| ext_stats.total_lines), // Placeholder
            SortBy::Functions => extensions.sort_by_key(|(_, ext_stats)| ext_stats.file_count), // Placeholder
            SortBy::DocRatio => extensions.sort_by(|(_, a), (_, b)| {
                let a_ratio = if a.total_lines > 0 { a.doc_lines as f64 / a.total_lines as f64 } else { 0.0 };
                let b_ratio = if b.total_lines > 0 { b.doc_lines as f64 / b.total_lines as f64 } else { 0.0 };
                a_ratio.partial_cmp(&b_ratio).unwrap_or(std::cmp::Ordering::Equal)
            }),
        }
        
        if descending {
            extensions.reverse();
        }
        
        // Apply top-n limit
        if let Some(top_n) = config.top_n {
            extensions.truncate(top_n);
        }
        
        for (ext, ext_stats) in extensions {
            println!("  {}: {} files, {} lines ({} code, {} docs, {} comments)",
                ext, ext_stats.file_count, ext_stats.total_lines, ext_stats.code_lines,
                ext_stats.doc_lines, ext_stats.comment_lines);
        }
    }
    
    if !individual_files.is_empty() && config.show_files {
        println!();
        println!("=== Individual Files ===");
        
        let mut files = individual_files.to_vec();
        
        // Apply top-n limit to individual files too
        if let Some(top_n) = config.top_n {
            files.truncate(top_n);
        }
        
        for (file_path, file_stats) in files {
            println!("  {}: {} lines ({} code)", file_path, file_stats.total_lines, file_stats.code_lines);
        }
    }
    
    Ok(())
}

/// Print summary-only output
fn print_summary_only(aggregated_stats: &AggregatedStats, config: &Config) {
    println!("Summary: {} files, {} lines ({} code, {} comments)", 
        aggregated_stats.basic.total_files,
        aggregated_stats.basic.total_lines,
        aggregated_stats.basic.code_lines,
        aggregated_stats.basic.comment_lines
    );
    
    if config.show_quality {
        println!("Quality: {:.1}/100", aggregated_stats.ratios.quality_metrics.overall_quality_score);
    }
}

/// Print compact output
fn print_compact_output(aggregated_stats: &AggregatedStats, config: &Config) {
    println!("{} files | {} lines | {} code | {} comments", 
        aggregated_stats.basic.total_files,
        aggregated_stats.basic.total_lines,
        aggregated_stats.basic.code_lines,
        aggregated_stats.basic.comment_lines
    );
    
    if config.show_quality {
        println!("Quality: {:.1}/100", aggregated_stats.ratios.quality_metrics.overall_quality_score);
    }
}

/// Format numbers with optional color
fn format_number(num: usize, use_color: bool) -> String {
    if use_color && num > 1000 {
        format!("\x1b[36m{}\x1b[0m", num) // Cyan for large numbers
    } else {
        num.to_string()
    }
}

fn output_json(
    aggregated_stats: &AggregatedStats,
    _individual_files: &[(String, FileStats)],
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
    
    // Use comprehensive report generation with real AggregatedStats
    reporter.generate_comprehensive_report(aggregated_stats, individual_files, output_path)?;
    println!("HTML report generated: {}", output_path.display());
    
    Ok(())
}

fn output_sarif(
    aggregated_stats: &AggregatedStats,
    individual_files: &[(String, FileStats)],
) -> Result<()> {
    use howmany::ui::sarif::SarifReporter;
    
    let reporter = SarifReporter::new();
    let output_path = Path::new("howmany-report.sarif");
    
    // Use comprehensive report generation with AggregatedStats
    reporter.generate_comprehensive_report(aggregated_stats, individual_files, output_path)?;
    println!("SARIF report generated: {}", output_path.display());
    
    Ok(())
}

/// Simple CLI output showing just basic file and line counts
fn simple_cli_output(
    path: &Path,
    max_depth: Option<usize>,
    include_hidden: bool,
    ignore_patterns: Vec<String>,
    extensions: Vec<String>,
    filter_options: FilterOptions,
) -> Result<()> {
    // Check if we need enhanced output (requires full analysis)
    let needs_enhanced_output = filter_options.show_complexity 
        || filter_options.show_quality 
        || filter_options.show_ratios;
    
    if needs_enhanced_output {
        // Run full analysis for enhanced output
        let (mut aggregated_stats, individual_files) = analyze_code_comprehensive(
            path,
            max_depth,
            include_hidden,
            ignore_patterns.clone(),
            extensions.clone(),
            false, // Don't need individual files for CLI output
            &OutputFormat::Text,
        )?;
        
        // Apply filters to the aggregated stats
        if !filter_options.include_languages.is_empty() 
            || !filter_options.exclude_languages.is_empty()
            || filter_options.min_lines.is_some()
            || filter_options.max_lines.is_some()
            || filter_options.min_size_bytes.is_some()
            || filter_options.max_size_bytes.is_some() {
            
            use howmany::ui::filters::ProjectFilter;
            let project_filter = ProjectFilter::new(filter_options.clone());
            let filtered_extensions = project_filter.filter_extensions(&aggregated_stats.basic.stats_by_extension);
            
            // Recalculate totals based on filtered extensions
            let mut total_files = 0;
            let mut total_lines = 0;
            let mut total_code_lines = 0;
            let mut total_comment_lines = 0;
            let mut total_blank_lines = 0;
            let mut total_size = 0;
            let mut total_doc_lines = 0;
            
            for stats in filtered_extensions.values() {
                total_files += stats.file_count;
                total_lines += stats.total_lines;
                total_code_lines += stats.code_lines;
                total_comment_lines += stats.comment_lines;
                total_blank_lines += stats.blank_lines;
                total_size += stats.total_size;
                total_doc_lines += stats.doc_lines;
            }
            
            // Update the basic stats with filtered totals
            aggregated_stats.basic.total_files = total_files;
            aggregated_stats.basic.total_lines = total_lines;
            aggregated_stats.basic.code_lines = total_code_lines;
            aggregated_stats.basic.comment_lines = total_comment_lines;
            aggregated_stats.basic.blank_lines = total_blank_lines;
            aggregated_stats.basic.total_size = total_size;
            aggregated_stats.basic.doc_lines = total_doc_lines;
            aggregated_stats.basic.stats_by_extension = filtered_extensions;
            
            // Recalculate ratios based on filtered data
            if total_lines > 0 {
                aggregated_stats.ratios.code_ratio = total_code_lines as f64 / total_lines as f64;
                aggregated_stats.ratios.comment_ratio = total_comment_lines as f64 / total_lines as f64;
                aggregated_stats.ratios.doc_ratio = total_doc_lines as f64 / total_lines as f64;
                aggregated_stats.ratios.blank_ratio = total_blank_lines as f64 / total_lines as f64;
            }
        }
        
        let output = FilteredOutputFormatter::format_enhanced_cli_output(
            &aggregated_stats,
            &individual_files,
            &filter_options,
        );
        println!("{}", output);
        return Ok(());
    }
    
    // Simple counting for basic output
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
    
    // Collect and filter files
    let file_stats_filter = FileStatsFilter::new(filter_options.clone());
    let mut filtered_files = Vec::new();
    let mut total_lines = 0;
    let mut counter = CachedCodeCounter::new();
    
    for entry in filter.walk_directory(path) {
        let entry_path = entry.path();
        
        if !entry_path.is_file() {
            continue;
        }
        
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
        
        // Count lines for this file
        if let Ok(stats) = counter.count_file(entry_path) {
            // Apply filters
            if file_stats_filter.passes_filter(&entry_path.to_string_lossy(), &stats) {
                filtered_files.push(entry_path.to_path_buf());
                total_lines += stats.total_lines;
            }
        }
    }
    
    if filter_options.show_size_info {
        // Calculate total size
        let total_size: u64 = filtered_files.iter()
            .filter_map(|path| std::fs::metadata(path).ok())
            .map(|metadata| metadata.len())
            .sum();
        let size_mb = total_size as f64 / (1024.0 * 1024.0);
        println!("{} files, {} lines, {:.1} MB", filtered_files.len(), total_lines, size_mb);
    } else {
        println!("{} files, {} lines", filtered_files.len(), total_lines);
    }
    
    Ok(())
}

/// Quiet mode output - minimal information only
fn quiet_output(
    path: &Path,
    max_depth: Option<usize>,
    include_hidden: bool,
    ignore_patterns: Vec<String>,
    extensions: Vec<String>,
    _filter_options: FilterOptions,
) -> Result<()> {
    let (aggregated_stats, _) = analyze_code_comprehensive(
        path,
        max_depth,
        include_hidden,
        ignore_patterns,
        extensions,
        false,
        &OutputFormat::Text,
    )?;
    
    // Just print the essential numbers
    println!("{} files, {} lines", 
        aggregated_stats.basic.total_files, 
        aggregated_stats.basic.total_lines
    );
    
    Ok(())
}

 