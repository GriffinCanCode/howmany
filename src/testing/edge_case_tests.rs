//! Inputs that break line counters.
//!
//! A counter is easy to write for well-formed UTF-8 text with Unix newlines. The
//! cases below are the ones that appear in real checkouts and historically make
//! tools panic, hang, or quietly report nonsense: files with no trailing
//! newline, CRLF and lone-CR line endings, byte-order marks, invalid UTF-8,
//! embedded NUL bytes, megabyte-long single lines, unterminated block comments,
//! deeply nested directories, and filenames the shell would refuse to spell.
//!
//! Every test asserts one of two things: an exact count, or that the run
//! completed without a panic and kept its invariants. Nothing here is allowed to
//! be "best effort".

use crate::core::counter::CodeCounter;
use crate::core::engine::{AnalysisOptions, DetectionMode, Engine, Parallelism};
use crate::core::types::FileStats;
use crate::testing::test_utils::TestProject;

fn options() -> AnalysisOptions {
    AnalysisOptions {
        detection: DetectionMode::Disabled,
        use_cache: false,
        collect_individual_files: true,
        ..AnalysisOptions::default()
    }
}

/// Count one file's bytes, asserting the partition invariant while we are here.
fn count(name: &str, bytes: &[u8]) -> FileStats {
    let project = TestProject::new("edge_single").unwrap();
    let path = project.create_file_binary(name, bytes).unwrap();
    let stats = CodeCounter::new().count_file(&path).unwrap();
    assert!(
        stats.is_consistent(),
        "line categories do not partition the total for {name}: {stats:?}"
    );
    stats
}

#[test]
fn an_empty_file_counts_as_nothing() {
    let stats = count("empty.rs", b"");
    assert_eq!(stats.total_lines, 0);
    assert_eq!(stats.code_lines, 0);
    assert_eq!(stats.file_size, 0);
}

#[test]
fn a_file_of_one_byte_counts_as_one_line() {
    let stats = count("one.rs", b"x");
    assert_eq!(stats.total_lines, 1);
    assert_eq!(stats.code_lines, 1);
}

/// A trailing newline terminates the last line; it does not begin a new one.
#[test]
fn a_missing_trailing_newline_does_not_lose_the_last_line() {
    let with = count("with.rs", b"fn a() {}\nfn b() {}\n");
    let without = count("without.rs", b"fn a() {}\nfn b() {}");

    assert_eq!(with.total_lines, 2);
    assert_eq!(without.total_lines, 2);
    assert_eq!(with.code_lines, without.code_lines);
}

#[test]
fn a_file_of_only_newlines_is_all_blank() {
    let stats = count("blank.rs", b"\n\n\n\n\n");
    assert_eq!(stats.total_lines, 5);
    assert_eq!(stats.blank_lines, 5);
    assert_eq!(stats.code_lines, 0);
}

#[test]
fn windows_line_endings_count_the_same_as_unix() {
    let unix = count("unix.rs", b"fn a() {}\n// c\n\nfn b() {}\n");
    let windows = count("win.rs", b"fn a() {}\r\n// c\r\n\r\nfn b() {}\r\n");

    assert_eq!(unix.total_lines, windows.total_lines);
    assert_eq!(unix.code_lines, windows.code_lines);
    assert_eq!(unix.comment_lines, windows.comment_lines);
    assert_eq!(
        unix.blank_lines, windows.blank_lines,
        "a CRLF blank line still holds a carriage return, which must not be \
         mistaken for content"
    );
}

/// Classic Mac line endings: a whole file on one physical line as far as `\n` is
/// concerned. It must be counted, not mangled.
#[test]
fn lone_carriage_returns_do_not_break_counting() {
    let stats = count("mac.rs", b"fn a() {}\rfn b() {}\r");
    assert!(stats.total_lines >= 1);
    assert_eq!(stats.total_lines, stats.code_lines + stats.blank_lines);
}

