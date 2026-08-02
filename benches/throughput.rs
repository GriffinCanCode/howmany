//! Throughput benchmarks for the analysis pipeline.
//!
//! Each group answers one question, so a regression points at a stage rather
//! than at "the tool got slower":
//!
//! * `scan` -- lines classified per second, with no filesystem involved.
//! * `discover` -- files found per second, counting excluded.
//! * `pipeline` -- the whole run, which is what a user waits for.
//! * `threads` -- how the pipeline scales, so a scaling regression is visible.
//! * `cache` -- what a warm cache is worth on an unchanged tree.
//!
//! Corpora are generated in a temporary directory from a fixed seed, so results
//! are comparable between runs and between machines with the same core count.
//! `Throughput` is set on every group, which makes Criterion report MiB/s
//! instead of only wall time -- and, for discovery, files/s alongside it, since
//! the file count is the unit that walk cost actually tracks.

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use howmany::core::counter::{comment_patterns, scanner, CodeCounter};
use howmany::core::engine::{AnalysisOptions, DetectionMode, Engine, Parallelism};
use howmany::core::filters::FileFilter;
use std::hint::black_box;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// A generated corpus, kept alive for the duration of a benchmark.
struct Corpus {
    _dir: TempDir,
    root: PathBuf,
    files: usize,
    bytes: u64,
}

/// Deterministic pseudo-random source. A fixed algorithm keeps the corpus
/// identical across platforms, which `rand` would not guarantee.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        // xorshift64*: cheap, and stable across releases.
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    fn below(&mut self, bound: usize) -> usize {
        (self.next() % bound as u64) as usize
    }
}

const LANGUAGES: &[(&str, &str, Option<&str>)] = &[
    ("rs", "//", Some("///")),
    ("py", "#", None),
    ("js", "//", Some("/**")),
    ("go", "//", None),
    ("java", "//", Some("/**")),
    ("c", "//", Some("/**")),
];

/// Build one file's worth of source with a realistic mix of line kinds.
fn file_body(rng: &mut Rng, lines: usize, single: &str, doc: Option<&str>) -> String {
    let mut out = String::with_capacity(lines * 40);
    for index in 0..lines {
        match rng.below(100) {
            0..=11 => {}
            12..=27 => out.push_str(&format!("    {single} explanatory note {index}")),
            28..=35 => match doc {
                Some("/**") => out.push_str(&format!("/** documented {index} */")),
                Some(marker) => out.push_str(&format!("{marker} documented {index}")),
                None => out.push_str(&format!("    value_{index} = compute({index});")),
            },
            _ => out.push_str(&format!("    value_{index} = compute({index});")),
        }
        out.push('\n');
    }
    out
}

/// Generate a corpus with `files` source files spread over a realistic tree,
/// plus `files / 2` files inside directories that must be pruned.
///
/// The noise is the point of the discovery benchmark: a walk that prunes
/// `node_modules` never reads it, and the difference has to be measurable.
fn corpus(files: usize) -> Corpus {
    let dir = TempDir::new().expect("temporary directory");
    let root = dir.path().to_path_buf();
    let mut rng = Rng(0x1337_C0DE);
    let mut bytes = 0u64;

    for index in 0..files {
        let (ext, single, doc) = LANGUAGES[index % LANGUAGES.len()];
        // 8 files per directory, nested three deep: broad at the leaves.
        let parent = root
            .join(format!("crate{}", index / 512))
            .join(format!("mod{}", (index / 64) % 8))
            .join(format!("sub{}", (index / 8) % 8));
        std::fs::create_dir_all(&parent).expect("create source directory");

        let lines = 5 + rng.below(395);
        let body = file_body(&mut rng, lines, single, doc);
        bytes += body.len() as u64;
        std::fs::write(parent.join(format!("mod_{index}.{ext}")), &body).expect("write source");
    }

    for index in 0..files / 2 {
        let noise = root
            .join(["node_modules", "target", "dist", "__pycache__", ".git"][index % 5])
            .join(format!("pkg{}", index % 32));
        std::fs::create_dir_all(&noise).expect("create noise directory");
        let lines = 20 + rng.below(100);
        let body = file_body(&mut rng, lines, "//", None);
        std::fs::write(noise.join(format!("vendored_{index}.js")), &body).expect("write noise");
    }

    Corpus {
        _dir: dir,
        root,
        files,
        bytes,
    }
}

