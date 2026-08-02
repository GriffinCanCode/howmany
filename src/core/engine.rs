//! The analysis pipeline.
//!
//! This used to live inside `main.rs`, which meant the only way to exercise it
//! was to run the binary and parse its output -- so it was neither benchmarked
//! nor unit tested, and library consumers had to reimplement it. It is a
//! library concern, so it lives here.
//!
//! The pipeline has four stages:
//!
//! 1. **Detect** (optional) -- ask the external `sherlock` binary to classify
//!    languages. Runs concurrently with stage 2 because it is an independent
//!    process that re-walks the tree itself.
//! 2. **Discover** -- walk the tree, pruning build directories, and keep the
//!    metadata the walk already paid for.
//! 3. **Count** -- classify every file's lines in parallel.
//! 4. **Aggregate** -- sum the per-file results and derive project statistics.
//!
//! Two properties are load-bearing and are asserted by the tests:
//!
//! * **Determinism.** Discovered paths are sorted before counting, so the report
//!   does not depend on directory order, thread count, or scheduling.
//! * **Parallel/sequential equivalence.** Counting with any number of threads
//!   produces byte-identical statistics.

use crate::core::counter::{self, CodeCounter};
use crate::core::detector::{Classification, DetectionJob, FileDetector};
use crate::core::filters::FileFilter;
use crate::core::stats::{AggregatedStats, StatsCalculator};
use crate::core::types::{CodeStats, FileStats};
use crate::utils::cache::{CacheKey, FileCache};
use crate::utils::errors::Result;
use rayon::prelude::*;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// How language detection should be used.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DetectionMode {
    /// Use `sherlock` when it is installed, fall back silently otherwise.
    #[default]
    Auto,
    /// Never invoke `sherlock`.
    ///
    /// Results then depend only on the tree being analyzed, which is what makes
    /// a run reproducible across machines.
    Disabled,
}

/// Default ceiling on worker threads.
///
/// Counting is dominated by opening and reading tens of thousands of small
/// files, and that concurrency saturates well before core count. Measured on a
/// 26,000-file corpus on a 12+4 core machine, the counting phase took 227 ms on
/// 4 threads, 213 ms on 8, 320 ms on 12 and 380 ms on 16 -- so using every
/// logical CPU, which is what `available_parallelism` reports, made the default
/// run 58% slower than the plateau. Twelve threads is already past the knee even
/// though all twelve are performance cores, which puts the ceiling in the
/// filesystem rather than in the CPU topology.
///
/// `--threads` overrides this for anyone whose storage scales further.
const DEFAULT_MAX_THREADS: usize = 8;

/// How many worker threads to use.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Parallelism {
    /// One thread per available core, up to [`DEFAULT_MAX_THREADS`].
    #[default]
    Auto,
    /// A fixed number of threads, honoured as given. `1` means fully sequential.
    Fixed(usize),
}

impl Parallelism {
    pub fn threads(self) -> usize {
        match self {
            Parallelism::Auto => std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(1)
                .min(DEFAULT_MAX_THREADS),
            Parallelism::Fixed(n) => n.max(1),
        }
    }
}

/// Inputs to an analysis run.
#[derive(Debug, Clone)]
pub struct AnalysisOptions {
    pub max_depth: Option<usize>,
    pub include_hidden: bool,
    pub ignore_patterns: Vec<String>,
    /// Restrict to these extensions; empty means every recognized extension.
    pub extensions: Vec<String>,
    /// Retain per-file statistics. Required for complexity analysis.
    pub collect_individual_files: bool,
    pub detection: DetectionMode,
    pub parallelism: Parallelism,
    /// Reuse results for unchanged files across runs.
    pub use_cache: bool,
    /// Compute complexity, quality and ratio statistics.
    pub compute_complexity: bool,
}

impl Default for AnalysisOptions {
    fn default() -> Self {
        Self {
            max_depth: None,
            include_hidden: false,
            ignore_patterns: Vec::new(),
            extensions: Vec::new(),
            collect_individual_files: false,
            detection: DetectionMode::Auto,
            parallelism: Parallelism::Auto,
            use_cache: true,
            compute_complexity: true,
        }
    }
}

