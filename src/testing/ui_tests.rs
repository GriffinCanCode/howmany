//! Everything the tool hands to something else.
//!
//! Reports are the product's real interface: a CI job parses the SARIF, a
//! browser renders the HTML, a script pipes the JSON into `jq`. A report that is
//! subtly malformed fails in someone else's system, far from here, so these
//! tests parse what is written rather than checking that a file appeared.

use crate::core::engine::{AnalysisOptions, DetectionMode, Engine};
use crate::core::stats::AggregatedStats;
use crate::core::types::{CodeStats, FileStats};
use crate::testing::test_utils::TestProject;
use crate::ui::filters::{FileFilter as OutputFilter, FilterOptions, FilterParser};
use crate::ui::html::HtmlReporter;
use crate::ui::sarif::SarifReporter;

struct Fixture {
    _project: TestProject,
    stats: AggregatedStats,
    basic: CodeStats,
    files: Vec<(String, FileStats)>,
}

fn fixture(name: &str) -> Fixture {
    let project = TestProject::new(name).unwrap();
    for i in 0..6 {
        project
            .create_rust_file(&format!("src/m{i}.rs"), i % 3 + 1, 1)
            .unwrap();
        project
            .create_python_file(&format!("py/m{i}.py"), i % 2 + 1)
            .unwrap();
    }
    project
        .create_file("README.md", "# Title\n\nProse.\n")
        .unwrap();

    let analysis = Engine::new()
        .analyze(
            project.path(),
            &AnalysisOptions {
                detection: DetectionMode::Disabled,
                use_cache: false,
                collect_individual_files: true,
                ..AnalysisOptions::default()
            },
        )
        .unwrap();

    Fixture {
        _project: project,
        stats: analysis.stats,
        basic: analysis.basic,
        files: analysis.individual_files,
    }
}

#[test]
fn the_html_report_is_a_complete_document_carrying_the_totals() {
    let f = fixture("ui_html");
    let out = tempfile::tempdir().unwrap();
    let path = out.path().join("report.html");

    HtmlReporter::new()
        .generate_comprehensive_report(&f.stats, &f.files, &path)
        .unwrap();

    let html = std::fs::read_to_string(&path).unwrap();
    assert!(html.starts_with("<!DOCTYPE html>") || html.starts_with("<!doctype html>"));
    assert!(html.contains("</html>"), "the document is truncated");
    assert_eq!(
        html.matches("<html").count(),
        1,
        "a template was emitted twice"
    );
    assert!(
        html.contains(&f.basic.total_files.to_string()),
        "the report does not mention how many files were analyzed"
    );
}

/// A path or language name containing `<` must not become markup.
#[test]
fn the_html_report_escapes_values_that_could_close_a_tag() {
    let out = tempfile::tempdir().unwrap();
    let path = out.path().join("report.html");
    let hostile = "src/<script>alert('x')</script>.rs".to_string();
    let files = vec![(
        hostile,
        FileStats {
            total_lines: 3,
            code_lines: 3,
            file_size: 12,
            ..FileStats::default()
        },
    )];
    let stats = CodeStats {
        total_files: 1,
        total_lines: 3,
        total_code_lines: 3,
        total_size: 12,
        ..CodeStats::default()
    };

    HtmlReporter::new()
        .generate_report(&stats, &files, &path)
        .unwrap();

    let html = std::fs::read_to_string(&path).unwrap();
    assert!(
        !html.contains("<script>alert('x')</script>"),
        "a file path was injected into the document as live markup"
    );
}

#[test]
fn the_sarif_report_parses_and_declares_its_schema() {
    let f = fixture("ui_sarif");
    let out = tempfile::tempdir().unwrap();
    let path = out.path().join("report.sarif");

    let reporter = SarifReporter::new();
    reporter
        .generate_comprehensive_report(&f.stats, &f.files, &path)
        .unwrap();

    let text = std::fs::read_to_string(&path).unwrap();
    let json: serde_json::Value = serde_json::from_str(&text).expect("SARIF must be valid JSON");

    assert_eq!(json["version"], "2.1.0");
    assert!(
        json["$schema"]
            .as_str()
            .is_some_and(|s| s.contains("sarif")),
        "consumers locate the schema by URI; it must be present"
    );
    let runs = json["runs"].as_array().expect("runs must be an array");
    assert_eq!(runs.len(), 1);
    assert!(runs[0]["tool"]["driver"]["name"].as_str().is_some());

    reporter
        .validate_sarif_output(&text)
        .expect("the tool's own validator must accept its own output");
}

#[test]
fn sarif_rejects_content_that_is_not_sarif() {
    let reporter = SarifReporter::new();
    assert!(reporter.validate_sarif_output("not json at all").is_err());
    assert!(reporter.validate_sarif_output("{}").is_err());
}

#[test]
fn an_empty_project_still_produces_valid_reports() {
    let project = TestProject::new("ui_empty").unwrap();
    let analysis = Engine::new()
        .analyze(
            project.path(),
            &AnalysisOptions {
                detection: DetectionMode::Disabled,
                use_cache: false,
                collect_individual_files: true,
                ..AnalysisOptions::default()
            },
        )
        .unwrap();

    let out = tempfile::tempdir().unwrap();
    let html = out.path().join("empty.html");
    let sarif = out.path().join("empty.sarif");

    HtmlReporter::new()
        .generate_comprehensive_report(&analysis.stats, &[], &html)
        .unwrap();
    SarifReporter::new()
        .generate_comprehensive_report(&analysis.stats, &[], &sarif)
        .unwrap();

    assert!(std::fs::read_to_string(&html).unwrap().contains("</html>"));
    let value: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&sarif).unwrap()).unwrap();
    assert_eq!(value["version"], "2.1.0");
}

