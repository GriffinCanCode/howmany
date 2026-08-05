//! Path classification patterns.
//!
//! Three invariants govern this module:
//!
//! 1. **Patterns are matched against a repository-relative path, never an
//!    absolute one.** `build/`, `tmp/`, `env/`, `bin/` and friends are ordinary
//!    words that routinely appear in a user's home directory or in a CI
//!    workspace root. Matching them against an absolute path made the whole
//!    analysis collapse to zero files depending on where the project happened
//!    to live. Callers are responsible for relativizing; see
//!    [`PatternMatcher::relative_path`].
//!
//! 2. **Build patterns match whole path segments.** `log/` names a directory
//!    called `log`, not any directory whose name merely ends in those letters.
//!    Left unanchored it also matched `billog/`, `catalog/`, `blog/` and
//!    `changelog/`, and `out/` matched `layout/`, `rollout/` and `checkout/` --
//!    so entire hand-written source trees vanished from the report with no
//!    indication that anything had been skipped. See [`segment_anchored`].
//!
//! 3. **Matching is one pass, not one pass per pattern.** All patterns in a
//!    category are compiled into a single [`RegexSet`], which evaluates every
//!    alternative in a single scan of the input.

use regex::{Regex, RegexSet};
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::LazyLock;

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
/// Every entry is anchored to a path-segment boundary when it is compiled (see
/// [`segment_anchored`]), so it is written as if it began at the start of a
/// segment. A pattern that must match a segment *suffix* -- an Xcode bundle is
/// `MyApp.xcodeproj`, never `.xcodeproj` -- says so with an explicit `[^/]*`.
///
/// Entries containing a `/` are genuinely multi-segment and can only be decided
/// from a path; single-segment entries are additionally used to prune whole
/// directories during traversal (see [`PRUNE_DIR_SET`]).
const LANGUAGE_BUILD_SPECS: &[(&str, &[&str])] = &[
    (
        "nodejs",
        &[
            r"node_modules/",
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
        ],
    ),
    ("rust", &[r"\.cargo/", r"\.rustup/"]),
    (
        "java",
        &[r"\.gradle/", r"\.m2/", r"\.mvn/", r"\.sbt/", r"\.ivy2/"],
    ),
    ("go", &[r"\.go/pkg/"]),
    (
        "cpp",
        &[r"\.ccache/", r"\.sccache/", r"cmake-build-", r"CMakeFiles/"],
    ),
    ("dotnet", &[r"\.nuget/", r"TestResults/", r"\.publish/"]),
    (
        "php",
        &[
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
            r"Carthage/",
            r"[^/]*\.xcodeproj/",
            r"[^/]*\.xcworkspace/",
            r"[^/]*\.xcarchive/",
        ],
    ),
];

/// What must be true on disk for an ambiguously-named directory to be build
/// output.
///
/// `beside` names files in the directory that *contains* the candidate --
/// `packages/` is a NuGet restore directory when a solution sits next to it,
/// and a pnpm workspace otherwise. `within` names files inside the candidate
/// itself, which is how a virtualenv identifies itself.
struct Evidence {
    beside: &'static [&'static str],
    within: &'static [&'static str],
}

