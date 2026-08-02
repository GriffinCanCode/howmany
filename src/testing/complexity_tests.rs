//! Complexity, structure and quality metrics.
//!
//! This is the half of the report that is not a line count, and it had no tests
//! at all -- which is how it shipped emitting a section of zeros with a
//! maintainability index of 100 for every JSON, HTML and SARIF report without
//! anything noticing.
//!
//! The metrics are heuristic, so these tests do not pin exact numbers that any
//! tuning would break. They assert the properties a consumer actually relies
//! on: that the analysers find what is plainly there, that scores stay inside
//! their documented ranges, that every number is finite, that more complex code
//! never scores as simpler, and that a file which cannot be read degrades to
//! zero instead of failing the run.

use crate::core::stats::complexity::analyzer::CodeAnalyzer;
use crate::core::stats::complexity::calculator::ComplexityCalculator;
use crate::core::types::{CodeStats, FileStats};
use crate::testing::test_utils::TestProject;

/// Line counts consistent with `source`, so quality maths has sane inputs.
fn stats_for(source: &str) -> FileStats {
    let total = source.lines().count();
    let blank = source.lines().filter(|l| l.trim().is_empty()).count();
    let comment = source
        .lines()
        .filter(|l| {
            let t = l.trim();
            t.starts_with("//") || t.starts_with('#')
        })
        .count();
    FileStats {
        total_lines: total,
        code_lines: total - blank - comment,
        comment_lines: comment,
        blank_lines: blank,
        doc_lines: 0,
        file_size: source.len() as u64,
    }
}

const BRANCHY_RUST: &str = r#"
pub fn classify(value: i32, mode: &str) -> i32 {
    if value > 0 && value < 10 {
        for index in 0..value {
            if index % 2 == 0 {
                return index;
            } else if index % 3 == 0 {
                continue;
            }
        }
    } else if mode == "wide" {
        while value > 0 {
            match value {
                1 => return 1,
                2 => return 2,
                _ => break,
            }
        }
    }
    0
}

pub fn trivial() -> i32 {
    1
}
"#;

const STRUCTURED_RUST: &str = r#"
pub struct Config {
    pub name: String,
    pub retries: usize,
}

pub enum Mode {
    Fast,
    Slow,
}

pub trait Runner {
    fn run(&self) -> bool;
}

impl Runner for Config {
    fn run(&self) -> bool {
        self.retries > 0
    }
}
"#;

#[cfg(test)]
mod analyzer {
    use super::*;

    /// The analyser must find the functions that are plainly in the file.
    #[test]
    fn functions_in_a_rust_file_are_found() {
        let project = TestProject::new("complexity_functions").unwrap();
        let path = project.create_file("src/lib.rs", BRANCHY_RUST).unwrap();

        let analysis = CodeAnalyzer::new()
            .analyze_file(&path.to_string_lossy())
            .unwrap();

        let names: Vec<&str> = analysis.functions.iter().map(|f| f.name.as_str()).collect();
        assert!(
            names.contains(&"classify") && names.contains(&"trivial"),
            "expected both functions, found {names:?}"
        );
    }

    /// A branchy function must not score as simple as a straight-line one. The
    /// absolute numbers are heuristic; the ordering is the contract.
    #[test]
    fn branching_raises_complexity_above_straight_line_code() {
        let project = TestProject::new("complexity_ordering").unwrap();
        let path = project.create_file("src/lib.rs", BRANCHY_RUST).unwrap();

        let analysis = CodeAnalyzer::new()
            .analyze_file(&path.to_string_lossy())
            .unwrap();
        let find = |name: &str| {
            analysis
                .functions
                .iter()
                .find(|f| f.name == name)
                .unwrap_or_else(|| panic!("{name} was not found"))
                .clone()
        };

        let branchy = find("classify");
        let trivial = find("trivial");

        assert!(
            branchy.cyclomatic_complexity > trivial.cyclomatic_complexity,
            "a function with six branches scored {} against {} for `1`",
            branchy.cyclomatic_complexity,
            trivial.cyclomatic_complexity
        );
        assert!(
            branchy.nesting_depth > trivial.nesting_depth,
            "three levels of nesting scored no deeper than none"
        );
        assert!(
            trivial.cyclomatic_complexity >= 1,
            "every function has at least one path"
        );
    }

    #[test]
    fn structures_are_found_and_typed() {
        let project = TestProject::new("complexity_structures").unwrap();
        let path = project.create_file("src/lib.rs", STRUCTURED_RUST).unwrap();

        let analysis = CodeAnalyzer::new()
            .analyze_file(&path.to_string_lossy())
            .unwrap();
        let names: Vec<&str> = analysis
            .structures
            .iter()
            .map(|s| s.name.as_str())
            .collect();

        assert!(
            names.contains(&"Config"),
            "the struct was not found: {names:?}"
        );
        assert!(
            analysis.structures.len() >= 2,
            "expected several structures, found {names:?}"
        );
    }

    /// A language with no analyser is not an error, and costs no file read.
    #[test]
    fn an_unsupported_language_yields_an_empty_analysis() {
        let project = TestProject::new("complexity_unsupported").unwrap();
        let path = project.create_file("data.qqq", "nothing here\n").unwrap();

        let analysis = CodeAnalyzer::new()
            .analyze_file(&path.to_string_lossy())
            .unwrap();
        assert!(analysis.functions.is_empty());
        assert!(analysis.structures.is_empty());
    }

    /// Reading the file again can fail -- it may have been moved since it was
    /// counted -- and that must not fail the whole run.
    #[test]
    fn a_missing_file_is_an_error_not_a_panic() {
        let analysis = CodeAnalyzer::new().analyze_file("/definitely/not/here.rs");
        assert!(analysis.is_err(), "a missing file should report an error");
    }

