use howmany::core::detector::SkipReason;
use howmany::core::engine::{Analysis, AnalysisOptions, AnalysisReport, Engine};
use howmany::core::languages::{Breakdown, Category, LanguageRow};
use howmany::core::stats::AggregatedStats;
use howmany::core::types::FileStats;
use howmany::ui::cli::{OutputFormat, SortBy};
use howmany::ui::filters::{
    FileComplexity, FileFilter as FileStatsFilter, FilterOptions, FilteredOutputFormatter,
};
use howmany::{Config, InteractiveDisplay, Result};
use std::cmp::Reverse;
use std::io::{IsTerminal, Write};
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

    // A path that does not exist is a typo, not an empty project. Reporting
    // "0 files, 0 lines" for it looks like a successful answer and has sent
    // people looking for the bug in their code instead of in their command.
    if !path.exists() {
        return Err(howmany::utils::errors::HowManyError::file_processing(
            format!("{} does not exist", path.display()),
        ));
    }

    if config.list_files {
        return list_files(path, &config);
    }

    // Quiet mode - suppress most output except essential results
    if config.quiet && !config.cli_mode {
        return quiet_output(path, &config);
    }

    // Simple CLI mode - just show basic counts
    if config.cli_mode {
        return simple_cli_output(path, &config, config.get_filter_options());
    }

    // Interactive mode (default unless --no-interactive or an explicit format).
    // It needs a terminal: without one the TUI cannot start and the run falls
    // back to a display that lists every file and then blocks on a keypress,
    // which is not something a pipe or a CI job can answer.
    if config.interactive()
        && matches!(config.format, OutputFormat::Text)
        && !config.quiet
        && std::io::stdout().is_terminal()
    {
        // Interactive mode analyses per-file so its drill-down views have data.
        let analysis = analyze(path, &config, true)?;

        let mut display = InteractiveDisplay::new();
        display.show_welcome()?;
        let pb = display.show_scanning_progress(&path.display().to_string())?;
        pb.finish_and_clear();
        return display
            .show_comprehensive_results(&analysis.stats, &analysis.individual_files)
            .map_err(|e| {
                howmany::utils::errors::HowManyError::display(format!(
                    "Interactive display error: {}",
                    e
                ))
            });
    }

    let analysis = analyze(path, &config, config.show_files)?;

    output_comprehensive_results(&analysis, &config)
}

/// Run the analysis pipeline, reporting progress only for human-readable output.
fn analyze(path: &Path, config: &Config, collect_individual_files: bool) -> Result<Analysis> {
    let should_print = matches!(config.format, OutputFormat::Text) && !config.quiet;
    let options = config.analysis_options(collect_individual_files);

    if should_print {
        println!("Analyzing directory: {}", path.display());
    }

    let analysis = Engine::new().analyze(path, &options)?;

    if should_print {
        report_run(&analysis, config);
    }

    Ok(analysis)
}

/// Print what the run cost, and anything it could not read.
fn report_run(analysis: &Analysis, config: &Config) {
    let report = &analysis.report;

    if report.files_counted == 0 && report.files_failed == 0 {
        println!("No files found matching the criteria.");
        return;
    }

    if report.detection_unavailable {
        eprintln!(
            "Note: the optional 'sherlock' language detector is not installed; \
             classifying files by extension."
        );
    }

    println!("Processed {} files", report.files_counted);

    if config.verbose {
        println!("Performance Summary:");
        println!("   - Files processed: {}", report.files_counted);
        println!(
            "   - Processing time: {:.3}s (discover {:.3}s, count {:.3}s, aggregate {:.3}s)",
            report.total_time.as_secs_f64(),
            report.discovery_time.as_secs_f64(),
            report.counting_time.as_secs_f64(),
            report.aggregation_time.as_secs_f64(),
        );
        if let Some(rate) = report.throughput_files_per_second() {
            println!("   - Throughput: {rate:.0} files/s");
        }
        if report.cache_hits + report.cache_misses > 0 {
            println!(
                "   - Cache hit rate: {:.1}%",
                report.cache_hit_rate() * 100.0
            );
            println!("   - Cache hits: {}", report.cache_hits);
            println!("   - Cache misses: {}", report.cache_misses);
        }
    }

    if report.files_failed > 0 {
        eprintln!(
            "Warning: {} file(s) could not be read:",
            report.files_failed
        );
        for (path, message) in report.failures.iter().take(10) {
            eprintln!("  {}: {}", path.display(), message);
        }
        if report.failures.len() > 10 {
            eprintln!("  ... and {} more", report.failures.len() - 10);
        }
    }
}

