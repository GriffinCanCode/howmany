//! What a file *is*, as opposed to what it is called.
//!
//! Reporting was previously keyed on the raw filename extension, which is not
//! the same question. Three things went wrong because of it.
//!
//! *Extensions are not languages.* A monorepo carries `contracts/net.firewall`,
//! `budget.budget` and `resolver.dns`; listing those beside `go` and `py` reads
//! as if the project were written in eleven exotic languages. Conversely `yml`
//! and `yaml`, or `mk` and `Makefile`, are one language reported as two.
//!
//! *Extension-less files lost their identity.* Every `Dockerfile`, `Makefile`
//! and `Justfile` in the tree aggregated into a single `no_ext` bucket, even
//! though the line counter already knew exactly what they were.
//!
//! *Prose and data were counted as if they were source.* On a large repository
//! Markdown is the most numerous file type by a wide margin and JSON fixtures
//! are among the largest, so a single flat list ranked by file count opened
//! with documentation and test data and buried the languages the project is
//! actually written in.
//!
//! The table below is keyed by [`crate::core::counter::classify_key`], the same
//! key the comment-syntax table uses, so a language that can be *counted* can
//! always be *named*. [`every_countable_format_has_a_language`] asserts it.

use crate::core::stats::basic::ExtensionStats;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::path::Path;
use std::sync::LazyLock;

/// What a file contributes to a project.
///
/// The split exists so that a report can lead with the thing the reader asked
/// about. "How much code is there" is not answered by a number that includes
/// the changelog and the test fixtures.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Category {
    /// Hand-written instructions: source, schemas, queries, templates, build
    /// recipes. The headline number.
    Code,
    /// Files that configure a tool rather than instruct a machine.
    Config,
    /// Payloads: fixtures, datasets, lockfiles, serialized records.
    Data,
    /// Prose written for people.
    Docs,
}

impl Category {
    /// Every category, in report order: code first, prose last.
    pub const ALL: [Category; 4] = [
        Category::Code,
        Category::Config,
        Category::Data,
        Category::Docs,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Category::Code => "Code",
            Category::Config => "Config",
            Category::Data => "Data",
            Category::Docs => "Docs",
        }
    }
}

impl std::fmt::Display for Category {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.label())
    }
}

/// A named format and the kind of content it holds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Language {
    pub name: &'static str,
    pub category: Category,
}

const fn code(name: &'static str) -> Language {
    Language {
        name,
        category: Category::Code,
    }
}
const fn config(name: &'static str) -> Language {
    Language {
        name,
        category: Category::Config,
    }
}
const fn data(name: &'static str) -> Language {
    Language {
        name,
        category: Category::Data,
    }
}
const fn docs(name: &'static str) -> Language {
    Language {
        name,
        category: Category::Docs,
    }
}

