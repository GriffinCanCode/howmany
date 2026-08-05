//! The binary, exercised the way a user installs and runs it.
//!
//! Everything else in this repository tests the library. What ships is the
//! executable, and the failures a new user actually hits live in the last inch:
//! a flag that is accepted but does nothing, output that is not valid JSON, a
//! non-zero exit status on an empty directory, a report written somewhere
//! unexpected, or a run that depends on an optional binary being installed.
//!
//! Each test spawns the real executable in a temporary directory, so nothing
//! here depends on the machine it runs on -- including whether the external
//! `sherlock` detector exists.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// The binary Cargo just built for this test run.
fn howmany() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_howmany"));
    // Reports must never be cached across tests, and no test may depend on a
    // cache the developer's own runs left behind.
    command.env(
        "HOWMANY_CACHE_DIR",
        std::env::temp_dir().join("howmany-cli-tests"),
    );
    command
}

struct Project {
    dir: tempfile::TempDir,
}

impl Project {
    /// A small, fixed tree: 3 Rust files, 1 Python file, a README, and noise
    /// that must not be counted.
    fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let project = Self { dir };

        project.write("src/main.rs", "fn main() {\n    println!(\"hi\");\n}\n");
        project.write("src/lib.rs", "// lib\npub fn f() {}\n");
        project.write("src/util.rs", "pub fn g() -> i32 {\n    1\n}\n");
        project.write("app/main.py", "def main():\n    pass\n");
        project.write("README.md", "# Title\n\nProse.\n");
        project.write("node_modules/dep/index.js", "module.exports = 1;\n");
        // Cargo's own tag file, which is what makes `target/` build output
        // rather than a directory that happens to be called that.
        project.write("target/CACHEDIR.TAG", "Signature: 8a477f597d28d172\n");
        project.write("target/debug/gen.rs", "fn gen() {}\n");
        project.write("package-lock.json", "{}\n");
        project
    }

    fn write(&self, relative: &str, contents: &str) -> PathBuf {
        let path = self.dir.path().join(relative);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, contents).unwrap();
        path
    }

    fn path(&self) -> &Path {
        self.dir.path()
    }

    /// Run the binary against this project with `args`.
    fn run(&self, args: &[&str]) -> Output {
        howmany()
            .arg(self.path())
            .args(args)
            .current_dir(self.path())
            .output()
            .expect("the binary should be runnable")
    }
}

fn stdout_of(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).to_string()
}