/// Directory names that mean "build output" only in the right company.
///
/// These are ordinary words, and every one of them is a real source directory
/// somewhere. Treating the name alone as proof deleted hand-written code from
/// the report with no indication anything had been skipped: `packages/` is the
/// source root of every pnpm and yarn workspace, and it cost one monorepo its
/// entire web client -- 588 TypeScript files and 252,000 lines -- because the
/// name also happens to be where NuGet restores its dependencies. `build/`
/// took a Go package that builds things, and `vendor/` took a directory of
/// vendored *source* that was being read, not generated.
///
/// So the name only raises the question, and the file system answers it.
const AMBIGUOUS_BUILD_DIRS: &[(&str, Evidence)] = &[
    (
        "packages",
        Evidence {
            beside: &["*.sln", "*.csproj", "packages.config", "nuget.config"],
            within: &[],
        },
    ),
    (
        "bin",
        Evidence {
            beside: &["*.sln", "*.csproj", "*.fsproj", "*.vbproj"],
            within: &[],
        },
    ),
    (
        "obj",
        Evidence {
            beside: &["*.sln", "*.csproj", "*.fsproj", "*.vbproj"],
            within: &[],
        },
    ),
    (
        "publish",
        Evidence {
            beside: &["*.sln", "*.csproj", "*.fsproj", "*.vbproj"],
            within: &[],
        },
    ),
    (
        "build",
        Evidence {
            beside: &[
                "CMakeLists.txt",
                "meson.build",
                "pom.xml",
                "build.gradle",
                "build.gradle.kts",
                "settings.gradle",
                "settings.gradle.kts",
            ],
            within: &[],
        },
    ),
    (
        "target",
        Evidence {
            beside: &["Cargo.toml", "pom.xml", "build.gradle", "build.gradle.kts"],
            within: &[".rustc_info.json"],
        },
    ),
    (
        "dist",
        Evidence {
            beside: &["package.json", "pyproject.toml", "setup.py", "setup.cfg"],
            within: &[],
        },
    ),
    (
        "out",
        Evidence {
            beside: &["package.json", "tsconfig.json"],
            within: &[],
        },
    ),
    (
        "coverage",
        Evidence {
            beside: &["package.json", "Gemfile", "pyproject.toml", ".coveragerc"],
            within: &[],
        },
    ),
    (
        "vendor",
        Evidence {
            beside: &["go.mod", "composer.json", "Gemfile"],
            within: &["modules.txt", "autoload.php"],
        },
    ),
    (
        "Pods",
        Evidence {
            beside: &["Podfile"],
            within: &["Manifest.lock"],
        },
    ),
    (
        "log",
        Evidence {
            beside: &["Gemfile", "config.ru"],
            within: &[],
        },
    ),
    (
        "tmp",
        Evidence {
            beside: &["Gemfile", "config.ru"],
            within: &[],
        },
    ),
    (
        "storage",
        Evidence {
            beside: &["composer.json", "artisan"],
            within: &[],
        },
    ),
    (
        "env",
        Evidence {
            beside: &[],
            within: VENV_MARKERS,
        },
    ),
    (
        ".env",
        Evidence {
            beside: &[],
            within: VENV_MARKERS,
        },
    ),
    (
        "venv",
        Evidence {
            beside: &[],
            within: VENV_MARKERS,
        },
    ),
    (
        ".venv",
        Evidence {
            beside: &[],
            within: VENV_MARKERS,
        },
    ),
    (
        "virtualenv",
        Evidence {
            beside: &[],
            within: VENV_MARKERS,
        },
    ),
];

/// How a Python virtual environment identifies itself from the inside.
const VENV_MARKERS: &[&str] = &["pyvenv.cfg", "bin/activate", "Scripts/activate.bat"];

static AMBIGUOUS_DIR_MAP: LazyLock<HashMap<&'static str, &'static Evidence>> =
    LazyLock::new(|| AMBIGUOUS_BUILD_DIRS.iter().map(|(n, e)| (*n, e)).collect());

/// True when `directory` contains something matching `marker`.
///
/// A marker is either a literal name or `*.ext`; the wildcard form has to read
/// the directory, so it is tried only after the literal names have missed.
fn marker_present(directory: &Path, marker: &str) -> bool {
    match marker.strip_prefix("*.") {
        None => directory.join(marker).exists(),
        Some(extension) => std::fs::read_dir(directory).is_ok_and(|entries| {
            entries.flatten().any(|entry| {
                Path::new(&entry.file_name())
                    .extension()
                    .is_some_and(|found| found.eq_ignore_ascii_case(extension))
            })
        }),
    }
}

/// The cross-tool convention for "this directory is a cache": a tag file
/// standardised for exactly this purpose, which Cargo and others write.
/// Evidence enough on its own, whatever the directory is called.
const CACHE_TAG: &str = "CACHEDIR.TAG";

