//! End-to-end runs with counts known in advance.
//!
//! Unit tests fix each stage's behaviour; these fix the *composition*. A tree is
//! built whose exact line breakdown is written down here by hand, and the whole
//! pipeline has to reproduce it. That makes any change in classification,
//! discovery or aggregation show up as a specific number moving, which is the
//! only way to tell a real improvement from a regression.

use crate::core::engine::{AnalysisOptions, DetectionMode, Engine, Parallelism};
use crate::testing::test_utils::TestProject;

fn options() -> AnalysisOptions {
    AnalysisOptions {
        collect_individual_files: true,
        ..AnalysisOptions::reproducible()
    }
}

/// A project whose every line is accounted for in [`GOLDEN`].
fn golden_project() -> TestProject {
    let project = TestProject::new("golden").unwrap();

    // 8 lines: 4 code, 2 comment, 1 doc, 1 blank.
    project
        .create_file(
            "src/main.rs",
            "//! Crate docs\n\
             // ordinary comment\n\
             use std::io;\n\
             \n\
             // another comment\n\
             fn main() {\n\
                 println!(\"hi\");\n\
             }\n",
        )
        .unwrap();

    // 6 lines: 3 code, 1 comment, 1 doc, 1 blank.
    project
        .create_file(
            "app/main.py",
            "\"\"\"Module docstring.\"\"\"\n\
             # a comment\n\
             import os\n\
             \n\
             def main():\n\
             \x20   pass\n",
        )
        .unwrap();

    // 5 lines: 2 code, 2 comment, 0 doc, 1 blank.
    project
        .create_file(
            "web/app.js",
            "// entry point\n\
             const x = 1;\n\
             \n\
             export default x;\n\
             // trailing\n",
        )
        .unwrap();

    // 4 lines: 3 doc, 1 blank.
    project
        .create_file("README.md", "# Title\n\nProse line.\nMore prose.\n")
        .unwrap();

    // Excluded: build output, a lock file, generated code and a binary.
    project
        .create_file("node_modules/dep/index.js", "module.exports = 1;\n")
        .unwrap();
    // `target/` is Cargo output because Cargo says so, in the tag file it
    // writes there; the name alone is not evidence.
    project
        .create_file("target/CACHEDIR.TAG", "Signature: 8a477f597d28d172\n")
        .unwrap();
    project
        .create_file("target/debug/gen.rs", "fn gen() {}\n")
        .unwrap();
    project.create_file("package-lock.json", "{}\n").unwrap();
    project
        .create_file("src/api.pb.go", "package api\n")
        .unwrap();
    project
        .create_file_binary("assets/logo.png", &[0x89, b'P', b'N', b'G'])
        .unwrap();
    project
        .create_file("LICENSE", "MIT License\n\nText.\n")
        .unwrap();

    project
}

/// Expected totals for [`golden_project`], derived by reading the fixtures
/// above rather than by running the tool.
struct Golden {
    files: usize,
    total: usize,
    code: usize,
    comment: usize,
    doc: usize,
    blank: usize,
}

const GOLDEN: Golden = Golden {
    files: 4,
    total: 8 + 6 + 5 + 4,
    code: 4 + 3 + 2,
    comment: 2 + 1 + 2,
    doc: (1 + 1) + 3,
    blank: 1 + 1 + 1 + 1,
};

#[test]
fn the_pipeline_reproduces_a_hand_counted_project() {
    let project = golden_project();
    let analysis = Engine::new().analyze(project.path(), &options()).unwrap();
    let basic = &analysis.basic;

    let breakdown: Vec<String> = analysis
        .individual_files
        .iter()
        .map(|(path, s)| {
            format!(
                "{path}: total={} code={} comment={} doc={} blank={}",
                s.total_lines, s.code_lines, s.comment_lines, s.doc_lines, s.blank_lines
            )
        })
        .collect();
    let context = breakdown.join("\n");

    assert_eq!(basic.total_files, GOLDEN.files, "\n{context}");
    assert_eq!(basic.total_lines, GOLDEN.total, "\n{context}");
    assert_eq!(basic.total_code_lines, GOLDEN.code, "\n{context}");
    assert_eq!(basic.total_comment_lines, GOLDEN.comment, "\n{context}");
    assert_eq!(basic.total_doc_lines, GOLDEN.doc, "\n{context}");
    assert_eq!(basic.total_blank_lines, GOLDEN.blank, "\n{context}");
    assert!(basic.is_consistent());
}

#[test]
fn the_golden_project_excludes_exactly_what_it_should() {
    let project = golden_project();
    let analysis = Engine::new().analyze(project.path(), &options()).unwrap();
    let counted: Vec<&str> = analysis
        .individual_files
        .iter()
        .map(|(p, _)| p.as_str())
        .collect();

    for wanted in ["main.rs", "main.py", "app.js", "README.md"] {
        assert!(
            counted.iter().any(|p| p.ends_with(wanted)),
            "{wanted} is missing from {counted:?}"
        );
    }
    for unwanted in [
        "node_modules",
        "/target/",
        "package-lock.json",
        "api.pb.go",
        ".png",
        "LICENSE",
    ] {
        assert!(
            !counted.iter().any(|p| p.contains(unwanted)),
            "{unwanted} was counted: {counted:?}"
        );
    }
}