impl AnalysisOptions {
    /// Options for a fully reproducible run: no language detection, no cache,
    /// single threaded.
    pub fn reproducible() -> Self {
        Self {
            detection: DetectionMode::Disabled,
            parallelism: Parallelism::Fixed(1),
            use_cache: false,
            ..Self::default()
        }
    }

    pub fn with_extensions(mut self, extensions: Vec<String>) -> Self {
        self.extensions = extensions;
        self
    }

    pub fn collecting_files(mut self, collect: bool) -> Self {
        self.collect_individual_files = collect;
        self
    }
}

/// What the run cost and what it had to skip.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AnalysisReport {
    pub files_discovered: usize,
    pub files_counted: usize,
    /// Files that could not be read; each is listed in `failures`.
    pub files_failed: usize,
    pub cache_hits: usize,
    pub cache_misses: usize,
    pub bytes_read: u64,
    /// True when language detection was requested but unavailable, so the
    /// extension fallback was used.
    pub detection_unavailable: bool,
    pub failures: Vec<(PathBuf, String)>,
    pub discovery_time: Duration,
    pub counting_time: Duration,
    pub aggregation_time: Duration,
    pub total_time: Duration,
}

impl AnalysisReport {
    pub fn cache_hit_rate(&self) -> f64 {
        let total = self.cache_hits + self.cache_misses;
        if total == 0 {
            0.0
        } else {
            self.cache_hits as f64 / total as f64
        }
    }

    /// Lines counted per second, or `None` when the run was too short to time.
    pub fn throughput_files_per_second(&self) -> Option<f64> {
        let secs = self.total_time.as_secs_f64();
        (secs > 0.0).then(|| self.files_counted as f64 / secs)
    }
}

/// The result of an analysis run.
#[derive(Debug, Clone)]
pub struct Analysis {
    pub stats: AggregatedStats,
    pub basic: CodeStats,
    /// Per-file statistics keyed by path, empty unless requested. Sorted by
    /// path.
    pub individual_files: Vec<(String, FileStats)>,
    pub report: AnalysisReport,
}

/// A file discovered by traversal, carrying the metadata the walk already read.
struct Candidate {
    path: PathBuf,
    size: u64,
    key: Option<CacheKey>,
}

/// Runs the analysis pipeline.
pub struct Engine {
    counter: CodeCounter,
    stats_calculator: StatsCalculator,
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}

impl Engine {
    pub fn new() -> Self {
        Self {
            counter: CodeCounter::new(),
            stats_calculator: StatsCalculator::new(),
        }
    }

    /// Analyze `root` under `options`.
    pub fn analyze(&self, root: &Path, options: &AnalysisOptions) -> Result<Analysis> {
        let started = Instant::now();
        let mut report = AnalysisReport::default();

        // Sherlock is a separate process that walks the tree itself, so its cost
        // overlaps with our own traversal instead of preceding it.
        let detection = self.spawn_detection(root, options, &mut report);

        let discovery_start = Instant::now();
        let filter = self.build_filter(options);
        let candidates = self.discover(&filter, root, options)?;
        report.discovery_time = discovery_start.elapsed();

        let mut candidates = self.select(candidates, root, options, detection, &mut report);

        // Sorting makes the report independent of directory order and of how
        // work happened to be distributed across threads.
        candidates.sort_unstable_by(|a, b| a.path.cmp(&b.path));
        report.files_discovered = candidates.len();

        let counting_start = Instant::now();
        let counted = self.count_all(root, &candidates, options, &mut report)?;
        report.counting_time = counting_start.elapsed();

        let aggregation_start = Instant::now();
        let analysis = self.aggregate(counted, options, report, started)?;
        let mut analysis = analysis;
        analysis.report.aggregation_time = aggregation_start.elapsed();
        analysis.report.total_time = started.elapsed();

        Ok(analysis)
    }

