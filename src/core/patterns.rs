//! Path classification patterns.
//!
//! Two invariants govern this module:
//!
//! 1. **Patterns are matched against a repository-relative path, never an
//!    absolute one.** `build/`, `tmp/`, `env/`, `bin/` and friends are ordinary
//!    words that routinely appear in a user's home directory or in a CI
//!    workspace root. Matching them against an absolute path made the whole
//!    analysis collapse to zero files depending on where the project happened
//!    to live. Callers are responsible for relativizing; see
//!    [`PatternMatcher::relative_path`].
//!
//! 2. **Matching is one pass, not one pass per pattern.** All patterns in a
//!    category are compiled into a single [`RegexSet`], which evaluates every
//!    alternative in a single scan of the input.

use once_cell::sync::Lazy;
use regex::{Regex, RegexSet};
use std::collections::{HashMap, HashSet};
use std::path::Path;

/// Files produced by the OS or a desktop environment.
const OS_PATTERNS: &[&str] = &[
    r"\.DS_Store",
    r"Thumbs\.db",
    r"desktop\.ini",
    r"\.directory",
    r"\.Spotlight-V100",
    r"\.Trashes",
    r"\.VolumeIcon\.icns",
    r"\.AppleDouble",
    r"\.LSOverride",
    r"\.DocumentRevisions-V100",
    r"\.fseventsd",
    r"\.TemporaryItems",
    r"\.com\.apple\.timemachine\.donotpresent",
    r"\.AppleDB",
    r"\.AppleDesktop",
    r"Network Trash Folder",
    r"Temporary Items",
    r"\.apdisk",
    r"ehthumbs\.db",
    r"\._ ",
];

/// Editor and IDE metadata.
const IDE_PATTERNS: &[&str] = &[
    r"\.vscode/",
    r"\.idea/",
    r"\.vs/",
    r"\.sublime-",
    r"\.atom/",
    r"\.eclipse/",
    r"\.metadata/",
    r"\.settings/",
    r"\.spyproject/",
    r"\.nova/",
    r"\.zed/",
    r"\.brackets\.json",
    r"\.emacs\.d/",
    r"\.vim/",
    r"nbproject/",
    r"\.buildpath",
    r"\.project",
    r"\.classpath",
    r"\.aptana/",
    r"\.phpstorm\.meta\.php",
];

/// Scratch files, editor swap files and backups.
const TEMP_PATTERNS: &[&str] = &[
    r"\.tmp$",
    r"\.temp$",
    r"\.swp$",
    r"\.swo$",
    r"~$",
    r"\.bak$",
    r"\.backup$",
    r"\.orig$",
    r"\.rej$",
    r"\.cache$",
    r"\.log$",
    r"\.out$",
    r"\.err$",
    r"\.pid$",
    r"\.lock$",
    r"\.lockb$",
    r"\.moved-aside",
    r"\._ ",
];

/// Version-control metadata.
///
/// Only the repositories' own internal stores are noise. `.gitignore`,
/// `.gitattributes` and `.gitmodules` are hand-written project configuration --
/// the same class of file as `.editorconfig` or `.dockerignore`, which are
/// counted -- so they are not excluded here; `.gitkeep` is an empty placeholder.
const VCS_PATTERNS: &[&str] = &[r"\.git/", r"\.svn/", r"\.hg/", r"\.bzr/", r"\.gitkeep"];

