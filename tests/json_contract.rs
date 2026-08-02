//! The JSON output is a published interface, so it is pinned here.
//!
//! Two other products parse `-o json` and neither can fail this crate's build:
//! the VS Code extension (`howmany-vscode`) and the GitHub Action
//! (`howmany-actions`) each keep their own hand-written copy of the schema. That
//! arrangement has already broken once. A `time` section was removed from this
//! crate's output while `howmany-actions` still required
//! `time.total_time_minutes`, and because the repositories are separate, nothing
//! failed until somebody ran the action.
//!
//! Every field below is read by name in one of those consumers, with the reader
//! cited. Deleting or renaming one is a breaking change for a downstream product,
//! so it has to fail here -- in the repository where the change is made -- rather
//! than in a CI run somebody else owns.
//!
//! When a field genuinely has to go, the honest sequence is: update the
//! consumers, then update this list, then remove it. Editing this list alone just
//! restores the silence.

use std::path::Path;
use std::process::Command;

/// Field paths the downstream consumers read, and who reads each one.
///
/// `howmany-actions/src/utils/quality-gate.ts` decides whether a CI run passes
/// using the four `quality_metrics` entries; the rest are asserted by
/// `howmany-actions/test/test-integration.js` and typed in
/// `howmany-actions/src/types/howmany.ts` and
/// `howmany-vscode/src/types/HowManyTypes.ts`.
const CONSUMED_FIELDS: &[&str] = &[
    // Totals shown by both consumers.
    "basic.total_files",
    "basic.total_lines",
    "basic.code_lines",
    "basic.average_file_size",
    // Complexity summary.
    "complexity.function_count",
    // The quality gate's four inputs. Losing any of these silently turns a CI
    // gate into a no-op, which is worse than an error.
    "complexity.quality_metrics.code_health_score",
    "complexity.quality_metrics.maintainability_index",
    "complexity.quality_metrics.documentation_coverage",
    "complexity.quality_metrics.avg_complexity",
    "ratios.quality_metrics.overall_quality_score",
    // Consumers report which version produced a result.
    "metadata.version",
];

/// The top-level sections. A consumer that indexes a missing section throws
/// before it ever reaches a leaf, so these are worth naming separately.
const TOP_LEVEL_SECTIONS: &[&str] = &["basic", "complexity", "ratios", "metadata"];

fn analyze_to_json(dir: &Path) -> serde_json::Value {
    let output = Command::new(env!("CARGO_BIN_EXE_howmany"))
        .arg(dir)
        .args(["-o", "json", "--no-cache"])
        .env(
            "HOWMANY_CACHE_DIR",
            std::env::temp_dir().join("howmany-contract"),
        )
        .output()
        .expect("the binary should be runnable");

    assert!(
        output.status.success(),
        "analysis failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    serde_json::from_str(&String::from_utf8_lossy(&output.stdout))
        .expect("-o json must emit valid JSON")
}

/// A project with enough shape that every consumed metric has something to say:
/// a branching function, a documented function, a comment, and prose.
fn fixture() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    let write = |relative: &str, contents: &str| {
        let path = dir.path().join(relative);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, contents).unwrap();
    };

    write(
        "src/lib.rs",
        "//! Crate docs.\n\n\
         /// Doc comment.\n\
         pub fn classify(n: i32) -> &'static str {\n\
         \x20   // a comment\n\
         \x20   if n > 10 {\n\
         \x20       \"big\"\n\
         \x20   } else if n > 0 {\n\
         \x20       \"small\"\n\
         \x20   } else {\n\
         \x20       \"none\"\n\
         \x20   }\n\
         }\n",
    );
    write("src/main.rs", "fn main() {\n    println!(\"hi\");\n}\n");
    write("app/util.py", "def add(a, b):\n    return a + b\n");
    write("README.md", "# Title\n\nProse describing the project.\n");
    dir
}

/// Resolve a dotted path, reporting the first segment that is absent.
fn lookup<'a>(root: &'a serde_json::Value, path: &str) -> Result<&'a serde_json::Value, String> {
    let mut current = root;
    for (depth, key) in path.split('.').enumerate() {
        current = current.get(key).ok_or_else(|| {
            let prefix: Vec<&str> = path.split('.').take(depth).collect();
            let context = if prefix.is_empty() {
                "the top level".to_string()
            } else {
                prefix.join(".")
            };
            format!("`{path}` is missing: {context} has no `{key}`")
        })?;
    }
    Ok(current)
}

#[test]
fn every_top_level_section_a_consumer_indexes_is_present() {
    let project = fixture();
    let json = analyze_to_json(project.path());

    for section in TOP_LEVEL_SECTIONS {
        let value = lookup(&json, section).unwrap_or_else(|err| panic!("{err}"));
        assert!(
            value.is_object(),
            "`{section}` must be an object; consumers index into it"
        );
    }
}

#[test]
fn every_field_a_consumer_reads_is_present_and_numeric() {
    let project = fixture();
    let json = analyze_to_json(project.path());

    let mut missing = Vec::new();
    for path in CONSUMED_FIELDS {
        match lookup(&json, path) {
            Err(err) => missing.push(err),
            Ok(value) => {
                // `metadata.version` is the only string in the set; everything
                // else is arithmetic in a consumer, and a null or a string would
                // become NaN there rather than an error.
                if *path == "metadata.version" {
                    assert!(
                        value.as_str().is_some_and(|v| !v.is_empty()),
                        "`{path}` must be a non-empty string, got {value}"
                    );
                } else {
                    assert!(
                        value.as_f64().is_some_and(f64::is_finite),
                        "`{path}` must be a finite number, got {value}"
                    );
                }
            }
        }
    }

    assert!(
        missing.is_empty(),
        "the JSON output no longer carries fields that howmany-vscode and \
         howmany-actions read. Update those consumers before removing a field:\n  {}",
        missing.join("\n  ")
    );
}

/// The gate's inputs are scores, and a consumer compares them against a
/// user-supplied 0-100 threshold. A score outside that range makes every
/// threshold meaningless without ever looking wrong.
#[test]
fn quality_scores_stay_in_the_range_consumers_compare_against() {
    let project = fixture();
    let json = analyze_to_json(project.path());

    for path in [
        "complexity.quality_metrics.code_health_score",
        "complexity.quality_metrics.maintainability_index",
        "complexity.quality_metrics.documentation_coverage",
        "ratios.quality_metrics.overall_quality_score",
    ] {
        let score = lookup(&json, path)
            .unwrap_or_else(|err| panic!("{err}"))
            .as_f64()
            .unwrap_or_else(|| panic!("`{path}` must be a number"));
        assert!(
            (0.0..=100.0).contains(&score),
            "`{path}` is {score}, outside the 0-100 range thresholds assume"
        );
    }
}

/// An empty project is the case a consumer is most likely to hit on a first run,
/// and the one most likely to divide by zero. The contract still has to hold.
#[test]
fn the_contract_holds_for_a_project_with_nothing_in_it() {
    let empty = tempfile::tempdir().unwrap();
    let json = analyze_to_json(empty.path());

    for path in CONSUMED_FIELDS {
        let value = lookup(&json, path).unwrap_or_else(|err| {
            panic!("{err}\nan empty project must still produce the full schema")
        });
        if *path != "metadata.version" {
            assert!(
                value.as_f64().is_some_and(f64::is_finite),
                "`{path}` is {value} for an empty project; consumers do arithmetic \
                 on it, so NaN or null becomes a silently wrong verdict"
            );
        }
    }
}
