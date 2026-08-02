//! The supporting machinery: cache, errors, configuration, metrics.
//!
//! These are the parts that decide whether a *fresh* install behaves. A cache
//! from an older version, a config file someone hand-edited, a home directory
//! that is not writable -- each must degrade into "run normally", never into a
//! failure or a wrong answer. The cache tests here concentrate on correctness
//! under change; the ones next to the implementation cover its file format.

use crate::core::types::FileStats;
use crate::utils::cache::{CacheKey, FileCache};
use crate::utils::config::HowManyConfig;
use crate::utils::errors::HowManyError;
use crate::utils::metrics::{MetricsCollector, PerformanceMetrics, Timer};
use crate::utils::progress::FileProgress;
use std::time::Duration;

fn stats(lines: usize) -> FileStats {
    FileStats {
        total_lines: lines,
        code_lines: lines,
        file_size: lines as u64 * 10,
        ..FileStats::default()
    }
}

#[test]
fn a_cached_entry_is_returned_for_an_unchanged_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("a.rs");
    std::fs::write(&path, "fn a() {}\n").unwrap();

    let mut cache = FileCache::new();
    cache.insert(path.clone(), stats(1)).unwrap();

    assert_eq!(cache.get(&path).map(|s| s.total_lines), Some(1));
    assert_eq!(cache.size(), 1);
}

/// The cache must key on content, not just on the path: an editor that rewrites
/// a file with the same size must still invalidate it.
#[test]
fn a_rewritten_file_is_not_served_from_cache() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("a.rs");
    std::fs::write(&path, "fn a() {}\n").unwrap();

    let mut cache = FileCache::new();
    cache.insert(path.clone(), stats(1)).unwrap();

    // Same byte length, different content and a new modification time.
    std::thread::sleep(Duration::from_millis(10));
    std::fs::write(&path, "fn b() {}\n").unwrap();

    let key = CacheKey::for_path(&path).unwrap();
    assert!(
        cache.get_with_key(&path, &key).is_none(),
        "a modified file was served from the cache"
    );
}

#[test]
fn a_deleted_file_is_dropped_from_the_cache() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("gone.rs");
    std::fs::write(&path, "fn a() {}\n").unwrap();

    let mut cache = FileCache::new();
    cache.insert(path.clone(), stats(1)).unwrap();
    std::fs::remove_file(&path).unwrap();

    cache.cleanup_missing_files();
    assert!(
        cache.is_empty(),
        "the cache would grow without bound across branch switches"
    );
}

#[test]
fn clearing_and_removing_entries_works() {
    let dir = tempfile::tempdir().unwrap();
    let a = dir.path().join("a.rs");
    let b = dir.path().join("b.rs");
    std::fs::write(&a, "fn a() {}\n").unwrap();
    std::fs::write(&b, "fn b() {}\n").unwrap();

    let mut cache = FileCache::new();
    cache.insert(a.clone(), stats(1)).unwrap();
    cache.insert(b.clone(), stats(1)).unwrap();
    assert_eq!(cache.size(), 2);

    cache.remove(&a);
    assert_eq!(cache.size(), 1);
    cache.clear();
    assert!(cache.is_empty());
}

#[test]
fn a_cache_key_needs_a_real_file() {
    assert!(CacheKey::for_path(std::path::Path::new("/definitely/not/here")).is_none());
}

/// A cache file from a future or older layout must be discarded rather than
/// misread -- and discarding it must not fail the run.
#[test]
fn an_unreadable_cache_falls_back_to_an_empty_one() {
    let dir = tempfile::tempdir().unwrap();
    let previous = std::env::var_os("HOWMANY_CACHE_DIR");
    std::env::set_var("HOWMANY_CACHE_DIR", dir.path());

    let project = tempfile::tempdir().unwrap();
    let cache_path = FileCache::cache_path_for(project.path()).unwrap();
    std::fs::create_dir_all(cache_path.parent().unwrap()).unwrap();
    std::fs::write(&cache_path, b"\x00\x01not json at all").unwrap();

    assert!(FileCache::scoped(project.path()).is_empty());

    match previous {
        Some(value) => std::env::set_var("HOWMANY_CACHE_DIR", value),
        None => std::env::remove_var("HOWMANY_CACHE_DIR"),
    }
}