/// Classification key -> language.
///
/// Keys are whatever [`crate::core::counter::classify_key`] produces: a
/// lowercase extension, or a lowercase filename for a file that has none.
/// Several keys deliberately share one name -- that is the point, it is how
/// `yml` and `yaml` stop being reported as two different things.
const LANGUAGE_TABLE: &[(&str, Language)] = &[
    // Systems
    ("rs", code("Rust")),
    ("go", code("Go")),
    ("zig", code("Zig")),
    ("c", code("C")),
    ("h", code("C/C++ Header")),
    ("cpp", code("C++")),
    ("cc", code("C++")),
    ("cxx", code("C++")),
    ("hpp", code("C++ Header")),
    ("hxx", code("C++ Header")),
    ("mm", code("Objective-C++")),
    ("swift", code("Swift")),
    ("dart", code("Dart")),
    // Scripting and dynamic
    ("py", code("Python")),
    ("pyi", code("Python Stubs")),
    ("rb", code("Ruby")),
    ("rake", code("Ruby")),
    ("php", code("PHP")),
    ("pl", code("Perl")),
    ("pm", code("Perl")),
    ("pod", code("Perl")),
    ("lua", code("Lua")),
    ("jl", code("Julia")),
    ("r", code("R")),
    ("rmd", code("R Markdown")),
    ("m", code("MATLAB")),
    ("mlx", code("MATLAB")),
    // Web
    ("js", code("JavaScript")),
    ("mjs", code("JavaScript")),
    ("cjs", code("JavaScript")),
    ("jsx", code("JavaScript (React)")),
    ("ts", code("TypeScript")),
    ("mts", code("TypeScript")),
    ("cts", code("TypeScript")),
    ("tsx", code("TypeScript (React)")),
    ("vue", code("Vue")),
    ("svelte", code("Svelte")),
    ("html", code("HTML")),
    ("htm", code("HTML")),
    ("css", code("CSS")),
    ("scss", code("Sass")),
    ("sass", code("Sass")),
    ("less", code("Less")),
    ("tcss", code("Textual CSS")),
    // JVM and .NET
    ("java", code("Java")),
    ("kt", code("Kotlin")),
    ("kts", code("Kotlin")),
    ("scala", code("Scala")),
    ("groovy", code("Groovy")),
    ("gradle", code("Gradle")),
    ("cs", code("C#")),
    ("fs", code("F#")),
    ("fsx", code("F#")),
    ("fsi", code("F#")),
    // Functional
    ("hs", code("Haskell")),
    ("lhs", code("Haskell")),
    ("elm", code("Elm")),
    ("ml", code("OCaml")),
    ("mli", code("OCaml")),
    ("clj", code("Clojure")),
    ("cljs", code("Clojure")),
    ("cljc", code("Clojure")),
    ("scm", code("Scheme")),
    ("ss", code("Scheme")),
    ("rkt", code("Racket")),
    ("lisp", code("Lisp")),
    ("el", code("Emacs Lisp")),
    ("erl", code("Erlang")),
    ("hrl", code("Erlang")),
    ("ex", code("Elixir")),
    ("exs", code("Elixir")),
    // Shells
    ("sh", code("Shell")),
    ("bash", code("Shell")),
    ("zsh", code("Shell")),
    ("fish", code("Fish")),
    ("ps1", code("PowerShell")),
    ("psm1", code("PowerShell")),
    ("bat", code("Batch")),
    ("cmd", code("Batch")),
    // Interface and policy definitions: hand-written contracts, not config.
    ("sql", code("SQL")),
    ("proto", code("Protocol Buffers")),
    ("capnp", code("Cap'n Proto")),
    ("thrift", code("Thrift")),
    ("graphql", code("GraphQL")),
    ("gql", code("GraphQL")),
    ("rego", code("Rego")),
    ("cedar", code("Cedar")),
    ("cedarschema", code("Cedar")),
    ("cue", code("CUE")),
    ("tf", code("Terraform")),
    ("nix", code("Nix")),
    // Build recipes
    ("mk", code("Makefile")),
    ("makefile", code("Makefile")),
    ("gnumakefile", code("Makefile")),
    ("justfile", code("Justfile")),
    ("dockerfile", code("Dockerfile")),
    ("containerfile", code("Dockerfile")),
    ("jenkinsfile", code("Jenkinsfile")),
    ("rakefile", code("Ruby")),
    ("vagrantfile", code("Ruby")),
    ("fastfile", code("Ruby")),
    ("appfile", code("Ruby")),
    ("berksfile", code("Ruby")),
    // Templates
    ("j2", code("Jinja")),
    ("jinja", code("Jinja")),
    ("jinja2", code("Jinja")),
    ("tpl", code("Go Template")),
    ("gotmpl", code("Go Template")),
    ("hbs", code("Handlebars")),
    ("ejs", code("EJS")),
    ("erb", code("ERB")),
    ("mustache", code("Mustache")),
    // Configuration
    ("toml", config("TOML")),
    ("yaml", config("YAML")),
    ("yml", config("YAML")),
    ("ini", config("INI")),
    ("cfg", config("INI")),
    ("conf", config("Config")),
    ("properties", config("Properties")),
    ("json5", config("JSON5")),
    ("jsonc", config("JSON with Comments")),
    ("hcl", config("HCL")),
    ("tfvars", config("Terraform Vars")),
    ("plist", config("Property List")),
    ("entitlements", config("Property List")),
    ("xcprivacy", config("Property List")),
    ("xctestplan", config("Xcode Test Plan")),
    ("zon", config("Zig Object Notation")),
    ("service", config("systemd Unit")),
    ("editorconfig", config("EditorConfig")),
    ("gitignore", config("Git Config")),
    ("gitattributes", config("Git Config")),
    ("gitmodules", config("Git Config")),
    ("dockerignore", config("Ignore Rules")),
    ("npmignore", config("Ignore Rules")),
    ("prettierignore", config("Ignore Rules")),
    ("eslintignore", config("Ignore Rules")),
    ("codeowners", config("Code Owners")),
    ("brewfile", config("Brewfile")),
    ("gemfile", config("Gemfile")),
    ("podfile", config("Podfile")),
    ("procfile", config("Procfile")),
    ("env", config("Environment")),
    // Data
    ("json", data("JSON")),
    ("jsonl", data("JSON Lines")),
    ("ndjson", data("JSON Lines")),
    ("csv", data("CSV")),
    ("tsv", data("TSV")),
    ("xml", data("XML")),
    ("sum", data("Checksums")),
    ("lock", data("Lockfile")),
    ("dic", data("Dictionary")),
    ("dict", data("Dictionary")),
    // Documentation
    ("md", docs("Markdown")),
    ("markdown", docs("Markdown")),
    ("mdx", docs("MDX")),
    ("mdc", docs("Markdown")),
    ("rst", docs("reStructuredText")),
    ("adoc", docs("AsciiDoc")),
    ("asciidoc", docs("AsciiDoc")),
    ("txt", docs("Plain Text")),
    ("tex", docs("LaTeX")),
    ("readme", docs("Markdown")),
    ("changelog", docs("Markdown")),
    ("contributing", docs("Markdown")),
];

