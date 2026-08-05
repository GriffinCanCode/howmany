pub mod comment_patterns;
pub mod scanner;

pub use comment_patterns::CommentPattern;
pub use scanner::LineTally;

use crate::core::stats::{AggregatedStats, StatsCalculator};
use crate::core::types::{CodeStats, FileStats};
use crate::utils::cache::{CacheKey, FileCache};
use crate::utils::errors::{HowManyError, Result};
use std::borrow::Cow;
use std::fs;
use std::io::BufReader;
use std::path::Path;

/// Read buffer size for line scanning.
///
/// Source files cluster well below this, so most files are read in one syscall.
const READ_BUFFER_BYTES: usize = 64 * 1024;

/// Lowercase `s`, borrowing when it is already lowercase.
fn lower(s: &str) -> Cow<'_, str> {
    if s.bytes().any(|b| b.is_ascii_uppercase()) {
        Cow::Owned(s.to_lowercase())
    } else {
        Cow::Borrowed(s)
    }
}

/// Lowercase `path`'s extension, borrowing when it is already lowercase.
fn lower_extension(path: &Path) -> Option<Cow<'_, str>> {
    path.extension()?.to_str().map(lower)
}

/// The comment-syntax key for `path`.
///
/// Its lowercase extension, or -- for a file that has none -- its lowercase
/// name without a leading dot. That is what lets `Dockerfile`, `Makefile` and
/// `.editorconfig` be classified with their real comment syntax instead of
/// being treated as an unknown format whose every line is code.
pub fn classify_key(path: &Path) -> Cow<'_, str> {
    lower_extension(path)
        .or_else(|| {
            let name = path.file_name()?.to_str()?;
            Some(lower(name.strip_prefix('.').unwrap_or(name)))
        })
        .unwrap_or(Cow::Borrowed(""))
}

/// The syntax key implied by a `#!` line.
///
/// An extension-less `bootstrap` or `configure` script is ordinary source, and
/// its shebang says which language. Without this its comments were counted as
/// code, because no extension meant no comment syntax.
pub fn shebang_key(head: &[u8]) -> Option<&'static str> {
    let rest = head.strip_prefix(b"#!")?;
    let line = &rest[..rest.iter().position(|b| *b == b'\n').unwrap_or(rest.len())];
    let text = std::str::from_utf8(line).ok()?;

    // `#!/usr/bin/env -S python3 -u` names the interpreter after the `env` and
    // any of its flags.
    let interpreter = text
        .split_whitespace()
        .find(|word| !word.starts_with('-') && !word.ends_with("env"))?
        .rsplit('/')
        .next()?;

    Some(
        match interpreter.trim_end_matches(|c: char| c.is_ascii_digit() || c == '.') {
            "sh" | "bash" | "dash" | "ksh" | "ash" => "sh",
            "zsh" => "zsh",
            "fish" => "fish",
            "python" | "python-" => "py",
            "node" | "deno" | "bun" => "js",
            "ruby" => "rb",
            "perl" => "pl",
            "php" => "php",
            "lua" => "lua",
            "Rscript" => "r",
            "pwsh" | "powershell" => "ps1",
            "tclsh" | "wish" => "sh",
            _ => return None,
        },
    )
}

/// Classify `reader` under the syntax registered for `key`.
///
/// When `key` is unknown the reader's buffered head is consulted for a shebang.
/// `fill_buf` peeks, so this costs no extra syscall and consumes nothing.
fn classify_with<R: std::io::BufRead>(reader: &mut R, key: &str) -> Result<scanner::LineTally> {
    let resolved = if comment_patterns::is_known(key) {
        Cow::Borrowed(key)
    } else {
        match shebang_key(reader.fill_buf()?) {
            Some(from_shebang) => Cow::Borrowed(from_shebang),
            None => Cow::Borrowed(key),
        }
    };

    Ok(if comment_patterns::is_prose_format(&resolved) {
        scanner::classify_markdown(reader)?
    } else {
        scanner::classify(reader, comment_patterns::lookup_or_empty(&resolved))?
    })
}

/// Classify `bytes` under the syntax registered for `key`.
fn classify_bytes_with(bytes: &[u8], key: &str) -> scanner::LineTally {
    let resolved = if comment_patterns::is_known(key) {
        Cow::Borrowed(key)
    } else {
        Cow::Borrowed(shebang_key(bytes).unwrap_or(key))
    };

    if comment_patterns::is_prose_format(&resolved) {
        scanner::classify_markdown_bytes(bytes)
    } else {
        scanner::classify_bytes(bytes, comment_patterns::lookup_or_empty(&resolved))
    }
}

/// Files up to this size are read whole into a reused buffer; larger ones are
/// streamed so that one enormous file cannot dictate a worker's memory use.
///
/// Source files are overwhelmingly smaller than this -- the threshold exists for
/// the vendored bundle and the checked-in dataset, not for code.
const WHOLE_FILE_LIMIT: u64 = 8 * 1024 * 1024;

thread_local! {
    /// Per-worker scratch space for whole-file reads.
    ///
    /// Reused across every file a thread handles, so counting a repository
    /// performs a handful of allocations rather than one per file. The previous
    /// `BufReader::with_capacity(64 KiB)` per file made the allocator the
    /// bottleneck: counting on all sixteen threads of a 12+4 core machine was
    /// slower than counting on four.
    static READ_SCRATCH: std::cell::RefCell<Vec<u8>> =
        std::cell::RefCell::new(Vec::with_capacity(READ_BUFFER_BYTES));
}

