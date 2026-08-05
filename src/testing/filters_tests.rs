//! What gets discovered, and why.
//!
//! Every question here is about the *set* of files an analysis considers, which
//! is the decision that most often surprises a user: a file they expected to be
//! counted is missing, or a vendored dependency inflated their numbers. The
//! suite asserts the rules directly rather than through totals, so a failure
//! names the rule that changed.

use crate::core::engine::{AnalysisOptions, DetectionMode, Engine, Parallelism};
use crate::core::filters::FileFilter;
use crate::testing::test_utils::TestProject;
use std::collections::BTreeSet;
use std::path::Path;

/// The paths an analysis would count, relative to `root` and slash-separated.
fn discovered(root: &Path, options: &AnalysisOptions) -> BTreeSet<String> {
    Engine::new()
        .discover_files(root, options)
        .unwrap()
        .into_iter()
        .map(|path| {
            path.strip_prefix(root)
                .unwrap_or(&path)
                .to_string_lossy()
                .replace('\\', "/")
        })
        .collect()
}

/// Discovery only; detection and cache are off so the answer depends on nothing
/// but the tree.
fn options() -> AnalysisOptions {
    AnalysisOptions {
        detection: DetectionMode::Disabled,
        use_cache: false,
        parallelism: Parallelism::Fixed(1),
        ..AnalysisOptions::default()
    }
}

#[test]
fn sources_are_found_and_noise_is_not() {
    let project = TestProject::new("filters_basic").unwrap();
    project
        .create_file("src/main.rs", "fn main() {}\n")
        .unwrap();
    project.create_file("script.py", "print(1)\n").unwrap();
    project.create_file("app.js", "console.log(1)\n").unwrap();
    project
        .create_file("style.css", "a { color: red }\n")
        .unwrap();
    project.create_file("notes.txt", "hello\n").unwrap();
    project
        .create_file_binary("logo.png", &[0x89, b'P'])
        .unwrap();
    project.create_file("app.exe", "MZ").unwrap();

    let found = discovered(project.path(), &options());

    for expected in ["src/main.rs", "script.py", "app.js", "style.css"] {
        assert!(
            found.contains(expected),
            "{expected} was not discovered: {found:?}"
        );
    }
    for rejected in ["logo.png", "app.exe"] {
        assert!(
            !found.contains(rejected),
            "{rejected} should not be counted"
        );
    }
}

/// Ambiguously-named build directories are excluded only when the toolchain
/// that produces them is present, so the project has to declare one -- see
/// [`crate::core::patterns`].
#[test]
fn build_output_and_dependency_caches_are_excluded() {
    let project = TestProject::new("filters_build").unwrap();
    project
        .create_file("src/lib.rs", "pub fn f() {}\n")
        .unwrap();
    for manifest in ["Cargo.toml", "package.json", "go.mod", "CMakeLists.txt"] {
        project.create_file(manifest, "\n").unwrap();
    }
    for noise in [
        "node_modules/express/index.js",
        "target/debug/build.rs",
        "build/generated.js",
        "dist/bundle.js",
        "out/app.js",
        "__pycache__/mod.py",
        ".venv/lib/site.py",
        "vendor/dep/dep.go",
        ".next/server/page.js",
        "coverage/report.js",
    ] {
        project.create_file(noise, "// noise\n").unwrap();
    }

    let found = discovered(project.path(), &options());
    assert!(found.contains("src/lib.rs"), "the source was dropped");
    let leaked: Vec<_> = found
        .iter()
        .filter(|name| name.contains('/') && !name.starts_with("src/"))
        .collect();
    assert!(leaked.is_empty(), "build output leaked: {leaked:?}");
}

/// The other half of the same rule: a directory that merely *shares a name*
/// with build output, in a project with no such toolchain, holds source.
#[test]
fn directories_named_like_build_output_are_kept_without_a_toolchain() {
    let project = TestProject::new("filters_ambiguous").unwrap();
    for source in [
        "packages/ui/src/button.ts",
        "build/pipeline.go",
        "vendor/graphify/main.go",
        "internal/log/logger.go",
        "cmd/out/render.go",
    ] {
        project.create_file(source, "// source\n").unwrap();
    }

    let found = discovered(project.path(), &options());
    assert_eq!(
        found.len(),
        5,
        "hand-written source was discarded because a directory shares a name \
         with some toolchain's output: {found:?}"
    );
}

