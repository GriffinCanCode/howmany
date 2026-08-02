//! Throughput properties, asserted rather than measured.
//!
//! Wall-clock numbers belong in `benches/throughput.rs`, where Criterion can
//! account for noise. What belongs in a test is the *shape* of the performance
//! work: that parallelism is actually used, that the cache actually avoids
//! reading files, that pruning avoids descending into dependency directories,
//! and that none of it changes the answer. Every threshold below is deliberately
//! loose, so a failure means a mechanism broke, not that the machine was busy.

use crate::core::engine::{AnalysisOptions, DetectionMode, Engine, Parallelism};
use crate::testing::test_utils::TestProject;
use std::time::Instant;

fn options() -> AnalysisOptions {
    AnalysisOptions {
        detection: DetectionMode::Disabled,
        use_cache: false,
        ..AnalysisOptions::default()
    }
}

/// A tree big enough that per-file costs dominate, small enough for CI.
fn corpus(name: &str, files: usize) -> TestProject {
    let project = TestProject::new(name).unwrap();
    let body: String = (0..40)
        .map(|i| match i % 4 {
            0 => "// a comment\n".to_string(),
            1 => "\n".to_string(),
            2 => format!("let value_{i} = {i};\n"),
            _ => format!("fn helper_{i}() -> i32 {{ {i} }}\n"),
        })
        .collect();

    for i in 0..files {
        project
            .create_file(&format!("src/pkg{}/mod{i}.rs", i % 32), &body)
            .unwrap();
    }
    project
}

#[test]
fn a_large_tree_is_counted_completely_and_quickly() {
    let files = 3_000;
    let project = corpus("perf_large", files);

    let started = Instant::now();
    let analysis = Engine::new().analyze(project.path(), &options()).unwrap();
    let elapsed = started.elapsed();

    assert_eq!(analysis.report.files_counted, files);
    assert_eq!(analysis.basic.total_lines, files * 40);
    assert!(
        elapsed.as_secs() < 30,
        "counting {files} small files took {elapsed:?}, which means something is \
         quadratic rather than merely slow"
    );
}

/// Parallel counting exists to be faster; at minimum it must not be
/// dramatically slower, which is what happens when work is serialised behind a
/// lock or the pool is rebuilt per file.
#[test]
fn parallel_counting_is_not_slower_than_sequential() {
    if std::thread::available_parallelism().map_or(1, |n| n.get()) < 2 {
        return;
    }
    let project = corpus("perf_scaling", 2_000);
    let engine = Engine::new();

    let timed = |threads: usize| {
        let started = Instant::now();
        let analysis = engine
            .analyze(
                project.path(),
                &AnalysisOptions {
                    parallelism: Parallelism::Fixed(threads),
                    ..options()
                },
            )
            .unwrap();
        (started.elapsed(), analysis)
    };

    let (sequential, one) = timed(1);
    let (parallel, many) = timed(8);

    assert_eq!(one.basic, many.basic, "parallelism changed the result");
    assert!(
        parallel.as_secs_f64() < sequential.as_secs_f64() * 3.0 + 0.5,
        "eight threads took {parallel:?} against {sequential:?} sequentially, so \
         the work is not really running in parallel"
    );
}

/// The cache exists to avoid reading unchanged files. That is observable without
/// timing anything: the second run must report hits, not misses.
#[test]
fn a_warm_cache_avoids_re_reading_files() {
    let project = corpus("perf_cache", 500);
    let cache_dir = tempfile::tempdir().unwrap();
    let previous = std::env::var_os("HOWMANY_CACHE_DIR");
    std::env::set_var("HOWMANY_CACHE_DIR", cache_dir.path());

    let engine = Engine::new();
    let cached = AnalysisOptions {
        use_cache: true,
        ..options()
    };
    let cold = engine.analyze(project.path(), &cached).unwrap();
    let warm = engine.analyze(project.path(), &cached).unwrap();

    match previous {
        Some(value) => std::env::set_var("HOWMANY_CACHE_DIR", value),
        None => std::env::remove_var("HOWMANY_CACHE_DIR"),
    }

    assert_eq!(cold.report.cache_hits, 0);
    assert_eq!(cold.report.cache_misses, cold.report.files_counted);
    assert_eq!(warm.report.cache_hits, warm.report.files_counted);
    assert_eq!(cold.basic, warm.basic, "the cache changed the answer");
    assert!(
        warm.report.counting_time <= cold.report.counting_time * 2,
        "a fully cached run spent {:?} counting against {:?} cold",
        warm.report.counting_time,
        cold.report.counting_time
    );
}