static LANGUAGES: LazyLock<HashMap<&'static str, Language>> =
    LazyLock::new(|| LANGUAGE_TABLE.iter().copied().collect());

/// The language registered for a classification key, if any.
pub fn lookup(key: &str) -> Option<Language> {
    LANGUAGES.get(key).copied()
}

/// The language of `path`, if this build recognizes it.
pub fn of_path(path: &Path) -> Option<Language> {
    lookup(&crate::core::counter::classify_key(path))
}

/// The language to report a key under, naming unrecognized keys after
/// themselves.
///
/// An unknown format is still counted -- dropping it would understate the
/// project -- but it is filed under [`Category::Data`] rather than being
/// presented as if it were a programming language somebody chose to write in.
/// A repository holding `net.firewall`, `svc.socket` and `daily.budget` was
/// reported as containing the languages Firewall, Socket and Budget.
pub fn describe(key: &str) -> (String, Category) {
    match lookup(key) {
        Some(language) => (language.name.to_string(), language.category),
        None if key.is_empty() || key == NO_EXTENSION => {
            ("Other (no extension)".to_string(), Category::Data)
        }
        None => (format!(".{key}"), Category::Data),
    }
}

/// The bucket [`crate::core::counter::extension_key`] uses for a file whose
/// name this build does not recognize.
pub const NO_EXTENSION: &str = "no_ext";

/// One language's contribution to a project.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LanguageRow {
    pub language: String,
    pub category: Category,
    pub file_count: usize,
    pub total_lines: usize,
    pub code_lines: usize,
    pub comment_lines: usize,
    pub doc_lines: usize,
    pub blank_lines: usize,
    pub total_size: u64,
}