#[test]
fn gitignore_is_respected_without_a_git_directory() {
    let project = TestProject::new("filters_gitignore").unwrap();
    project
        .create_file(".gitignore", "secret.rs\nlogs/\n")
        .unwrap();
    project.create_file("keep.rs", "fn keep() {}\n").unwrap();
    project
        .create_file("secret.rs", "fn secret() {}\n")
        .unwrap();
    project
        .create_file("logs/debug.rs", "fn log() {}\n")
        .unwrap();

    let found = discovered(project.path(), &options());
    assert!(found.contains("keep.rs"));
    assert!(
        !found.contains("secret.rs"),
        "a .gitignore without a .git directory was ignored, so an exported or \
         vendored tree counts files its own rules exclude"
    );
    assert!(!found.contains("logs/debug.rs"));
}

#[test]
fn nested_gitignore_files_apply_to_their_own_subtree() {
    let project = TestProject::new("filters_nested_gitignore").unwrap();
    project.create_file("a/keep.rs", "fn a() {}\n").unwrap();
    project.create_file("a/.gitignore", "skip.rs\n").unwrap();
    project.create_file("a/skip.rs", "fn skip() {}\n").unwrap();
    project.create_file("b/skip.rs", "fn other() {}\n").unwrap();

    let found = discovered(project.path(), &options());
    assert!(found.contains("a/keep.rs"));
    assert!(!found.contains("a/skip.rs"));
    assert!(
        found.contains("b/skip.rs"),
        "a nested ignore rule escaped its own directory"
    );
}

#[test]
fn hidden_files_are_opt_in() {
    let project = TestProject::new("filters_hidden").unwrap();
    project.create_file("visible.rs", "fn v() {}\n").unwrap();
    project.create_file(".hidden.rs", "fn h() {}\n").unwrap();
    project
        .create_file(".config/settings.toml", "k = 1\n")
        .unwrap();

    let default = discovered(project.path(), &options());
    assert!(default.contains("visible.rs"));
    assert!(!default.contains(".hidden.rs"));

    let with_hidden = discovered(
        project.path(),
        &AnalysisOptions {
            include_hidden: true,
            ..options()
        },
    );
    assert!(with_hidden.contains(".hidden.rs"));
    assert!(with_hidden.contains(".config/settings.toml"));
}

#[test]
fn custom_ignore_patterns_exclude_directories_and_their_contents() {
    let project = TestProject::new("filters_custom").unwrap();
    project.create_file("keep.rs", "fn k() {}\n").unwrap();
    project
        .create_file("generated/a.rs", "fn a() {}\n")
        .unwrap();
    project
        .create_file("generated/deep/b.rs", "fn b() {}\n")
        .unwrap();
    project.create_file("scratch.rs", "fn s() {}\n").unwrap();

    let found = discovered(
        project.path(),
        &AnalysisOptions {
            ignore_patterns: vec!["generated".to_string(), "scratch.rs".to_string()],
            ..options()
        },
    );

    assert_eq!(found, BTreeSet::from(["keep.rs".to_string()]));
}

#[test]
fn an_invalid_ignore_pattern_is_an_error_not_an_empty_result() {
    let project = TestProject::new("filters_bad_pattern").unwrap();
    project.create_file("keep.rs", "fn k() {}\n").unwrap();

    let result = Engine::new().discover_files(
        project.path(),
        &AnalysisOptions {
            // An unclosed character class cannot be compiled into a glob.
            ignore_patterns: vec!["[".to_string()],
            ..options()
        },
    );

    assert!(
        result.is_err(),
        "a pattern that cannot be compiled must be reported, because silently \
         counting everything looks like success"
    );
}