/// Options that measure the tool itself: no external detector, no cache.
fn bench_options(parallelism: Parallelism) -> AnalysisOptions {
    AnalysisOptions {
        detection: DetectionMode::Disabled,
        parallelism,
        use_cache: false,
        collect_individual_files: false,
        compute_complexity: false,
        ..AnalysisOptions::default()
    }
}

/// Line classification with no filesystem in the way.
fn bench_scan(c: &mut Criterion) {
    let mut group = c.benchmark_group("scan");

    for (ext, single, doc) in LANGUAGES {
        let mut rng = Rng(0xABCD_EF01);
        let body = file_body(&mut rng, 20_000, single, *doc);
        let pattern = comment_patterns::lookup_or_empty(ext);

        group.throughput(Throughput::Bytes(body.len() as u64));
        group.bench_with_input(BenchmarkId::new("lines", ext), &body, |b, body| {
            b.iter(|| {
                let tally = scanner::classify(&mut body.as_bytes(), pattern).expect("classify");
                black_box(tally)
            })
        });
    }

    group.finish();
}

/// Counting one file end to end, including the open and the stat.
fn bench_count_file(c: &mut Criterion) {
    let dir = TempDir::new().expect("temporary directory");
    let mut rng = Rng(0x5EED);
    let body = file_body(&mut rng, 2_000, "//", Some("///"));
    let path = dir.path().join("bench.rs");
    std::fs::write(&path, &body).expect("write source");

    let counter = CodeCounter::new();
    let mut group = c.benchmark_group("count_file");
    group.throughput(Throughput::Bytes(body.len() as u64));
    group.bench_function("2000_lines", |b| {
        b.iter(|| black_box(counter.count_file(&path).expect("count")))
    });
    group.finish();
}

/// Traversal only: how fast candidates are produced, and what pruning saves.
fn bench_discover(c: &mut Criterion) {
    let corpus = corpus(4_000);
    let engine = Engine::new();

    let mut group = c.benchmark_group("discover");
    group.sample_size(30);
    group.throughput(Throughput::ElementsAndBytes {
        elements: corpus.files as u64,
        bytes: corpus.bytes,
    });

    for (label, parallelism) in [
        ("sequential", Parallelism::Fixed(1)),
        ("parallel", Parallelism::Auto),
    ] {
        let options = bench_options(parallelism);
        group.bench_function(label, |b| {
            b.iter(|| {
                black_box(
                    engine
                        .discover_files(&corpus.root, &options)
                        .expect("discover"),
                )
            })
        });
    }

    // Without pruning, the walk reads every vendored file only to reject it.
    group.bench_function("unpruned", |b| {
        let filter = FileFilter::new().prune_build_dirs(false);
        b.iter(|| {
            let count = filter
                .try_walk_directory(&corpus.root)
                .expect("walk")
                .count();
            black_box(count)
        })
    });
    group.bench_function("pruned", |b| {
        let filter = FileFilter::new().prune_build_dirs(true);
        b.iter(|| {
            let count = filter
                .try_walk_directory(&corpus.root)
                .expect("walk")
                .count();
            black_box(count)
        })
    });

    group.finish();
}

/// The whole pipeline, at the sizes a user actually runs it on.
fn bench_pipeline(c: &mut Criterion) {
    let engine = Engine::new();
    let mut group = c.benchmark_group("pipeline");
    group.sample_size(20);

    for files in [500usize, 4_000] {
        let corpus = corpus(files);
        group.throughput(Throughput::Bytes(corpus.bytes));

        for (label, parallelism) in [
            ("sequential", Parallelism::Fixed(1)),
            ("parallel", Parallelism::Auto),
        ] {
            let options = bench_options(parallelism);
            group.bench_with_input(BenchmarkId::new(label, files), &corpus.root, |b, root| {
                b.iter(|| black_box(engine.analyze(root, &options).expect("analyze")))
            });
        }
    }

    group.finish();
}

/// Scaling. A flat or rising curve here is the signal that parallel counting
/// has stopped paying for itself.
fn bench_threads(c: &mut Criterion) {
    let corpus = corpus(4_000);
    let engine = Engine::new();

    let mut group = c.benchmark_group("threads");
    group.sample_size(20);
    group.throughput(Throughput::Bytes(corpus.bytes));

    let available = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1);
    for threads in [1, 2, 4, 8, 16]
        .into_iter()
        .filter(|t| *t <= available.max(1))
    {
        let options = bench_options(Parallelism::Fixed(threads));
        group.bench_with_input(BenchmarkId::from_parameter(threads), &threads, |b, _| {
            b.iter(|| black_box(engine.analyze(&corpus.root, &options).expect("analyze")))
        });
    }

    group.finish();
}