/// Read `file` of known `size` into `buf`, replacing its contents.
fn read_into(file: &mut fs::File, size: u64, buf: &mut Vec<u8>) -> Result<()> {
    use std::io::Read;
    buf.clear();
    buf.reserve(size as usize + 1);
    file.read_to_end(buf)?;
    Ok(())
}

/// The error a caller gets for handing a directory to a file counter.
fn is_a_directory(path: &Path) -> HowManyError {
    HowManyError::file_processing(format!("{} is a directory, not a file", path.display()))
}

/// Counts lines of code, comments, documentation and blanks.
///
/// Construction is free -- all comment tables are process-wide statics -- so one
/// counter per worker thread costs nothing.
pub struct CodeCounter {
    stats_calculator: StatsCalculator,
}

impl Default for CodeCounter {
    fn default() -> Self {
        Self::new()
    }
}

impl CodeCounter {
    pub fn new() -> Self {
        Self {
            stats_calculator: StatsCalculator::new(),
        }
    }

    /// Comment syntax registered for `extension`, if any.
    pub fn comment_pattern(&self, extension: &str) -> Option<CommentPattern> {
        comment_patterns::lookup(extension)
    }

    /// True when `extension` has registered comment syntax.
    pub fn supports_extension(&self, extension: &str) -> bool {
        comment_patterns::is_known(extension)
    }

    /// Count one file.
    ///
    /// A single `open` supplies both the contents and the size; the previous
    /// implementation opened the file and then issued a second `stat` for its
    /// length, doubling the syscalls per file.
    pub fn count_file(&self, path: &Path) -> Result<FileStats> {
        // Unix opens a directory happily and fails only on the first read;
        // Windows refuses the open outright with a permission error. Checking
        // on the way out of a failed open costs nothing on the happy path and
        // makes the error identical on every platform.
        let file = fs::File::open(path).map_err(|err| {
            if path.is_dir() {
                is_a_directory(path)
            } else {
                err.into()
            }
        })?;
        let metadata = file.metadata()?;

        if metadata.is_dir() {
            return Err(is_a_directory(path));
        }

        let file_size = metadata.len();
        let mut reader = BufReader::with_capacity(READ_BUFFER_BYTES, file);
        let tally = classify_with(&mut reader, &classify_key(path))?;

        Ok(tally.into_file_stats(file_size))
    }

    /// Count a file whose size is already known, skipping the metadata call.
    ///
    /// Traversal already paid for the size; reusing it removes a `stat` per
    /// file from the hot path.
    pub fn count_file_with_size(&self, path: &Path, file_size: u64) -> Result<FileStats> {
        let mut file = fs::File::open(path)?;
        let key = classify_key(path);

        if file_size > WHOLE_FILE_LIMIT {
            let mut reader = BufReader::with_capacity(READ_BUFFER_BYTES, file);
            let tally = classify_with(&mut reader, &key)?;
            return Ok(tally.into_file_stats(file_size));
        }

        READ_SCRATCH.with(|scratch| {
            let mut buf = scratch.borrow_mut();
            read_into(&mut file, file_size, &mut buf)?;
            Ok(classify_bytes_with(&buf, &key).into_file_stats(file_size))
        })
    }

    /// Classify an in-memory buffer as if it had extension `extension`.
    ///
    /// Exposed so that classification can be exercised without touching disk.
    pub fn count_bytes(&self, contents: &[u8], extension: &str) -> Result<FileStats> {
        let mut reader = contents;
        let tally = classify_with(&mut reader, &lower(extension))?;
        Ok(tally.into_file_stats(contents.len() as u64))
    }

    /// Comprehensive statistics for a single file.
    pub fn calculate_file_stats(&self, path: &Path) -> Result<AggregatedStats> {
        let file_stats = self.count_file(path)?;
        let path_str = path.to_string_lossy().to_string();

        let start_time = std::time::Instant::now();
        let mut aggregated_stats = self
            .stats_calculator
            .calculate_file_stats(&file_stats, &path_str)?;
        crate::core::stats::aggregation::StatsAggregator::update_timing(
            &mut aggregated_stats,
            start_time,
        );

        Ok(aggregated_stats)
    }

    /// Comprehensive statistics for a project.
    pub fn calculate_project_stats(
        &self,
        code_stats: &CodeStats,
        individual_files: &[(String, FileStats)],
    ) -> Result<AggregatedStats> {
        let start_time = std::time::Instant::now();
        let mut aggregated_stats = self
            .stats_calculator
            .calculate_project_stats(code_stats, individual_files)?;
        crate::core::stats::aggregation::StatsAggregator::update_timing(
            &mut aggregated_stats,
            start_time,
        );

        Ok(aggregated_stats)
    }

    pub fn stats_calculator(&self) -> &StatsCalculator {
        &self.stats_calculator
    }

    /// Sum per-file statistics into project totals, keyed by extension.
    pub fn aggregate_stats(&self, file_stats: Vec<(String, FileStats)>) -> CodeStats {
        aggregate(file_stats)
    }
}