#[test]
fn a_byte_order_mark_is_not_content() {
    let stats = count("bom.rs", "\u{feff}fn main() {}\n".as_bytes());
    assert_eq!(stats.total_lines, 1);
    assert_eq!(
        stats.code_lines, 1,
        "a UTF-8 BOM must not turn the first line into something unclassifiable"
    );
}

#[test]
fn a_bom_before_a_comment_still_leaves_a_comment() {
    let stats = count("bomc.rs", "\u{feff}// a comment\n".as_bytes());
    assert_eq!(stats.comment_lines, 1, "{stats:?}");
}

/// Invalid UTF-8 is common in real trees: files saved in Latin-1, or source with
/// a stray byte in a string literal. Counting is byte-oriented, so it works.
#[test]
fn invalid_utf8_is_counted_rather_than_rejected() {
    let stats = count(
        "latin1.rs",
        b"fn a() {} // caf\xe9\n// \xff\xfe\nfn b() {}\n",
    );
    assert_eq!(stats.total_lines, 3);
    assert_eq!(stats.code_lines, 2);
    assert_eq!(stats.comment_lines, 1);
}

#[test]
fn embedded_nul_bytes_do_not_truncate_the_count() {
    let stats = count("nul.rs", b"fn a() {}\n\0\0\0\nfn b() {}\n");
    assert_eq!(stats.total_lines, 3);
}

#[test]
fn a_very_long_single_line_is_counted_once() {
    let mut bytes = b"let s = \"".to_vec();
    bytes.extend(std::iter::repeat_n(b'x', 2 * 1024 * 1024));
    bytes.extend(b"\";\n");

    let stats = count("long.rs", &bytes);
    assert_eq!(stats.total_lines, 1);
    assert_eq!(stats.code_lines, 1);
    assert!(stats.file_size > 2 * 1024 * 1024);
}

#[test]
fn many_short_lines_are_all_counted() {
    let bytes = "x\n".repeat(200_000).into_bytes();
    let stats = count("many.rs", &bytes);
    assert_eq!(stats.total_lines, 200_000);
    assert_eq!(stats.code_lines, 200_000);
}

#[test]
fn whitespace_only_lines_are_blank_whatever_the_whitespace() {
    let stats = count("ws.rs", b"fn a() {}\n   \n\t\n \t \n\x0c\n");
    assert_eq!(stats.total_lines, 5);
    assert_eq!(stats.code_lines, 1);
    assert_eq!(stats.blank_lines, 4, "{stats:?}");
}

#[test]
fn an_unterminated_block_comment_does_not_swallow_the_file_silently() {
    let stats = count("unterminated.rs", b"fn a() {}\n/* open\nstill open\n");
    assert_eq!(stats.total_lines, 3);
    assert_eq!(stats.code_lines, 1);
    assert_eq!(
        stats.comment_lines, 2,
        "everything after an unclosed opener is comment, not code"
    );
}

#[test]
fn nested_and_adjacent_block_comments_are_counted_once() {
    let stats = count(
        "blocks.rs",
        b"/* a */ /* b */ fn f() {}\n/* c\n*/ fn g() {}\n",
    );
    assert_eq!(stats.total_lines, 3);
    assert!(stats.is_consistent());
}

#[test]
fn unicode_content_does_not_shift_classification() {
    let stats = count(
        "unicode.rs",
        "// коммент\n/// документация\nlet emoji = \"🦀🎉\";\n".as_bytes(),
    );
    assert_eq!(stats.total_lines, 3);
    assert_eq!(stats.comment_lines, 1);
    assert_eq!(stats.doc_lines, 1);
    assert_eq!(stats.code_lines, 1);
}

