//! Derived statistics, checked as invariants.
//!
//! Ratios, quality scores and complexity figures are all functions of the same
//! line counts, and every one of them is a division that can produce `NaN`,
//! infinity, or a percentage above 100 when its denominator is zero. These tests
//! feed the pipeline real projects plus the degenerate inputs (no files, empty
//! files, comments only) and assert the properties every consumer relies on:
//! sums that partition, ratios inside `0..=1`, scores inside `0..=100`, and no
//! non-finite number anywhere in the report.

use crate::core::engine::{AnalysisOptions, DetectionMode, Engine};
use crate::core::stats::{AggregatedStats, StatsCalculator};
use crate::core::types::{CodeStats, FileStats};
use crate::testing::test_utils::TestProject;

fn options() -> AnalysisOptions {
    AnalysisOptions {
        detection: DetectionMode::Disabled,
        use_cache: false,
        collect_individual_files: true,
        compute_complexity: true,
        ..AnalysisOptions::default()
    }
}

fn analyze(project: &TestProject) -> (AggregatedStats, CodeStats) {
    let analysis = Engine::new().analyze(project.path(), &options()).unwrap();
    (analysis.stats, analysis.basic)
}

/// A project with several languages, sizes and comment densities.
fn mixed_project(name: &str) -> TestProject {
    let project = TestProject::new(name).unwrap();
    for i in 0..8 {
        project
            .create_rust_file(&format!("src/m{i}.rs"), i % 3 + 1, i % 2)
            .unwrap();
        project
            .create_python_file(&format!("py/m{i}.py"), i % 3 + 1)
            .unwrap();
    }
    project.create_typescript_file("web/app.ts", 3).unwrap();
    project.create_javascript_file("web/util.js", 2).unwrap();
    project.create_html_file("web/index.html").unwrap();
    project.create_css_file("web/style.css").unwrap();
    project
        .create_file("README.md", "# Title\n\nProse.\n")
        .unwrap();
    project
}

/// Every ratio and score in a report, with the range it must stay inside.
fn bounded_values(stats: &AggregatedStats) -> Vec<(&'static str, f64, f64, f64)> {
    let r = &stats.ratios;
    let q = &r.quality_metrics;
    let c = &stats.complexity;
    vec![
        ("code_ratio", r.code_ratio, 0.0, 1.0),
        ("comment_ratio", r.comment_ratio, 0.0, 1.0),
        ("doc_ratio", r.doc_ratio, 0.0, 1.0),
        ("blank_ratio", r.blank_ratio, 0.0, 1.0),
        ("documentation_score", q.documentation_score, 0.0, 100.0),
        ("maintainability_score", q.maintainability_score, 0.0, 100.0),
        ("readability_score", q.readability_score, 0.0, 100.0),
        ("consistency_score", q.consistency_score, 0.0, 100.0),
        ("overall_quality_score", q.overall_quality_score, 0.0, 100.0),
        (
            "code_health_score",
            c.quality_metrics.code_health_score,
            0.0,
            100.0,
        ),
        (
            "documentation_coverage",
            c.quality_metrics.documentation_coverage,
            0.0,
            100.0,
        ),
        ("maintainability_index", c.maintainability_index, 0.0, 100.0),
    ]
}

fn assert_all_bounded(stats: &AggregatedStats, context: &str) {
    for (name, value, low, high) in bounded_values(stats) {
        assert!(
            value.is_finite(),
            "{context}: {name} is not a finite number ({value})"
        );
        assert!(
            (low..=high).contains(&value),
            "{context}: {name} = {value} escapes {low}..={high}"
        );
    }
}

#[test]
fn line_categories_partition_the_total() {
    let project = mixed_project("stats_partition");
    let (stats, basic) = analyze(&project);

    assert!(basic.is_consistent());
    assert_eq!(
        stats.basic.code_lines
            + stats.basic.comment_lines
            + stats.basic.doc_lines
            + stats.basic.blank_lines,
        stats.basic.total_lines,
        "the four line categories must exactly partition the total"
    );
    assert_eq!(stats.basic.total_lines, basic.total_lines);
    assert_eq!(stats.basic.total_files, basic.total_files);
}