fn list_files(path: &Path, config: &Config) -> Result<()> {
    let options = config.analysis_options(false);
    let files = Engine::new().discover_files(path, &options)?;

    if matches!(config.format, OutputFormat::Text) && !config.quiet {
        println!("Files that would be counted:");
    }

    let stdout = std::io::stdout();
    let mut out = std::io::BufWriter::new(stdout.lock());
    for file in files {
        let _ = writeln!(out, "  {}", file.display());
    }
    let _ = out.flush();

    Ok(())
}

fn output_comprehensive_results(analysis: &Analysis, config: &Config) -> Result<()> {
    let (stats, files) = (&analysis.stats, &analysis.individual_files[..]);
    match config.format {
        OutputFormat::Text => output_text(analysis, config),
        OutputFormat::Json => output_json(stats),
        OutputFormat::Csv => output_csv(stats),
        OutputFormat::Html => output_html(stats, files, config),
        OutputFormat::Sarif => output_sarif(stats, files, config),
    }
}

fn output_text(analysis: &Analysis, config: &Config) -> Result<()> {
    let aggregated_stats = &analysis.stats;
    let individual_files = &analysis.individual_files[..];
    let report = &analysis.report;
    let (sort_by, ascending, verbose) = (config.sort_by, config.ascending, config.verbose);
    if config.summary_only {
        print_summary_only(aggregated_stats, config);
        return Ok(());
    }

    if config.compact_output {
        print_compact_output(aggregated_stats, config);
        return Ok(());
    }

    let use_color = !config.no_color && std::io::stdout().is_terminal();

    println!();
    println!("=== Code Statistics ===");

    println!(
        "Total files: {}",
        format_number(aggregated_stats.basic.total_files, use_color)
    );
    println!(
        "Total lines: {}",
        format_number(aggregated_stats.basic.total_lines, use_color)
    );
    println!(
        "Code lines: {}",
        format_number(aggregated_stats.basic.code_lines, use_color)
    );
    println!(
        "Comment lines: {}",
        format_number(aggregated_stats.basic.comment_lines, use_color)
    );
    println!(
        "Documentation lines: {}",
        format_number(aggregated_stats.basic.doc_lines, use_color)
    );
    println!(
        "Blank lines: {}",
        format_number(aggregated_stats.basic.blank_lines, use_color)
    );

    if config.show_size {
        let size_mb = aggregated_stats.basic.total_size as f64 / (1024.0 * 1024.0);
        println!(
            "Total size: {} bytes ({:.2} MB)",
            format_number(aggregated_stats.basic.total_size as usize, use_color),
            size_mb
        );
    }

    if config.show_time_estimates {
        println!();
        println!("=== Time Estimates ===");

        let hours = (aggregated_stats.basic.code_lines as f64 * 0.5) / 60.0;
        let days = hours / 8.0;

        if days >= 1.0 {
            println!(
                "Estimated development time: {:.1} days ({:.1} hours)",
                days, hours
            );
        } else {
            println!("Estimated development time: {:.1} hours", hours);
        }
    }

    if config.show_complexity && aggregated_stats.complexity.function_count > 0 {
        println!();
        println!("=== Complexity Analysis ===");
        println!(
            "Functions: {}",
            format_number(aggregated_stats.complexity.function_count, use_color)
        );
        println!(
            "Average complexity: {:.1}",
            aggregated_stats.complexity.cyclomatic_complexity
        );
        println!(
            "Max nesting depth: {}",
            aggregated_stats.complexity.max_nesting_depth
        );

        if config.show_function_details {
            println!(
                "Average function length: {:.1} lines",
                aggregated_stats.complexity.average_function_length
            );
            println!(
                "Methods per class: {:.1}",
                aggregated_stats.complexity.methods_per_class
            );
        }
    }

    if config.show_quality {
        println!();
        println!("=== Quality Metrics ===");

        let quality_score = aggregated_stats
            .ratios
            .quality_metrics
            .overall_quality_score;
        let quality_color = if use_color {
            if quality_score >= 80.0 {
                "\x1b[32m"
            } else if quality_score >= 60.0 {
                "\x1b[33m"
            } else {
                "\x1b[31m"
            }
        } else {
            ""
        };
        let reset = if use_color { "\x1b[0m" } else { "" };

        println!(
            "Overall quality score: {}{:.1}/100{}",
            quality_color, quality_score, reset
        );
        println!(
            "Documentation score: {:.1}/100",
            aggregated_stats.ratios.quality_metrics.documentation_score
        );
        println!(
            "Maintainability score: {:.1}/100",
            aggregated_stats
                .ratios
                .quality_metrics
                .maintainability_score
        );
    }

    if config.show_ratios {
        println!();
        println!("=== Code Ratios ===");
        println!(
            "Code ratio: {:.1}%",
            aggregated_stats.ratios.code_ratio * 100.0
        );
        println!(
            "Comment ratio: {:.1}%",
            aggregated_stats.ratios.comment_ratio * 100.0
        );
        println!(
            "Documentation ratio: {:.1}%",
            aggregated_stats.ratios.doc_ratio * 100.0
        );
    }

    if !aggregated_stats.basic.stats_by_extension.is_empty() {
        print_language_breakdown(
            &Breakdown::from_extensions(&aggregated_stats.basic.stats_by_extension),
            sort_by,
            ascending,
            verbose,
            config,
            use_color,
        );
    }

    print_skipped(report, use_color);

    if !individual_files.is_empty() && config.show_files {
        println!();
        println!("=== Individual Files ===");

        let mut files = individual_files.to_vec();

        if let Some(top_n) = config.top_n {
            files.truncate(top_n);
        }

        for (file_path, file_stats) in files {
            println!(
                "  {}: {} lines ({} code)",
                file_path, file_stats.total_lines, file_stats.code_lines
            );
        }
    }

    Ok(())
}