    /// The paths `analyze` would count, in the same order.
    ///
    /// `--list` uses this so that what it prints is exactly what gets counted,
    /// rather than a second, separately-maintained traversal that could drift.
    pub fn discover_files(&self, root: &Path, options: &AnalysisOptions) -> Result<Vec<PathBuf>> {
        let mut report = AnalysisReport::default();
        let detection = self.spawn_detection(root, options, &mut report);
        let filter = self.build_filter(options);
        let candidates = self.discover(&filter, root, options)?;
        let selected = self.select(candidates, root, options, detection, &mut report);

        let mut paths: Vec<PathBuf> = selected.into_iter().map(|c| c.path).collect();
        paths.sort_unstable();
        Ok(paths)
    }

    /// Reduce discovered files to the ones to count.
    ///
    /// Classification runs first and in parallel; the pending detection process
    /// is consulted only for the files that classification could not settle. On
    /// a tree with no such files -- which is most trees -- the process is
    /// cancelled and the run never pays for it.
    fn select(
        &self,
        candidates: Vec<Candidate>,
        root: &Path,
        options: &AnalysisOptions,
        detection: Option<DetectionJob>,
        report: &mut AnalysisReport,
    ) -> Vec<Candidate> {
        let detector = FileDetector::new().with_root(root);

        let classify = |candidate: Candidate| -> Option<(Candidate, Classification)> {
            matches_extension_filter(&candidate.path, &options.extensions)
                .then(|| (detector.classify(&candidate.path), candidate))
                .and_then(|(class, candidate)| match class {
                    Classification::Rejected => None,
                    class => Some((candidate, class)),
                })
        };

        let threads = options.parallelism.threads();
        let classified: Vec<(Candidate, Classification)> = if threads <= 1 {
            candidates.into_iter().filter_map(classify).collect()
        } else {
            // Classification touches the filesystem for extension-less files
            // (the shebang probe), so it is worth spreading over the pool.
            candidates.into_par_iter().filter_map(classify).collect()
        };

        let mut selected = Vec::with_capacity(classified.len());
        let mut unknown = Vec::new();
        for (candidate, class) in classified {
            match class {
                Classification::Source => selected.push(candidate),
                _ => unknown.push(candidate),
            }
        }

        match detection {
            Some(job) if unknown.is_empty() => job.cancel(),
            Some(job) => match job.finish() {
                Ok(result) => {
                    let detector = detector.with_sherlock_result(result);
                    selected.extend(
                        unknown
                            .into_iter()
                            .filter(|c| detector.detected_as_source(&c.path)),
                    );
                }
                Err(_) => report.detection_unavailable = true,
            },
            None => {}
        }

        selected
    }

    fn build_filter(&self, options: &AnalysisOptions) -> FileFilter {
        let mut filter = FileFilter::new()
            .respect_hidden(!options.include_hidden)
            .respect_gitignore(true);

        if let Some(depth) = options.max_depth {
            filter = filter.with_max_depth(depth);
        }
        if !options.ignore_patterns.is_empty() {
            filter = filter.with_custom_ignores(options.ignore_patterns.clone());
        }
        filter
    }

    /// Start language detection without waiting for it, if it is usable.
    fn spawn_detection(
        &self,
        root: &Path,
        options: &AnalysisOptions,
        report: &mut AnalysisReport,
    ) -> Option<DetectionJob> {
        if options.detection == DetectionMode::Disabled {
            return None;
        }

        match FileDetector::start_detection(root) {
            Ok(job) => Some(job),
            // Sherlock not being installed is the normal case, not an error.
            Err(_) => {
                report.detection_unavailable = true;
                None
            }
        }
    }