/// True when `path` -- a directory whose name is in [`AMBIGUOUS_BUILD_DIRS`] --
/// is really build output.
fn is_corroborated_build_dir(path: &Path, name: &str) -> bool {
    let Some(evidence) = AMBIGUOUS_DIR_MAP.get(name) else {
        return false;
    };
    if marker_present(path, CACHE_TAG) {
        return true;
    }
    evidence
        .within
        .iter()
        .any(|marker| marker_present(path, marker))
        || path.parent().is_some_and(|parent| {
            evidence
                .beside
                .iter()
                .any(|marker| marker_present(parent, marker))
        })
}

/// Bind `pattern` to the start of a path segment.
///
/// Every build pattern names a directory or a file, and a name only means
/// anything as a whole path component. Without this, matching is by substring:
/// `log/` swallowed `libs/kernels/billog/`, `out/` swallowed `web/layout/`, and
/// the files inside were dropped from the analysis entirely.
fn segment_anchored(pattern: &str) -> String {
    format!(r"(?:^|/){pattern}")
}

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
    // protoc-gen-es / protoc-gen-connect-es, the TypeScript equivalents of
    // the `.pb.go` convention above
    "_pb.ts",
    "_pb.js",
    "_pb.d.ts",
    "_connect.ts",
    "_connect.d.ts",
    "_connectquery.ts",
    "_connectquery.d.ts",
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

static OS_SET: LazyLock<RegexSet> = LazyLock::new(|| compile_set(OS_PATTERNS));
static IDE_SET: LazyLock<RegexSet> = LazyLock::new(|| compile_set(IDE_PATTERNS));
static TEMP_SET: LazyLock<RegexSet> = LazyLock::new(|| compile_set(TEMP_PATTERNS));
static VCS_SET: LazyLock<RegexSet> = LazyLock::new(|| compile_set(VCS_PATTERNS));

