use crate::core::counter::comment_patterns;
use crate::core::languages;
use crate::core::patterns::PatternMatcher;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SherlockLanguage {
    pub name: String,
    pub color: String,
    pub file_count: usize,
    pub files: Vec<String>,
    pub percentage: f64,
    pub total_bytes: u64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SherlockSummary {
    pub languages_detected: usize,
    pub total_bytes: u64,
    pub total_files: usize,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SherlockResult {
    pub languages: Vec<SherlockLanguage>,
    pub summary: SherlockSummary,
    pub unknown_files: Vec<String>,
}

impl SherlockResult {
    /// A result carrying no detections, used when detection is unavailable.
    pub fn empty() -> Self {
        Self {
            languages: Vec::new(),
            summary: SherlockSummary {
                languages_detected: 0,
                total_bytes: 0,
                total_files: 0,
            },
            unknown_files: Vec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.languages.is_empty()
    }
}

/// Normalize a path emitted by Sherlock (or discovered by us) into a stable
/// comparison key: forward slashes, no `./` prefix, no leading separator.
fn normalize_key(raw: &str) -> String {
    let unified = raw.replace('\\', "/");
    let trimmed = unified.trim_start_matches("./").trim_start_matches('/');
    trimmed.to_string()
}

/// A lookup table built once from a [`SherlockResult`].
///
/// The original implementation compared each candidate path against every file
/// of every detected language with `ends_with`, which is O(files x detections)
/// -- on a 10k-file repository that is upwards of 10^8 string comparisons. The
/// same question is answered here by two hash lookups.
#[derive(Debug, Default, Clone)]
struct SherlockIndex {
    /// Detected file paths, normalized and keyed both by full relative path and
    /// by trailing path segment, so a candidate matches regardless of whether
    /// Sherlock reported paths relative to the scan root or to the CWD.
    by_path: HashSet<String>,
    /// Extension -> language name, for display.
    language_by_extension: HashMap<String, String>,
}

impl SherlockIndex {
    fn build(result: &SherlockResult, root: Option<&Path>) -> Self {
        let root_key = root.map(|r| normalize_key(&r.to_string_lossy()));
        // Sherlock echoes back the path it was handed, so a relative invocation
        // yields entries prefixed with only the root's last component.
        let root_leaf = root
            .and_then(|r| r.file_name())
            .map(|n| n.to_string_lossy().to_string());
        let mut by_path = HashSet::new();
        let mut language_by_extension = HashMap::new();

        for language in &result.languages {
            for file in &language.files {
                let key = normalize_key(file);

                // Candidates are compared as root-relative paths, so store that
                // form for every prefix shape Sherlock might have used.
                for prefix in [root_key.as_deref(), root_leaf.as_deref()]
                    .into_iter()
                    .flatten()
                {
                    if let Some(rel) = key.strip_prefix(prefix) {
                        let rel = rel.trim_start_matches('/');
                        if !rel.is_empty() && rel != key {
                            by_path.insert(rel.to_string());
                        }
                    }
                }

                if let Some(ext) = Path::new(&key).extension().and_then(|e| e.to_str()) {
                    language_by_extension
                        .entry(ext.to_lowercase())
                        .or_insert_with(|| language.name.clone());
                }

                by_path.insert(key);
            }
        }

        Self {
            by_path,
            language_by_extension,
        }
    }

    fn contains(&self, relative: &str, absolute: &str) -> bool {
        self.by_path.contains(relative) || self.by_path.contains(&normalize_key(absolute))
    }
}

/// Why a path was left out of the analysis.
///
/// Carried rather than discarded so the report can say what it skipped. A run
/// over one monorepo dropped 1.15 million lines of generated protobuf bindings
/// and said nothing at all, which is indistinguishable from not having found
/// them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SkipReason {
    /// Inside build output, a dependency cache, or vendored code.
    NotAuthoredHere,
    /// Machine-written: protobuf bindings, minified bundles, codegen output.
    Generated,
    /// An image, archive, font or compiled artifact -- not line-oriented text.
    Binary,
    /// Legal or credits boilerplate nobody in the project wrote.
    Boilerplate,
}

impl SkipReason {
    pub const ALL: [SkipReason; 4] = [
        SkipReason::NotAuthoredHere,
        SkipReason::Generated,
        SkipReason::Binary,
        SkipReason::Boilerplate,
    ];

    pub fn label(self) -> &'static str {
        match self {
            SkipReason::NotAuthoredHere => "build output or vendored",
            SkipReason::Generated => "generated",
            SkipReason::Binary => "binary or asset",
            SkipReason::Boilerplate => "licence boilerplate",
        }
    }
}

/// What the built-in classifier can conclude about a path on its own.
///
/// The distinction between [`Rejected`](Classification::Rejected) and
/// [`Unknown`](Classification::Unknown) is what keeps an optional external
/// detector from changing the answer. Before it existed, both cases fell
/// through to Sherlock, so installing Sherlock re-admitted files this build
/// deliberately excludes -- on this repository its entire effect was to add
/// three `LICENSE` files back, for three quarters of the run time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Classification {
    /// Hand-written source, counted whether or not a detector is installed.
    Source,
    /// Deliberately not counted. No detector overrides this.
    Rejected(SkipReason),
    /// Nothing in this build recognizes the name, and no rule rejects it.
    /// Only a language detector can settle it.
    Unknown,
}