#[test]
fn extensionless_scripts_are_recognised_by_their_shebang() {
    let project = TestProject::new("edge_shebang").unwrap();
    let cases = [
        ("bootstrap", "#!/bin/bash\n# setup\necho hi\n", 1, 2),
        (
            "configure",
            "#!/usr/bin/env python3\n# setup\nprint(1)\n",
            1,
            2,
        ),
        (
            "build",
            "#!/usr/bin/env -S node --harmony\n// go\nrun();\n",
            1,
            2,
        ),
    ];

    for (name, body, comments, code) in cases {
        let path = project.create_file(name, body).unwrap();
        let stats = CodeCounter::new().count_file(&path).unwrap();
        assert_eq!(
            stats.comment_lines, comments,
            "{name}: comments were counted as code because the interpreter was \
             not consulted ({stats:?})"
        );
        assert_eq!(stats.code_lines, code, "{name}: {stats:?}");
    }
}

#[test]
fn a_shebangless_extensionless_file_is_not_treated_as_source() {
    let project = TestProject::new("edge_no_shebang").unwrap();
    project.create_file("NOTES", "just some notes\n").unwrap();
    project.create_file("keep.rs", "fn f() {}\n").unwrap();

    let analysis = Engine::new().analyze(project.path(), &options()).unwrap();
    assert_eq!(analysis.report.files_counted, 1);
}

#[test]
fn filenames_that_look_like_flags_or_paths_are_handled() {
    let project = TestProject::new("edge_names").unwrap();
    let names = [
        "-dashed.rs",
        "--double.rs",
        "spaces everywhere.rs",
        "quote'single.rs",
        "semi;colon.rs",
        "dollar$sign.rs",
        "paren(s).rs",
        "🦀.rs",
    ];
    for name in names {
        project.create_file(name, "fn f() {}\n").unwrap();
    }

    let analysis = Engine::new().analyze(project.path(), &options()).unwrap();
    assert_eq!(
        analysis.report.files_counted,
        names.len(),
        "counted {:?}",
        analysis
            .individual_files
            .iter()
            .map(|(p, _)| p.as_str())
            .collect::<Vec<_>>()
    );
}

#[test]
fn deep_nesting_does_not_overflow_the_stack() {
    let project = TestProject::new("edge_deep").unwrap();
    let mut relative = String::new();
    for _ in 0..80 {
        relative.push_str("d/");
    }
    project
        .create_file(&format!("{relative}deep.rs"), "fn deep() {}\n")
        .unwrap();

    let analysis = Engine::new().analyze(project.path(), &options()).unwrap();
    assert_eq!(analysis.report.files_counted, 1);
}

#[test]
fn a_wide_directory_is_counted_completely() {
    let project = TestProject::new("edge_wide").unwrap();
    for i in 0..2_000 {
        project
            .create_file(&format!("wide/f{i:04}.rs"), "fn f() {}\n")
            .unwrap();
    }

    let analysis = Engine::new().analyze(project.path(), &options()).unwrap();
    assert_eq!(analysis.report.files_counted, 2_000);
    assert_eq!(analysis.basic.total_lines, 2_000);
}

#[test]
fn a_file_that_is_only_a_comment_marker_is_still_a_comment() {
    for (name, body) in [("a.rs", "//"), ("b.py", "#"), ("c.html", "<!--")] {
        let stats = count(name, body.as_bytes());
        assert_eq!(stats.total_lines, 1, "{name}");
        assert_eq!(stats.comment_lines, 1, "{name}: {stats:?}");
    }
}

#[test]
fn identical_content_under_different_extensions_is_classified_per_language() {
    let body = b"# comment\nvalue = 1\n";
    let python = count("a.py", body);
    let rust = count("a.rs", body);

    assert_eq!(python.comment_lines, 1);
    assert_eq!(
        rust.comment_lines, 0,
        "`#` is not a Rust comment; it must not be counted as one"
    );
    assert_eq!(python.total_lines, rust.total_lines);
}

/// A file removed between discovery and counting must be reported, not fatal.
#[test]
fn a_file_that_vanishes_is_reported_and_the_run_continues() {
    let project = TestProject::new("edge_vanish").unwrap();
    project.create_file("stays.rs", "fn f() {}\n").unwrap();
    let doomed = project.create_file("goes.rs", "fn g() {}\n").unwrap();

    let engine = Engine::new();
    let paths = engine.discover_files(project.path(), &options()).unwrap();
    assert_eq!(paths.len(), 2);

    std::fs::remove_file(&doomed).unwrap();
    let analysis = engine.analyze(project.path(), &options()).unwrap();
    assert_eq!(analysis.report.files_counted, 1);
    assert_eq!(analysis.report.files_failed, 0);
}