/// Per-ecosystem build and dependency-cache locations.
///
/// Entries containing a `/` are genuinely multi-segment and can only be decided
/// from a path; single-segment entries are additionally used to prune whole
/// directories during traversal (see [`PRUNE_DIRS`]).
const LANGUAGE_BUILD_SPECS: &[(&str, &[&str])] = &[
    (
        "nodejs",
        &[
            r"node_modules/",
            r"dist/",
            r"out/",
            r"\.npm/",
            r"\.yarn/",
            r"\.pnpm-store/",
            r"\.bun/",
            r"\.next/",
            r"\.nuxt/",
            r"\.output/",
            r"\.svelte-kit/",
            r"\.astro/",
            r"\.remix/",
            r"\.vercel/",
            r"\.netlify/",
            r"\.firebase/",
            r"\.parcel-cache/",
            r"\.turbo/",
            r"\.webpack/",
            r"\.rollup\.cache/",
            r"\.vite/",
            r"\.swc/",
            r"\.esbuild/",
            r"\.nyc_output/",
            r"\.eslintcache",
            r"\.stylelintcache",
        ],
    ),
    (
        "python",
        &[
            r"__pycache__/",
            r"\.pytest_cache/",
            r"\.tox/",
            r"\.nox/",
            r"\.coverage",
            r"htmlcov/",
            r"\.mypy_cache/",
            r"\.pytype/",
            r"\.pyre/",
            r"\.ruff_cache/",
            r"\.ipynb_checkpoints/",
            r"\.eggs/",
            r"\.pip/",
            r"\.venv/",
            r"venv/",
            r"env/",
            r"\.env/",
            r"virtualenv/",
        ],
    ),
    ("rust", &[r"target/", r"\.cargo/", r"\.rustup/"]),
    (
        "java",
        &[
            r"target/",
            r"build/",
            r"\.gradle/",
            r"\.m2/",
            r"\.mvn/",
            r"\.sbt/",
            r"\.ivy2/",
        ],
    ),
    ("go", &[r"vendor/", r"\.go/pkg/"]),
    (
        "cpp",
        &[
            r"\.ccache/",
            r"\.sccache/",
            r"build/",
            r"cmake-build-",
            r"CMakeFiles/",
        ],
    ),
    (
        "dotnet",
        &[
            r"bin/",
            r"obj/",
            r"packages/",
            r"\.nuget/",
            r"TestResults/",
            r"publish/",
            r"\.publish/",
        ],
    ),
    (
        "php",
        &[
            r"vendor/",
            r"bootstrap/cache/",
            r"storage/framework/",
            r"storage/logs/",
            r"var/cache/",
            r"var/logs/",
            r"tmp/cache/",
            r"application/cache/",
        ],
    ),
    (
        "ruby",
        &[
            r"\.bundle/",
            r"vendor/bundle/",
            r"\.gem/",
            r"log/",
            r"tmp/",
            r"coverage/",
            r"\.yardoc/",
            r"\.sass-cache/",
            r"\.spring/",
        ],
    ),
    (
        "swift",
        &[
            r"\.build/",
            r"\.swiftpm/",
            r"DerivedData/",
            r"Pods/",
            r"Carthage/",
            r"\.xcodeproj/",
            r"\.xcworkspace/",
            r"\.xcarchive/",
        ],
    ),
];

/// Prunable directories that are not build output: version control metadata and
/// editor state. The build directories are derived from
/// [`LANGUAGE_BUILD_SPECS`] instead of being listed again here.
const PRUNE_DIRS_NON_BUILD: &[&str] = &[
    // VCS
    ".git",
    ".svn",
    ".hg",
    ".bzr",
    // Editors
    ".vscode",
    ".idea",
    ".vs",
    ".atom",
    ".eclipse",
    ".metadata",
    ".settings",
    ".spyproject",
    ".nova",
    ".zed",
    ".emacs.d",
    ".vim",
    "nbproject",
    ".aptana",
];

/// The directory name a build pattern denotes, when it denotes exactly one.
///
/// `node_modules/` yields `node_modules` and `\.venv/` yields `.venv`;
/// `vendor/bundle/` and `cmake-build-` yield nothing, because neither names a
/// single directory that can be decided from its own name alone.
fn prunable_dir_name(pattern: &str) -> Option<String> {
    let name = pattern.strip_suffix('/')?;
    if name.contains('/') {
        return None;
    }
    // `\.` is the only escape these lists use; any other metacharacter means
    // the pattern is a real regex and cannot be compared as a literal name.
    let literal = name.replace(r"\.", ".");
    (!literal.contains(|c: char| r"\[]()*+?{}|^$".contains(c))).then_some(literal)
}