/// Print the breakdown, one section per category, biggest language first.
///
/// The previous rendering was a single flat list of filename extensions ordered
/// by ascending file count. On any real repository that put the one-off
/// extensions at the top and the languages the project is written in at the
/// bottom, with prose and test fixtures -- the two most numerous file kinds in
/// a monorepo -- interleaved among them. Splitting by category and leading with
/// the largest answers the question the reader actually asked.
fn print_language_breakdown(
    breakdown: &Breakdown,
    sort_by: SortBy,
    ascending: bool,
    verbose: bool,
    config: &Config,
    use_color: bool,
) {
    let project_lines = breakdown.total_lines();
    let name_width = breakdown
        .rows
        .iter()
        .map(|row| row.language.chars().count())
        .max()
        .unwrap_or(0)
        .clamp(12, 28);

    for category in Category::ALL {
        let category_lines = breakdown.lines_in(category);
        if category_lines == 0 && breakdown.in_category(category).next().is_none() {
            continue;
        }

        let mut rows: Vec<&LanguageRow> = breakdown.in_category(category).collect();
        sort_rows(&mut rows, sort_by);
        if ascending {
            rows.reverse();
        }
        if let Some(top_n) = config.top_n {
            rows.truncate(top_n);
        }

        println!();
        println!(
            "=== {} === {} lines in {} files, {:.1}% of the project",
            category.label(),
            format_number(category_lines, use_color),
            format_number(breakdown.files_in(category), use_color),
            percentage(category_lines, project_lines),
        );

        for row in rows {
            // Code lines are shown next to the total on every row, because the
            // total is what the percentage is a share of but the code figure is
            // what `--sort code` orders by; printing only one of them made the
            // ordering look wrong.
            print!(
                "  {:<name_width$} {:>7} files {:>13} lines  {:>5.1}%  {:>13} code",
                row.language,
                group_digits(row.file_count),
                group_digits(row.total_lines),
                row.share_of(category_lines),
                group_digits(row.code_lines),
            );
            if verbose {
                print!(
                    "   ({} docs, {} comments, {} blank)",
                    group_digits(row.doc_lines),
                    group_digits(row.comment_lines),
                    group_digits(row.blank_lines),
                );
            }
            println!();
        }
    }
}

/// Order `rows` largest-first under the requested metric.
fn sort_rows(rows: &mut [&LanguageRow], sort_by: SortBy) {
    // Name first, so that rows tied on the metric resolve identically on every
    // run rather than inheriting whatever order they were collected in.
    rows.sort_by(|a, b| a.language.cmp(&b.language));

    let doc_ratio = |row: &LanguageRow| {
        if row.total_lines > 0 {
            row.doc_lines as f64 / row.total_lines as f64
        } else {
            0.0
        }
    };

    match sort_by {
        SortBy::Files | SortBy::Functions => rows.sort_by_key(|r| Reverse(r.file_count)),
        SortBy::Lines | SortBy::Complexity | SortBy::Quality => {
            rows.sort_by_key(|r| Reverse(r.total_lines))
        }
        SortBy::Code => rows.sort_by_key(|r| Reverse(r.code_lines)),
        SortBy::Comments => rows.sort_by_key(|r| Reverse(r.comment_lines)),
        SortBy::Size => rows.sort_by_key(|r| Reverse(r.total_size)),
        SortBy::DocRatio => rows.sort_by(|a, b| {
            doc_ratio(b)
                .partial_cmp(&doc_ratio(a))
                .unwrap_or(std::cmp::Ordering::Equal)
        }),
    }
}