/// What a warm cache buys on an unchanged tree.
fn bench_cache(c: &mut Criterion) {
    let corpus = corpus(2_000);
    let cache_dir = TempDir::new().expect("temporary directory");
    std::env::set_var("HOWMANY_CACHE_DIR", cache_dir.path());

    let engine = Engine::new();
    let cold = AnalysisOptions {
        use_cache: false,
        ..bench_options(Parallelism::Auto)
    };
    let warm = AnalysisOptions {
        use_cache: true,
        ..bench_options(Parallelism::Auto)
    };

    // Populate before measuring, so the warm case is genuinely warm.
    engine.analyze(&corpus.root, &warm).expect("populate cache");

    let mut group = c.benchmark_group("cache");
    group.sample_size(20);
    group.throughput(Throughput::Bytes(corpus.bytes));
    group.bench_function("cold", |b| {
        b.iter(|| black_box(engine.analyze(&corpus.root, &cold).expect("analyze")))
    });
    group.bench_function("warm", |b| {
        b.iter(|| black_box(engine.analyze(&corpus.root, &warm).expect("analyze")))
    });
    group.finish();

    std::env::remove_var("HOWMANY_CACHE_DIR");
}

/// What language detection costs when it cannot change the answer.
///
/// Detection is a separate process that re-walks the whole tree, and blocking on
/// it used to account for roughly three quarters of a run. It is now started but
/// cancelled once discovery shows that every file was classified without it, so
/// `auto` and `disabled` should be indistinguishable on a corpus of well-known
/// extensions. A gap reappearing here means the cancellation stopped working.
fn bench_detection(c: &mut Criterion) {
    let corpus = corpus(4_000);
    let engine = Engine::new();

    let mut group = c.benchmark_group("detection");
    group.sample_size(20);
    group.throughput(Throughput::Bytes(corpus.bytes));

    for (label, detection) in [
        ("disabled", DetectionMode::Disabled),
        ("auto", DetectionMode::Auto),
    ] {
        let options = AnalysisOptions {
            detection,
            ..bench_options(Parallelism::Auto)
        };
        group.bench_function(label, |b| {
            b.iter(|| black_box(engine.analyze(&corpus.root, &options).expect("analyze")))
        });
    }

    group.finish();
}

/// What the complexity and quality metrics cost on top of counting.
///
/// Reports that print those metrics now compute them, so this is the price of
/// the fix and the number to watch if a report path gets slow.
fn bench_complexity(c: &mut Criterion) {
    let corpus = corpus(2_000);
    let engine = Engine::new();

    let mut group = c.benchmark_group("complexity");
    group.sample_size(20);
    group.throughput(Throughput::Bytes(corpus.bytes));

    for (label, compute) in [("counts_only", false), ("with_complexity", true)] {
        let options = AnalysisOptions {
            compute_complexity: compute,
            ..bench_options(Parallelism::Auto)
        };
        group.bench_function(label, |b| {
            b.iter(|| black_box(engine.analyze(&corpus.root, &options).expect("analyze")))
        });
    }

    group.finish();
}

/// A corpus of files the counter has to handle without a fast path: no trailing
/// newline, CRLF, invalid UTF-8, one enormous line.
fn bench_pathological(c: &mut Criterion) {
    let dir = TempDir::new().expect("temporary directory");
    let root: &Path = dir.path();

    std::fs::write(
        root.join("huge_line.rs"),
        format!("// {}\n", "x".repeat(4_000_000)),
    )
    .expect("write");
    std::fs::write(
        root.join("crlf.rs"),
        "fn main() {}\r\n// c\r\n\r\n".repeat(20_000),
    )
    .expect("write");
    std::fs::write(
        root.join("not_utf8.rs"),
        [b"fn a() {}\n\xff\xfe\x80\n".as_slice(); 20_000].concat(),
    )
    .expect("write");

    let engine = Engine::new();
    let options = bench_options(Parallelism::Auto);

    let mut group = c.benchmark_group("pathological");
    group.sample_size(20);
    group.bench_function("mixed", |b| {
        b.iter(|| black_box(engine.analyze(root, &options).expect("analyze")))
    });
    group.finish();
}

criterion_group!(
    benches,
    bench_scan,
    bench_count_file,
    bench_discover,
    bench_pipeline,
    bench_threads,
    bench_cache,
    bench_detection,
    bench_complexity,
    bench_pathological
);
criterion_main!(benches);