/// Extensions whose contents are not line-oriented text.
const BINARY_EXTENSIONS: &[&str] = &[
    // Executables and objects
    "exe", "dll", "so", "dylib", "a", "lib", "o", "obj", "bin", "dat", "rlib", "pdb",
    // Archives
    "zip", "tar", "gz", "bz2", "rar", "7z", "dmg", "iso", // Images
    "jpg", "jpeg", "png", "gif", "bmp", "tiff", "ico", "svg", "webp", "heic", "heif",
    // Audio / video
    "mp3", "mp4", "avi", "mov", "wmv", "flv", "wav", "flac", "m4a", "m4v", "aiff",
    // Fonts
    "ttf", "otf", "woff", "woff2", "eot", // Documents
    "pdf", "doc", "docx", "xls", "xlsx", "ppt", "pptx", // Databases
    "db", "sqlite", "sqlite3", // Compiled code
    "class", "jar", "war", "ear", "pyc", "pyo", "pyd", // Mobile
    "apk", "ipa", "aab", "dex", // Packages
    "whl", "egg", "gem", "nupkg", "snupkg", "phar",
];

/// Filename endings that identify machine-generated code.
///
/// Matched as **suffixes**, not as substrings. The previous implementation
/// asked whether the lowercased filename *contained* any of `auto`, `out`,
/// `bin`, `obj`, `build`, `dist` or `schema`, which rejected ordinary source:
/// `automation.rs`, `output.rs`, `binary_search.rs`, `objects.py`,
/// `builder.go`, `distance.c` and `schema_test.rb` were all discarded as
/// "generated".
const GENERATED_SUFFIXES: &[&str] = &[
    // Minified and bundled web assets
    ".min.js",
    ".min.mjs",
    ".min.css",
    ".bundle.js",
    ".bundle.css",
    ".js.map",
    ".css.map",
    // Protocol buffers and gRPC
    ".pb.go",
    ".pb.gw.go",
    ".pb.cc",
    ".pb.h",
    ".pb.rs",
    ".pb.swift",
    "_pb2.py",
    "_pb2_grpc.py",
    ".pbobjc.h",
    ".pbobjc.m",
    // Explicit codegen conventions. `*.generated.*` is handled structurally in
    // `is_generated_file`, so only the underscore and abbreviated spellings
    // need listing here.
    "_generated.go",
    "_generated.rs",
    ".gen.go",
    ".gen.ts",
    ".gen.rs",
    ".g.dart",
    ".g.cs",
    ".freezed.dart",
    ".designer.cs",
    // Parser and binding generators
    ".tab.c",
    ".tab.h",
    ".yy.c",
    "_bindata.go",
];

/// Exact filenames (case-insensitive) that are always machine-generated.
///
/// Lock files dominate this list: they are large, they change on every install
/// and nobody writes them by hand, so counting them misrepresents a project by
/// thousands of lines. Suffix-based `.lock` matching in [`TEMP_PATTERNS`] covers
/// most of them; these are the ones that wear an ordinary extension.
const GENERATED_NAMES: &[&str] = &[
    "package-lock.json",
    "npm-shrinkwrap.json",
    "pnpm-lock.yaml",
    "bun.lockb",
    "go.sum",
    "flake.lock",
    "bundle.js",
    "bundle.css",
    "vendor.js",
    "polyfills.js",
];

fn compile_set(patterns: &[&str]) -> RegexSet {
    RegexSet::new(patterns).expect("built-in patterns must compile")
}

static OS_SET: Lazy<RegexSet> = Lazy::new(|| compile_set(OS_PATTERNS));
static IDE_SET: Lazy<RegexSet> = Lazy::new(|| compile_set(IDE_PATTERNS));
static TEMP_SET: Lazy<RegexSet> = Lazy::new(|| compile_set(TEMP_PATTERNS));
static VCS_SET: Lazy<RegexSet> = Lazy::new(|| compile_set(VCS_PATTERNS));

/// Every "ignore this outright" pattern, evaluated in a single scan.
static IGNORE_SET: Lazy<RegexSet> = Lazy::new(|| {
    let all: Vec<&str> = OS_PATTERNS
        .iter()
        .chain(IDE_PATTERNS)
        .chain(TEMP_PATTERNS)
        .chain(VCS_PATTERNS)
        .copied()
        .collect();
    compile_set(&all)
});