    /// Two analyses of the same file must agree, so a report does not depend on
    /// which worker happened to run it.
    #[test]
    fn analysis_is_deterministic() {
        let project = TestProject::new("complexity_determinism").unwrap();
        let path = project.create_file("src/lib.rs", BRANCHY_RUST).unwrap();
        let analyzer = CodeAnalyzer::new();
        let name = path.to_string_lossy().to_string();

        let first = analyzer.analyze_file(&name).unwrap();
        for _ in 0..3 {
            let again = analyzer.analyze_file(&name).unwrap();
            assert_eq!(first.functions.len(), again.functions.len());
            assert_eq!(first.structures.len(), again.structures.len());
            for (a, b) in first.functions.iter().zip(&again.functions) {
                assert_eq!(a.name, b.name);
                assert_eq!(a.cyclomatic_complexity, b.cyclomatic_complexity);
                assert_eq!(a.nesting_depth, b.nesting_depth);
            }
        }
    }
}

#[cfg(test)]
mod calculator {
    use super::*;

    /// Every number in the block must be finite and inside its documented
    /// range. A NaN reaches the report as `null` and a score above 100 reads as
    /// a bug in the tool.
    fn assert_well_formed(stats: &crate::core::stats::complexity::ComplexityStats) {
        for (label, value) in [
            ("cyclomatic_complexity", stats.cyclomatic_complexity),
            ("cognitive_complexity", stats.cognitive_complexity),
            ("average_function_length", stats.average_function_length),
            ("average_nesting_depth", stats.average_nesting_depth),
            ("methods_per_class", stats.methods_per_class),
            (
                "average_parameters_per_function",
                stats.average_parameters_per_function,
            ),
        ] {
            assert!(value.is_finite(), "{label} was {value}");
            assert!(value >= 0.0, "{label} was negative: {value}");
        }

        assert!(
            (0.0..=100.0).contains(&stats.maintainability_index),
            "maintainability index outside 0..=100: {}",
            stats.maintainability_index
        );
        assert!(
            stats.max_function_length >= stats.min_function_length
                || stats.max_function_length == 0,
            "max function length {} is below min {}",
            stats.max_function_length,
            stats.min_function_length
        );
    }

    #[test]
    fn a_single_file_produces_well_formed_metrics() {
        let project = TestProject::new("complexity_single").unwrap();
        let path = project.create_file("src/lib.rs", BRANCHY_RUST).unwrap();

        let stats = ComplexityCalculator::new()
            .calculate_complexity_stats(&stats_for(BRANCHY_RUST), &path.to_string_lossy())
            .unwrap();

        assert!(stats.function_count >= 2, "{stats:?}");
        assert!(stats.cyclomatic_complexity > 0.0);
        assert_well_formed(&stats);
    }

    /// The whole reason this module exists: a file the calculator cannot read
    /// contributes zeros rather than aborting the report.
    #[test]
    fn an_unreadable_file_degrades_to_zero_metrics() {
        let stats = ComplexityCalculator::new()
            .calculate_complexity_stats(&FileStats::default(), "/definitely/not/here.rs")
            .unwrap();

        assert_eq!(stats.function_count, 0);
        assert_eq!(stats.total_structures, 0);
        assert_well_formed(&stats);
    }

    #[test]
    fn a_project_with_no_files_produces_well_formed_zeros() {
        let stats = ComplexityCalculator::new()
            .calculate_project_complexity_stats(&CodeStats::default(), &[])
            .unwrap();

        assert_eq!(stats.function_count, 0);
        assert!(stats.complexity_by_extension.is_empty());
        assert_well_formed(&stats);
    }

    /// Project metrics must account for every file handed in, and the
    /// per-extension breakdown must not invent or drop languages.
    #[test]
    fn project_metrics_cover_every_file() {
        let project = TestProject::new("complexity_project").unwrap();
        let rust = project.create_file("src/lib.rs", BRANCHY_RUST).unwrap();
        let more = project
            .create_file("src/model.rs", STRUCTURED_RUST)
            .unwrap();

        let files = vec![
            (rust.to_string_lossy().to_string(), stats_for(BRANCHY_RUST)),
            (
                more.to_string_lossy().to_string(),
                stats_for(STRUCTURED_RUST),
            ),
        ];

        let stats = ComplexityCalculator::new()
            .calculate_project_complexity_stats(&CodeStats::default(), &files)
            .unwrap();

        assert!(stats.function_count >= 2, "{stats:?}");
        assert_eq!(
            stats.complexity_by_extension.keys().collect::<Vec<_>>(),
            vec!["rs"],
            "only Rust was analysed, so only Rust should appear"
        );
        assert_well_formed(&stats);
    }

    /// Repeated calculation over the same input must be identical, including the
    /// per-extension ordering that ends up in serialised reports.
    #[test]
    fn project_metrics_are_reproducible() {
        let project = TestProject::new("complexity_repro").unwrap();
        let path = project.create_file("src/lib.rs", BRANCHY_RUST).unwrap();
        let files = vec![(path.to_string_lossy().to_string(), stats_for(BRANCHY_RUST))];
        let calculator = ComplexityCalculator::new();

        let first = calculator
            .calculate_project_complexity_stats(&CodeStats::default(), &files)
            .unwrap();
        for _ in 0..3 {
            let again = calculator
                .calculate_project_complexity_stats(&CodeStats::default(), &files)
                .unwrap();
            assert_eq!(first.function_count, again.function_count);
            assert_eq!(first.cyclomatic_complexity, again.cyclomatic_complexity);
            assert_eq!(
                first.complexity_by_extension.keys().collect::<Vec<_>>(),
                again.complexity_by_extension.keys().collect::<Vec<_>>()
            );
        }
    }
}