impl LanguageRow {
    fn empty(language: String, category: Category) -> Self {
        Self {
            language,
            category,
            file_count: 0,
            total_lines: 0,
            code_lines: 0,
            comment_lines: 0,
            doc_lines: 0,
            blank_lines: 0,
            total_size: 0,
        }
    }

    fn absorb(&mut self, stats: &ExtensionStats) {
        self.file_count += stats.file_count;
        self.total_lines += stats.total_lines;
        self.code_lines += stats.code_lines;
        self.comment_lines += stats.comment_lines;
        self.doc_lines += stats.doc_lines;
        self.blank_lines += stats.blank_lines;
        self.total_size += stats.total_size;
    }

    /// Lines this language contributes out of `whole`, as a percentage.
    pub fn share_of(&self, whole: usize) -> f64 {
        if whole == 0 {
            0.0
        } else {
            100.0 * self.total_lines as f64 / whole as f64
        }
    }
}

/// A project's languages, grouped into categories.
///
/// Built once and read by every renderer, so the text report, the JSON document
/// and the interactive view cannot disagree about what the project is made of.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Breakdown {
    pub rows: Vec<LanguageRow>,
}

impl Breakdown {
    /// Roll per-extension statistics up into languages.
    ///
    /// Keys that name the same language are merged: `yml` and `yaml` become one
    /// YAML row rather than two rows a reader has to add up by hand.
    pub fn from_extensions(by_extension: &BTreeMap<String, ExtensionStats>) -> Self {
        let mut merged: BTreeMap<(Category, String), LanguageRow> = BTreeMap::new();

        for (key, stats) in by_extension {
            let (name, category) = describe(key);
            merged
                .entry((category, name.clone()))
                .or_insert_with(|| LanguageRow::empty(name, category))
                .absorb(stats);
        }

        let mut rows: Vec<LanguageRow> = merged.into_values().collect();
        // Largest first, because that is the question being asked, and by the
        // same measure [`LanguageRow::share_of`] reports -- an order that
        // disagreed with the percentages beside it would read as a bug. Ties
        // break on name so that two identical runs render identically.
        rows.sort_by(|a, b| {
            b.total_lines
                .cmp(&a.total_lines)
                .then(b.code_lines.cmp(&a.code_lines))
                .then(a.language.cmp(&b.language))
        });

        Self { rows }
    }

    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    /// Rows in `category`, largest first.
    pub fn in_category(&self, category: Category) -> impl Iterator<Item = &LanguageRow> {
        self.rows.iter().filter(move |row| row.category == category)
    }

    /// Total lines in `category`.
    pub fn lines_in(&self, category: Category) -> usize {
        self.in_category(category).map(|row| row.total_lines).sum()
    }

    /// Total files in `category`.
    pub fn files_in(&self, category: Category) -> usize {
        self.in_category(category).map(|row| row.file_count).sum()
    }