/// Every build/cache pattern across all ecosystems, evaluated in a single scan.
static BUILD_SET: Lazy<RegexSet> = Lazy::new(|| {
    let all: Vec<&str> = LANGUAGE_BUILD_SPECS
        .iter()
        .flat_map(|(_, pats)| pats.iter())
        .copied()
        .collect();
    compile_set(&all)
});

static LANGUAGE_BUILD_PATTERNS: Lazy<HashMap<&'static str, Vec<Regex>>> = Lazy::new(|| {
    LANGUAGE_BUILD_SPECS
        .iter()
        .map(|(lang, pats)| {
            let compiled = pats
                .iter()
                .map(|p| Regex::new(p).expect("built-in patterns must compile"))
                .collect();
            (*lang, compiled)
        })
        .collect()
});

/// Directory names that can be skipped during traversal without inspecting
/// their contents.
///
/// Pruning at the directory level is what makes traversal cheap: a
/// `node_modules` tree is skipped after one name comparison instead of being
/// read, stat'ed and then discarded file by file.
///
/// The build entries are *derived* from [`LANGUAGE_BUILD_SPECS`] rather than
/// restated, so the fast path cannot fall behind the pattern list -- adding
/// `dist/` there prunes `dist` here, with no second edit to forget.
static PRUNE_DIR_SET: Lazy<HashSet<String>> = Lazy::new(|| {
    LANGUAGE_BUILD_SPECS
        .iter()
        .flat_map(|(_, patterns)| patterns.iter())
        .filter_map(|pattern| prunable_dir_name(pattern))
        .chain(PRUNE_DIRS_NON_BUILD.iter().map(|d| d.to_string()))
        .collect()
});

static BINARY_EXTENSION_SET: Lazy<HashSet<&'static str>> =
    Lazy::new(|| BINARY_EXTENSIONS.iter().copied().collect());

static GENERATED_NAME_SET: Lazy<HashSet<&'static str>> =
    Lazy::new(|| GENERATED_NAMES.iter().copied().collect());

/// Legal and credits boilerplate, matched on the stem so that `LICENSE`,
/// `LICENSE.md` and `LICENSE.txt` are all treated alike.
///
/// Nobody in the project authored these, and a 200-line Apache licence counted
/// as documentation misrepresents the project it sits in. Keeping the decision
/// here -- rather than only in the comment-syntax table, which sees `LICENSE`
/// but not `LICENSE.md` -- is what makes it hold for every spelling.
const BOILERPLATE_STEMS: &[&str] = &[
    "license",
    "licence",
    "copying",
    "copyright",
    "notice",
    "patents",
    "authors",
    "contributors",
];

static BOILERPLATE_STEM_SET: Lazy<HashSet<&'static str>> =
    Lazy::new(|| BOILERPLATE_STEMS.iter().copied().collect());

/// Patterns shared between the detector and the file filter.
#[derive(Debug, Default, Clone, Copy)]
pub struct CommonPatterns;

impl CommonPatterns {
    pub fn new() -> Self {
        Self
    }