/// `part` as a percentage of `whole`, and zero rather than NaN when empty.
fn percentage(part: usize, whole: usize) -> f64 {
    if whole == 0 {
        0.0
    } else {
        100.0 * part as f64 / whole as f64
    }
}

/// Report the files the run deliberately left out.
///
/// Without this the totals cannot be reconciled with any other tool: on one
/// monorepo the generated protobuf bindings alone were 1.15 million lines, and
/// they simply were not in the output and were not mentioned.
fn print_skipped(report: &AnalysisReport, use_color: bool) {
    let total: usize = report.skipped.values().sum::<usize>() + report.files_unrecognized;
    if total == 0 {
        return;
    }

    println!();
    println!(
        "=== Not counted === {} files",
        format_number(total, use_color)
    );
    for reason in SkipReason::ALL {
        if let Some(count) = report.skipped.get(&reason).filter(|c| **c > 0) {
            println!("  {:<24} {:>7}", reason.label(), count);
        }
    }
    if report.files_unrecognized > 0 {
        println!(
            "  {:<24} {:>7}",
            "unrecognized format", report.files_unrecognized
        );
    }
}

fn print_summary_only(aggregated_stats: &AggregatedStats, config: &Config) {
    println!(
        "Summary: {} files, {} lines ({} code, {} comments)",
        aggregated_stats.basic.total_files,
        aggregated_stats.basic.total_lines,
        aggregated_stats.basic.code_lines,
        aggregated_stats.basic.comment_lines
    );

    if config.show_quality {
        println!(
            "Quality: {:.1}/100",
            aggregated_stats
                .ratios
                .quality_metrics
                .overall_quality_score
        );
    }
}

fn print_compact_output(aggregated_stats: &AggregatedStats, config: &Config) {
    println!(
        "{} files | {} lines | {} code | {} comments",
        aggregated_stats.basic.total_files,
        aggregated_stats.basic.total_lines,
        aggregated_stats.basic.code_lines,
        aggregated_stats.basic.comment_lines
    );

    if config.show_quality {
        println!(
            "Quality: {:.1}/100",
            aggregated_stats
                .ratios
                .quality_metrics
                .overall_quality_score
        );
    }
}

fn format_number(num: usize, use_color: bool) -> String {
    let grouped = group_digits(num);
    if use_color && num > 1000 {
        format!("\x1b[36m{grouped}\x1b[0m")
    } else {
        grouped
    }
}

/// Render `num` with thousands separators.
///
/// A report whose headline figure is `2202324` makes the reader count digits to
/// find out whether it says two million or two hundred thousand.
fn group_digits(num: usize) -> String {
    let digits = num.to_string();
    let mut out = String::with_capacity(digits.len() + digits.len() / 3);
    for (index, digit) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index).is_multiple_of(3) {
            out.push(',');
        }
        out.push(digit);
    }
    out
}

fn output_json(aggregated_stats: &AggregatedStats) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(aggregated_stats)?);
    Ok(())
}

fn output_csv(aggregated_stats: &AggregatedStats) -> Result<()> {
    println!(
        "Extension,Files,Total Lines,Code Lines,Comment Lines,Doc Lines,Blank Lines,Size (bytes)"
    );

    // Sorted so that the same project always produces the same CSV.
    let mut rows: Vec<_> = aggregated_stats.basic.stats_by_extension.iter().collect();
    rows.sort_by_key(|(a, _)| *a);

    for (ext, ext_stats) in rows {
        println!(
            "{},{},{},{},{},{},{},{}",
            ext,
            ext_stats.file_count,
            ext_stats.total_lines,
            ext_stats.code_lines,
            ext_stats.comment_lines,
            ext_stats.doc_lines,
            ext_stats.blank_lines,
            ext_stats.total_size
        );
    }

    Ok(())
}

fn output_html(
    aggregated_stats: &AggregatedStats,
    individual_files: &[(String, FileStats)],
    config: &Config,
) -> Result<()> {
    use howmany::ui::html::HtmlReporter;

    let output_path = config.report_path("howmany-report.html");
    HtmlReporter::new().generate_comprehensive_report(
        aggregated_stats,
        individual_files,
        &output_path,
    )?;
    println!("HTML report generated: {}", output_path.display());

    Ok(())
}

