//! Comment syntax per file extension.
//!
//! The table is a process-wide lazy static of `&'static str` slices. It used to
//! be rebuilt -- roughly five hundred `String` allocations plus a `HashMap` --
//! every time a `CodeCounter` was constructed, which made per-thread counters
//! prohibitively expensive. Now construction is free and lookups hand back a
//! `Copy` descriptor, so nothing is cloned per file either.

use std::collections::HashMap;
use std::sync::LazyLock;

/// How a language spells its comments.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CommentPattern {
    /// Prefixes that make the rest of the line a comment.
    pub single_line: &'static [&'static str],
    /// Tokens that open a block comment.
    pub multi_line_start: &'static [&'static str],
    /// Tokens that close a block comment, positionally paired with
    /// `multi_line_start`.
    pub multi_line_end: &'static [&'static str],
    /// Prefixes that mark documentation rather than an ordinary comment.
    pub doc_patterns: &'static [&'static str],
}

impl CommentPattern {
    /// A language with no comment syntax we know about.
    pub const EMPTY: Self = Self {
        single_line: &[],
        multi_line_start: &[],
        multi_line_end: &[],
        doc_patterns: &[],
    };

    const fn new(
        single_line: &'static [&'static str],
        multi_line_start: &'static [&'static str],
        multi_line_end: &'static [&'static str],
        doc_patterns: &'static [&'static str],
    ) -> Self {
        Self {
            single_line,
            multi_line_start,
            multi_line_end,
            doc_patterns,
        }
    }
}

const C_STYLE: CommentPattern = CommentPattern::new(&["//"], &["/*"], &["*/"], &["/**", "/*!"]);
const JS_STYLE: CommentPattern = CommentPattern::new(&["//"], &["/*"], &["*/"], &["/**", "//!"]);
const JAVADOC_STYLE: CommentPattern = CommentPattern::new(&["//"], &["/*"], &["*/"], &["/**"]);
const TRIPLE_SLASH_STYLE: CommentPattern =
    CommentPattern::new(&["//"], &["/*"], &["*/"], &["///", "/**"]);
const HASH_ONLY: CommentPattern = CommentPattern::new(&["#"], &[], &[], &["##"]);
/// `<!-- ... -->` is a comment, not documentation. Listing it as a doc pattern
/// made every HTML and XML comment count as documentation and left those
/// languages reporting zero comment lines.
const XML_STYLE: CommentPattern = CommentPattern::new(&[], &["<!--"], &["-->"], &[]);
const FSHARP_STYLE: CommentPattern =
    CommentPattern::new(&["//"], &["(*"], &["*)"], &["///", "(**"]);
const PERL_STYLE: CommentPattern = CommentPattern::new(&["#"], &["=pod"], &["=cut"], &["=pod"]);
const R_STYLE: CommentPattern = CommentPattern::new(&["#"], &[], &[], &["#'"]);
const RMD_STYLE: CommentPattern = CommentPattern::new(&["#"], &["<!--"], &["-->"], &[]);
const MATLAB_STYLE: CommentPattern = CommentPattern::new(&["%"], &["%{"], &["%}"], &["%%"]);
const BATCH_STYLE: CommentPattern = CommentPattern::new(&["REM", "rem", "::"], &[], &[], &["REM"]);
const ELIXIR_STYLE: CommentPattern = CommentPattern::new(&["#"], &[], &[], &["@doc", "@moduledoc"]);
const LISP_STYLE: CommentPattern = CommentPattern::new(&[";"], &["#_"], &[], &[";;"]);
const ML_STYLE: CommentPattern = CommentPattern::new(&[], &["(*"], &["*)"], &["(**"]);
const HASKELL_STYLE: CommentPattern =
    CommentPattern::new(&["--"], &["{-"], &["-}"], &["-- |", "-- ^"]);
const WEB_COMPONENT_STYLE: CommentPattern =
    CommentPattern::new(&["//"], &["<!--", "/*"], &["-->", "*/"], &["/**"]);
const ASCIIDOC_STYLE: CommentPattern = CommentPattern::new(&["//"], &["////"], &["////"], &[]);
/// Prose formats: only an HTML comment is a comment. Prose itself is classified
/// as documentation by `scanner::classify_markdown`, not by this pattern.
const MARKDOWN_STYLE: CommentPattern = CommentPattern::new(&[], &["<!--"], &["-->"], &[]);

