use crate::core::patterns::PatternMatcher;
use crate::utils::errors::{HowManyError, Result};
use ignore::overrides::{Override, OverrideBuilder};
use ignore::{DirEntry, WalkBuilder, WalkParallel};
use std::path::{Path, PathBuf};

/// Directory traversal with gitignore awareness and build-output pruning.
#[derive(Debug, Clone)]
pub struct FileFilter {
    respect_gitignore: bool,
    respect_hidden: bool,
    max_depth: Option<usize>,
    custom_ignores: Vec<String>,
    prune_build_dirs: bool,
    pattern_matcher: PatternMatcher,
}

impl Default for FileFilter {
    fn default() -> Self {
        Self::new()
    }
}

impl FileFilter {
    pub fn new() -> Self {
        Self {
            respect_gitignore: true,
            respect_hidden: true,
            max_depth: None,
            custom_ignores: Vec::new(),
            prune_build_dirs: true,
            pattern_matcher: PatternMatcher::new(),
        }
    }

    pub fn with_max_depth(mut self, depth: usize) -> Self {
        self.max_depth = Some(depth);
        self
    }

    pub fn with_custom_ignores(mut self, ignores: Vec<String>) -> Self {
        self.custom_ignores
            .extend(ignores.into_iter().filter(|p| !p.trim().is_empty()));
        self
    }

    pub fn respect_gitignore(mut self, respect: bool) -> Self {
        self.respect_gitignore = respect;
        self
    }

    pub fn respect_hidden(mut self, respect: bool) -> Self {
        self.respect_hidden = respect;
        self
    }

    /// Skip descending into known build output and dependency caches.
    ///
    /// Enabled by default. Disabling it does not change *which* files are
    /// reported -- the same paths are excluded by pattern afterwards -- only
    /// how much of the tree is read to find that out.
    pub fn prune_build_dirs(mut self, prune: bool) -> Self {
        self.prune_build_dirs = prune;
        self
    }

    /// Translate `--ignore` patterns into gitignore-style exclusion globs.
    ///
    /// Each pattern is registered both bare and as a directory prefix so that
    /// `--ignore node_modules` excludes the directory *and* everything beneath
    /// it, matching what users expect from `.gitignore`.
    fn build_overrides(&self, root: &Path) -> Result<Option<Override>> {
        if self.custom_ignores.is_empty() {
            return Ok(None);
        }

        let mut builder = OverrideBuilder::new(root);
        for pattern in &self.custom_ignores {
            let pattern = pattern.trim().trim_end_matches('/');
            for glob in [format!("!{pattern}"), format!("!{pattern}/**")] {
                builder.add(&glob).map_err(|e| {
                    HowManyError::filter(format!("invalid --ignore pattern {pattern:?}: {e}"))
                })?;
            }
        }

        builder
            .build()
            .map(Some)
            .map_err(|e| HowManyError::filter(format!("could not build ignore rules: {e}")))
    }

    fn configure(&self, root: &Path) -> Result<WalkBuilder> {
        let mut builder = WalkBuilder::new(root);

        builder
            .git_ignore(self.respect_gitignore)
            .git_global(self.respect_gitignore)
            .git_exclude(self.respect_gitignore)
            .hidden(self.respect_hidden)
            .parents(self.respect_gitignore)
            .ignore(true)
            // A source tree extracted from a tarball or vendored into another
            // repository has a `.gitignore` but no `.git`; its rules still
            // describe what is generated, so honour them.
            .require_git(false)
            // Symlinks are not followed: a link back up the tree would make the
            // walk unbounded, and a link to a file already in the tree would
            // count it twice.
            .follow_links(false);

        if let Some(depth) = self.max_depth {
            builder.max_depth(Some(depth));
        }

        if let Some(overrides) = self.build_overrides(root)? {
            builder.overrides(overrides);
        }

        if self.prune_build_dirs {
            let matcher = self.pattern_matcher;
            builder.filter_entry(move |entry| !is_prunable_dir(&matcher, entry));
        }

        Ok(builder)
    }

    /// Walk `path`, yielding entries that survive gitignore rules and pruning.
    ///
    /// Errors while configuring the walk (an invalid `--ignore` pattern, say)
    /// surface here rather than silently producing an empty listing.
    pub fn try_walk_directory(&self, path: &Path) -> Result<impl Iterator<Item = DirEntry>> {
        Ok(self.configure(path)?.build().filter_map(|entry| entry.ok()))
    }