/// Sum per-file statistics into project totals, keyed by extension.
pub fn aggregate<I>(file_stats: I) -> CodeStats
where
    I: IntoIterator<Item = (String, FileStats)>,
{
    let mut totals = CodeStats::default();

    for (extension, stats) in file_stats {
        totals.total_files += 1;
        totals.total_lines += stats.total_lines;
        totals.total_code_lines += stats.code_lines;
        totals.total_comment_lines += stats.comment_lines;
        totals.total_blank_lines += stats.blank_lines;
        totals.total_size += stats.file_size;
        totals.total_doc_lines += stats.doc_lines;

        let entry = totals
            .stats_by_extension
            .entry(extension)
            .or_insert_with(|| (0, FileStats::default()));

        entry.0 += 1;
        entry.1.total_lines += stats.total_lines;
        entry.1.code_lines += stats.code_lines;
        entry.1.comment_lines += stats.comment_lines;
        entry.1.blank_lines += stats.blank_lines;
        entry.1.file_size += stats.file_size;
        entry.1.doc_lines += stats.doc_lines;
    }

    totals
}

/// A [`CodeCounter`] that reuses results for files that have not changed.
pub struct CachedCodeCounter {
    counter: CodeCounter,
    cache: FileCache,
    cache_hits: usize,
    cache_misses: usize,
}

impl Default for CachedCodeCounter {
    fn default() -> Self {
        Self::new()
    }
}

impl CachedCodeCounter {
    /// A counter whose cache lives only as long as it does.
    pub fn new() -> Self {
        Self::with_cache(FileCache::new())
    }

    /// A counter backed by the on-disk cache for `root`.
    ///
    /// The cache is scoped to the project so that its load and save cost is
    /// proportional to that project rather than to every directory the machine
    /// has ever analyzed.
    pub fn for_root(root: &Path) -> Self {
        Self::with_cache(FileCache::scoped(root))
    }

    /// Build a counter over an explicit cache, bypassing the on-disk one.
    pub fn with_cache(cache: FileCache) -> Self {
        Self {
            counter: CodeCounter::new(),
            cache,
            cache_hits: 0,
            cache_misses: 0,
        }
    }

    pub fn count_file(&mut self, path: &Path) -> Result<FileStats> {
        let key = CacheKey::for_path(path);

        if let Some(cached) = key.as_ref().and_then(|k| self.cache.get_with_key(path, k)) {
            self.cache_hits += 1;
            return Ok(cached.clone());
        }

        self.cache_misses += 1;
        let file_stats = match &key {
            Some(k) => self.counter.count_file_with_size(path, k.size)?,
            None => self.counter.count_file(path)?,
        };

        if let Some(k) = key {
            self.cache
                .insert_with_key(path.to_path_buf(), file_stats.clone(), k);
        }

        Ok(file_stats)
    }

    pub fn save_cache(&self) -> Result<()> {
        self.cache.save()
    }

    pub fn cleanup_cache(&mut self) {
        self.cache.cleanup_missing_files();
    }