#[test]
fn ratios_and_scores_stay_inside_their_ranges() {
    let project = mixed_project("stats_bounds");
    let (stats, _) = analyze(&project);
    assert_all_bounded(&stats, "mixed project");

    let sum = stats.ratios.code_ratio
        + stats.ratios.comment_ratio
        + stats.ratios.doc_ratio
        + stats.ratios.blank_ratio;
    assert!(
        (sum - 1.0).abs() < 1e-9,
        "the four line ratios should sum to 1, got {sum}"
    );
}

/// Division by zero is the failure mode of every derived statistic, so each
/// degenerate shape gets its own check.
#[test]
fn degenerate_projects_produce_finite_statistics() {
    let cases: Vec<(&str, Box<dyn Fn(&TestProject)>)> = vec![
        ("empty project", Box::new(|_: &TestProject| {})),
        (
            "one empty file",
            Box::new(|p: &TestProject| {
                p.create_file("empty.rs", "").unwrap();
            }),
        ),
        (
            "only blank lines",
            Box::new(|p: &TestProject| {
                p.create_file("blank.rs", "\n\n\n\n").unwrap();
            }),
        ),
        (
            "only comments",
            Box::new(|p: &TestProject| {
                p.create_file("c.rs", "// a\n// b\n// c\n").unwrap();
            }),
        ),
        (
            "only documentation",
            Box::new(|p: &TestProject| {
                p.create_file("d.rs", "//! module\n/// item\n").unwrap();
            }),
        ),
        (
            "one code line, no newline",
            Box::new(|p: &TestProject| {
                p.create_file("one.rs", "fn f() {}").unwrap();
            }),
        ),
    ];

    for (name, populate) in cases {
        let project = TestProject::new("stats_degenerate").unwrap();
        populate(&project);
        let (stats, basic) = analyze(&project);

        assert_all_bounded(&stats, name);
        assert!(basic.is_consistent(), "{name}: totals are inconsistent");
        assert!(
            stats.complexity.cyclomatic_complexity.is_finite(),
            "{name}: cyclomatic complexity is not finite"
        );
        assert!(
            stats.complexity.average_function_length.is_finite(),
            "{name}: average function length is not finite"
        );
    }
}

#[test]
fn an_empty_project_reports_zeroes_not_errors() {
    let project = TestProject::new("stats_empty").unwrap();
    let (stats, basic) = analyze(&project);

    assert_eq!(basic.total_files, 0);
    assert_eq!(stats.basic.total_lines, 0);
    assert_eq!(stats.complexity.function_count, 0);
    assert_eq!(stats.ratios.code_ratio, 0.0);
    assert!(stats.ratios.language_distribution.is_empty());
}

#[test]
fn language_distributions_are_percentages_that_sum_to_a_hundred() {
    let project = mixed_project("stats_distribution");
    let (stats, _) = analyze(&project);

    for (label, distribution) in [
        ("lines", &stats.ratios.language_distribution),
        ("files", &stats.ratios.file_distribution),
        ("size", &stats.ratios.size_distribution),
    ] {
        assert!(!distribution.is_empty(), "{label} distribution is empty");
        let total: f64 = distribution.values().sum();
        assert!(
            (total - 100.0).abs() < 0.5,
            "{label} distribution sums to {total}, not 100"
        );
        for (language, share) in distribution {
            assert!(
                (0.0..=100.0).contains(share),
                "{label} share for {language} is {share}"
            );
        }
    }
}

/// The per-extension breakdown must agree with the project totals; a mismatch
/// means a file was counted in one place and not the other.
#[test]
fn per_extension_totals_agree_with_the_project_total() {
    let project = mixed_project("stats_by_extension");
    let (_, basic) = analyze(&project);

    let files: usize = basic.stats_by_extension.values().map(|(n, _)| n).sum();
    let lines: usize = basic
        .stats_by_extension
        .values()
        .map(|(_, s)| s.total_lines)
        .sum();
    let size: u64 = basic
        .stats_by_extension
        .values()
        .map(|(_, s)| s.file_size)
        .sum();

    assert_eq!(files, basic.total_files);
    assert_eq!(lines, basic.total_lines);
    assert_eq!(size, basic.total_size);
}