    /// Walk `root` and collect every file, with its size and cache key.
    fn discover(
        &self,
        filter: &FileFilter,
        root: &Path,
        options: &AnalysisOptions,
    ) -> Result<Vec<Candidate>> {
        let threads = options.parallelism.threads();

        if threads <= 1 {
            let mut candidates = Vec::new();
            for entry in filter.try_walk_directory(root)? {
                if let Some(candidate) = candidate_from_entry(&entry) {
                    candidates.push(candidate);
                }
            }
            return Ok(candidates);
        }

        let (sender, receiver) = std::sync::mpsc::channel();
        let walker = filter.walk_parallel(root, threads)?;

        // The collector runs on this thread's behalf while the walkers feed it,
        // so directory reads and channel drains overlap.
        let collector = std::thread::spawn(move || {
            let mut candidates: Vec<Candidate> = Vec::new();
            while let Ok(candidate) = receiver.recv() {
                candidates.push(candidate);
            }
            candidates
        });

        walker.run(|| {
            let sender = sender.clone();
            Box::new(move |result| {
                if let Ok(entry) = result {
                    if let Some(candidate) = candidate_from_entry(&entry) {
                        // A closed channel means the collector is gone; stop.
                        if sender.send(candidate).is_err() {
                            return ignore::WalkState::Quit;
                        }
                    }
                }
                ignore::WalkState::Continue
            })
        });

        drop(sender);
        Ok(collector.join().unwrap_or_default())
    }

    /// Count every candidate, in parallel unless a single thread was requested.
    fn count_all(
        &self,
        root: &Path,
        candidates: &[Candidate],
        options: &AnalysisOptions,
        report: &mut AnalysisReport,
    ) -> Result<Vec<(PathBuf, FileStats)>> {
        let cache = if options.use_cache {
            Some(FileCache::scoped(root))
        } else {
            None
        };

        let count_one = |candidate: &Candidate| -> CountOutcome {
            if let (Some(cache), Some(key)) = (cache.as_ref(), candidate.key.as_ref()) {
                if let Some(hit) = cache.get_with_key(&candidate.path, key) {
                    return CountOutcome::Hit(hit.clone());
                }
            }
            match self
                .counter
                .count_file_with_size(&candidate.path, candidate.size)
            {
                Ok(stats) => CountOutcome::Miss(stats),
                Err(err) => CountOutcome::Failed(err.to_string()),
            }
        };

        let threads = options.parallelism.threads();
        let outcomes: Vec<CountOutcome> = if threads <= 1 {
            candidates.iter().map(count_one).collect()
        } else {
            let pool = rayon::ThreadPoolBuilder::new()
                .num_threads(threads)
                .build()
                .map_err(|e| {
                    crate::utils::errors::HowManyError::counter(format!(
                        "could not start worker pool: {e}"
                    ))
                })?;
            // `collect` preserves input order regardless of scheduling, so the
            // result does not depend on the number of threads.
            pool.install(|| candidates.par_iter().map(count_one).collect())
        };

        let mut counted = Vec::with_capacity(candidates.len());
        let mut updates = Vec::new();

        for (candidate, outcome) in candidates.iter().zip(outcomes) {
            match outcome {
                CountOutcome::Hit(stats) => {
                    report.cache_hits += 1;
                    report.bytes_read += stats.file_size;
                    counted.push((candidate.path.clone(), stats));
                }
                CountOutcome::Miss(stats) => {
                    report.cache_misses += 1;
                    report.bytes_read += stats.file_size;
                    if let Some(key) = candidate.key {
                        updates.push((candidate.path.clone(), stats.clone(), key));
                    }
                    counted.push((candidate.path.clone(), stats));
                }
                CountOutcome::Failed(message) => {
                    report.files_failed += 1;
                    report.failures.push((candidate.path.clone(), message));
                }
            }
        }

        report.files_counted = counted.len();

        if let Some(mut cache) = cache {
            cache.extend_from(updates);
            cache.cleanup_missing_files();
            // A cache that cannot be written is not a reason to fail a run.
            let _ = cache.save();
        }

        Ok(counted)
    }

