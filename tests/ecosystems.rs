//! Build directories require ecosystem evidence; ordinary source names survive.

use howmany::{AnalysisOptions, Engine, FileFilter, PatternMatcher};
use std::collections::BTreeSet;
use std::path::Path;
use std::process::Command;

fn write(root: &Path, path: &str, text: &str) {
    let full = root.join(path);
    std::fs::create_dir_all(full.parent().unwrap()).unwrap();
    std::fs::write(full, text).unwrap();
}

fn git(root: &Path, args: &[&str]) {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn selected(root: &Path, tracked_only: bool) -> BTreeSet<String> {
    let options = AnalysisOptions {
        include_hidden: true,
        source_only: true,
        tracked_only,
        compute_complexity: false,
        ..AnalysisOptions::reproducible()
    };
    Engine::new()
        .discover_files(root, &options)
        .unwrap()
        .into_iter()
        .map(|path| {
            path.strip_prefix(root)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/")
        })
        .collect()
}

#[test]
fn lean_caches_and_compiled_outputs_stay_excluded_when_hidden_or_tracked() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    git(root, &["init", "-q"]);
    write(root, "src/main.rs", "fn main() {}\n");
    for path in [
        ".lake/packages/dependency/foreign.c",
        ".lake/build/ir/Main.c",
        ".lake/build/ir/Main.ir",
        "nested/.lake/build/ir/Proof.c",
        "lake-packages/dependency/foreign.c",
        "nested/lake-packages/dependency/foreign.c",
        "artifacts/Main.olean",
        "artifacts/Main.olean.server",
        "artifacts/Main.olean.private",
        "artifacts/Main.ilean",
    ] {
        write(root, path, "int machine_output;\n");
    }
    write(root, "lake-manifest.json", "{\"packages\": []}\n");
    write(
        root,
        "src/handwritten.ir",
        "authored intermediate language\n",
    );
    git(root, &["add", "-f", "."]);
    for tracked_only in [false, true] {
        assert_eq!(
            selected(root, tracked_only),
            BTreeSet::from(["src/main.rs".to_owned()])
        );
    }
    let filter = FileFilter::new().respect_hidden(false);
    for path in [
        ".lake/build/ir/Main.c",
        "lake-packages/dependency/foreign.c",
        "artifacts/Main.olean.server",
        "lake-manifest.json",
    ] {
        assert!(!filter.should_include_file(&root.join(path), Some(root)));
    }
    assert!(filter.should_include_file(&root.join("src/main.rs"), Some(root)));
    assert!(filter.should_include_file(&root.join("src/handwritten.ir"), Some(root)));
    assert!(!PatternMatcher::new().is_build_output(&root.join("src/handwritten.ir"), Some(root)));
}

#[test]
fn cmake_fetch_content_needs_a_configured_build_or_subbuild() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    git(root, &["init", "-q"]);
    write(root, "CMakeLists.txt", "project(example)\n");
    for marker in [
        "configured/CMakeCache.txt",
        "configured-by-files/CMakeFiles/cmake.check_cache",
        "fetch/_deps/library-subbuild/CMakeCache.txt",
    ] {
        write(root, marker, "# CMake output\n");
    }
    for path in [
        "configured/_deps/library-src/foreign.cc",
        "configured-by-files/_deps/library-src/foreign.cc",
        "fetch/_deps/library-src/foreign.cc",
        "fetch/_deps/library-subbuild/glue.cc",
    ] {
        write(root, path, "int foreign;\n");
    }
    let authored = [
        "src/_deps/fixture.cc",
        "_deps/authored.cc",
        "unconfigured/_deps/library-src/source.cc",
        "unconfigured/_deps/library-subbuild/source.cc",
        "engine/chromium/services/overlay.cc",
        "third_party/tests/copied.cc",
        "vendor/feature/service.cc",
        "tools/oracles/corpus/copied-test.cc",
    ];
    for path in authored {
        write(root, path, "int source;\n");
    }
    git(root, &["add", "."]);
    let expected: BTreeSet<_> = authored.into_iter().map(str::to_owned).collect();
    for tracked_only in [false, true] {
        assert_eq!(selected(root, tracked_only), expected);
    }
}

#[test]
fn gn_generated_trees_need_build_markers_without_hiding_authored_neighbors() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    git(root, &["init", "-q"]);
    for marker in [
        "gn/.gn",
        "gn/BUILD.gn",
        "gn/out/Debug/build.ninja",
        "gn/out/Debug/args.gn",
        "direct/out/build.ninja",
        "direct/out/toolchain.ninja",
        "custom-build/build.ninja",
        "custom-build/toolchain.ninja",
        "ninja-source/build.ninja",
        "source/BUILD.gn",
    ] {
        write(root, marker, "# build configuration\n");
    }
    for path in [
        "gn/out/Debug/gen/bindings.cc",
        "gn/out/Debug/obj/generated.cc",
        "direct/out/gen/bindings.cc",
        "custom-build/gen/bindings.cc",
    ] {
        write(root, path, "int generated;\n");
    }
    let authored = [
        "gn/out/source.cc",
        "gn/out/Scratch/source.cc",
        "gn/gen/source.cc",
        "source/gen/source.cc",
        "source/out/source.cc",
        "ninja-source/gen/source.cc",
        "custom-build/authored.cc",
        "engine/chromium/services/overlay.cc",
    ];
    for path in authored {
        write(root, path, "int source;\n");
    }
    git(root, &["add", "."]);
    let expected: BTreeSet<_> = authored.into_iter().map(str::to_owned).collect();
    for tracked_only in [false, true] {
        assert_eq!(selected(root, tracked_only), expected);
    }
    let matcher = PatternMatcher::new();
    assert!(matcher.is_prunable_dir_at(&root.join("gn/out/Debug"), "Debug"));
    assert!(!matcher.is_prunable_dir_at(&root.join("gn/out"), "out"));
    assert!(!matcher.is_prunable_dir_at(&root.join("gn/gen"), "gen"));
}