    /// A work-stealing parallel walker over `path`.
    ///
    /// Directory reads dominate traversal on large trees; spreading them across
    /// threads keeps the counting stage fed.
    pub fn walk_parallel(&self, path: &Path, threads: usize) -> Result<WalkParallel> {
        let mut builder = self.configure(path)?;
        builder.threads(threads.max(1));
        Ok(builder.build_parallel())
    }

    /// True when `path` should be counted, judged relative to `root`.
    ///
    /// `root` must be the directory the analysis was started from; patterns
    /// like `build/` describe locations inside a project and must never be
    /// matched against the absolute prefix that leads to it.
    pub fn should_include_file(&self, path: &Path, root: Option<&Path>) -> bool {
        let relative = self.pattern_matcher.relative_path(path, root);

        if self.pattern_matcher.should_ignore_file(&relative)
            || self.pattern_matcher.is_build_output(path, root)
        {
            return false;
        }

        if let Some(extension) = path.extension() {
            if self
                .pattern_matcher
                .is_binary_file(&extension.to_string_lossy())
            {
                return false;
            }
        }

        if let Some(filename) = path.file_name() {
            if self
                .pattern_matcher
                .is_generated_file(&filename.to_string_lossy())
            {
                return false;
            }
        }

        true
    }

    pub fn pattern_matcher(&self) -> &PatternMatcher {
        &self.pattern_matcher
    }

    pub fn max_depth(&self) -> Option<usize> {
        self.max_depth
    }

    pub fn custom_ignores(&self) -> &[String] {
        &self.custom_ignores
    }
}

/// True when `entry` is a directory that can be skipped wholesale.
///
/// The root of the walk is never pruned: analyzing a directory that happens to
/// be called `build` is a legitimate request.
fn is_prunable_dir(matcher: &PatternMatcher, entry: &DirEntry) -> bool {
    if entry.depth() == 0 {
        return false;
    }
    if !entry.file_type().is_some_and(|ft| ft.is_dir()) {
        return false;
    }
    entry
        .file_name()
        .to_str()
        .is_some_and(|name| matcher.is_prunable_dir_at(entry.path(), name))
}

/// A file discovered by traversal, with the metadata the walk already paid for.
///
/// Reusing `size`/`modified` downstream removes one `stat` per file from the
/// cache lookup and another from the counting stage.
#[derive(Debug, Clone)]
pub struct DiscoveredFile {
    pub path: PathBuf,
    pub size: u64,
    pub modified: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::test_utils::TestProject;
    use std::collections::HashSet;

    fn walked_names(filter: &FileFilter, root: &Path) -> HashSet<String> {
        filter
            .try_walk_directory(root)
            .unwrap()
            .filter(|e| e.file_type().is_some_and(|ft| ft.is_file()))
            .map(|e| {
                e.path()
                    .strip_prefix(root)
                    .unwrap_or(e.path())
                    .to_string_lossy()
                    .replace('\\', "/")
            })
            .collect()
    }

    #[test]
    fn prunes_build_directories_but_keeps_sources() {
        let project = TestProject::new("prune").unwrap();
        project.create_file("src/main.rs", "fn main() {}").unwrap();
        // `target/` is Cargo's only when Cargo is here to have made it.
        project.create_file("Cargo.toml", "[package]\n").unwrap();
        project
            .create_file("node_modules/dep/index.js", "module.exports = 1;")
            .unwrap();
        project
            .create_file("target/debug/gen.rs", "fn gen() {}")
            .unwrap();
        project.create_file("__pycache__/m.py", "x = 1").unwrap();

        let names = walked_names(&FileFilter::new(), project.path());
        assert!(names.contains("src/main.rs"));
        assert!(!names.iter().any(|n| n.starts_with("node_modules/")));
        assert!(!names.iter().any(|n| n.starts_with("target/")));
        assert!(!names.iter().any(|n| n.starts_with("__pycache__/")));
    }