/// Extension -> comment syntax.
///
/// Where several extensions share a syntax they share one descriptor; the table
/// is intentionally flat so a lookup is a single hash.
const PATTERN_TABLE: &[(&str, CommentPattern)] = &[
    // Rust
    (
        "rs",
        CommentPattern::new(&["//"], &["/*"], &["*/"], &["///", "//!", "/**"]),
    ),
    // JavaScript family
    ("js", JS_STYLE),
    ("ts", JS_STYLE),
    ("jsx", JS_STYLE),
    ("tsx", JS_STYLE),
    ("mjs", JS_STYLE),
    ("cjs", JS_STYLE),
    ("mts", JS_STYLE),
    ("cts", JS_STYLE),
    // Python
    (
        "py",
        CommentPattern::new(
            &["#"],
            &["\"\"\"", "'''"],
            &["\"\"\"", "'''"],
            &["\"\"\"", "'''"],
        ),
    ),
    (
        "pyi",
        CommentPattern::new(
            &["#"],
            &["\"\"\"", "'''"],
            &["\"\"\"", "'''"],
            &["\"\"\"", "'''"],
        ),
    ),
    // JVM
    ("java", JAVADOC_STYLE),
    ("kt", JAVADOC_STYLE),
    ("kts", JAVADOC_STYLE),
    ("scala", JAVADOC_STYLE),
    ("groovy", JAVADOC_STYLE),
    // C family
    ("c", C_STYLE),
    ("cpp", C_STYLE),
    ("cc", C_STYLE),
    ("cxx", C_STYLE),
    ("h", C_STYLE),
    ("hpp", C_STYLE),
    ("hxx", C_STYLE),
    ("mm", JAVADOC_STYLE),
    ("cs", TRIPLE_SLASH_STYLE),
    // PHP
    (
        "php",
        CommentPattern::new(&["//", "#"], &["/*"], &["*/"], &["/**"]),
    ),
    // Ruby
    (
        "rb",
        CommentPattern::new(&["#"], &["=begin"], &["=end"], &["##"]),
    ),
    (
        "rake",
        CommentPattern::new(&["#"], &["=begin"], &["=end"], &["##"]),
    ),
    // Go uses // for documentation as well as comments.
    (
        "go",
        CommentPattern::new(&["//"], &["/*"], &["*/"], &["//"]),
    ),
    // Apple platforms
    ("swift", TRIPLE_SLASH_STYLE),
    // Shells
    ("sh", HASH_ONLY),
    ("bash", HASH_ONLY),
    ("zsh", HASH_ONLY),
    ("fish", HASH_ONLY),
    // Data / config
    ("yaml", HASH_ONLY),
    ("yml", HASH_ONLY),
    ("toml", HASH_ONLY),
    ("ini", CommentPattern::new(&[";", "#"], &[], &[], &[";;"])),
    ("json", CommentPattern::new(&["//"], &["/*"], &["*/"], &[])),
    ("jsonc", CommentPattern::new(&["//"], &["/*"], &["*/"], &[])),
    ("xml", XML_STYLE),
    ("html", XML_STYLE),
    ("htm", XML_STYLE),
    // `svg` is deliberately absent: it is listed in `patterns::BINARY_EXTENSIONS`
    // as the image asset it is. Registering syntax for it here contradicted that
    // and let the engine -- which classifies without consulting the file filter
    // -- report thousands of lines of generated path data as source.
    // Stylesheets
    ("css", CommentPattern::new(&[], &["/*"], &["*/"], &["/**"])),
    (
        "scss",
        CommentPattern::new(&["//"], &["/*"], &["*/"], &["/**", "///"]),
    ),
    ("sass", CommentPattern::new(&["//"], &[], &[], &["///"])),
    ("less", JAVADOC_STYLE),
    // Frontend components
    ("vue", WEB_COMPONENT_STYLE),
    ("svelte", WEB_COMPONENT_STYLE),
    // Markup / docs
    ("md", MARKDOWN_STYLE),
    ("markdown", MARKDOWN_STYLE),
    ("rst", CommentPattern::new(&[".."], &[], &[], &[])),
    ("adoc", ASCIIDOC_STYLE),
    ("asciidoc", ASCIIDOC_STYLE),
    // Functional
    ("hs", HASKELL_STYLE),
    ("lhs", HASKELL_STYLE),
    (
        "elm",
        CommentPattern::new(&["--"], &["{-"], &["-}"], &["{-|"]),
    ),
    ("ml", ML_STYLE),
    ("mli", ML_STYLE),
    ("fs", FSHARP_STYLE),
    ("fsx", FSHARP_STYLE),
    ("fsi", FSHARP_STYLE),
    ("clj", LISP_STYLE),
    ("cljs", LISP_STYLE),
    ("cljc", LISP_STYLE),
    ("erl", CommentPattern::new(&["%"], &[], &[], &["%%"])),
    ("hrl", CommentPattern::new(&["%"], &[], &[], &["%%"])),
    ("ex", ELIXIR_STYLE),
    ("exs", ELIXIR_STYLE),
    // Scientific
    (
        "jl",
        CommentPattern::new(&["#"], &["#="], &["=#"], &["\"\"\""]),
    ),
    ("r", R_STYLE),
    ("R", R_STYLE),
    ("rmd", RMD_STYLE),
    ("Rmd", RMD_STYLE),
    ("m", MATLAB_STYLE),
    ("mlx", MATLAB_STYLE),
    // Scripting
    (
        "lua",
        CommentPattern::new(&["--"], &["--[["], &["]]"], &["---"]),
    ),
    ("pl", PERL_STYLE),
    ("pm", PERL_STYLE),
    ("pod", PERL_STYLE),
    (
        "ps1",
        CommentPattern::new(&["#"], &["<#"], &["#>"], &["<#"]),
    ),
    (
        "psm1",
        CommentPattern::new(&["#"], &["<#"], &["#>"], &["<#"]),
    ),
    ("bat", BATCH_STYLE),
    ("cmd", BATCH_STYLE),
    // Others
    (
        "sql",
        CommentPattern::new(&["--"], &["/*"], &["*/"], &["--"]),
    ),
    ("dart", TRIPLE_SLASH_STYLE),
    (
        "zig",
        CommentPattern::new(&["//"], &[], &[], &["///", "//!"]),
    ),
    ("proto", JAVADOC_STYLE),
    ("tf", HASH_ONLY),
    ("nix", HASH_ONLY),
    ("gradle", JAVADOC_STYLE),
    // Extension-less project files, keyed by their lowercase file name (see
    // `counter::classify_key`). Every one of these is hand-written and carries
    // real comment syntax, so leaving them out both dropped them from the
    // totals and, when they were counted, called every line code.
    ("dockerfile", HASH_ONLY),
    ("containerfile", HASH_ONLY),
    ("makefile", HASH_ONLY),
    ("gnumakefile", HASH_ONLY),
    ("justfile", HASH_ONLY),
    ("rakefile", HASH_ONLY),
    ("gemfile", HASH_ONLY),
    ("podfile", HASH_ONLY),
    ("brewfile", HASH_ONLY),
    ("procfile", HASH_ONLY),
    ("vagrantfile", HASH_ONLY),
    ("berksfile", HASH_ONLY),
    ("fastfile", HASH_ONLY),
    ("appfile", HASH_ONLY),
    ("jenkinsfile", JAVADOC_STYLE),
    ("dockerignore", HASH_ONLY),
    ("gitignore", HASH_ONLY),
    ("gitattributes", HASH_ONLY),
    ("gitmodules", HASH_ONLY),
    ("npmignore", HASH_ONLY),
    ("prettierignore", HASH_ONLY),
    ("eslintignore", HASH_ONLY),
    ("codeowners", HASH_ONLY),
    ("editorconfig", HASH_ONLY),
    // Extension-less prose a human actually writes. LICENSE, COPYING and NOTICE
    // are deliberately absent: they are boilerplate nobody in the project
    // authored, and a 200-line Apache licence counted as documentation
    // misrepresents the project it sits in.
    ("readme", MARKDOWN_STYLE),
    ("changelog", MARKDOWN_STYLE),
    ("contributing", MARKDOWN_STYLE),
];