fn assert_success(output: &Output, context: &str) {
    assert!(
        output.status.success(),
        "{context} exited with {:?}\nstdout: {}\nstderr: {}",
        output.status.code(),
        stdout_of(output),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn version_and_help_work_without_a_project() {
    for flag in ["--version", "--help"] {
        let output = howmany().arg(flag).output().unwrap();
        assert_success(&output, flag);
        assert!(
            !stdout_of(&output).trim().is_empty(),
            "{flag} printed nothing"
        );
    }
}

/// `--version` must report the version that was built, not a hardcoded string
/// that drifts from the manifest.
#[test]
fn version_matches_the_package() {
    let output = howmany().arg("--version").output().unwrap();
    let text = stdout_of(&output);
    assert!(
        text.contains(env!("CARGO_PKG_VERSION")),
        "--version printed {text:?}, expected {}",
        env!("CARGO_PKG_VERSION")
    );
}

#[test]
fn cli_mode_reports_files_and_lines() {
    let project = Project::new();
    let output = project.run(&["--cli", "--reproducible"]);
    assert_success(&output, "--cli");

    let text = stdout_of(&output);
    assert!(
        text.contains("5 files"),
        "expected the five source files, got {text:?}"
    );
    assert!(text.contains("lines"), "got {text:?}");
}

#[test]
fn json_output_is_parseable_and_carries_the_totals() {
    let project = Project::new();
    let output = project.run(&["-o", "json", "--reproducible"]);
    assert_success(&output, "-o json");

    let json: serde_json::Value =
        serde_json::from_str(&stdout_of(&output)).expect("output of -o json must be valid JSON");

    assert_eq!(json["basic"]["total_files"], 5);
    assert!(json["basic"]["total_lines"].as_u64().unwrap() > 0);
    assert!(json["metadata"]["version"].as_str().is_some());
}

/// Nothing may contaminate stdout when a machine-readable format is requested:
/// progress, warnings and diagnostics belong on stderr.
#[test]
fn json_output_contains_nothing_but_json() {
    let project = Project::new();
    let output = project.run(&["-o", "json", "--reproducible"]);
    let text = stdout_of(&output);

    assert!(
        text.trim_start().starts_with('{'),
        "stdout begins with something other than JSON: {:?}",
        &text[..text.len().min(120)]
    );
    serde_json::from_str::<serde_json::Value>(&text).unwrap();
}

#[test]
fn csv_output_has_a_header_and_one_row_per_extension() {
    let project = Project::new();
    let output = project.run(&["-o", "csv", "--reproducible"]);
    assert_success(&output, "-o csv");

    let text = stdout_of(&output);
    let lines: Vec<&str> = text.lines().filter(|l| !l.trim().is_empty()).collect();
    assert!(
        lines.len() >= 4,
        "expected a header and three rows: {text:?}"
    );

    let header = lines[0];
    let columns = header.matches(',').count() + 1;
    for row in &lines[1..] {
        assert_eq!(
            row.matches(',').count() + 1,
            columns,
            "row {row:?} does not match the header {header:?}"
        );
    }
}

#[test]
fn html_and_sarif_reports_are_written_where_asked() {
    let project = Project::new();
    let output_dir = tempfile::tempdir().unwrap();

    for (format, name) in [("html", "report.html"), ("sarif", "report.sarif")] {
        let destination = output_dir.path().join(name);
        let output = project.run(&[
            "-o",
            format,
            "--reproducible",
            "--output-file",
            destination.to_str().unwrap(),
        ]);
        assert_success(&output, format);
        assert!(
            destination.exists(),
            "--output-file was ignored for {format}"
        );
        let written = std::fs::read_to_string(&destination).unwrap();
        assert!(!written.is_empty());
        if format == "sarif" {
            let json: serde_json::Value = serde_json::from_str(&written).unwrap();
            assert_eq!(json["version"], "2.1.0");
        } else {
            assert!(written.contains("</html>"));
        }
    }
}

#[test]
fn listing_prints_the_files_that_would_be_counted() {
    let project = Project::new();
    let output = project.run(&["--list", "--reproducible"]);
    assert_success(&output, "--list");

    let text = stdout_of(&output);
    for expected in ["main.rs", "lib.rs", "util.rs", "main.py", "README.md"] {
        assert!(text.contains(expected), "{expected} missing from {text}");
    }
    for excluded in ["node_modules", "package-lock.json"] {
        assert!(!text.contains(excluded), "{excluded} was listed");
    }
}

#[test]
fn an_empty_directory_succeeds_with_zero_counts() {
    let empty = tempfile::tempdir().unwrap();
    let output = howmany()
        .arg(empty.path())
        .args(["--cli", "--reproducible"])
        .output()
        .unwrap();

    assert_success(&output, "an empty directory");
    let text = stdout_of(&output);
    assert!(
        text.contains("0 files"),
        "an empty directory reported {text:?}"
    );
}

/// A path that does not exist is a user error: it must be reported on stderr
/// with a non-zero status, not silently counted as an empty project.
#[test]
fn a_missing_path_fails_loudly() {
    let output = howmany()
        .arg("/definitely/does/not/exist")
        .args(["--cli"])
        .output()
        .unwrap();

    assert!(
        !output.status.success(),
        "a nonexistent path exited successfully with: {}",
        stdout_of(&output)
    );
    assert!(
        !String::from_utf8_lossy(&output.stderr).trim().is_empty(),
        "a failure printed no explanation"
    );
}

#[test]
fn an_invalid_flag_value_is_rejected_with_an_explanation() {
    for args in [
        vec!["-o", "yaml"],
        vec!["--sort", "sideways"],
        vec!["--depth", "not-a-number"],
    ] {
        let output = howmany().arg(".").args(&args).output().unwrap();
        assert!(!output.status.success(), "{args:?} was accepted");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            !stderr.trim().is_empty(),
            "{args:?} failed without a message"
        );
    }
}

#[test]
fn extension_and_ignore_filters_apply() {
    let project = Project::new();

    let only_rust = stdout_of(&project.run(&["--cli", "--reproducible", "-e", "rs"]));
    assert!(
        only_rust.contains("3 files"),
        "-e rs reported {only_rust:?}"
    );

    let without_src = stdout_of(&project.run(&["--cli", "--reproducible", "--ignore", "src"]));
    assert!(
        without_src.contains("2 files"),
        "--ignore src reported {without_src:?}"
    );
}

#[test]
fn depth_limits_traversal() {
    let project = Project::new();
    let shallow = stdout_of(&project.run(&["--cli", "--reproducible", "--depth", "1"]));
    assert!(
        shallow.contains("1 files") || shallow.contains("1 file"),
        "--depth 1 reported {shallow:?}"
    );
}

/// Every documented filter flag must change the result or be absent. A flag that
/// parses and then does nothing is worse than no flag at all.
#[test]
fn filter_flags_actually_filter() {
    let project = Project::new();
    project.write(
        "src/big.rs",
        &(0..300)
            .map(|i| format!("let x{i} = {i};\n"))
            .collect::<String>(),
    );

    let baseline = stdout_of(&project.run(&["--cli", "--reproducible"]));
    let cases: Vec<(&str, Vec<&str>)> = vec![
        ("--min-lines", vec!["--min-lines", "100"]),
        ("--max-lines", vec!["--max-lines", "5"]),
        ("--min-size", vec!["--min-size", "1KB"]),
        ("--max-size", vec!["--max-size", "100"]),
        ("--only", vec!["--only", "py"]),
        ("--exclude", vec!["--exclude", "rs"]),
        ("--min-functions", vec!["--min-functions", "2"]),
        ("--min-quality", vec!["--min-quality", "99"]),
        ("--min-complexity", vec!["--min-complexity", "5"]),
    ];

    for (flag, args) in cases {
        let mut full = vec!["--cli", "--reproducible"];
        full.extend(args);
        let output = project.run(&full);
        assert_success(&output, flag);
        let text = stdout_of(&output);
        assert_ne!(
            text.trim(),
            baseline.trim(),
            "{flag} was accepted but changed nothing"
        );
    }
}

#[test]
fn quiet_mode_prints_a_single_line() {
    let project = Project::new();
    let output = project.run(&["--quiet", "--reproducible"]);
    assert_success(&output, "--quiet");

    let text = stdout_of(&output);
    assert_eq!(
        text.lines().filter(|l| !l.trim().is_empty()).count(),
        1,
        "--quiet printed more than one line: {text:?}"
    );
}

/// A report with the wall-clock fields removed.
///
/// Everything else must match exactly, key order included: reports get diffed
/// between commits and cached in CI, so hash-ordered output is a defect even
/// when the numbers agree.
fn stable_report(output: &Output) -> String {
    let mut json: serde_json::Value = serde_json::from_str(&stdout_of(output))
        .unwrap_or_else(|e| panic!("expected JSON, got: {e}\n{}", stdout_of(output)));
    if let Some(metadata) = json.get_mut("metadata").and_then(|m| m.as_object_mut()) {
        metadata.remove("timestamp");
        metadata.remove("calculation_time_ms");
    }
    serde_json::to_string_pretty(&json).unwrap()
}

/// The same tree must produce identical output on repeated runs and at every
/// thread count. This is what makes the tool usable in CI.
#[test]
fn output_is_reproducible_across_runs_and_thread_counts() {
    let project = Project::new();
    let baseline = stable_report(&project.run(&["-o", "json", "--reproducible"]));

    for _ in 0..3 {
        assert_eq!(
            baseline,
            stable_report(&project.run(&["-o", "json", "--reproducible"])),
            "two runs of the same tree disagreed"
        );
    }

    for threads in ["1", "2", "4", "8"] {
        let output = project.run(&["-o", "json", "--no-detect", "--no-cache", "-j", threads]);
        assert_eq!(
            baseline,
            stable_report(&output),
            "{threads} threads produced a different report"
        );
    }
}

/// A machine with no `sherlock` binary is the normal case, so an installed one
/// must not change a single number in the report.
///
/// The tree deliberately contains the two shapes that used to make detection
/// observable: boilerplate the built-in rules reject, and an extension nothing
/// recognizes. Detection was consulted for both, so installing it re-admitted
/// the licence files and two machines reported different totals for the same
/// commit.
#[test]
fn results_do_not_depend_on_the_optional_detector() {
    let project = Project::new();
    project.write("LICENSE", "MIT License\n\nboilerplate\n");
    project.write("LICENSE.md", "# MIT License\n\nboilerplate\n");
    project.write("COPYING", "GPL\n");
    project.write("data.qqq", "nothing recognises this extension\n");

    let with_detection = project.run(&["-o", "json", "--no-cache"]);
    let without = project.run(&["-o", "json", "--no-cache", "--no-detect"]);
    assert_success(&with_detection, "detection auto");
    assert_success(&without, "detection disabled");

    assert_eq!(
        stable_report(&with_detection),
        stable_report(&without),
        "language detection changed the report, so two machines would disagree \
         about the same commit"
    );

    let listed = stdout_of(&project.run(&["--list", "--no-cache"]));
    for boilerplate in ["LICENSE", "COPYING"] {
        assert!(
            !listed.contains(boilerplate),
            "{boilerplate} is boilerplate nobody in the project wrote, but it \
             was counted:\n{listed}"
        );
    }
}

/// Every format that prints a complexity section must have measured it.
///
/// Complexity used to be computed only when per-file statistics were also
/// requested, which no format does by default -- so every JSON, HTML and SARIF
/// report carried zeroed metrics and a maintainability index of 100. That reads
/// as a perfect score, not as "not measured", which is the worse of the two
/// failures.
#[test]
fn reports_that_show_complexity_have_measured_it() {
    let project = Project::new();
    project.write(
        "src/work.rs",
        "pub fn branchy(n: i32) -> i32 {\n\
         \x20   if n > 0 && n < 10 {\n\
         \x20       for i in 0..n { if i % 2 == 0 { return i; } }\n\
         \x20   }\n\
         \x20   0\n\
         }\n",
    );

    let report = project.run(&["-o", "json", "--no-cache"]);
    assert_success(&report, "json report");
    let json: serde_json::Value = serde_json::from_str(&stdout_of(&report)).unwrap();
    let complexity = &json["complexity"];

    assert!(
        complexity["function_count"].as_u64().unwrap_or(0) > 0,
        "the report claims zero functions for a tree full of them: {complexity}"
    );
    assert!(
        complexity["cyclomatic_complexity"].as_f64().unwrap_or(0.0) > 0.0,
        "cyclomatic complexity was reported as zero: {complexity}"
    );
}

/// The cache must be invisible: a cold run and a warm run agree, and a warm run
/// still works when the cache directory is read-only.
#[test]
fn the_cache_never_changes_the_answer() {
    let project = Project::new();
    let cache = tempfile::tempdir().unwrap();

    let run = || {
        howmany()
            .env("HOWMANY_CACHE_DIR", cache.path())
            .arg(project.path())
            .args(["-o", "json", "--no-detect"])
            .output()
            .unwrap()
    };

    let cold = run();
    let warm = run();
    assert_success(&cold, "cold cache");
    assert_success(&warm, "warm cache");
    assert_eq!(stable_report(&cold), stable_report(&warm));
}

/// An unwritable cache directory is common in containers and CI. It must not
/// fail the run.
#[cfg(unix)]
#[test]
fn an_unwritable_cache_directory_does_not_fail_the_run() {
    use std::os::unix::fs::PermissionsExt;

    let project = Project::new();
    let cache = tempfile::tempdir().unwrap();
    std::fs::set_permissions(cache.path(), std::fs::Permissions::from_mode(0o500)).unwrap();

    let output = howmany()
        .env("HOWMANY_CACHE_DIR", cache.path())
        .arg(project.path())
        .args(["--cli", "--no-detect"])
        .output()
        .unwrap();

    std::fs::set_permissions(cache.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
    assert_success(&output, "a read-only cache directory");
}

/// Output that is piped, rather than shown on a terminal, must not contain
/// cursor movement or colour escapes: it ends up in log files and diffs.
#[test]
fn piped_output_contains_no_terminal_escapes() {
    let project = Project::new();
    for args in [
        vec!["--cli", "--reproducible"],
        vec!["-o", "json", "--reproducible"],
        vec!["-o", "csv", "--reproducible"],
        vec!["--quiet", "--reproducible"],
    ] {
        let text = stdout_of(&project.run(&args));
        assert!(
            !text.contains('\u{1b}'),
            "{args:?} emitted an ANSI escape into piped output"
        );
    }
}

#[test]
fn a_project_in_a_directory_named_like_build_output_is_still_counted() {
    // The regression that made the tool useless anywhere under /tmp, /build or
    // /vendor: patterns were matched against the absolute path.
    for hostile in ["build", "dist", "target", "vendor", "tmp"] {
        let outer = tempfile::tempdir().unwrap();
        let root = outer.path().join(hostile).join("project");
        std::fs::create_dir_all(root.join("src")).unwrap();
        std::fs::write(root.join("src/main.rs"), "fn main() {}\n").unwrap();

        let output = howmany()
            .arg(&root)
            .args(["--cli", "--reproducible"])
            .output()
            .unwrap();
        assert_success(&output, hostile);
        let text = stdout_of(&output);
        assert!(
            text.contains("1 files") || text.contains("1 file"),
            "a project under {hostile:?} reported {text:?}"
        );
    }
}

#[test]
fn a_file_can_be_analyzed_directly() {
    let project = Project::new();
    let file = project.path().join("src/main.rs");
    let output = howmany()
        .arg(&file)
        .args(["--cli", "--reproducible"])
        .output()
        .unwrap();

    assert_success(&output, "a single file");
    let text = stdout_of(&output);
    assert!(
        text.contains("1 files") || text.contains("1 file"),
        "{text:?}"
    );
}

/// Unreadable files must be reported without aborting the run.
#[cfg(unix)]
#[test]
fn an_unreadable_file_does_not_abort_the_run() {
    use std::os::unix::fs::PermissionsExt;

    let project = Project::new();
    let locked = project.write("src/locked.rs", "fn locked() {}\n");
    std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o000)).unwrap();

    let output = project.run(&["--cli", "--reproducible"]);
    std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o644)).unwrap();

    assert_success(&output, "a tree containing an unreadable file");
    assert!(stdout_of(&output).contains("files"));
}