#[test]
fn per_extension_breakdown_matches_the_files_on_disk() {
    let project = golden_project();
    let analysis = Engine::new().analyze(project.path(), &options()).unwrap();
    let by_ext = &analysis.basic.stats_by_extension;

    let mut extensions: Vec<&str> = by_ext.keys().map(String::as_str).collect();
    extensions.sort_unstable();
    assert_eq!(extensions, ["js", "md", "py", "rs"]);

    for (extension, expected_lines) in [("rs", 8), ("py", 6), ("js", 5), ("md", 4)] {
        let (count, stats) = &by_ext[extension];
        assert_eq!(*count, 1, "{extension}");
        assert_eq!(stats.total_lines, expected_lines, "{extension}");
    }
}

/// The same tree, analyzed every way the tool can be configured, must produce
/// the same counts. Options may change cost and detail, never the answer.
#[test]
fn every_configuration_agrees_on_the_totals() {
    let project = golden_project();
    let engine = Engine::new();
    let baseline = engine.analyze(project.path(), &options()).unwrap();

    let variants = [
        (
            "auto threads",
            AnalysisOptions {
                parallelism: Parallelism::Auto,
                ..options()
            },
        ),
        (
            "sixteen threads",
            AnalysisOptions {
                parallelism: Parallelism::Fixed(16),
                ..options()
            },
        ),
        (
            "detection enabled",
            AnalysisOptions {
                detection: DetectionMode::Auto,
                ..options()
            },
        ),
        (
            "no per-file stats",
            AnalysisOptions {
                collect_individual_files: false,
                compute_complexity: false,
                ..options()
            },
        ),
        (
            "no complexity",
            AnalysisOptions {
                compute_complexity: false,
                ..options()
            },
        ),
    ];

    for (name, variant) in variants {
        let other = engine.analyze(project.path(), &variant).unwrap();
        assert_eq!(
            baseline.basic.total_lines, other.basic.total_lines,
            "{name} changed the line total"
        );
        assert_eq!(
            baseline.basic.total_files, other.basic.total_files,
            "{name} changed the file count"
        );
        assert_eq!(
            baseline.basic.stats_by_extension, other.basic.stats_by_extension,
            "{name} changed the per-extension breakdown"
        );
    }
}

/// Listing and counting must agree: what `--list` prints is what gets counted.
#[test]
fn listing_and_counting_see_the_same_files() {
    let project = golden_project();
    let engine = Engine::new();

    let listed: Vec<String> = engine
        .discover_files(project.path(), &options())
        .unwrap()
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();
    let counted: Vec<String> = engine
        .analyze(project.path(), &options())
        .unwrap()
        .individual_files
        .into_iter()
        .map(|(p, _)| p)
        .collect();

    assert_eq!(listed, counted);
}

#[test]
fn a_comprehensive_project_is_analyzed_without_surprises() {
    let project = TestProject::new("integration_comprehensive").unwrap();
    project.create_comprehensive_project().unwrap();

    let analysis = Engine::new().analyze(project.path(), &options()).unwrap();

    assert!(analysis.report.files_counted > 0);
    assert_eq!(
        analysis.report.files_failed, 0,
        "{:?}",
        analysis.report.failures
    );
    assert!(analysis.basic.is_consistent());
    assert!(analysis.stats.complexity.function_count > 0);
    assert_eq!(
        analysis.report.files_counted + analysis.report.files_failed,
        analysis.report.files_discovered
    );
}

/// Re-running over an unchanged tree must produce identical output *and* be
/// served from cache the second time.
#[test]
fn a_second_run_is_cached_and_identical() {
    let project = golden_project();
    let cache_dir = tempfile::tempdir().unwrap();
    let previous = std::env::var_os("HOWMANY_CACHE_DIR");
    std::env::set_var("HOWMANY_CACHE_DIR", cache_dir.path());

    let engine = Engine::new();
    let cached_options = AnalysisOptions {
        use_cache: true,
        ..options()
    };
    let first = engine.analyze(project.path(), &cached_options).unwrap();
    let second = engine.analyze(project.path(), &cached_options).unwrap();

    match previous {
        Some(value) => std::env::set_var("HOWMANY_CACHE_DIR", value),
        None => std::env::remove_var("HOWMANY_CACHE_DIR"),
    }

    assert_eq!(first.basic, second.basic);
    assert_eq!(first.individual_files, second.individual_files);
    assert_eq!(second.report.cache_hits, second.report.files_counted);
}

/// Editing one file must change that file's numbers and nothing else.
#[test]
fn an_edit_is_reflected_in_the_next_run() {
    let project = golden_project();
    let engine = Engine::new();
    let before = engine.analyze(project.path(), &options()).unwrap();

    project
        .create_file("src/main.rs", "fn main() {}\n")
        .unwrap();
    let after = engine.analyze(project.path(), &options()).unwrap();

    assert_eq!(after.basic.total_files, before.basic.total_files);
    assert_eq!(
        after.basic.total_lines,
        before.basic.total_lines - 8 + 1,
        "the edit was not picked up exactly"
    );
}