#[test]
fn every_error_kind_has_a_message_a_user_can_act_on() {
    let errors = [
        HowManyError::file_processing("could not read src/main.rs"),
        HowManyError::invalid_config("max_depth must be positive"),
        HowManyError::filter("invalid --ignore pattern"),
        HowManyError::counter("unsupported encoding"),
        HowManyError::display("terminal too small"),
    ];

    for error in errors {
        let rendered = error.to_string();
        assert!(!rendered.is_empty());
        assert!(
            rendered.chars().any(|c| c.is_alphabetic()),
            "an error rendered as punctuation only: {rendered:?}"
        );
        // The `Debug` form ends up in logs; it must not be empty either.
        assert!(!format!("{error:?}").is_empty());
    }
}

#[test]
fn io_errors_convert_without_losing_their_cause() {
    let io = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied");
    let converted: HowManyError = io.into();
    assert!(
        converted.to_string().to_lowercase().contains("denied"),
        "the underlying cause was discarded: {converted}"
    );
}

#[test]
fn default_configuration_is_usable_as_is() {
    let config = HowManyConfig::default();
    assert!(config.performance.chunk_size > 0);
    assert!(!config.output_preferences.default_format.is_empty());
    assert!(!config.output_preferences.default_sort_by.is_empty());
}

#[test]
fn configuration_survives_a_toml_round_trip() {
    let mut config = HowManyConfig {
        default_max_depth: Some(7),
        custom_ignore_patterns: vec!["scratch/".to_string()],
        ..Default::default()
    };
    config.performance.max_threads = Some(3);

    let text = toml::to_string(&config).unwrap();
    let back: HowManyConfig = toml::from_str(&text).unwrap();

    assert_eq!(back.default_max_depth, Some(7));
    assert_eq!(back.custom_ignore_patterns, vec!["scratch/".to_string()]);
    assert_eq!(back.performance.max_threads, Some(3));
}

/// A partial config file is the normal case: users set one key. Missing keys
/// must take their defaults instead of failing to parse.
#[test]
fn a_partial_configuration_file_takes_defaults_for_the_rest() {
    let config: HowManyConfig = toml::from_str("default_include_hidden = true\n").unwrap();
    assert!(config.default_include_hidden);
    assert_eq!(
        config.performance.chunk_size,
        HowManyConfig::default().performance.chunk_size
    );
}

#[test]
fn metrics_rates_are_finite_even_with_no_elapsed_time() {
    let metrics = PerformanceMetrics::new();
    for rate in [
        metrics.files_per_second(),
        metrics.lines_per_second(),
        metrics.bytes_per_second(),
        metrics.cache_hit_rate(),
    ] {
        assert!(rate.is_finite(), "a rate divided by zero elapsed time");
    }
}

#[test]
fn metrics_accumulate_what_they_are_told() {
    let mut collector = MetricsCollector::new();
    for _ in 0..3 {
        collector.record_file_processed(100, 1_000);
        collector.record_cache_hit();
    }
    collector.record_cache_miss();
    collector.add_phase_timing("counting", Duration::from_millis(5));

    let metrics = collector.finish();
    assert_eq!(metrics.files_processed, 3);
    assert_eq!(metrics.lines_processed, 300);
    assert_eq!(metrics.bytes_processed, 3_000);
    assert_eq!(metrics.cache_hits, 3);
    assert_eq!(metrics.cache_misses, 1);
    assert!((metrics.cache_hit_rate() - 0.75).abs() < 1e-9);
    assert!(metrics.phase_timings.contains_key("counting"));
}

#[test]
fn a_timer_reports_the_name_it_was_given() {
    let timer = Timer::new("discovery");
    let (name, elapsed) = timer.finish();
    assert_eq!(name, "discovery");
    assert!(elapsed < Duration::from_secs(5));
}

#[test]
fn progress_percentage_stays_between_zero_and_a_hundred() {
    let mut progress = FileProgress::new(4);
    assert_eq!(progress.percentage(), 0.0);

    for i in 0..4 {
        progress.update_file(&format!("f{i}.rs"));
        progress.add_lines(10);
        progress.add_bytes(100);
        assert!((0.0..=100.0).contains(&progress.percentage()));
    }
    assert!((progress.percentage() - 100.0).abs() < 1e-9);

    // Reporting more files than were promised must not exceed 100%.
    progress.update_file("extra.rs");
    assert!(progress.percentage() <= 100.0);
}

#[test]
fn progress_over_zero_files_does_not_divide_by_zero() {
    let progress = FileProgress::new(0);
    assert!(progress.percentage().is_finite());
}