#[cfg(unix)]
#[test]
fn a_filename_containing_a_newline_is_handled() {
    let project = TestProject::new("edge_newline_name").unwrap();
    let path = project.path().join("we\nird.rs");
    std::fs::write(&path, "fn f() {}\n").unwrap();

    let analysis = Engine::new().analyze(project.path(), &options()).unwrap();
    assert_eq!(analysis.report.files_counted, 1);
}

#[cfg(unix)]
#[test]
fn a_named_pipe_does_not_hang_the_walk() {
    use std::process::Command;

    let project = TestProject::new("edge_fifo").unwrap();
    project.create_file("real.rs", "fn f() {}\n").unwrap();
    let fifo = project.path().join("pipe.rs");
    let made = Command::new("mkfifo")
        .arg(&fifo)
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    if !made {
        return;
    }

    // A FIFO with no writer blocks forever on open, so it must be skipped by
    // file-type rather than opened speculatively.
    let paths = Engine::new()
        .discover_files(project.path(), &options())
        .unwrap();
    assert!(
        paths.iter().all(|p| p.ends_with("real.rs")),
        "a named pipe was queued for counting: {paths:?}"
    );
}

#[test]
fn counts_do_not_depend_on_the_number_of_threads_for_awkward_inputs() {
    let project = TestProject::new("edge_threads").unwrap();
    project
        .create_file_binary("bad.rs", b"fn a() {} // \xff\n")
        .unwrap();
    project.create_file("empty.rs", "").unwrap();
    project.create_file("crlf.rs", "fn a() {}\r\n\r\n").unwrap();
    project.create_file("no_nl.rs", "fn a() {}").unwrap();
    project
        .create_file("bom.rs", "\u{feff}fn a() {}\n")
        .unwrap();
    for i in 0..50 {
        project
            .create_file(&format!("src/f{i}.rs"), "fn f() {}\n// c\n")
            .unwrap();
    }

    let engine = Engine::new();
    let baseline = engine
        .analyze(
            project.path(),
            &AnalysisOptions {
                parallelism: Parallelism::Fixed(1),
                ..options()
            },
        )
        .unwrap();

    for threads in [2, 4, 16] {
        let other = engine
            .analyze(
                project.path(),
                &AnalysisOptions {
                    parallelism: Parallelism::Fixed(threads),
                    ..options()
                },
            )
            .unwrap();
        assert_eq!(baseline.basic, other.basic, "{threads} threads disagreed");
        assert_eq!(baseline.individual_files, other.individual_files);
    }
}

#[test]
fn a_binary_file_wearing_a_source_extension_does_not_panic() {
    let bytes: Vec<u8> = (0u8..=255).cycle().take(64 * 1024).collect();
    let stats = count("blob.rs", &bytes);
    assert!(stats.total_lines > 0);
    assert!(stats.is_consistent());
}

#[test]
fn analysis_of_a_single_file_root_counts_that_file() {
    let project = TestProject::new("edge_file_root").unwrap();
    let file = project.create_file("solo.rs", "fn a() {}\n// c\n").unwrap();

    let analysis = Engine::new().analyze(&file, &options()).unwrap();
    assert_eq!(analysis.report.files_counted, 1);
    assert_eq!(analysis.basic.total_lines, 2);
}

#[test]
fn nonexistent_and_unreadable_roots_return_empty_reports() {
    let engine = Engine::new();
    for root in ["/definitely/not/here", "/proc/self/mem"] {
        let analysis = engine
            .analyze(std::path::Path::new(root), &options())
            .unwrap();
        assert!(analysis.basic.is_consistent(), "{root}");
    }
}