#[test]
fn structure_counts_add_up_to_the_total() {
    let project = mixed_project("stats_structures");
    let (stats, _) = analyze(&project);
    let c = &stats.complexity;

    assert!(c.function_count > 0, "a project of functions found none");
    assert_eq!(
        c.class_count + c.interface_count + c.trait_count + c.enum_count + c.struct_count,
        c.total_structures,
        "the structure breakdown does not add up to total_structures"
    );
    assert_eq!(c.structure_distribution.classes, c.class_count);
    assert_eq!(c.structure_distribution.traits, c.trait_count);
}

#[test]
fn function_length_extremes_bracket_the_average() {
    let project = mixed_project("stats_lengths");
    let (stats, _) = analyze(&project);
    let c = &stats.complexity;

    assert!(c.min_function_length <= c.max_function_length);
    assert!(
        c.average_function_length >= c.min_function_length as f64 - 1e-9
            && c.average_function_length <= c.max_function_length as f64 + 1e-9,
        "average function length {} is outside {}..={}",
        c.average_function_length,
        c.min_function_length,
        c.max_function_length
    );
}

/// More code with the same shape must not reduce the counts that scale with it.
#[test]
fn statistics_grow_monotonically_with_the_project() {
    let small = TestProject::new("stats_small").unwrap();
    for i in 0..4 {
        small
            .create_rust_file(&format!("src/m{i}.rs"), 3, 1)
            .unwrap();
    }
    let large = TestProject::new("stats_large").unwrap();
    for i in 0..16 {
        large
            .create_rust_file(&format!("src/m{i}.rs"), 3, 1)
            .unwrap();
    }

    let (small_stats, small_basic) = analyze(&small);
    let (large_stats, large_basic) = analyze(&large);

    assert!(large_basic.total_files > small_basic.total_files);
    assert!(large_basic.total_lines > small_basic.total_lines);
    assert!(large_stats.complexity.function_count > small_stats.complexity.function_count);
    assert!(large_stats.complexity.total_structures >= small_stats.complexity.total_structures);
}

/// Quality scores describe *shape*, not size, so a project duplicated file for
/// file should score the same.
#[test]
fn quality_scores_describe_shape_not_size() {
    let one = TestProject::new("stats_shape_one").unwrap();
    let many = TestProject::new("stats_shape_many").unwrap();
    for i in 0..1 {
        one.create_rust_file(&format!("src/m{i}.rs"), 4, 2).unwrap();
    }
    for i in 0..8 {
        many.create_rust_file(&format!("src/m{i}.rs"), 4, 2)
            .unwrap();
    }

    let (one_stats, _) = analyze(&one);
    let (many_stats, _) = analyze(&many);

    for (name, a, b) in [
        (
            "code_ratio",
            one_stats.ratios.code_ratio,
            many_stats.ratios.code_ratio,
        ),
        (
            "comment_ratio",
            one_stats.ratios.comment_ratio,
            many_stats.ratios.comment_ratio,
        ),
        (
            "doc_ratio",
            one_stats.ratios.doc_ratio,
            many_stats.ratios.doc_ratio,
        ),
    ] {
        assert!(
            (a - b).abs() < 1e-9,
            "{name} changed with project size: {a} vs {b}"
        );
    }
}