#[test]
fn max_depth_counts_from_the_root() {
    let project = TestProject::new("filters_depth").unwrap();
    project.create_file("top.rs", "fn t() {}\n").unwrap();
    project.create_file("one/a.rs", "fn a() {}\n").unwrap();
    project.create_file("one/two/b.rs", "fn b() {}\n").unwrap();
    project
        .create_file("one/two/three/c.rs", "fn c() {}\n")
        .unwrap();

    let depths = [(1, 1), (2, 2), (3, 3), (4, 4)];
    for (depth, expected) in depths {
        let found = discovered(
            project.path(),
            &AnalysisOptions {
                max_depth: Some(depth),
                ..options()
            },
        );
        assert_eq!(found.len(), expected, "depth {depth} found {found:?}");
    }
}

#[test]
fn extension_filters_are_case_insensitive_and_exclusive() {
    let project = TestProject::new("filters_ext").unwrap();
    project.create_file("a.rs", "fn a() {}\n").unwrap();
    project.create_file("b.RS", "fn b() {}\n").unwrap();
    project.create_file("c.py", "pass\n").unwrap();

    let found = discovered(
        project.path(),
        &AnalysisOptions {
            extensions: vec!["rs".to_string()],
            ..options()
        },
    );
    assert_eq!(
        found,
        BTreeSet::from(["a.rs".to_string(), "b.RS".to_string()])
    );
}

#[test]
fn generated_files_are_excluded_whatever_their_extension() {
    let project = TestProject::new("filters_generated").unwrap();
    project
        .create_file("src/main.rs", "fn main() {}\n")
        .unwrap();
    for generated in [
        "src/api.pb.go",
        "src/api.generated.rs",
        "src/schema_generated.go",
        "web/app.min.js",
        "web/app.bundle.js",
        "package-lock.json",
        "go.sum",
    ] {
        project.create_file(generated, "// generated\n").unwrap();
    }

    let found = discovered(project.path(), &options());
    assert_eq!(found, BTreeSet::from(["src/main.rs".to_string()]));
}

/// Symlinks are not followed, so a link pointing back up the tree cannot make
/// the walk unbounded and a link to a counted file cannot count it twice.
#[cfg(unix)]
#[test]
fn symlinks_are_not_followed() {
    use std::os::unix::fs::symlink;

    let project = TestProject::new("filters_symlink").unwrap();
    project
        .create_file("src/main.rs", "fn main() {}\n")
        .unwrap();
    symlink(project.path(), project.path().join("src/loop")).unwrap();
    symlink(
        project.path().join("src/main.rs"),
        project.path().join("alias.rs"),
    )
    .unwrap();

    let found = discovered(project.path(), &options());
    assert_eq!(
        found,
        BTreeSet::from(["src/main.rs".to_string()]),
        "following symlinks would count files twice or never terminate"
    );
}

#[test]
fn a_project_inside_a_build_named_directory_is_still_analyzed() {
    // The bug this pins down made the tool report zero files for any checkout
    // living under a path segment that looked like build output.
    for hostile in [
        "build", "dist", "target", "tmp", "env", "vendor", "bin", "out",
    ] {
        let outer = tempfile::tempdir().unwrap();
        let root = outer.path().join(hostile).join("project");
        std::fs::create_dir_all(root.join("src")).unwrap();
        std::fs::write(root.join("src/main.rs"), "fn main() {}\n").unwrap();

        let found = discovered(&root, &options());
        assert_eq!(
            found,
            BTreeSet::from(["src/main.rs".to_string()]),
            "a checkout under a directory named {hostile:?} was discarded"
        );
    }
}

/// A directory that *is* the analysis root is never pruned, even when its own
/// name matches a build pattern: analyzing `./target` is a legitimate request.
#[test]
fn the_root_itself_is_never_pruned() {
    let outer = tempfile::tempdir().unwrap();
    let root = outer.path().join("node_modules");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("index.js"), "module.exports = 1;\n").unwrap();

    let found = discovered(&root, &options());
    assert_eq!(found, BTreeSet::from(["index.js".to_string()]));
}