    fn aggregate(
        &self,
        counted: Vec<(PathBuf, FileStats)>,
        options: &AnalysisOptions,
        report: AnalysisReport,
        _started: Instant,
    ) -> Result<Analysis> {
        // Complexity analysis needs the per-file list, so it is built whenever
        // complexity is wanted -- not only when the caller wants the list back.
        // Tying the two together meant every JSON, HTML and SARIF report
        // carried a complexity section of zeros and a maintainability index of
        // 100, which reads as a perfect score rather than as "not measured".
        let keep_files = options.collect_individual_files;
        let need_files = keep_files || options.compute_complexity;

        let mut by_extension = Vec::with_capacity(counted.len());
        let mut individual_files = Vec::with_capacity(if need_files { counted.len() } else { 0 });

        for (path, stats) in counted {
            by_extension.push((counter::extension_key(&path), stats.clone()));
            if need_files {
                individual_files.push((path.to_string_lossy().to_string(), stats));
            }
        }

        let basic = counter::aggregate(by_extension);

        let files_for_stats: &[(String, FileStats)] = if options.compute_complexity {
            &individual_files
        } else {
            &[]
        };

        let stats = self
            .stats_calculator
            .calculate_project_stats(&basic, files_for_stats)?;

        Ok(Analysis {
            stats,
            basic,
            individual_files: if keep_files {
                individual_files
            } else {
                Vec::new()
            },
            report,
        })
    }
}

enum CountOutcome {
    Hit(FileStats),
    Miss(FileStats),
    Failed(String),
}

/// Build a candidate from a walk entry, skipping anything that is not a file.
///
/// `file_type()` comes from the directory read and needs no syscall;
/// `metadata()` is the one `stat` this pipeline performs per file, and its
/// results are reused by both the cache lookup and the counter.
fn candidate_from_entry(entry: &ignore::DirEntry) -> Option<Candidate> {
    if !entry.file_type().is_some_and(|ft| ft.is_file()) {
        return None;
    }

    let metadata = entry.metadata().ok()?;
    Some(Candidate {
        path: entry.path().to_path_buf(),
        size: metadata.len(),
        key: CacheKey::from_metadata(&metadata),
    })
}