    /// Extensions treated as binary.
    pub fn binary_extensions(&self) -> &'static [&'static str] {
        BINARY_EXTENSIONS
    }

    /// Filename endings that identify generated code.
    pub fn generated_suffixes(&self) -> &'static [&'static str] {
        GENERATED_SUFFIXES
    }

    pub fn matches_os_pattern(&self, path_str: &str) -> bool {
        OS_SET.is_match(path_str)
    }

    pub fn matches_ide_pattern(&self, path_str: &str) -> bool {
        IDE_SET.is_match(path_str)
    }

    pub fn matches_temp_pattern(&self, path_str: &str) -> bool {
        TEMP_SET.is_match(path_str)
    }

    pub fn matches_vcs_pattern(&self, path_str: &str) -> bool {
        VCS_SET.is_match(path_str)
    }

    /// True when the extension denotes a binary file.
    pub fn is_binary_extension(&self, extension: &str) -> bool {
        if extension.bytes().any(|b| b.is_ascii_uppercase()) {
            BINARY_EXTENSION_SET.contains(extension.to_lowercase().as_str())
        } else {
            BINARY_EXTENSION_SET.contains(extension)
        }
    }

    /// True when the filename identifies the file as machine-generated.
    pub fn is_generated_file(&self, filename: &str) -> bool {
        let lower = filename.to_lowercase();
        if GENERATED_NAME_SET.contains(lower.as_str())
            || GENERATED_SUFFIXES
                .iter()
                .any(|suffix| lower.ends_with(suffix))
        {
            return true;
        }

        // `foo.generated.<ext>` is a convention no single language owns, so it is
        // matched structurally rather than enumerated once per extension --
        // which is how `api.generated.rs` slipped through and got counted.
        lower
            .rsplit_once('.')
            .and_then(|(stem, _)| stem.rsplit_once('.'))
            .is_some_and(|(_, marker)| marker == "generated")
    }

    /// True when the filename is legal or credits boilerplate.
    ///
    /// Only the extension-less and plain-text spellings qualify; a suffix like
    /// `.rs` means somebody wrote code in a file that happens to be called
    /// `license.rs`, which is source.
    pub fn is_boilerplate_file(filename: &str) -> bool {
        let lower = filename.to_lowercase();
        let (stem, extension) = match lower.rsplit_once('.') {
            Some((stem, extension)) => (stem, Some(extension)),
            None => (lower.as_str(), None),
        };

        matches!(extension, None | Some("md" | "txt" | "rst" | "markdown"))
            && BOILERPLATE_STEM_SET.contains(stem)
    }

    /// True when the path is OS/IDE/temp/VCS noise. Single scan over `path_str`.
    pub fn should_ignore(&self, path_str: &str) -> bool {
        IGNORE_SET.is_match(path_str)
    }
}

/// Build and cache locations, keyed by ecosystem.
#[derive(Debug, Default, Clone, Copy)]
pub struct LanguageBuildPatterns;

impl LanguageBuildPatterns {
    pub fn new() -> Self {
        Self
    }

    /// True when the path lies inside any known build or dependency cache.
    /// Single scan over `path_str`.
    pub fn matches_build_pattern(&self, path_str: &str) -> bool {
        BUILD_SET.is_match(path_str)
    }

    pub fn get_language_patterns(&self, language: &str) -> Option<&'static Vec<Regex>> {
        LANGUAGE_BUILD_PATTERNS.get(language)
    }
}

/// Combines the common and per-ecosystem pattern sets.
///
/// Construction is free: every table is a process-wide lazy static, so building
/// one matcher per worker thread costs nothing.
#[derive(Debug, Default, Clone, Copy)]
pub struct PatternMatcher {
    common: CommonPatterns,
    language_build: LanguageBuildPatterns,
}