    pub fn cache_size(&self) -> usize {
        self.cache.size()
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    pub fn cache_stats(&self) -> (usize, usize) {
        (self.cache_hits, self.cache_misses)
    }

    pub fn cache_hit_rate(&self) -> f64 {
        let total = self.cache_hits + self.cache_misses;
        if total > 0 {
            self.cache_hits as f64 / total as f64
        } else {
            0.0
        }
    }

    pub fn aggregate_stats(&self, file_stats: Vec<(String, FileStats)>) -> CodeStats {
        self.counter.aggregate_stats(file_stats)
    }

    pub fn calculate_file_stats(&self, path: &Path) -> Result<AggregatedStats> {
        self.counter.calculate_file_stats(path)
    }

    pub fn calculate_project_stats(
        &self,
        code_stats: &CodeStats,
        individual_files: &[(String, FileStats)],
    ) -> Result<AggregatedStats> {
        self.counter
            .calculate_project_stats(code_stats, individual_files)
    }

    pub fn stats_calculator(&self) -> &StatsCalculator {
        self.counter.stats_calculator()
    }

    /// Hand back the cache so it can be persisted or merged elsewhere.
    pub fn into_cache(self) -> FileCache {
        self.cache
    }
}

/// The bucket a file is reported under.
///
/// This is [`classify_key`] narrowed to keys that mean something to a reader.
/// Two differences from the raw extension it replaced:
///
/// *Case is normalized*, so a tree containing both `Foo.RS` and `bar.rs` is one
/// language rather than two rows that do not visibly differ.
///
/// *A recognized extension-less file keeps its name.* Every `Dockerfile` and
/// `Makefile` used to aggregate into a single anonymous `no_ext` bucket
/// alongside whatever else had no suffix, even though the counter had already
/// identified them well enough to apply their comment syntax. An extension-less
/// file that nothing recognizes still falls back to `no_ext`, so a tree of
/// one-off scripts cannot fan the report out into a row per filename.
pub fn extension_key(path: &Path) -> String {
    let key = classify_key(path);
    if path.extension().is_some() || comment_patterns::is_known(&key) {
        key.into_owned()
    } else {
        "no_ext".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::test_utils::TestProject;
    use std::collections::BTreeMap;

    #[test]
    fn test_rust_file_counting() {
        let project = TestProject::new("test_rust").unwrap();
        let file_path = project.create_rust_file("test.rs", 2, 3).unwrap();

        let counter = CodeCounter::new();
        let stats = counter.count_file(&file_path).unwrap();

        assert!(stats.total_lines > 0);
        assert!(stats.code_lines > 0);
        assert!(stats.comment_lines > 0);
        assert!(stats.doc_lines > 0);
        assert!(stats.blank_lines > 0);
    }

    #[test]
    fn test_python_file_counting() {
        let project = TestProject::new("test_python").unwrap();
        let file_path = project.create_python_file("test.py", 2).unwrap();

        let counter = CodeCounter::new();
        let stats = counter.count_file(&file_path).unwrap();

        assert!(stats.total_lines > 0);
        assert!(stats.code_lines > 0);
        assert!(stats.comment_lines > 0);
        assert!(stats.doc_lines > 0); // Python docstrings
    }

    #[test]
    fn test_javascript_file_counting() {
        let project = TestProject::new("test_javascript").unwrap();
        let file_path = project.create_javascript_file("test.js", 2).unwrap();

        let counter = CodeCounter::new();
        let stats = counter.count_file(&file_path).unwrap();

        assert!(stats.total_lines > 0);
        assert!(stats.code_lines > 0);
        assert!(stats.comment_lines > 0);
        assert!(stats.doc_lines > 0); // JSDoc comments
    }

    #[test]
    fn test_markdown_file_counting() {
        let project = TestProject::new("test_markdown").unwrap();
        let content = r#"# Title

This is documentation content.

```rust
fn main() {
    println!("Hello, world!");
}
```

More documentation.

<!-- HTML comment -->
"#;
        let file_path = project.create_file("test.md", content).unwrap();

        let counter = CodeCounter::new();
        let stats = counter.count_file(&file_path).unwrap();

        assert!(stats.total_lines > 0);
        assert!(stats.code_lines > 0); // Code blocks
        assert!(stats.comment_lines > 0); // HTML comments
        assert!(stats.doc_lines > 0); // Markdown content
    }

    #[test]
    fn test_empty_file() {
        let project = TestProject::new("test_empty").unwrap();
        let file_path = project.create_file("empty.rs", "").unwrap();

        let counter = CodeCounter::new();
        let stats = counter.count_file(&file_path).unwrap();

        assert_eq!(stats.total_lines, 0);
        assert_eq!(stats.code_lines, 0);
        assert_eq!(stats.comment_lines, 0);
        assert_eq!(stats.doc_lines, 0);
        assert_eq!(stats.blank_lines, 0);
        assert_eq!(stats.file_size, 0);
    }

    #[test]
    fn test_only_blank_lines() {
        let project = TestProject::new("test_blank").unwrap();
        let file_path = project.create_file("blank.rs", "\n\n\n\n").unwrap();

        let counter = CodeCounter::new();
        let stats = counter.count_file(&file_path).unwrap();

        assert_eq!(stats.total_lines, 4);
        assert_eq!(stats.code_lines, 0);
        assert_eq!(stats.comment_lines, 0);
        assert_eq!(stats.doc_lines, 0);
        assert_eq!(stats.blank_lines, 4);
    }

    #[test]
    fn test_aggregation() {
        let counter = CodeCounter::new();

        let file_stats = vec![
            (
                "rs".to_string(),
                FileStats {
                    total_lines: 100,
                    code_lines: 70,
                    comment_lines: 20,
                    blank_lines: 10,
                    file_size: 1000,
                    doc_lines: 15,
                },
            ),
            (
                "rs".to_string(),
                FileStats {
                    total_lines: 50,
                    code_lines: 35,
                    comment_lines: 10,
                    blank_lines: 5,
                    file_size: 500,
                    doc_lines: 8,
                },
            ),
            (
                "py".to_string(),
                FileStats {
                    total_lines: 80,
                    code_lines: 60,
                    comment_lines: 15,
                    blank_lines: 5,
                    file_size: 800,
                    doc_lines: 12,
                },
            ),
        ];

        let aggregated = counter.aggregate_stats(file_stats);

        assert_eq!(aggregated.total_files, 3);
        assert_eq!(aggregated.total_lines, 230);
        assert_eq!(aggregated.total_code_lines, 165);
        assert_eq!(aggregated.total_comment_lines, 45);
        assert_eq!(aggregated.total_blank_lines, 20);
        assert_eq!(aggregated.total_size, 2300);
        assert_eq!(aggregated.total_doc_lines, 35);

        assert_eq!(aggregated.stats_by_extension.len(), 2);

        let rust_stats = &aggregated.stats_by_extension["rs"];
        assert_eq!(rust_stats.0, 2);
        assert_eq!(rust_stats.1.total_lines, 150);

        let python_stats = &aggregated.stats_by_extension["py"];
        assert_eq!(python_stats.0, 1);
        assert_eq!(python_stats.1.total_lines, 80);
    }

    /// Aggregation must be a pure sum: whatever order files arrive in, the
    /// totals are identical. Parallel counting relies on this.
    #[test]
    fn aggregation_is_order_independent() {
        let counter = CodeCounter::new();
        let mut files: Vec<(String, FileStats)> = (0..50)
            .map(|i| {
                (
                    if i % 3 == 0 { "rs" } else { "py" }.to_string(),
                    FileStats {
                        total_lines: i + 1,
                        code_lines: i,
                        comment_lines: 1,
                        blank_lines: 0,
                        file_size: (i as u64) * 10,
                        doc_lines: 0,
                    },
                )
            })
            .collect();

        let forward = counter.aggregate_stats(files.clone());
        files.reverse();
        let reversed = counter.aggregate_stats(files);

        assert_eq!(forward, reversed);
    }

    #[test]
    fn test_comment_patterns() {
        let counter = CodeCounter::new();

        let rust_pattern = counter.comment_pattern("rs").unwrap();
        assert!(rust_pattern.single_line.contains(&"//"));
        assert!(rust_pattern.doc_patterns.contains(&"///"));
        assert!(rust_pattern.doc_patterns.contains(&"//!"));

        let python_pattern = counter.comment_pattern("py").unwrap();
        assert!(python_pattern.single_line.contains(&"#"));
        assert!(python_pattern.doc_patterns.contains(&"\"\"\""));

        let js_pattern = counter.comment_pattern("js").unwrap();
        assert!(js_pattern.single_line.contains(&"//"));
        assert!(js_pattern.doc_patterns.contains(&"/**"));
    }

    #[test]
    fn test_new_language_patterns() {
        let counter = CodeCounter::new();

        let ps_pattern = counter.comment_pattern("ps1").unwrap();
        assert!(ps_pattern.single_line.contains(&"#"));
        assert!(ps_pattern.multi_line_start.contains(&"<#"));

        let elm_pattern = counter.comment_pattern("elm").unwrap();
        assert!(elm_pattern.single_line.contains(&"--"));
        assert!(elm_pattern.multi_line_start.contains(&"{-"));
        assert!(elm_pattern.doc_patterns.contains(&"{-|"));

        let julia_pattern = counter.comment_pattern("jl").unwrap();
        assert!(julia_pattern.single_line.contains(&"#"));
        assert!(julia_pattern.multi_line_start.contains(&"#="));

        let sql_pattern = counter.comment_pattern("sql").unwrap();
        assert!(sql_pattern.single_line.contains(&"--"));
        assert!(sql_pattern.multi_line_start.contains(&"/*"));

        let elixir_pattern = counter.comment_pattern("ex").unwrap();
        assert!(elixir_pattern.single_line.contains(&"#"));
        assert!(elixir_pattern.doc_patterns.contains(&"@doc"));

        assert!(counter
            .comment_pattern("yaml")
            .unwrap()
            .single_line
            .contains(&"#"));

        let zig_pattern = counter.comment_pattern("zig").unwrap();
        assert!(zig_pattern.single_line.contains(&"//"));
        assert!(zig_pattern.doc_patterns.contains(&"///"));

        let clj_pattern = counter.comment_pattern("clj").unwrap();
        assert!(clj_pattern.single_line.contains(&";"));
        assert!(clj_pattern.doc_patterns.contains(&";;"));

        let fs_pattern = counter.comment_pattern("fs").unwrap();
        assert!(fs_pattern.single_line.contains(&"//"));
        assert!(fs_pattern.multi_line_start.contains(&"(*"));
        assert!(fs_pattern.doc_patterns.contains(&"///"));

        let dart_pattern = counter.comment_pattern("dart").unwrap();
        assert!(dart_pattern.single_line.contains(&"//"));
        assert!(dart_pattern.doc_patterns.contains(&"///"));

        let matlab_pattern = counter.comment_pattern("m").unwrap();
        assert!(matlab_pattern.single_line.contains(&"%"));
        assert!(matlab_pattern.doc_patterns.contains(&"%%"));

        let r_pattern = counter.comment_pattern("r").unwrap();
        assert!(r_pattern.single_line.contains(&"#"));
        assert!(r_pattern.doc_patterns.contains(&"#'"));
    }

    #[test]
    fn test_mixed_comment_types() {
        let project = TestProject::new("test_mixed").unwrap();
        let content = r#"
// Single line comment
/* Multi-line comment
   continues here */
/// Documentation comment
fn main() {
    // Another comment
    println!("Hello, world!");
    /* Inline comment */ let x = 5;
}
"#;
        let file_path = project.create_file("test.rs", content).unwrap();

        let counter = CodeCounter::new();
        let stats = counter.count_file(&file_path).unwrap();

        assert!(stats.comment_lines >= 4);
        assert!(stats.doc_lines >= 1);
        assert!(stats.code_lines >= 3);
    }

    #[test]
    fn test_multiline_strings_vs_comments() {
        let project = TestProject::new("test_multiline").unwrap();
        let content = r#"
def test_function():
    """This is a docstring
    that spans multiple lines
    and should be counted as doc"""
    # This is a comment
    code = '''This is a string
    not a comment'''
    return code
"#;
        let file_path = project.create_file("test.py", content).unwrap();

        let counter = CodeCounter::new();
        let stats = counter.count_file(&file_path).unwrap();

        assert!(stats.doc_lines >= 3);
        assert!(stats.comment_lines >= 1);
        assert!(stats.code_lines >= 3);
    }

    #[test]
    fn test_file_extension_detection() {
        let project = TestProject::new("test_extensions").unwrap();

        let files = vec![
            ("test.rs", "fn main() {}", "rs"),
            ("test.py", "def main():", "py"),
            ("test.js", "function main() {}", "js"),
            ("test.ts", "function main(): void {}", "ts"),
            ("test.java", "public class Test {}", "java"),
            ("test.cpp", "int main() {}", "cpp"),
            ("test.c", "int main() {}", "c"),
            ("test.go", "func main() {}", "go"),
            ("test.rb", "def main", "rb"),
            ("test.php", "<?php function main() {}", "php"),
            ("test.cs", "public class Test {}", "cs"),
            ("test.swift", "func main() {}", "swift"),
            ("test.kt", "fun main() {}", "kt"),
            ("test.scala", "object Main {}", "scala"),
            ("test.md", "# Header", "md"),
            ("test.html", "<html></html>", "html"),
            ("test.css", "body { color: red; }", "css"),
            ("test.json", "{\"key\": \"value\"}", "json"),
            ("test.xml", "<root></root>", "xml"),
            ("test.yaml", "key: value", "yaml"),
            ("test.yml", "key: value", "yml"),
            ("test.toml", "key = \"value\"", "toml"),
        ];

        let counter = CodeCounter::new();

        for (filename, content, expected_ext) in files {
            let file_path = project.create_file(filename, content).unwrap();
            let stats = counter.count_file(&file_path).unwrap();

            assert!(
                stats.total_lines > 0,
                "File {} should have content",
                filename
            );
            assert!(
                counter.supports_extension(expected_ext),
                "missing comment patterns for {expected_ext}"
            );
        }
    }

    /// Extensions differing only in case must classify identically.
    #[test]
    fn extension_matching_is_case_insensitive() {
        let project = TestProject::new("case").unwrap();
        let counter = CodeCounter::new();
        let content = "// comment\nfn main() {}\n";

        let lower = counter
            .count_file(&project.create_file("a.rs", content).unwrap())
            .unwrap();
        let upper = counter
            .count_file(&project.create_file("b.RS", content).unwrap())
            .unwrap();

        assert_eq!(lower, upper, "uppercase extension classified differently");
    }

    #[test]
    fn test_binary_file_handling() {
        let project = TestProject::new("test_binary").unwrap();

        let binary_content = vec![0u8, 1, 2, 3, 255, 254, 253];
        let file_path = project.root.join("binary.bin");
        std::fs::write(&file_path, binary_content).unwrap();

        let counter = CodeCounter::new();
        let stats = counter.count_file(&file_path).unwrap();

        // No newline, so the whole buffer is one line.
        assert_eq!(stats.total_lines, 1);
        assert!(stats.is_consistent());
    }

    /// A file that is not valid UTF-8 must be counted, not dropped. Previously
    /// the read failed and the file disappeared from the totals entirely.
    #[test]
    fn non_utf8_files_are_counted() {
        let project = TestProject::new("non_utf8").unwrap();
        let file_path = project.root.join("latin1.rs");
        std::fs::write(&file_path, b"// caf\xe9\nfn main() {}\n").unwrap();

        let counter = CodeCounter::new();
        let stats = counter.count_file(&file_path).unwrap();

        assert_eq!(stats.total_lines, 2);
        assert_eq!(stats.comment_lines, 1);
        assert_eq!(stats.code_lines, 1);
    }

    #[test]
    fn test_very_long_lines() {
        let project = TestProject::new("test_long_lines").unwrap();

        let long_line = "// ".to_string() + &"x".repeat(10000);
        let content = format!("{}\nfn main() {{}}\n{}", long_line, long_line);
        let file_path = project.create_file("long.rs", &content).unwrap();

        let counter = CodeCounter::new();
        let stats = counter.count_file(&file_path).unwrap();

        assert_eq!(stats.total_lines, 3);
        assert_eq!(stats.comment_lines, 2);
        assert_eq!(stats.code_lines, 1);
    }

    #[test]
    fn test_nested_comments() {
        let project = TestProject::new("test_nested").unwrap();
        let content = r#"
/* Outer comment
   /* Nested comment */
   Still outer comment */
fn main() {
    // Regular comment
}
"#;
        let file_path = project.create_file("nested.rs", content).unwrap();

        let counter = CodeCounter::new();
        let stats = counter.count_file(&file_path).unwrap();

        assert!(stats.comment_lines >= 3);
        assert!(stats.code_lines >= 2);
    }

    #[test]
    fn test_comment_patterns_comprehensive() {
        let counter = CodeCounter::new();

        let expected_languages = vec![
            "rs", "py", "js", "ts", "jsx", "tsx", "java", "c", "cpp", "cc", "cxx", "h", "hpp",
            "cs", "go", "rb", "php", "swift", "kt", "scala", "html", "css", "scss", "sass", "md",
            "yaml", "yml", "json", "toml", "xml", "sh", "bash", "zsh", "fish", "ps1", "elm", "jl",
            "sql", "ex", "exs", "zig", "clj", "cljs", "fs", "fsx", "fsi",
        ];

        for lang in expected_languages {
            let pattern = counter
                .comment_pattern(lang)
                .unwrap_or_else(|| panic!("Missing comment patterns for language: {lang}"));

            if !pattern.single_line.is_empty() {
                assert!(
                    !pattern.single_line[0].is_empty(),
                    "Empty single-line comment pattern for {lang}"
                );
            }
        }
    }

    #[test]
    fn test_code_vs_comment_detection() {
        let project = TestProject::new("test_detection").unwrap();
        let content = r#"
fn main() {
    let url = "https://example.com"; // Not a comment marker in string
    let comment = "// This is not a comment";
    // This IS a comment
    println!("/* Not a comment */");
    /* This IS a comment */
    let regex = r"//.*"; // Regex pattern, not comment
}
"#;
        let file_path = project.create_file("tricky.rs", content).unwrap();

        let counter = CodeCounter::new();
        let stats = counter.count_file(&file_path).unwrap();

        assert!(stats.comment_lines >= 2);
        assert!(stats.code_lines >= 4);
    }

    #[test]
    fn test_calculate_file_stats_comprehensive() {
        let project = TestProject::new("test_comprehensive").unwrap();
        let file_path = project
            .create_rust_file("comprehensive.rs", 20, 10)
            .unwrap();

        let counter = CodeCounter::new();
        let aggregated_stats = counter.calculate_file_stats(&file_path).unwrap();

        assert!(aggregated_stats.basic.total_lines > 0);
        assert_eq!(
            aggregated_stats.basic.code_lines
                + aggregated_stats.basic.comment_lines
                + aggregated_stats.basic.doc_lines
                + aggregated_stats.basic.blank_lines,
            aggregated_stats.basic.total_lines,
            "line categories must partition total_lines"
        );
        assert!(aggregated_stats.ratios.code_ratio > 0.0);
        assert!(aggregated_stats.ratios.code_ratio <= 1.0);

        assert!(!aggregated_stats.metadata.version.is_empty());
        assert!(!aggregated_stats.metadata.timestamp.is_empty());
        assert_eq!(aggregated_stats.metadata.file_count_analyzed, 1);
    }

    #[test]
    fn test_calculate_project_stats_comprehensive() {
        let project = TestProject::new("test_project_stats").unwrap();

        project.create_rust_file("main.rs", 15, 8).unwrap();
        project.create_rust_file("lib.rs", 25, 12).unwrap();
        project.create_python_file("script.py", 20).unwrap();

        let counter = CodeCounter::new();

        let mut stats_by_extension = BTreeMap::new();
        stats_by_extension.insert(
            "rs".to_string(),
            (
                2,
                FileStats {
                    total_lines: 100,
                    code_lines: 70,
                    comment_lines: 20,
                    doc_lines: 5,
                    blank_lines: 10,
                    file_size: 2000,
                },
            ),
        );
        stats_by_extension.insert(
            "py".to_string(),
            (
                1,
                FileStats {
                    total_lines: 50,
                    code_lines: 35,
                    comment_lines: 10,
                    doc_lines: 2,
                    blank_lines: 5,
                    file_size: 1000,
                },
            ),
        );

        let code_stats = CodeStats {
            total_files: 3,
            total_lines: 150,
            total_code_lines: 105,
            total_comment_lines: 30,
            total_doc_lines: 7,
            total_blank_lines: 15,
            total_size: 3000,
            stats_by_extension,
        };

        let individual_files = vec![
            (
                project.root.join("main.rs").to_string_lossy().to_string(),
                FileStats {
                    total_lines: 50,
                    code_lines: 35,
                    comment_lines: 10,
                    doc_lines: 2,
                    blank_lines: 5,
                    file_size: 1000,
                },
            ),
            (
                project.root.join("lib.rs").to_string_lossy().to_string(),
                FileStats {
                    total_lines: 50,
                    code_lines: 35,
                    comment_lines: 10,
                    doc_lines: 3,
                    blank_lines: 5,
                    file_size: 1000,
                },
            ),
            (
                project.root.join("script.py").to_string_lossy().to_string(),
                FileStats {
                    total_lines: 50,
                    code_lines: 35,
                    comment_lines: 10,
                    doc_lines: 2,
                    blank_lines: 5,
                    file_size: 1000,
                },
            ),
        ];

        let aggregated_stats = counter
            .calculate_project_stats(&code_stats, &individual_files)
            .unwrap();

        assert_eq!(aggregated_stats.basic.total_files, 3);
        assert_eq!(aggregated_stats.basic.total_lines, 150);
        assert_eq!(aggregated_stats.basic.code_lines, 105);
        assert!(aggregated_stats.ratios.code_ratio > 0.0);

        assert_eq!(aggregated_stats.metadata.file_count_analyzed, 3);
        assert!(aggregated_stats.metadata.languages_detected.len() >= 2);
    }

    #[test]
    fn test_aggregate_stats_functionality() {
        let counter = CodeCounter::new();

        let file_stats = vec![
            (
                "rs".to_string(),
                FileStats {
                    total_lines: 100,
                    code_lines: 70,
                    comment_lines: 20,
                    doc_lines: 5,
                    blank_lines: 10,
                    file_size: 2000,
                },
            ),
            (
                "rs".to_string(),
                FileStats {
                    total_lines: 50,
                    code_lines: 35,
                    comment_lines: 10,
                    doc_lines: 2,
                    blank_lines: 5,
                    file_size: 1000,
                },
            ),
            (
                "py".to_string(),
                FileStats {
                    total_lines: 80,
                    code_lines: 60,
                    comment_lines: 15,
                    doc_lines: 3,
                    blank_lines: 5,
                    file_size: 1500,
                },
            ),
        ];

        let aggregated = counter.aggregate_stats(file_stats);

        assert_eq!(aggregated.total_files, 3);
        assert_eq!(aggregated.total_lines, 230);
        assert_eq!(aggregated.total_code_lines, 165);
        assert_eq!(aggregated.total_comment_lines, 45);
        assert_eq!(aggregated.total_doc_lines, 10);
        assert_eq!(aggregated.total_blank_lines, 20);
        assert_eq!(aggregated.total_size, 4500);

        assert_eq!(aggregated.stats_by_extension.len(), 2);

        let rust_stats = &aggregated.stats_by_extension["rs"];
        assert_eq!(rust_stats.0, 2);
        assert_eq!(rust_stats.1.total_lines, 150);
        assert_eq!(rust_stats.1.code_lines, 105);

        let python_stats = &aggregated.stats_by_extension["py"];
        assert_eq!(python_stats.0, 1);
        assert_eq!(python_stats.1.total_lines, 80);
        assert_eq!(python_stats.1.code_lines, 60);
    }

    #[test]
    fn test_stats_calculator_access() {
        let counter = CodeCounter::new();
        let stats_calc = counter.stats_calculator();
        let stats_calc2 = counter.stats_calculator();
        assert!(std::ptr::eq(stats_calc, stats_calc2));
    }

    #[test]
    fn test_error_handling() {
        let counter = CodeCounter::new();

        let non_existent = std::path::Path::new("/non/existent/file.rs");
        assert!(counter.count_file(non_existent).is_err());

        let temp_dir = tempfile::tempdir().unwrap();
        let err = counter.count_file(temp_dir.path()).unwrap_err();
        assert!(
            err.to_string().contains("directory"),
            "directory error should say so, got: {err}"
        );
    }

    #[test]
    fn test_performance_with_large_file() {
        let project = TestProject::new("test_performance").unwrap();

        let mut large_content = String::new();
        for i in 0..1000 {
            large_content.push_str(&format!("// Comment line {}\n", i));
            large_content.push_str(&format!("fn function_{}() {{\n", i));
            large_content.push_str("    println!(\"Hello\");\n");
            large_content.push_str("}\n\n");
        }

        let file_path = project.create_file("large.rs", &large_content).unwrap();

        let counter = CodeCounter::new();
        let start = std::time::Instant::now();
        let stats = counter.count_file(&file_path).unwrap();
        let duration = start.elapsed();

        assert!(duration.as_secs() < 1);
        assert_eq!(stats.comment_lines, 1000);
        assert!(stats.code_lines >= 2000);
        assert!(stats.total_lines >= 4000);
    }

    /// Counting with a known size must agree with counting that discovers it.
    #[test]
    fn count_with_known_size_matches_full_count() {
        let project = TestProject::new("known_size").unwrap();
        let path = project.create_rust_file("a.rs", 5, 3).unwrap();
        let size = std::fs::metadata(&path).unwrap().len();

        let counter = CodeCounter::new();
        assert_eq!(
            counter.count_file(&path).unwrap(),
            counter.count_file_with_size(&path, size).unwrap()
        );
    }

    /// In-memory classification must agree with reading the same bytes off disk.
    #[test]
    fn count_bytes_matches_count_file() {
        let project = TestProject::new("bytes").unwrap();
        let counter = CodeCounter::new();

        for (name, ext, content) in [
            ("a.rs", "rs", "// c\nfn main() {}\n\n"),
            ("b.py", "py", "# c\ndef f():\n    pass\n"),
            ("c.md", "md", "# T\n\ntext\n"),
        ] {
            let path = project.create_file(name, content).unwrap();
            assert_eq!(
                counter.count_file(&path).unwrap(),
                counter.count_bytes(content.as_bytes(), ext).unwrap(),
                "{name} differed between disk and memory"
            );
        }
    }

    /// The cache must never change the answer, only the cost of getting it.
    #[test]
    fn cached_counting_matches_uncached() {
        let project = TestProject::new("cache_equiv").unwrap();
        let paths: Vec<_> = (0..12)
            .map(|i| {
                project
                    .create_rust_file(&format!("f{i}.rs"), i % 5 + 1, i % 3)
                    .unwrap()
            })
            .collect();

        let plain = CodeCounter::new();
        let mut cached = CachedCodeCounter::with_cache(FileCache::new());

        for path in &paths {
            let expected = plain.count_file(path).unwrap();
            assert_eq!(cached.count_file(path).unwrap(), expected, "first pass");
            assert_eq!(cached.count_file(path).unwrap(), expected, "cached pass");
        }

        let (hits, misses) = cached.cache_stats();
        assert_eq!(misses, paths.len(), "every file should miss exactly once");
        assert_eq!(hits, paths.len(), "every file should hit exactly once");
    }

    /// Editing a file must invalidate its cache entry.
    #[test]
    fn cache_invalidates_when_content_changes() {
        let project = TestProject::new("cache_invalidate").unwrap();
        let path = project.create_file("a.rs", "fn a() {}\n").unwrap();

        let mut cached = CachedCodeCounter::with_cache(FileCache::new());
        let before = cached.count_file(&path).unwrap();
        assert_eq!(before.total_lines, 1);

        project
            .create_file("a.rs", "fn a() {}\nfn b() {}\nfn c() {}\n")
            .unwrap();

        let after = cached.count_file(&path).unwrap();
        assert_eq!(after.total_lines, 3, "cache returned stale statistics");
    }
}