fn matches_extension_filter(path: &Path, extensions: &[String]) -> bool {
    if extensions.is_empty() {
        return true;
    }
    match path.extension().and_then(|e| e.to_str()) {
        Some(ext) => extensions
            .iter()
            .any(|allowed| allowed.eq_ignore_ascii_case(ext)),
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::test_utils::TestProject;

    /// A project with enough files, languages and noise to exercise every stage.
    fn fixture(name: &str) -> TestProject {
        let project = TestProject::new(name).unwrap();
        for i in 0..24 {
            project
                .create_rust_file(&format!("src/mod{i}/lib.rs"), i % 4 + 1, i % 3)
                .unwrap();
        }
        for i in 0..12 {
            project
                .create_python_file(&format!("app/pkg{i}/main.py"), i % 3 + 1)
                .unwrap();
        }
        project
            .create_file("README.md", "# Title\n\nProse.\n")
            .unwrap();
        project.create_file("src/empty.rs", "").unwrap();
        project
            .create_file("node_modules/dep/index.js", "module.exports = 1;\n")
            .unwrap();
        project
            .create_file("target/debug/gen.rs", "fn gen() {}\n")
            .unwrap();
        project
            .create_file_binary("assets/logo.png", &[0x89, 0x50, 0x4e, 0x47])
            .unwrap();
        project
    }

    fn options() -> AnalysisOptions {
        AnalysisOptions {
            use_cache: false,
            detection: DetectionMode::Disabled,
            collect_individual_files: true,
            ..AnalysisOptions::default()
        }
    }

    #[test]
    fn analyzes_a_project_and_reports_totals() {
        let project = fixture("engine_basic");
        let analysis = Engine::new().analyze(project.path(), &options()).unwrap();

        assert!(analysis.report.files_counted > 30, "{:?}", analysis.report);
        assert!(analysis.basic.total_lines > 0);
        assert_eq!(analysis.report.files_failed, 0);
        assert!(
            analysis.basic.is_consistent(),
            "aggregate totals disagree with the per-extension breakdown"
        );
    }

    #[test]
    fn excludes_build_output_and_binaries() {
        let project = fixture("engine_exclude");
        let analysis = Engine::new().analyze(project.path(), &options()).unwrap();

        let paths: Vec<&str> = analysis
            .individual_files
            .iter()
            .map(|(p, _)| p.as_str())
            .collect();

        assert!(!paths.iter().any(|p| p.contains("node_modules")));
        assert!(!paths.iter().any(|p| p.contains("/target/")));
        assert!(!paths.iter().any(|p| p.ends_with(".png")));
        assert!(paths.iter().any(|p| p.ends_with("README.md")));
    }

    /// The property that makes parallel counting trustworthy: the number of
    /// threads must not be observable in the result.
    #[test]
    fn results_are_identical_at_every_thread_count() {
        let project = fixture("engine_threads");
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

        for threads in [2, 3, 4, 8, 16] {
            let candidate = engine
                .analyze(
                    project.path(),
                    &AnalysisOptions {
                        parallelism: Parallelism::Fixed(threads),
                        ..options()
                    },
                )
                .unwrap();

            assert_eq!(
                baseline.basic, candidate.basic,
                "totals differed with {threads} threads"
            );
            assert_eq!(
                baseline.individual_files, candidate.individual_files,
                "per-file results differed with {threads} threads"
            );
            assert_eq!(
                baseline.stats.complexity.function_count, candidate.stats.complexity.function_count,
                "function count differed with {threads} threads"
            );
        }
    }

    /// Repeated runs must agree exactly, including the order of per-file rows.
    #[test]
    fn repeated_runs_are_byte_identical() {
        let project = fixture("engine_determinism");
        let engine = Engine::new();
        let first = engine.analyze(project.path(), &options()).unwrap();

        for _ in 0..4 {
            let again = engine.analyze(project.path(), &options()).unwrap();
            assert_eq!(first.basic, again.basic);
            assert_eq!(first.individual_files, again.individual_files);
        }
    }

    #[test]
    fn individual_files_are_sorted_by_path() {
        let project = fixture("engine_sorted");
        let analysis = Engine::new().analyze(project.path(), &options()).unwrap();
        let paths: Vec<&String> = analysis.individual_files.iter().map(|(p, _)| p).collect();
        let mut sorted = paths.clone();
        sorted.sort();
        assert_eq!(paths, sorted, "per-file rows are not in a stable order");
    }

    /// The cache must be invisible in the output, and must actually hit.
    #[test]
    fn cache_changes_cost_not_results() {
        let project = fixture("engine_cache");
        let cache_dir = tempfile::tempdir().unwrap();
        let engine = Engine::new();

        let uncached = engine.analyze(project.path(), &options()).unwrap();

        let cached_options = AnalysisOptions {
            use_cache: true,
            ..options()
        };

        let guard = CacheDirGuard::set(cache_dir.path());
        let first = engine.analyze(project.path(), &cached_options).unwrap();
        let second = engine.analyze(project.path(), &cached_options).unwrap();
        drop(guard);

        assert_eq!(uncached.basic, first.basic);
        assert_eq!(uncached.basic, second.basic);
        assert_eq!(
            first.report.cache_hits, 0,
            "first run should populate, not hit"
        );
        assert_eq!(
            second.report.cache_hits, second.report.files_counted,
            "second run should be served entirely from cache"
        );
    }

    #[test]
    fn extension_filter_restricts_results() {
        let project = fixture("engine_ext");
        let analysis = Engine::new()
            .analyze(
                project.path(),
                &AnalysisOptions {
                    extensions: vec!["py".to_string()],
                    ..options()
                },
            )
            .unwrap();

        assert!(analysis.report.files_counted > 0);
        assert!(analysis
            .individual_files
            .iter()
            .all(|(p, _)| p.ends_with(".py")));
        assert_eq!(analysis.basic.stats_by_extension.len(), 1);
    }

    #[test]
    fn extension_filter_is_case_insensitive() {
        let project = TestProject::new("engine_ext_case").unwrap();
        project.create_file("a.rs", "fn a() {}\n").unwrap();

        let analysis = Engine::new()
            .analyze(
                project.path(),
                &AnalysisOptions {
                    extensions: vec!["RS".to_string()],
                    ..options()
                },
            )
            .unwrap();
        assert_eq!(analysis.report.files_counted, 1);
    }

    #[test]
    fn empty_directory_yields_empty_stats() {
        let project = TestProject::new("engine_empty").unwrap();
        let analysis = Engine::new().analyze(project.path(), &options()).unwrap();

        assert_eq!(analysis.report.files_counted, 0);
        assert_eq!(analysis.basic.total_files, 0);
        assert_eq!(analysis.basic.total_lines, 0);
        assert!(analysis.individual_files.is_empty());
        assert!(analysis.basic.is_consistent());
    }

    #[test]
    fn missing_directory_is_not_a_panic() {
        let analysis = Engine::new()
            .analyze(Path::new("/definitely/does/not/exist"), &options())
            .unwrap();
        assert_eq!(analysis.report.files_counted, 0);
    }

    /// A project rooted in a directory whose name matches a build pattern must
    /// still be analyzed. This is the end-to-end form of the bug that made the
    /// tool report zero files depending on where it was checked out.
    #[test]
    fn project_under_a_build_named_directory_is_analyzed() {
        for hostile in ["build", "tmp", "env", "bin", "target", "vendor", "dist"] {
            let outer = tempfile::tempdir().unwrap();
            let root = outer.path().join(hostile).join("myproject");
            std::fs::create_dir_all(root.join("src")).unwrap();
            std::fs::write(root.join("src/main.rs"), "fn main() {}\n").unwrap();

            let analysis = Engine::new().analyze(&root, &options()).unwrap();
            assert_eq!(
                analysis.report.files_counted, 1,
                "a project under a directory named {hostile:?} reported \
                 {} files instead of 1",
                analysis.report.files_counted
            );
        }
    }

    #[test]
    fn max_depth_limits_traversal() {
        let project = TestProject::new("engine_depth").unwrap();
        project.create_file("a.rs", "fn a() {}\n").unwrap();
        project.create_file("one/b.rs", "fn b() {}\n").unwrap();
        project.create_file("one/two/c.rs", "fn c() {}\n").unwrap();

        let analysis = Engine::new()
            .analyze(
                project.path(),
                &AnalysisOptions {
                    max_depth: Some(2),
                    ..options()
                },
            )
            .unwrap();
        assert_eq!(analysis.report.files_counted, 2);
    }

    /// Unreadable files must be reported, not silently dropped and not fatal.
    #[cfg(unix)]
    #[test]
    fn unreadable_files_are_reported_and_the_run_continues() {
        use std::os::unix::fs::PermissionsExt;

        let project = TestProject::new("engine_perm").unwrap();
        project.create_file("ok.rs", "fn ok() {}\n").unwrap();
        let locked = project
            .create_file("locked.rs", "fn locked() {}\n")
            .unwrap();
        std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o000)).unwrap();

        let analysis = Engine::new().analyze(project.path(), &options()).unwrap();

        // Running as root defeats the permission bits; only assert when it took.
        if std::fs::File::open(&locked).is_err() {
            assert_eq!(analysis.report.files_counted, 1);
            assert_eq!(analysis.report.files_failed, 1);
            assert_eq!(analysis.report.failures.len(), 1);
            assert!(analysis.report.failures[0].0.ends_with("locked.rs"));
        }

        std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o644)).unwrap();
    }

    /// Every counted file, and the aggregate, must satisfy the partition
    /// invariant that all derived ratios depend on.
    #[test]
    fn line_categories_partition_every_file_and_the_total() {
        let project = fixture("engine_invariants");
        let analysis = Engine::new().analyze(project.path(), &options()).unwrap();

        for (path, stats) in &analysis.individual_files {
            assert!(
                stats.is_consistent(),
                "line categories do not partition total for {path}: {stats:?}"
            );
        }
        assert!(analysis.basic.is_consistent());

        let summed: usize = analysis
            .individual_files
            .iter()
            .map(|(_, s)| s.total_lines)
            .sum();
        assert_eq!(summed, analysis.basic.total_lines);
    }

    #[test]
    fn report_accounts_for_every_discovered_file() {
        let project = fixture("engine_accounting");
        let analysis = Engine::new().analyze(project.path(), &options()).unwrap();
        let r = &analysis.report;
        assert_eq!(
            r.files_counted + r.files_failed,
            r.files_discovered,
            "discovered files are unaccounted for: {r:?}"
        );
        assert_eq!(r.cache_hits + r.cache_misses, r.files_counted);
    }

    #[test]
    fn disabled_detection_never_probes_for_sherlock() {
        let project = fixture("engine_no_detect");
        let analysis = Engine::new()
            .analyze(
                project.path(),
                &AnalysisOptions {
                    detection: DetectionMode::Disabled,
                    ..options()
                },
            )
            .unwrap();
        assert!(
            !analysis.report.detection_unavailable,
            "detection was disabled, so availability should not be reported"
        );
    }

    /// Results must not depend on whether the optional `sherlock` binary is
    /// installed -- that is what makes an install reproducible.
    #[test]
    fn detection_mode_does_not_change_counts_for_known_languages() {
        let project = fixture("engine_detect_equiv");
        let engine = Engine::new();

        let without = engine
            .analyze(
                project.path(),
                &AnalysisOptions {
                    detection: DetectionMode::Disabled,
                    ..options()
                },
            )
            .unwrap();
        let auto = engine
            .analyze(
                project.path(),
                &AnalysisOptions {
                    detection: DetectionMode::Auto,
                    ..options()
                },
            )
            .unwrap();

        assert_eq!(
            without.basic, auto.basic,
            "language detection changed the totals for a project made only of \
             well-known extensions; results would differ between machines"
        );
    }

    #[test]
    fn reproducible_options_are_fully_deterministic() {
        let project = fixture("engine_repro");
        let engine = Engine::new();
        let opts = AnalysisOptions {
            collect_individual_files: true,
            ..AnalysisOptions::reproducible()
        };
        let a = engine.analyze(project.path(), &opts).unwrap();
        let b = engine.analyze(project.path(), &opts).unwrap();
        assert_eq!(a.basic, b.basic);
        assert_eq!(a.individual_files, b.individual_files);
    }

    #[test]
    fn parallelism_resolves_thread_counts() {
        assert!(Parallelism::Auto.threads() >= 1);
        assert_eq!(Parallelism::Fixed(0).threads(), 1);
        assert_eq!(Parallelism::Fixed(7).threads(), 7);
    }

    /// The default must not oversubscribe. Past the measured knee, more threads
    /// make counting slower, so `Auto` is capped while an explicit `--threads`
    /// is honoured as given.
    #[test]
    fn auto_parallelism_is_capped_but_explicit_requests_are_not() {
        assert!(Parallelism::Auto.threads() <= DEFAULT_MAX_THREADS);
        assert_eq!(
            Parallelism::Fixed(DEFAULT_MAX_THREADS * 4).threads(),
            DEFAULT_MAX_THREADS * 4,
            "an explicit thread count should not be silently reduced"
        );
    }

    /// Serialised guard for the process-wide cache directory override.
    struct CacheDirGuard(Option<std::ffi::OsString>);

    impl CacheDirGuard {
        fn set(dir: &Path) -> Self {
            let previous = std::env::var_os("HOWMANY_CACHE_DIR");
            std::env::set_var("HOWMANY_CACHE_DIR", dir);
            Self(previous)
        }
    }

    impl Drop for CacheDirGuard {
        fn drop(&mut self) {
            match self.0.take() {
                Some(value) => std::env::set_var("HOWMANY_CACHE_DIR", value),
                None => std::env::remove_var("HOWMANY_CACHE_DIR"),
            }
        }
    }
}