/// Every "ignore this outright" pattern, evaluated in a single scan.
static IGNORE_SET: LazyLock<RegexSet> = LazyLock::new(|| {
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
static BUILD_SET: LazyLock<RegexSet> = LazyLock::new(|| {
    let all: Vec<String> = LANGUAGE_BUILD_SPECS
        .iter()
        .flat_map(|(_, pats)| pats.iter())
        .map(|p| segment_anchored(p))
        .collect();
    RegexSet::new(&all).expect("built-in patterns must compile")
});

static LANGUAGE_BUILD_PATTERNS: LazyLock<HashMap<&'static str, Vec<Regex>>> = LazyLock::new(|| {
    LANGUAGE_BUILD_SPECS
        .iter()
        .map(|(lang, pats)| {
            let compiled = pats
                .iter()
                .map(|p| Regex::new(&segment_anchored(p)).expect("built-in patterns must compile"))
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
static PRUNE_DIR_SET: LazyLock<HashSet<String>> = LazyLock::new(|| {
    LANGUAGE_BUILD_SPECS
        .iter()
        .flat_map(|(_, patterns)| patterns.iter())
        .filter_map(|pattern| prunable_dir_name(pattern))
        .chain(PRUNE_DIRS_NON_BUILD.iter().map(|d| d.to_string()))
        .collect()
});

static BINARY_EXTENSION_SET: LazyLock<HashSet<&'static str>> =
    LazyLock::new(|| BINARY_EXTENSIONS.iter().copied().collect());

static GENERATED_NAME_SET: LazyLock<HashSet<&'static str>> =
    LazyLock::new(|| GENERATED_NAMES.iter().copied().collect());

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

static BOILERPLATE_STEM_SET: LazyLock<HashSet<&'static str>> =
    LazyLock::new(|| BOILERPLATE_STEMS.iter().copied().collect());

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

    /// True when `path` can be skipped: either its name is unconditionally
    /// build output, or its name is ambiguous and the surrounding files say it
    /// is. See [`AMBIGUOUS_BUILD_DIRS`].
    pub fn is_prunable_dir_at(&self, path: &Path, dir_name: &str) -> bool {
        self.is_prunable_dir(dir_name) || is_corroborated_build_dir(path, dir_name)
    }

    /// True when `path` lies inside a build output or dependency cache.
    ///
    /// Unlike [`Self::matches_build_cache_pattern`] this can also decide the
    /// ambiguously-named directories, because it is given the real path and
    /// can look for the toolchain that would have produced them.
    pub fn is_build_output(&self, path: &Path, root: Option<&Path>) -> bool {
        let relative = self.relative_path(path, root);
        if self.matches_build_cache_pattern(&relative) {
            return true;
        }

        // Only pay for filesystem checks when a segment name asks for them.
        let mut prefix = root.map(Path::to_path_buf).unwrap_or_default();
        let mut ancestors = Path::new(relative.as_ref()).components().peekable();
        while let Some(component) = ancestors.next() {
            prefix.push(component);
            if ancestors.peek().is_none() {
                break; // The file itself, not a directory containing it.
            }
            let name = component.as_os_str().to_string_lossy();
            if AMBIGUOUS_DIR_MAP.contains_key(name.as_ref())
                && is_corroborated_build_dir(&prefix, &name)
            {
                return true;
            }
        }
        false
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

    /// Lay out `paths` (a trailing `/` makes a directory) under a temp root.
    fn tree(paths: &[&str]) -> tempfile::TempDir {
        let dir = tempfile::TempDir::new().expect("temp dir");
        for path in paths {
            let full = dir.path().join(path.trim_end_matches('/'));
            if path.ends_with('/') {
                std::fs::create_dir_all(&full).expect("mkdir");
            } else {
                if let Some(parent) = full.parent() {
                    std::fs::create_dir_all(parent).expect("mkdir");
                }
                std::fs::write(&full, "x\n").expect("write");
            }
        }
        dir
    }

    /// `dist/` is the most common build output name in the JavaScript and
    /// Python ecosystems; leaving it out silently doubled the reported size of
    /// any project that ships a bundle.
    #[test]
    fn common_build_output_directories_are_excluded() {
        let m = PatternMatcher::new();
        let project = tree(&[
            "Cargo.toml",
            "package.json",
            "CMakeLists.txt",
            "dist/bundle.js",
            "out/index.html",
            "node_modules/left-pad/index.js",
            "target/debug/deps/x.rs",
            "build/classes/A.class",
            "src/distance.rs",
            "src/outbox/handler.rs",
        ]);
        let root = project.path();

        for probe in [
            "dist/bundle.js",
            "out/index.html",
            "node_modules/left-pad/index.js",
            "target/debug/deps/x.rs",
            "build/classes/A.class",
        ] {
            assert!(
                m.is_build_output(&root.join(probe), Some(root)),
                "{probe:?} should be excluded"
            );
        }
        for probe in ["src/distance.rs", "src/outbox/handler.rs"] {
            assert!(!m.is_build_output(&root.join(probe), Some(root)));
        }
    }

    /// An ambiguously-named directory is build output only when the toolchain
    /// that would have produced it is there too. Without that check, `packages/`
    /// -- the source root of every pnpm and yarn workspace -- was deleted from
    /// the report because it is also where NuGet restores dependencies.
    #[test]
    fn ambiguous_directory_names_need_a_toolchain_to_be_build_output() {
        let m = PatternMatcher::new();
        let project = tree(&[
            "packages/ui/src/button.ts",
            "build/pipeline.go",
            "vendor/graphify/main.go",
            "out/render.go",
            "internal/log/logger.go",
            "env/settings.py",
        ]);
        let root = project.path();

        for probe in [
            "packages/ui/src/button.ts",
            "build/pipeline.go",
            "vendor/graphify/main.go",
            "out/render.go",
            "internal/log/logger.go",
            "env/settings.py",
        ] {
            assert!(
                !m.is_build_output(&root.join(probe), Some(root)),
                "{probe:?} was discarded as build output, but nothing in the \
                 project produces build output under that name"
            );
        }
    }

    /// A cache may also identify itself, whatever it is called: `CACHEDIR.TAG`
    /// is the cross-tool convention for saying so.
    #[test]
    fn a_directory_that_tags_itself_as_a_cache_is_build_output() {
        let m = PatternMatcher::new();
        let project = tree(&["out/CACHEDIR.TAG", "out/app.js", "venv/pyvenv.cfg"]);
        let root = project.path();

        assert!(m.is_build_output(&root.join("out/app.js"), Some(root)));
        assert!(m.is_prunable_dir_at(&root.join("venv"), "venv"));
        assert!(!m.is_prunable_dir_at(&root.join("out/nope"), "nope"));
    }

    /// A build pattern names a whole path segment. Matching it as a substring
    /// deleted entire source trees: `log/` claimed `libs/kernels/billog/`,
    /// `catalog/`, `blog/` and `changelog/`; `out/` claimed `layout/`,
    /// `rollout/` and `scout/`. On one monorepo that silently discarded 272
    /// hand-written files, including every Zig source in the project.
    #[test]
    fn build_patterns_do_not_match_partial_segment_names() {
        let m = PatternMatcher::new();
        for probe in [
            "libs/kernels/billog/src/root.zig",
            "api/handler/admin/catalog/routes.go",
            "site/blog/post.md",
            "docs/changelog/2024.md",
            "web/components/layout/Grid.tsx",
            "ops/rollout/plan.py",
            "tools/scout/main.rs",
            "src/distance/metric.rs",
            "src/binary/search.c",
            "src/environment/setup.py",
            "third_party/vendored_notes.md",
        ] {
            assert!(
                !m.is_excluded_path(probe),
                "{probe:?} was excluded because a directory name merely contains \
                 a build word as a substring"
            );
        }
    }

    /// The other half of the same rule: a genuine build directory must still be
    /// excluded wherever it appears in the path.
    #[test]
    fn build_patterns_still_match_whole_segments_at_any_depth() {
        let m = PatternMatcher::new();
        let project = tree(&[
            "var/Gemfile",
            "var/log/app.txt",
            "packages/ui/package.json",
            "packages/ui/out/index.html",
            "services/web/node_modules/left-pad/index.js",
            "crates/core/Cargo.toml",
            "crates/core/target/debug/x.rs",
            "apps/api/__pycache__/m.pyc",
        ]);
        let root = project.path();
        for probe in [
            "var/log/app.txt",
            "packages/ui/out/index.html",
            "services/web/node_modules/left-pad/index.js",
            "crates/core/target/debug/x.rs",
            "apps/api/__pycache__/m.pyc",
        ] {
            assert!(
                m.is_build_output(&root.join(probe), Some(root)),
                "{probe:?} should be excluded"
            );
        }
    }

    /// An Xcode bundle is `MyApp.xcodeproj`, so its pattern has to match a
    /// segment *suffix*. Anchoring every pattern to a segment start without
    /// saying so would have stopped excluding them.
    #[test]
    fn segment_suffix_patterns_still_match_their_bundles() {
        let m = PatternMatcher::new();
        for probe in [
            "MyApp.xcodeproj/project.pbxproj",
            "ios/MyApp.xcworkspace/contents.xcworkspacedata",
            "build/MyApp.xcarchive/Info.plist",
        ] {
            assert!(m.is_excluded_path(probe), "{probe:?} should be excluded");
        }
    }

    #[test]
    fn segment_anchoring_binds_to_a_component_boundary() {
        let anchored = Regex::new(&segment_anchored("log/")).unwrap();
        assert!(anchored.is_match("log/x"));
        assert!(anchored.is_match("var/log/x"));
        assert!(!anchored.is_match("billog/x"));
        assert!(!anchored.is_match("catalog/x"));
    }

    #[test]
    fn build_patterns_match_relative_locations() {
        let m = PatternMatcher::new();
        assert!(m.matches_build_cache_pattern("node_modules/left-pad/index.js"));
        assert!(m.matches_build_cache_pattern("__pycache__/mod.pyc"));
        assert!(m.matches_build_cache_pattern("DerivedData/Build/x.o"));
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
        assert!(m.is_excluded_path("/home/dev/node_modules/repo/src/main.rs"));
        assert!(m.is_excluded_path("/srv/DerivedData/workspace/src/main.rs"));
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