    /// Pruning is an optimization, so the set of *countable* files it yields
    /// must equal what pattern filtering alone would yield.
    #[test]
    fn pruning_and_pattern_filtering_agree() {
        let project = TestProject::new("agree").unwrap();
        project.create_file("src/main.rs", "fn main() {}").unwrap();
        project.create_file("src/lib.rs", "pub fn a() {}").unwrap();
        project
            .create_file("node_modules/dep/index.js", "x")
            .unwrap();
        project.create_file("target/debug/gen.rs", "y").unwrap();
        project
            .create_file("vendor/pkg/lib.go", "package pkg")
            .unwrap();
        project.create_file("docs/guide.md", "# Guide").unwrap();

        let root = project.path();
        let pruned = FileFilter::new();
        let unpruned = FileFilter::new().prune_build_dirs(false);

        let pruned_set: HashSet<_> = walked_names(&pruned, root)
            .into_iter()
            .filter(|n| pruned.should_include_file(&root.join(n), Some(root)))
            .collect();
        let unpruned_set: HashSet<_> = walked_names(&unpruned, root)
            .into_iter()
            .filter(|n| unpruned.should_include_file(&root.join(n), Some(root)))
            .collect();

        assert_eq!(
            pruned_set, unpruned_set,
            "directory pruning changed the result set; it must only change the cost"
        );
    }

    /// A project whose own directory name is a build word must still be walked.
    #[test]
    fn root_named_like_a_build_dir_is_still_analyzed() {
        let project = TestProject::new("build").unwrap();
        project.create_file("src/main.rs", "fn main() {}").unwrap();

        let root = project.path();
        assert_eq!(root.file_name().unwrap(), "build");

        let names = walked_names(&FileFilter::new(), root);
        assert!(
            names.contains("src/main.rs"),
            "root directory was pruned by its own name"
        );
        assert!(
            FileFilter::new().should_include_file(&root.join("src/main.rs"), Some(root)),
            "file was excluded because its project root is called 'build'"
        );
    }

    #[test]
    fn custom_ignore_patterns_actually_exclude() {
        let project = TestProject::new("ignores").unwrap();
        project.create_file("src/main.rs", "fn main() {}").unwrap();
        project
            .create_file("generated/api.rs", "fn api() {}")
            .unwrap();
        project.create_file("src/big.snap.rs", "fn s() {}").unwrap();

        let filter = FileFilter::new()
            .with_custom_ignores(vec!["generated".to_string(), "*.snap.rs".to_string()]);
        let names = walked_names(&filter, project.path());

        assert!(names.contains("src/main.rs"));
        assert!(
            !names.iter().any(|n| n.starts_with("generated/")),
            "--ignore did not exclude the directory: {names:?}"
        );
        assert!(
            !names.contains("src/big.snap.rs"),
            "--ignore glob did not exclude the file: {names:?}"
        );
    }

    #[test]
    fn invalid_ignore_pattern_reports_an_error() {
        let project = TestProject::new("bad_ignore").unwrap();
        project.create_file("src/main.rs", "fn main() {}").unwrap();

        let filter = FileFilter::new().with_custom_ignores(vec!["[".to_string()]);
        assert!(
            filter.try_walk_directory(project.path()).is_err(),
            "a malformed --ignore pattern must not be silently accepted"
        );
    }

    #[test]
    fn max_depth_is_respected() {
        let project = TestProject::new("depth").unwrap();
        project.create_file("a.rs", "fn a() {}").unwrap();
        project.create_file("one/b.rs", "fn b() {}").unwrap();
        project.create_file("one/two/c.rs", "fn c() {}").unwrap();

        let names = walked_names(&FileFilter::new().with_max_depth(2), project.path());
        assert!(names.contains("a.rs"));
        assert!(names.contains("one/b.rs"));
        assert!(!names.contains("one/two/c.rs"));
    }

    #[test]
    fn gitignored_files_are_skipped_unless_disabled() {
        let project = TestProject::new("gitignore").unwrap();
        project.create_file(".gitignore", "secret.rs\n").unwrap();
        project.create_file("src/main.rs", "fn main() {}").unwrap();
        project.create_file("secret.rs", "fn s() {}").unwrap();

        let respected = walked_names(&FileFilter::new(), project.path());
        assert!(!respected.contains("secret.rs"));

        let ignored = walked_names(&FileFilter::new().respect_gitignore(false), project.path());
        assert!(ignored.contains("secret.rs"));
    }

    #[test]
    fn binary_and_generated_files_are_excluded() {
        let project = TestProject::new("excl").unwrap();
        let root = project.path();
        let filter = FileFilter::new();

        assert!(!filter.should_include_file(&root.join("assets/logo.png"), Some(root)));
        assert!(!filter.should_include_file(&root.join("src/api.generated.rs"), Some(root)));
        assert!(filter.should_include_file(&root.join("src/main.rs"), Some(root)));
    }
}