/// Put a rendered path in the form the patterns are written in.
///
/// Every pattern separates components with `/`, which is what Windows hands
/// back as `\` and would therefore match nothing. Elsewhere a backslash is a
/// legal character in a file name, so it must survive untouched.
fn to_pattern_separators(path: std::borrow::Cow<'_, str>) -> std::borrow::Cow<'_, str> {
    if cfg!(windows) && path.contains('\\') {
        std::borrow::Cow::Owned(path.replace('\\', "/"))
    } else {
        path
    }
}

impl PatternMatcher {
    pub fn new() -> Self {
        Self {
            common: CommonPatterns,
            language_build: LanguageBuildPatterns,
        }
    }

    /// Render `path` relative to `root` for pattern matching.
    ///
    /// Patterns describe locations *within* a project, so an absolute prefix
    /// must never participate in matching. Falls back to the path's own file
    /// name when it lies outside `root`, which keeps filename-oriented patterns
    /// working without ever exposing the ancestor directories.
    pub fn relative_path<'a>(
        &self,
        path: &'a Path,
        root: Option<&Path>,
    ) -> std::borrow::Cow<'a, str> {
        let rendered = match root {
            Some(root) => match path.strip_prefix(root) {
                Ok(rel) => rel.to_string_lossy(),
                Err(_) => path
                    .file_name()
                    .map(|n| n.to_string_lossy())
                    .unwrap_or_else(|| path.to_string_lossy()),
            },
            None => path.to_string_lossy(),
        };
        to_pattern_separators(rendered)
    }

    /// True when a directory with this name never contains user-authored
    /// source and can be skipped without descending into it.
    pub fn is_prunable_dir(&self, dir_name: &str) -> bool {
        PRUNE_DIR_SET.contains(dir_name)
    }

    /// Every prunable directory name, for tests and diagnostics.
    pub fn prunable_dirs(&self) -> impl Iterator<Item = &'static str> {
        PRUNE_DIR_SET.iter().map(String::as_str)
    }

    /// True when the path is OS/IDE/temp/VCS noise.
    pub fn should_ignore_file(&self, path_str: &str) -> bool {
        self.common.should_ignore(path_str)
    }

    pub fn is_binary_file(&self, extension: &str) -> bool {
        self.common.is_binary_extension(extension)
    }

    pub fn is_generated_file(&self, filename: &str) -> bool {
        self.common.is_generated_file(filename)
    }

    /// True when the filename is legal or credits boilerplate.
    pub fn is_boilerplate_file(filename: &str) -> bool {
        CommonPatterns::is_boilerplate_file(filename)
    }

    /// True when the path lies inside a build output or dependency cache.
    pub fn matches_build_cache_pattern(&self, path_str: &str) -> bool {
        self.language_build.matches_build_pattern(path_str)
    }

    /// Both noise classes in one call.
    pub fn is_excluded_path(&self, relative_path: &str) -> bool {
        self.should_ignore_file(relative_path) || self.matches_build_cache_pattern(relative_path)
    }

    pub fn common_patterns(&self) -> &CommonPatterns {
        &self.common
    }

    pub fn language_patterns(&self) -> &LanguageBuildPatterns {
        &self.language_build
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn every_builtin_pattern_compiles() {
        // Touching each lazy static forces compilation; a bad pattern panics.
        assert!(!OS_SET.is_empty());
        assert!(!IDE_SET.is_empty());
        assert!(!TEMP_SET.is_empty());
        assert!(!VCS_SET.is_empty());
        assert_eq!(
            IGNORE_SET.len(),
            OS_PATTERNS.len() + IDE_PATTERNS.len() + TEMP_PATTERNS.len() + VCS_PATTERNS.len()
        );
        let build_total: usize = LANGUAGE_BUILD_SPECS.iter().map(|(_, p)| p.len()).sum();
        assert_eq!(BUILD_SET.len(), build_total);
    }

    /// The combined set must agree with the per-category sets it replaced.
    #[test]
    fn combined_ignore_set_agrees_with_categories() {
        let m = PatternMatcher::new();
        let probes = [
            ".DS_Store",
            "src/.DS_Store",
            ".vscode/settings.json",
            "a/.idea/workspace.xml",
            "notes.tmp",
            "editor.swp",
            "backup~",
            ".git/config",
            ".gitignore",
            "src/main.rs",
            "lib/util.py",
        ];
        for p in probes {
            let per_category = m.common.matches_os_pattern(p)
                || m.common.matches_ide_pattern(p)
                || m.common.matches_temp_pattern(p)
                || m.common.matches_vcs_pattern(p);
            assert_eq!(
                m.should_ignore_file(p),
                per_category,
                "combined set disagreed with per-category union for {p:?}"
            );
        }
    }

    /// Every prunable directory name must also be excluded by the pattern
    /// sets, otherwise pruning would drop files the slow path would have kept.
    #[test]
    fn pruning_never_drops_files_the_patterns_would_keep() {
        let m = PatternMatcher::new();
        for dir in m.prunable_dirs() {
            let probe = format!("{dir}/some/file.rs");
            assert!(
                m.is_excluded_path(&probe),
                "{dir:?} is pruned during traversal but {probe:?} is not excluded by any pattern; \
                 pruning and filtering would disagree"
            );
        }
    }

    /// The other direction: any build directory the filter would reject must be
    /// prunable, or we pay to walk a tree whose every file we then discard.
    /// This is the property that keeps traversal cheap as patterns are added.
    #[test]
    fn every_single_segment_build_pattern_is_pruned() {
        let m = PatternMatcher::new();
        for (language, patterns) in LANGUAGE_BUILD_SPECS {
            for pattern in *patterns {
                let Some(name) = prunable_dir_name(pattern) else {
                    continue;
                };
                assert!(
                    m.is_prunable_dir(&name),
                    "{language} excludes {pattern:?} by pattern but {name:?} is not pruned, so \
                     the whole tree is walked only to be discarded"
                );
            }
        }
    }

    #[test]
    fn prunable_dir_name_only_accepts_literal_single_segments() {
        assert_eq!(
            prunable_dir_name("node_modules/").as_deref(),
            Some("node_modules")
        );
        assert_eq!(prunable_dir_name(r"\.venv/").as_deref(), Some(".venv"));
        assert_eq!(
            prunable_dir_name(r"\.rollup\.cache/").as_deref(),
            Some(".rollup.cache")
        );
        assert_eq!(prunable_dir_name("vendor/bundle/"), None);
        assert_eq!(prunable_dir_name("cmake-build-"), None);
        assert_eq!(prunable_dir_name(r"\.coverage"), None);
    }

    /// `dist/` is the most common build output name in the JavaScript and
    /// Python ecosystems; leaving it out silently doubled the reported size of
    /// any project that ships a bundle.
    #[test]
    fn common_build_output_directories_are_excluded() {
        let m = PatternMatcher::new();
        for probe in [
            "dist/bundle.js",
            "out/index.html",
            "node_modules/left-pad/index.js",
            "target/debug/deps/x.rs",
            "build/classes/A.class",
        ] {
            assert!(m.is_excluded_path(probe), "{probe:?} should be excluded");
        }
        assert!(!m.is_excluded_path("src/distance.rs"));
        assert!(!m.is_excluded_path("src/outbox/handler.rs"));
    }

    #[test]
    fn build_patterns_match_relative_locations() {
        let m = PatternMatcher::new();
        assert!(m.matches_build_cache_pattern("node_modules/left-pad/index.js"));
        assert!(m.matches_build_cache_pattern("target/debug/build.rs"));
        assert!(m.matches_build_cache_pattern("__pycache__/mod.pyc"));
        assert!(!m.matches_build_cache_pattern("src/main.rs"));
    }

    /// The regression that motivated root-relative matching: a project whose
    /// absolute path contains a build-directory word must still be analyzed.
    #[test]
    fn absolute_ancestors_never_exclude_a_project() {
        let m = PatternMatcher::new();
        let hostile_roots = [
            "/tmp/checkout",
            "/build/ci/workspace",
            "/home/dev/env/projects",
            "/var/log/app",
            "/Users/dev/bin/scratch",
            "/srv/vendor/site",
            "/opt/target/repo",
            "/data/coverage/repo",
            "/mnt/dist/repo",
            "/home/obj/repo",
        ];
        for root in hostile_roots {
            let root = PathBuf::from(root);
            let file = root.join("src/main.rs");
            let rel = m.relative_path(&file, Some(&root));
            assert_eq!(rel, "src/main.rs");
            assert!(
                !m.is_excluded_path(&rel),
                "a project rooted at {root:?} was excluded by its own ancestor directories"
            );
        }
    }

    /// Without relativization the same paths are (wrongly) excluded -- this
    /// pins the bug so it cannot silently return.
    #[test]
    fn absolute_paths_would_be_excluded_without_relativization() {
        let m = PatternMatcher::new();
        assert!(m.is_excluded_path("/tmp/checkout/src/main.rs"));
        assert!(m.is_excluded_path("/build/ci/workspace/src/main.rs"));
    }

    #[test]
    fn relative_path_falls_back_to_file_name_outside_root() {
        let m = PatternMatcher::new();
        let root = PathBuf::from("/tmp/project");
        let outside = PathBuf::from("/elsewhere/build/main.rs");
        assert_eq!(m.relative_path(&outside, Some(&root)), "main.rs");
    }

    #[test]
    fn binary_extensions_are_case_insensitive() {
        let m = PatternMatcher::new();
        for ext in ["png", "PNG", "PnG", "exe", "EXE"] {
            assert!(m.is_binary_file(ext), "{ext} should be binary");
        }
        for ext in ["rs", "py", "RS"] {
            assert!(!m.is_binary_file(ext), "{ext} should not be binary");
        }
    }

    #[test]
    fn matcher_is_cheap_to_clone_and_send() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<PatternMatcher>();
        assert_eq!(std::mem::size_of::<PatternMatcher>(), 0);
    }
}