static PATTERNS: LazyLock<HashMap<&'static str, CommentPattern>> =
    LazyLock::new(|| PATTERN_TABLE.iter().copied().collect());

/// Comment syntax for a lowercase extension, if known.
pub fn lookup(extension: &str) -> Option<CommentPattern> {
    PATTERNS.get(extension).copied()
}

/// Comment syntax for `extension`, falling back to "no comment syntax".
pub fn lookup_or_empty(extension: &str) -> CommentPattern {
    lookup(extension).unwrap_or(CommentPattern::EMPTY)
}

/// True when the extension has a registered comment syntax.
pub fn is_known(extension: &str) -> bool {
    PATTERNS.contains_key(extension)
}

/// Formats whose ordinary lines are prose rather than code.
///
/// These are classified by `scanner::classify_markdown`, which counts prose as
/// documentation and only fenced or indented blocks as code. Classifying them
/// with a comment pattern instead would report a README as pure code.
const PROSE_FORMATS: &[&str] = &[
    "md",
    "markdown",
    "rst",
    "adoc",
    "asciidoc",
    "readme",
    "changelog",
    "contributing",
];

pub fn is_prose_format(key: &str) -> bool {
    PROSE_FORMATS.contains(&key)
}

/// Every extension with a registered comment syntax.
pub fn known_extensions() -> impl Iterator<Item = &'static str> {
    PATTERN_TABLE.iter().map(|(ext, _)| *ext)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_has_no_conflicting_duplicates() {
        let mut seen: HashMap<&str, CommentPattern> = HashMap::new();
        for (ext, pattern) in PATTERN_TABLE {
            if let Some(existing) = seen.insert(ext, *pattern) {
                assert_eq!(
                    existing, *pattern,
                    "extension {ext:?} is listed twice with different comment syntax"
                );
            }
        }
    }

    /// A block-comment opener with no paired closer would leave the counter
    /// stuck in "inside a comment" for the rest of the file.
    #[test]
    fn block_delimiters_are_paired_or_deliberately_unpaired() {
        for (ext, p) in PATTERN_TABLE {
            if p.multi_line_start.is_empty() {
                assert!(
                    p.multi_line_end.is_empty(),
                    "{ext:?} has block closers but no openers"
                );
                continue;
            }
            // Clojure's `#_` elides the next form and has no closer by design.
            if *ext == "clj" || *ext == "cljs" || *ext == "cljc" {
                continue;
            }
            assert_eq!(
                p.multi_line_start.len(),
                p.multi_line_end.len(),
                "{ext:?} has {} block openers but {} closers",
                p.multi_line_start.len(),
                p.multi_line_end.len()
            );
        }
    }

    #[test]
    fn no_pattern_is_an_empty_string() {
        for (ext, p) in PATTERN_TABLE {
            for group in [
                p.single_line,
                p.multi_line_start,
                p.multi_line_end,
                p.doc_patterns,
            ] {
                assert!(
                    group.iter().all(|s| !s.is_empty()),
                    "{ext:?} contains an empty pattern, which would match every line"
                );
            }
        }
    }

    #[test]
    fn lookups_resolve_expected_languages() {
        assert_eq!(lookup("rs").unwrap().single_line, &["//"]);
        assert!(lookup("rs").unwrap().doc_patterns.contains(&"///"));
        assert_eq!(lookup("py").unwrap().single_line, &["#"]);
        assert!(lookup("py").unwrap().multi_line_start.contains(&"\"\"\""));
        assert!(lookup("unknown-ext").is_none());
        assert_eq!(lookup_or_empty("unknown-ext"), CommentPattern::EMPTY);
    }

    #[test]
    fn every_extension_is_lowercase_or_deliberately_cased() {
        // Two R extensions are conventionally capitalised on disk.
        let allowed_uppercase = ["R", "Rmd"];
        for (ext, _) in PATTERN_TABLE {
            assert!(
                ext.chars().all(|c| !c.is_ascii_uppercase()) || allowed_uppercase.contains(ext),
                "{ext:?} would never be found: lookups use a lowercase extension"
            );
        }
    }
}