#[test]
fn unusual_but_legal_filenames_are_handled() {
    let project = TestProject::new("filters_names").unwrap();
    let names = [
        "with space.rs",
        "with-dash.rs",
        "with.dots.rs",
        "ünïcödé.rs",
        "日本語.rs",
        "UPPER.RS",
        "no_vowels.rs",
    ];
    for name in names {
        project.create_file(name, "fn f() {}\n").unwrap();
    }

    let found = discovered(project.path(), &options());
    for name in names {
        assert!(found.contains(name), "{name} was dropped: {found:?}");
    }
}

#[test]
fn discovery_is_identical_however_the_walk_is_parallelised() {
    let project = TestProject::new("filters_parallel").unwrap();
    for i in 0..40 {
        project
            .create_file(&format!("src/pkg{}/mod{i}.rs", i % 5), "fn f() {}\n")
            .unwrap();
    }
    project.create_file("node_modules/dep/i.js", "1\n").unwrap();

    let sequential = discovered(project.path(), &options());
    for threads in [2, 4, 8] {
        let parallel = discovered(
            project.path(),
            &AnalysisOptions {
                parallelism: Parallelism::Fixed(threads),
                ..options()
            },
        );
        assert_eq!(
            sequential, parallel,
            "discovery differed with {threads} threads"
        );
    }
}

#[test]
fn discovered_paths_are_sorted() {
    let project = TestProject::new("filters_sorted").unwrap();
    for name in ["z.rs", "a.rs", "m/n.rs", "b/c.rs"] {
        project.create_file(name, "fn f() {}\n").unwrap();
    }

    let paths = Engine::new()
        .discover_files(project.path(), &options())
        .unwrap();
    let mut sorted = paths.clone();
    sorted.sort();
    assert_eq!(paths, sorted, "listing order is not stable across runs");
}

#[test]
fn missing_and_unreadable_roots_do_not_panic() {
    let engine = Engine::new();
    assert!(engine
        .discover_files(Path::new("/definitely/not/here"), &options())
        .unwrap()
        .is_empty());

    // A file, rather than a directory, as the root.
    let project = TestProject::new("filters_file_root").unwrap();
    let file = project.create_file("only.rs", "fn f() {}\n").unwrap();
    let found = engine.discover_files(&file, &options()).unwrap();
    assert_eq!(found.len(), 1, "a single file root should count that file");
}

#[test]
fn build_pruning_does_not_change_which_files_are_reported() {
    let project = TestProject::new("filters_prune_equivalence").unwrap();
    project
        .create_file("src/main.rs", "fn main() {}\n")
        .unwrap();
    project
        .create_file("node_modules/dep/index.js", "module.exports = 1;\n")
        .unwrap();
    project
        .create_file("target/debug/gen.rs", "fn g() {}\n")
        .unwrap();

    let root = project.path();
    let names = |filter: &FileFilter| -> BTreeSet<String> {
        filter
            .try_walk_directory(root)
            .unwrap()
            .filter(|entry| entry.file_type().is_some_and(|ft| ft.is_file()))
            .map(|entry| entry.path().to_path_buf())
            .filter(|path| filter.should_include_file(path, Some(root)))
            .map(|path| {
                path.strip_prefix(root)
                    .unwrap_or(&path)
                    .to_string_lossy()
                    .replace('\\', "/")
            })
            .collect()
    };

    assert_eq!(
        names(&FileFilter::new().prune_build_dirs(true)),
        names(&FileFilter::new().prune_build_dirs(false)),
        "pruning is an optimisation and must not change the answer"
    );
}

#[test]
fn filter_builder_settings_are_recorded() {
    let filter = FileFilter::new()
        .with_max_depth(5)
        .with_custom_ignores(vec!["*.tmp".to_string(), "  ".to_string()]);

    assert_eq!(filter.max_depth(), Some(5));
    assert_eq!(
        filter.custom_ignores(),
        ["*.tmp".to_string()],
        "blank ignore patterns would exclude everything, so they are dropped"
    );
}