#[test]
fn single_file_statistics_match_the_file() {
    let calculator = StatsCalculator::new();
    let file = FileStats {
        total_lines: 100,
        code_lines: 60,
        comment_lines: 20,
        doc_lines: 10,
        blank_lines: 10,
        file_size: 2048,
    };

    let stats = calculator
        .calculate_file_stats(&file, "src/main.rs")
        .unwrap();

    assert_eq!(stats.basic.total_lines, 100);
    assert_eq!(stats.basic.code_lines, 60);
    assert!((stats.ratios.code_ratio - 0.6).abs() < 1e-9);
    assert!((stats.ratios.comment_ratio - 0.2).abs() < 1e-9);
    // Per-file ratios are reported to two decimal places.
    assert!((stats.ratios.comment_to_code_ratio - 0.33).abs() < 1e-9);
    assert_all_bounded(&stats, "single file");
}

/// Two decimals is the reported precision for a single file's ratios; pinning it
/// keeps a formatting change from silently altering the numbers.
#[test]
fn per_file_ratios_are_reported_to_two_decimals() {
    let stats = StatsCalculator::new()
        .calculate_file_stats(
            &FileStats {
                total_lines: 3,
                code_lines: 1,
                comment_lines: 1,
                doc_lines: 1,
                file_size: 30,
                ..FileStats::default()
            },
            "thirds.rs",
        )
        .unwrap();

    assert!((stats.ratios.code_ratio - 0.33).abs() < 1e-9);
    assert!((stats.ratios.comment_ratio - 0.33).abs() < 1e-9);
    assert!((stats.ratios.doc_ratio - 0.33).abs() < 1e-9);
}

/// A file with no lines, and a path that no longer exists, must still produce a
/// report: the line counts are already known and complexity simply has nothing
/// to find.
#[test]
fn a_zeroed_file_does_not_divide_by_zero() {
    let stats = StatsCalculator::new()
        .calculate_file_stats(&FileStats::default(), "/definitely/not/here.rs")
        .unwrap();

    assert_eq!(stats.basic.total_lines, 0);
    assert_eq!(stats.ratios.code_ratio, 0.0);
    assert_eq!(stats.ratios.comment_to_code_ratio, 0.0);
    assert_all_bounded(&stats, "zeroed file");
}

#[test]
fn metadata_records_what_was_analyzed() {
    let project = mixed_project("stats_metadata");
    let (stats, basic) = analyze(&project);

    assert_eq!(stats.metadata.file_count_analyzed, basic.total_files);
    assert_eq!(stats.metadata.total_bytes_analyzed, basic.total_size);
    assert!(
        !stats.metadata.version.is_empty(),
        "reports must name the version that produced them"
    );
    assert!(!stats.metadata.languages_detected.is_empty());
}

#[test]
fn statistics_serialise_and_survive_a_round_trip() {
    let project = mixed_project("stats_serde");
    let (stats, _) = analyze(&project);

    let json = serde_json::to_string(&stats).unwrap();
    let back: AggregatedStats = serde_json::from_str(&json).unwrap();

    assert_eq!(stats.basic.total_lines, back.basic.total_lines);
    assert_eq!(
        stats.complexity.function_count,
        back.complexity.function_count
    );
    assert!((stats.ratios.code_ratio - back.ratios.code_ratio).abs() < 1e-12);
}

/// `NaN` and `Infinity` are not valid JSON; serde would emit `null` and any
/// consumer parsing the report would then fail. Nothing may be non-finite.
#[test]
fn no_statistic_serialises_to_null() {
    let project = TestProject::new("stats_no_null").unwrap();
    project
        .create_file("only_comments.rs", "// a\n// b\n")
        .unwrap();
    project.create_file("empty.rs", "").unwrap();
    let (stats, _) = analyze(&project);

    let json = serde_json::to_string(&stats).unwrap();
    assert!(
        !json.contains("null"),
        "a division by zero leaked into the report as null: {json}"
    );
}

#[test]
fn aggregated_and_basic_views_of_the_same_run_agree() {
    let project = mixed_project("stats_views");
    let (stats, basic) = analyze(&project);
    let converted = StatsCalculator::new().to_code_stats(&stats);

    assert_eq!(converted.total_files, basic.total_files);
    assert_eq!(converted.total_lines, basic.total_lines);
    assert_eq!(converted.total_code_lines, basic.total_code_lines);
    assert_eq!(converted.total_size, basic.total_size);
}