/// Pruning is what keeps a `node_modules` beside your source from dominating the
/// run: the walk must not descend into it at all.
#[test]
fn dependency_directories_are_not_descended_into() {
    let project = TestProject::new("perf_prune").unwrap();
    for i in 0..50 {
        project
            .create_file(&format!("src/m{i}.rs"), "fn f() {}\n")
            .unwrap();
    }
    // Ten times as many files in a dependency directory as in the source tree.
    for i in 0..2_000 {
        project
            .create_file(
                &format!("node_modules/pkg{}/file{i}.js", i % 50),
                "module.exports = 1;\n",
            )
            .unwrap();
    }

    let started = Instant::now();
    let analysis = Engine::new().analyze(project.path(), &options()).unwrap();
    let elapsed = started.elapsed();

    assert_eq!(analysis.report.files_counted, 50);
    assert!(
        elapsed.as_secs() < 10,
        "a pruned dependency tree still cost {elapsed:?}, so it is being walked"
    );
}

/// Discovery must be linear in the number of files. Doubling the tree roughly
/// doubles the work; a quadratic step would show up as a large multiple.
#[test]
fn discovery_cost_grows_linearly_with_the_tree() {
    let small = corpus("perf_linear_small", 500);
    let large = corpus("perf_linear_large", 2_000);
    let engine = Engine::new();

    let timed = |project: &TestProject| {
        // Warm the directory cache so the comparison measures our work.
        let _ = engine.discover_files(project.path(), &options()).unwrap();
        let started = Instant::now();
        let found = engine.discover_files(project.path(), &options()).unwrap();
        (started.elapsed().as_secs_f64(), found.len())
    };

    let (small_secs, small_files) = timed(&small);
    let (large_secs, large_files) = timed(&large);

    assert_eq!(small_files, 500);
    assert_eq!(large_files, 2_000);

    // Four times the files, allowed twenty times the time before we call it
    // superlinear; the floor keeps a fast machine from dividing by ~zero.
    let ratio = large_secs / small_secs.max(0.001);
    assert!(
        ratio < 20.0,
        "discovery took {large_secs:.3}s for 2000 files against \
         {small_secs:.3}s for 500 -- a factor of {ratio:.1} for 4x the files"
    );
}

/// Skipping per-file collection is the fast path for `--cli`, and it must agree
/// with the detailed path on every total it still reports.
#[test]
fn the_aggregate_only_path_matches_the_detailed_one() {
    let project = corpus("perf_aggregate_only", 800);
    let engine = Engine::new();

    let detailed = engine
        .analyze(
            project.path(),
            &AnalysisOptions {
                collect_individual_files: true,
                compute_complexity: true,
                ..options()
            },
        )
        .unwrap();
    let aggregate = engine
        .analyze(
            project.path(),
            &AnalysisOptions {
                collect_individual_files: false,
                compute_complexity: false,
                ..options()
            },
        )
        .unwrap();

    assert_eq!(detailed.basic, aggregate.basic);
    assert!(
        aggregate.individual_files.is_empty(),
        "the fast path allocated one record per file anyway"
    );
}

#[test]
fn the_report_accounts_for_every_file_and_byte() {
    let project = corpus("perf_accounting", 400);
    let analysis = Engine::new().analyze(project.path(), &options()).unwrap();
    let report = &analysis.report;

    assert_eq!(
        report.files_counted + report.files_failed,
        report.files_discovered
    );
    assert_eq!(
        report.cache_hits + report.cache_misses,
        report.files_counted
    );
    assert_eq!(report.bytes_read, analysis.basic.total_size);
    assert!(report.total_time >= report.counting_time);
    assert!(report.throughput_files_per_second().unwrap_or(0.0) > 0.0);
}