/// A running Sherlock process whose answer has not been needed yet.
///
/// Detection re-walks the whole tree in a separate process and, on a 7,700-file
/// corpus, takes about three times as long as counting every line. Holding the
/// child rather than blocking on it lets the caller discover the tree first and
/// then decide: if nothing it found is [`Classification::Unknown`], Sherlock's
/// answer cannot change the result, so the process is cancelled instead of
/// waited on.
pub struct DetectionJob {
    child: std::process::Child,
}

impl DetectionJob {
    fn start(path: &Path) -> std::io::Result<Self> {
        Command::new("sherlock")
            .arg(path.as_os_str())
            .arg("--format")
            .arg("json")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map(|child| Self { child })
    }

    /// Wait for the report.
    pub fn finish(self) -> std::result::Result<SherlockResult, Box<dyn std::error::Error>> {
        let output = self.child.wait_with_output()?;
        if !output.status.success() {
            return Err(format!(
                "sherlock exited with {}: {}",
                output.status,
                String::from_utf8_lossy(&output.stderr).trim()
            )
            .into());
        }
        Ok(serde_json::from_slice(&output.stdout)?)
    }

    /// Stop the process; its answer is not needed.
    ///
    /// Killed rather than detached so it stops competing for CPU and file
    /// descriptors with the counting threads that are still running.
    pub fn cancel(mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// Decides whether a discovered path is user-authored source.
#[derive(Debug, Clone)]
pub struct FileDetector {
    pattern_matcher: PatternMatcher,
    sherlock_result: Option<SherlockResult>,
    sherlock_index: SherlockIndex,
    root: Option<PathBuf>,
}

impl Default for FileDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl FileDetector {
    pub fn new() -> Self {
        Self {
            pattern_matcher: PatternMatcher::new(),
            sherlock_result: None,
            sherlock_index: SherlockIndex::default(),
            root: None,
        }
    }

    /// Anchor pattern matching to the directory being analyzed.
    ///
    /// Without a root, exclusion patterns are matched against absolute paths
    /// and words like `build`, `tmp` or `env` anywhere above the project
    /// silently exclude everything. Always set this.
    pub fn with_root(mut self, root: impl Into<PathBuf>) -> Self {
        self.root = Some(root.into());
        self.reindex();
        self
    }

    pub fn with_sherlock_result(mut self, sherlock_result: SherlockResult) -> Self {
        self.sherlock_result = Some(sherlock_result);
        self.reindex();
        self
    }

    fn reindex(&mut self) {
        self.sherlock_index = match &self.sherlock_result {
            Some(result) => SherlockIndex::build(result, self.root.as_deref()),
            None => SherlockIndex::default(),
        };
    }

    pub fn root(&self) -> Option<&Path> {
        self.root.as_deref()
    }

    pub fn sherlock_result(&self) -> Option<&SherlockResult> {
        self.sherlock_result.as_ref()
    }

    /// Classify `path` using only rules compiled into this build.
    ///
    /// Deliberately free of any external process, so that the answer is the
    /// same on every machine. Only [`Classification::Unknown`] leaves room for
    /// a detector to contribute.
    pub fn classify(&self, path: &Path) -> Classification {
        let relative = self
            .pattern_matcher
            .relative_path(path, self.root.as_deref());

        if self.pattern_matcher.should_ignore_file(&relative)
            || self
                .pattern_matcher
                .is_build_output(path, self.root.as_deref())
        {
            return Classification::Rejected(SkipReason::NotAuthoredHere);
        }

        // Binary, generated and boilerplate files are rejected here rather than
        // only in the walker, so that every entry point agrees. `api.pb.go`
        // used to be counted as hand-written Go because this check lived only
        // in the file filter -- and for the same reason the engine, which never
        // calls the filter, counted the "lines" of every `.svg` in the tree.
        if let Some(extension) = path.extension().and_then(|e| e.to_str()) {
            if self.pattern_matcher.is_binary_file(extension) {
                return Classification::Rejected(SkipReason::Binary);
            }
        }

        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if self.pattern_matcher.is_generated_file(name) {
                return Classification::Rejected(SkipReason::Generated);
            }
            if PatternMatcher::is_boilerplate_file(name) {
                return Classification::Rejected(SkipReason::Boilerplate);
            }
        }

        if Self::is_recognized_source(path) || Self::is_script_by_shebang(path) {
            return Classification::Source;
        }

        Classification::Unknown
    }

    pub fn is_user_created_file(&self, path: &Path) -> bool {
        match self.classify(path) {
            Classification::Source => true,
            Classification::Rejected(_) => false,
            Classification::Unknown => self.detected_as_source(path),
        }
    }

    /// Whether the external detector claims this path is a known language.
    ///
    /// Only consulted for [`Classification::Unknown`] paths; with no detection
    /// result loaded it is always false, which is the reproducible default.
    pub fn detected_as_source(&self, path: &Path) -> bool {
        if self.sherlock_index.by_path.is_empty() {
            return false;
        }
        let relative = self
            .pattern_matcher
            .relative_path(path, self.root.as_deref());
        self.sherlock_index
            .contains(&relative, &path.to_string_lossy())
    }

    pub fn is_code_file(&self, path: &Path) -> bool {
        Self::is_recognized_source(path) || Self::is_script_by_shebang(path)
    }

    /// True when an extension-less file announces an interpreter.
    ///
    /// `configure`, `bootstrap` and `pre-commit` are source code that happens to
    /// carry no extension. The probe reads at most 128 bytes and only for files
    /// no other rule recognized, so a tree of ordinary `.rs` and `.ts` files
    /// never pays for it.
    fn is_script_by_shebang(path: &Path) -> bool {
        if path.extension().is_some() {
            return false;
        }

        let mut head = [0u8; 128];
        let Ok(mut file) = fs::File::open(path) else {
            return false;
        };
        let Ok(read) = file.read(&mut head) else {
            return false;
        };

        crate::core::counter::shebang_key(&head[..read]).is_some()
    }

    /// True when the file is source without help from the external detector.
    ///
    /// This is the classifier that decides a run on a machine where `sherlock`
    /// is not installed, which is most machines, so it has to be complete on its
    /// own. `Dockerfile` and `Makefile` are matched by name because they carry
    /// no extension.
    pub fn is_recognized_source(path: &Path) -> bool {
        comment_patterns::is_known(&crate::core::counter::classify_key(path))
    }

    /// Extensions counted when language detection is unavailable.
    ///
    /// Derived from the counter's comment-syntax table rather than restated, so
    /// a language the counter can classify is never dropped before it is
    /// counted. The two lists disagreed before: `vue`, `svelte`, `sql`, `proto`,
    /// `tf`, `Dockerfile` and a dozen others were classifiable but discarded.
    pub fn is_code_extension(ext: &str) -> bool {
        if ext.bytes().any(|b| b.is_ascii_uppercase()) {
            comment_patterns::is_known(&ext.to_lowercase())
        } else {
            comment_patterns::is_known(ext)
        }
    }

    /// True when the external `sherlock` binary is callable.
    ///
    /// Checked before spawning so that a machine without Sherlock pays a single
    /// cheap probe instead of a failed process spawn, and so callers can report
    /// the degraded mode honestly instead of emitting a scary error.
    pub fn detection_available() -> bool {
        Command::new("sherlock")
            .arg("--version")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }

    /// Run Sherlock over `path` and parse its report.
    pub fn detect_languages(
        &self,
        path: &Path,
    ) -> std::result::Result<SherlockResult, Box<dyn std::error::Error>> {
        DetectionJob::start(path)?.finish()
    }

    /// Start Sherlock without waiting for it.
    pub fn start_detection(path: &Path) -> std::io::Result<DetectionJob> {
        DetectionJob::start(path)
    }

    /// Language name for `extension`, preferring Sherlock's own labelling.
    pub fn get_language_from_extension(
        &self,
        extension: &str,
        sherlock_result: &SherlockResult,
    ) -> Option<String> {
        let key = extension.to_lowercase();

        if let Some(name) = self.sherlock_index.language_by_extension.get(&key) {
            return Some(name.clone());
        }

        // Caller-supplied result that was never indexed on this detector.
        for language in &sherlock_result.languages {
            for file in &language.files {
                if Path::new(file)
                    .extension()
                    .and_then(|e| e.to_str())
                    .is_some_and(|e| e.eq_ignore_ascii_case(&key))
                {
                    return Some(language.name.clone());
                }
            }
        }

        Self::language_name_for_extension(&key)
    }

    /// Built-in extension to language mapping, from [`crate::core::languages`].
    pub fn language_name_for_extension(extension: &str) -> Option<String> {
        languages::lookup(&extension.to_lowercase()).map(|l| l.name.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn result_with(files: &[&str]) -> SherlockResult {
        SherlockResult {
            languages: vec![SherlockLanguage {
                name: "Rust".to_string(),
                color: "#000".to_string(),
                file_count: files.len(),
                files: files.iter().map(|f| f.to_string()).collect(),
                percentage: 100.0,
                total_bytes: 0,
            }],
            summary: SherlockSummary {
                languages_detected: 1,
                total_bytes: 0,
                total_files: files.len(),
            },
            unknown_files: Vec::new(),
        }
    }

    #[test]
    fn code_extensions_are_recognized_case_insensitively() {
        let d = FileDetector::new();
        let root = Path::new("/p");
        for name in ["a.rs", "a.RS", "b.Py", "c.TS"] {
            assert!(
                d.clone()
                    .with_root(root)
                    .is_user_created_file(&root.join(name)),
                "{name} should count"
            );
        }
    }

    #[test]
    fn non_code_extensions_are_rejected() {
        let root = Path::new("/p");
        let d = FileDetector::new().with_root(root);
        for name in ["a.png", "a.exe", "LICENSE"] {
            assert!(
                !d.is_user_created_file(&root.join(name)),
                "{name} must not count"
            );
        }
    }

    /// The regression that made the tool report zero files: a project living
    /// under a directory whose name matches a build pattern.
    #[test]
    fn hostile_ancestor_directories_do_not_exclude_sources() {
        for root in [
            "/tmp/checkout",
            "/build/workspace",
            "/home/u/env/proj",
            "/var/log/proj",
            "/x/bin/proj",
            "/x/target/proj",
            "/x/vendor/proj",
        ] {
            let root = Path::new(root);
            let detector = FileDetector::new().with_root(root);
            assert!(
                detector.is_user_created_file(&root.join("src/main.rs")),
                "sources under {root:?} were excluded by an ancestor directory name"
            );
        }
    }

    #[test]
    fn build_directories_inside_the_project_are_still_excluded() {
        let root = Path::new("/tmp/checkout");
        let detector = FileDetector::new().with_root(root);
        for rel in [
            "node_modules/dep/index.js",
            "DerivedData/Build/x.swift",
            "__pycache__/m.py",
            ".git/hooks/pre-commit.sh",
        ] {
            assert!(
                !detector.is_user_created_file(&root.join(rel)),
                "{rel} should be excluded"
            );
        }
    }

    #[test]
    fn sherlock_detections_are_honoured_for_unknown_extensions() {
        let root = Path::new("/p");
        let detector = FileDetector::new()
            .with_root(root)
            .with_sherlock_result(result_with(&["/p/weird/thing.customext"]));

        assert!(detector.is_user_created_file(&root.join("weird/thing.customext")));
        assert!(!detector.is_user_created_file(&root.join("weird/other.customext")));
    }

    /// Sherlock reports paths relative to whatever it was handed; the index
    /// must match candidates under either shape.
    #[test]
    fn sherlock_paths_match_under_relative_and_absolute_forms() {
        let root = Path::new("/p/corpus");
        for reported in [
            "./corpus/deep/thing.customext",
            "/p/corpus/deep/thing.customext",
            "deep/thing.customext",
        ] {
            let detector = FileDetector::new()
                .with_root(root)
                .with_sherlock_result(result_with(&[reported]));
            assert!(
                detector.is_user_created_file(&root.join("deep/thing.customext")),
                "reported form {reported:?} did not match"
            );
        }
    }

    /// Sherlock must never resurrect a path that pattern exclusion rejected.
    #[test]
    fn sherlock_cannot_override_build_exclusions() {
        let root = Path::new("/p");
        let detector = FileDetector::new()
            .with_root(root)
            .with_sherlock_result(result_with(&["/p/node_modules/dep/index.js"]));
        assert!(!detector.is_user_created_file(&root.join("node_modules/dep/index.js")));
    }

    /// With Sherlock unavailable the extension fallback must still classify
    /// ordinary source correctly -- this is what guarantees identical results on
    /// a machine that lacks the binary.
    #[test]
    fn extension_fallback_matches_sherlock_for_known_languages() {
        let root = Path::new("/p");
        let files = [
            "src/main.rs",
            "app/util.py",
            "web/index.js",
            "svc/handler.go",
        ];
        let reported: Vec<String> = files.iter().map(|f| format!("/p/{f}")).collect();
        let reported_refs: Vec<&str> = reported.iter().map(String::as_str).collect();
        let with_detection = FileDetector::new()
            .with_root(root)
            .with_sherlock_result(result_with(&reported_refs));
        let without_detection = FileDetector::new().with_root(root);

        for f in files {
            let p = root.join(f);
            assert_eq!(
                with_detection.is_user_created_file(&p),
                without_detection.is_user_created_file(&p),
                "{f} classified differently with and without language detection"
            );
        }
    }

    #[test]
    fn language_names_resolve_from_extension() {
        assert_eq!(
            FileDetector::language_name_for_extension("rs").as_deref(),
            Some("Rust")
        );
        assert_eq!(
            FileDetector::language_name_for_extension("RS").as_deref(),
            Some("Rust")
        );
        assert_eq!(FileDetector::language_name_for_extension("qqq"), None);
    }

    #[test]
    fn empty_result_is_inert() {
        let root = Path::new("/p");
        let detector = FileDetector::new()
            .with_root(root)
            .with_sherlock_result(SherlockResult::empty());
        assert!(detector.is_user_created_file(&root.join("src/main.rs")));
        assert!(!detector.is_user_created_file(&root.join("src/thing.customext")));
    }
}