    pub fn total_lines(&self) -> usize {
        self.rows.iter().map(|row| row.total_lines).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::counter::comment_patterns;

    #[test]
    fn table_has_no_conflicting_duplicates() {
        let mut seen: HashMap<&str, Language> = HashMap::new();
        for (key, language) in LANGUAGE_TABLE {
            if let Some(existing) = seen.insert(key, *language) {
                assert_eq!(
                    existing, *language,
                    "{key:?} is listed twice with different languages"
                );
            }
        }
    }

    /// Any format the counter can classify must also be nameable, or the report
    /// will show a bare extension next to real language names.
    #[test]
    fn every_countable_format_has_a_language() {
        let missing: Vec<&str> = comment_patterns::known_extensions()
            .filter(|key| lookup(&key.to_lowercase()).is_none())
            .collect();
        assert!(
            missing.is_empty(),
            "these formats have comment syntax but no language name: {missing:?}"
        );
    }

    #[test]
    fn keys_are_lowercase_so_lookups_can_find_them() {
        for (key, _) in LANGUAGE_TABLE {
            assert!(
                !key.chars().any(|c| c.is_ascii_uppercase()),
                "{key:?} would never be found: lookups use a lowercase key"
            );
        }
    }

    /// The spellings that used to be reported as separate languages.
    #[test]
    fn equivalent_spellings_collapse_to_one_language() {
        for (a, b) in [
            ("yml", "yaml"),
            ("mk", "makefile"),
            ("mjs", "js"),
            ("cts", "ts"),
            ("markdown", "md"),
            ("containerfile", "dockerfile"),
        ] {
            assert_eq!(
                lookup(a).unwrap().name,
                lookup(b).unwrap().name,
                "{a:?} and {b:?} should report as one language"
            );
        }
    }

    /// Extension-less build files are named, not lumped into one anonymous
    /// bucket -- the counter already knows what they are.
    #[test]
    fn extension_less_project_files_are_named() {
        for (path, expected) in [
            ("Dockerfile", "Dockerfile"),
            ("Makefile", "Makefile"),
            ("justfile", "Justfile"),
            ("services/api/Dockerfile", "Dockerfile"),
        ] {
            assert_eq!(
                of_path(Path::new(path)).map(|l| l.name),
                Some(expected),
                "{path:?} was not recognized"
            );
        }
    }

    #[test]
    fn categories_separate_source_from_prose_and_payload() {
        for (key, expected) in [
            ("rs", Category::Code),
            ("sql", Category::Code),
            ("dockerfile", Category::Code),
            ("toml", Category::Config),
            ("yml", Category::Config),
            ("json", Category::Data),
            ("csv", Category::Data),
            ("md", Category::Docs),
            ("txt", Category::Docs),
        ] {
            assert_eq!(lookup(key).unwrap().category, expected, "{key:?}");
        }
    }

    /// An unrecognized extension is still counted, but it is not dressed up as
    /// a language: `net.firewall` must not sit in the report beside Go and Rust
    /// as though somebody wrote the project in Firewall.
    #[test]
    fn unknown_extensions_are_named_after_themselves_and_are_not_code() {
        let (name, category) = describe("firewall");
        assert_eq!(name, ".firewall");
        assert_eq!(category, Category::Data);
    }

    #[test]
    fn describe_prefers_the_registered_language_name() {
        assert_eq!(describe("rs"), ("Rust".to_string(), Category::Code));
        assert_eq!(describe("yml"), ("YAML".to_string(), Category::Config));
    }

    fn ext(file_count: usize, total_lines: usize, code_lines: usize) -> ExtensionStats {
        ExtensionStats {
            file_count,
            total_lines,
            code_lines,
            comment_lines: 0,
            doc_lines: total_lines - code_lines,
            blank_lines: 0,
            total_size: total_lines as u64 * 32,
            average_lines_per_file: total_lines as f64 / file_count.max(1) as f64,
            average_size_per_file: 0.0,
        }
    }

    fn breakdown_of(entries: &[(&str, ExtensionStats)]) -> Breakdown {
        Breakdown::from_extensions(
            &entries
                .iter()
                .map(|(k, v)| ((*k).to_string(), v.clone()))
                .collect(),
        )
    }

    #[test]
    fn spellings_of_one_language_merge_into_a_single_row() {
        let breakdown = breakdown_of(&[
            ("yml", ext(3, 300, 250)),
            ("yaml", ext(2, 200, 150)),
            ("ts", ext(1, 100, 90)),
        ]);

        let yaml: Vec<&LanguageRow> = breakdown
            .rows
            .iter()
            .filter(|r| r.language == "YAML")
            .collect();
        assert_eq!(
            yaml.len(),
            1,
            "YAML was reported twice: {:?}",
            breakdown.rows
        );
        assert_eq!(yaml[0].file_count, 5);
        assert_eq!(yaml[0].total_lines, 500);
        assert_eq!(yaml[0].code_lines, 400);
    }

    /// The failure that started this: a report ordered by ascending file count
    /// opened with one-off extensions and buried the languages the project is
    /// written in.
    #[test]
    fn rows_lead_with_the_largest_language() {
        let breakdown = breakdown_of(&[
            ("firewall", ext(1, 1_071, 1_055)),
            ("go", ext(2_898, 587_158, 393_163)),
            ("py", ext(5_790, 1_018_247, 674_002)),
        ]);

        let code: Vec<&str> = breakdown
            .in_category(Category::Code)
            .map(|r| r.language.as_str())
            .collect();
        assert_eq!(code, ["Python", "Go"]);
    }

    /// Rows are ordered by the same measure the percentages report, so a reader
    /// never sees a larger share printed below a smaller one.
    #[test]
    fn row_order_agrees_with_the_percentages_beside_it() {
        let breakdown = breakdown_of(&[
            ("rs", ext(517, 148_412, 95_022)),
            ("pyi", ext(349, 144_391, 97_808)),
            ("go", ext(2_928, 592_325, 397_052)),
        ]);

        let shares: Vec<f64> = breakdown
            .in_category(Category::Code)
            .map(|r| r.share_of(breakdown.lines_in(Category::Code)))
            .collect();
        assert!(
            shares.windows(2).all(|pair| pair[0] >= pair[1]),
            "percentages are not monotonically decreasing: {shares:?}"
        );
    }

    /// Prose and payload are counted, but they are not code and must not
    /// dominate the answer to "what is this project written in".
    #[test]
    fn documentation_and_data_are_kept_out_of_the_code_category() {
        let breakdown = breakdown_of(&[
            ("md", ext(3_678, 273_627, 31_922)),
            ("json", ext(500, 323_952, 250_052)),
            ("toml", ext(322, 76_312, 55_058)),
            ("rs", ext(511, 146_336, 93_455)),
        ]);

        assert_eq!(breakdown.lines_in(Category::Code), 146_336);
        assert_eq!(breakdown.lines_in(Category::Docs), 273_627);
        assert_eq!(breakdown.lines_in(Category::Data), 323_952);
        assert_eq!(breakdown.lines_in(Category::Config), 76_312);
        assert_eq!(
            breakdown.total_lines(),
            146_336 + 273_627 + 323_952 + 76_312,
            "every counted line must land in exactly one category"
        );
    }

    #[test]
    fn every_row_belongs_to_exactly_one_category() {
        let breakdown = breakdown_of(&[
            ("rs", ext(1, 10, 10)),
            ("md", ext(1, 10, 0)),
            ("json", ext(1, 10, 10)),
            ("toml", ext(1, 10, 10)),
        ]);

        let counted: usize = Category::ALL
            .iter()
            .map(|c| breakdown.in_category(*c).count())
            .sum();
        assert_eq!(counted, breakdown.rows.len());
    }

    #[test]
    fn shares_are_relative_to_the_whole_they_are_given() {
        let breakdown = breakdown_of(&[("rs", ext(1, 750, 700)), ("go", ext(1, 250, 200))]);
        let code_lines = breakdown.lines_in(Category::Code);
        let shares: Vec<f64> = breakdown
            .in_category(Category::Code)
            .map(|r| r.share_of(code_lines))
            .collect();
        assert_eq!(shares, [75.0, 25.0]);
    }

    #[test]
    fn an_empty_project_produces_an_empty_breakdown() {
        let breakdown = Breakdown::from_extensions(&BTreeMap::new());
        assert!(breakdown.is_empty());
        assert_eq!(breakdown.total_lines(), 0);
        assert_eq!(breakdown.lines_in(Category::Code), 0);
    }

    #[test]
    fn unrecognized_extension_less_files_get_one_shared_row() {
        let breakdown = breakdown_of(&[(NO_EXTENSION, ext(190, 1_091, 682))]);
        assert_eq!(breakdown.rows.len(), 1);
        assert_eq!(breakdown.rows[0].language, "Other (no extension)");
        assert_eq!(breakdown.rows[0].category, Category::Data);
    }
}