#[test]
fn writing_a_report_to_an_unwritable_path_is_an_error_not_a_panic() {
    let f = fixture("ui_unwritable");
    let missing = std::path::Path::new("/definitely/not/here/report.html");
    assert!(HtmlReporter::new()
        .generate_comprehensive_report(&f.stats, &f.files, missing)
        .is_err());
}

#[test]
fn auto_report_selection_needs_some_statistics() {
    let f = fixture("ui_auto");
    let out = tempfile::tempdir().unwrap();

    assert!(HtmlReporter::new()
        .generate_auto_report(None, Some(&f.stats), &f.files, &out.path().join("a.html"))
        .is_ok());
    assert!(HtmlReporter::new()
        .generate_auto_report(Some(&f.basic), None, &f.files, &out.path().join("b.html"))
        .is_ok());
    assert!(
        HtmlReporter::new()
            .generate_auto_report(None, None, &f.files, &out.path().join("c.html"))
            .is_err(),
        "with nothing to report, an empty file is worse than an error"
    );
}

#[test]
fn json_output_of_a_run_is_parseable_and_complete() {
    let f = fixture("ui_json");
    let json = serde_json::to_value(&f.stats).unwrap();

    assert_eq!(
        json["basic"]["total_files"].as_u64().unwrap() as usize,
        f.basic.total_files
    );
    assert!(json["complexity"]["function_count"].as_u64().is_some());
    assert!(json["metadata"]["version"].as_str().is_some());
}

#[test]
fn size_filters_accept_the_units_people_type() {
    let cases = [
        ("1024", Some(1024)),
        ("1kb", Some(1024)),
        ("1KB", Some(1024)),
        ("2mb", Some(2 * 1024 * 1024)),
        ("1gb", Some(1024 * 1024 * 1024)),
        ("", None),
        ("banana", None),
        ("-5", None),
    ];
    for (input, expected) in cases {
        assert_eq!(FilterParser::parse_size(input), expected, "input {input:?}");
    }
}

#[test]
fn language_filters_split_and_normalise_their_input() {
    let parsed = FilterParser::parse_languages("rust, Python ,,JS");
    assert_eq!(parsed, vec!["rust", "python", "js"]);
    assert!(FilterParser::parse_languages("").is_empty());
}

#[test]
fn per_file_filters_select_by_size_and_line_count() {
    let small = FileStats {
        total_lines: 10,
        code_lines: 8,
        file_size: 100,
        ..FileStats::default()
    };
    let large = FileStats {
        total_lines: 5_000,
        code_lines: 4_000,
        file_size: 200_000,
        ..FileStats::default()
    };

    let by_lines = OutputFilter::new(FilterOptions {
        min_lines: Some(100),
        ..FilterOptions::default()
    });
    assert!(!by_lines.passes_filter("small.rs", &small));
    assert!(by_lines.passes_filter("large.rs", &large));

    let by_size = OutputFilter::new(FilterOptions {
        max_size_bytes: Some(1_000),
        ..FilterOptions::default()
    });
    assert!(by_size.passes_filter("small.rs", &small));
    assert!(!by_size.passes_filter("large.rs", &large));
}

#[test]
fn language_filters_include_and_exclude_by_extension() {
    let stats = FileStats {
        total_lines: 10,
        code_lines: 10,
        file_size: 100,
        ..FileStats::default()
    };

    let only_rust = OutputFilter::new(FilterOptions {
        include_languages: vec!["rs".to_string()],
        ..FilterOptions::default()
    });
    assert!(only_rust.passes_filter("src/main.rs", &stats));
    assert!(!only_rust.passes_filter("src/main.py", &stats));

    let no_python = OutputFilter::new(FilterOptions {
        exclude_languages: vec!["py".to_string()],
        ..FilterOptions::default()
    });
    assert!(no_python.passes_filter("src/main.rs", &stats));
    assert!(!no_python.passes_filter("src/main.py", &stats));
}

/// Retaining one record per file is pure overhead when nothing can reject a
/// file, and on a large tree that overhead is the whole cost of `--cli`.
#[test]
fn per_file_statistics_are_only_collected_when_a_filter_needs_them() {
    assert!(!OutputFilter::needs_per_file_stats(
        &FilterOptions::default()
    ));

    let needing = [
        FilterOptions {
            min_lines: Some(1),
            ..FilterOptions::default()
        },
        FilterOptions {
            max_lines: Some(1),
            ..FilterOptions::default()
        },
        FilterOptions {
            min_size_bytes: Some(1),
            ..FilterOptions::default()
        },
        FilterOptions {
            max_size_bytes: Some(1),
            ..FilterOptions::default()
        },
        FilterOptions {
            include_languages: vec!["rs".to_string()],
            ..FilterOptions::default()
        },
        FilterOptions {
            exclude_languages: vec!["rs".to_string()],
            ..FilterOptions::default()
        },
    ];
    for options in needing {
        assert!(
            OutputFilter::needs_per_file_stats(&options),
            "a per-file filter was not detected: {options:?}"
        );
    }

    // Display-only switches change formatting, not the set of files.
    assert!(!OutputFilter::needs_per_file_stats(&FilterOptions {
        show_complexity: true,
        show_quality: true,
        compact_output: true,
        ..FilterOptions::default()
    }));
}