fn output_sarif(
    aggregated_stats: &AggregatedStats,
    individual_files: &[(String, FileStats)],
    config: &Config,
) -> Result<()> {
    use howmany::ui::sarif::SarifReporter;

    let output_path = config.report_path("howmany-report.sarif");
    SarifReporter::new().generate_comprehensive_report(
        aggregated_stats,
        individual_files,
        &output_path,
    )?;
    println!("SARIF report generated: {}", output_path.display());

    Ok(())
}

/// Simple CLI output showing just basic file and line counts.
fn simple_cli_output(path: &Path, config: &Config, filter_options: FilterOptions) -> Result<()> {
    let needs_enhanced_output =
        filter_options.show_complexity || filter_options.show_quality || filter_options.show_ratios;

    if needs_enhanced_output {
        let analysis = analyze(path, config, true)?;
        let mut aggregated_stats = analysis.stats;

        if !filter_options.include_languages.is_empty()
            || !filter_options.exclude_languages.is_empty()
            || filter_options.min_lines.is_some()
            || filter_options.max_lines.is_some()
            || filter_options.min_size_bytes.is_some()
            || filter_options.max_size_bytes.is_some()
        {
            use howmany::ui::filters::ProjectFilter;
            let project_filter = ProjectFilter::new(filter_options.clone());
            let filtered_extensions =
                project_filter.filter_extensions(&aggregated_stats.basic.stats_by_extension);

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

            aggregated_stats.basic.total_files = total_files;
            aggregated_stats.basic.total_lines = total_lines;
            aggregated_stats.basic.code_lines = total_code_lines;
            aggregated_stats.basic.comment_lines = total_comment_lines;
            aggregated_stats.basic.blank_lines = total_blank_lines;
            aggregated_stats.basic.total_size = total_size;
            aggregated_stats.basic.doc_lines = total_doc_lines;
            aggregated_stats.basic.stats_by_extension = filtered_extensions;

            if total_lines > 0 {
                aggregated_stats.ratios.code_ratio = total_code_lines as f64 / total_lines as f64;
                aggregated_stats.ratios.comment_ratio =
                    total_comment_lines as f64 / total_lines as f64;
                aggregated_stats.ratios.doc_ratio = total_doc_lines as f64 / total_lines as f64;
                aggregated_stats.ratios.blank_ratio = total_blank_lines as f64 / total_lines as f64;
            }
        }

        println!(
            "{}",
            FilteredOutputFormatter::format_enhanced_cli_output(&aggregated_stats, &filter_options)
        );
        return Ok(());
    }

    // Per-file records are only retained when a filter needs them; otherwise the
    // aggregate the engine already computed is the answer. Complexity is parsed
    // only when a complexity filter is actually in play.
    let per_file = FileStatsFilter::needs_per_file_stats(&filter_options);
    let needs_complexity = FileStatsFilter::needs_complexity(&filter_options);
    let options = AnalysisOptions {
        compute_complexity: needs_complexity,
        ..config.analysis_options(per_file)
    };
    let analysis = Engine::new().analyze(path, &options)?;

    let (file_count, total_lines, total_size) = if per_file {
        let file_filter = FileStatsFilter::new(filter_options.clone());
        let complexity = if needs_complexity {
            FileComplexity::index(&analysis.stats.complexity.function_complexity_details)
        } else {
            Default::default()
        };
        analysis
            .individual_files
            .iter()
            .filter(|(path, stats)| {
                file_filter.passes_filter(path, stats)
                    && (!needs_complexity
                        || file_filter.passes_complexity_filter(complexity.get(path.as_str())))
            })
            .fold((0, 0, 0u64), |(files, lines, size), (_, stats)| {
                (files + 1, lines + stats.total_lines, size + stats.file_size)
            })
    } else {
        (
            analysis.basic.total_files,
            analysis.basic.total_lines,
            analysis.basic.total_size,
        )
    };

    if filter_options.show_size_info {
        let size_mb = total_size as f64 / (1024.0 * 1024.0);
        println!(
            "{} files, {} lines, {:.1} MB",
            file_count, total_lines, size_mb
        );
    } else {
        println!("{} files, {} lines", file_count, total_lines);
    }

    Ok(())
}

/// Quiet mode output - minimal information only.
fn quiet_output(path: &Path, config: &Config) -> Result<()> {
    let options = AnalysisOptions {
        compute_complexity: false,
        collect_individual_files: false,
        ..config.analysis_options(false)
    };
    let analysis = Engine::new().analyze(path, &options)?;

    println!(
        "{} files, {} lines",
        analysis.basic.total_files, analysis.basic.total_lines
    );

    Ok(())
}
