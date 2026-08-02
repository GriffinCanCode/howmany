use crate::core::detector::FileDetector;
use crate::core::patterns::PatternMatcher;
use crate::testing::test_utils::TestProject;
use std::path::Path;

#[cfg(test)]
mod detection {
    use super::*;

    /// A detector built with no root judges paths whole, which is only correct
    /// for paths that are already project-relative. Everything below anchors to
    /// the project root, the way the engine does.
    #[test]
    fn detector_without_a_root_still_classifies_relative_paths() {
        let detector = FileDetector::new();
        assert!(detector.is_user_created_file(Path::new("src/main.rs")));
        assert!(!detector.is_user_created_file(Path::new("node_modules/x/index.js")));
    }

    #[test]
    fn test_user_created_rust_files() {
        let project = TestProject::new("test_project").unwrap();
        let detector = FileDetector::new().with_root(project.path());

        // Create user Rust files
        let main_rs = project.create_file("src/main.rs", "fn main() {}").unwrap();
        let lib_rs = project.create_file("src/lib.rs", "pub mod test;").unwrap();
        let mod_rs = project
            .create_file("src/utils/mod.rs", "// Utils module")
            .unwrap();

        assert!(detector.is_user_created_file(&main_rs));
        assert!(detector.is_user_created_file(&lib_rs));
        assert!(detector.is_user_created_file(&mod_rs));
    }

    #[test]
    fn test_ignore_target_directory() {
        let project = TestProject::new("test_project").unwrap();
        let detector = FileDetector::new().with_root(project.path());

        // Create files in target directory (should be ignored)
        project.create_directory("target/debug").unwrap();
        let target_file = project
            .create_file("target/debug/myapp", "binary content")
            .unwrap();
        let target_lib = project
            .create_file("target/debug/lib.rlib", "library")
            .unwrap();

        assert!(!detector.is_user_created_file(&target_file));
        assert!(!detector.is_user_created_file(&target_lib));
    }

    #[test]
    fn test_ignore_node_modules() {
        let project = TestProject::new("test_project").unwrap();
        let detector = FileDetector::new().with_root(project.path());

        // Create node_modules structure
        project.create_node_modules().unwrap();
        let node_file = project
            .create_file("node_modules/express/index.js", "module.exports = {};")
            .unwrap();
        let nested_file = project
            .create_file("node_modules/lodash/dist/lodash.min.js", "//minified")
            .unwrap();

        assert!(!detector.is_user_created_file(&node_file));
        assert!(!detector.is_user_created_file(&nested_file));
    }

    #[test]
    fn test_ignore_python_cache() {
        let project = TestProject::new("test_project").unwrap();
        let detector = FileDetector::new().with_root(project.path());

        // Create Python cache files
        project.create_directory("__pycache__").unwrap();
        let pyc_file = project
            .create_file("__pycache__/module.cpython-39.pyc", "bytecode")
            .unwrap();
        let cache_file = project
            .create_file("src/__pycache__/test.pyc", "cache")
            .unwrap();

        assert!(!detector.is_user_created_file(&pyc_file));
        assert!(!detector.is_user_created_file(&cache_file));
    }

    #[test]
    fn test_ignore_build_artifacts() {
        let project = TestProject::new("test_project").unwrap();
        let detector = FileDetector::new().with_root(project.path());

        // Create various build artifacts
        let exe_file = project.create_file("myapp.exe", "executable").unwrap();
        let dll_file = project
            .create_file("library.dll", "dynamic library")
            .unwrap();
        let so_file = project.create_file("library.so", "shared object").unwrap();
        let class_file = project.create_file("Main.class", "java bytecode").unwrap();
        let jar_file = project.create_file("app.jar", "java archive").unwrap();

        assert!(!detector.is_user_created_file(&exe_file));
        assert!(!detector.is_user_created_file(&dll_file));
        assert!(!detector.is_user_created_file(&so_file));
        assert!(!detector.is_user_created_file(&class_file));
        assert!(!detector.is_user_created_file(&jar_file));
    }

    #[test]
    fn test_ignore_editor_files() {
        let project = TestProject::new("test_project").unwrap();
        let detector = FileDetector::new().with_root(project.path());

        // Create editor temporary files
        let vim_swap = project.create_file(".test.rs.swp", "vim swap").unwrap();
        let emacs_backup = project.create_file("test.rs~", "emacs backup").unwrap();
        let ds_store = project.create_file(".DS_Store", "macos").unwrap();

        assert!(!detector.is_user_created_file(&vim_swap));
        assert!(!detector.is_user_created_file(&emacs_backup));
        assert!(!detector.is_user_created_file(&ds_store));
    }

    #[test]
    fn test_ignore_version_control() {
        let project = TestProject::new("test_project").unwrap();
        let detector = FileDetector::new().with_root(project.path());

        // Create version control files
        project.create_directory(".git").unwrap();
        let git_config = project.create_file(".git/config", "git config").unwrap();
        let git_head = project
            .create_file(".git/HEAD", "ref: refs/heads/main")
            .unwrap();
        let gitignore = project.create_file(".gitignore", "target/\n*.log").unwrap();

        assert!(!detector.is_user_created_file(&git_config));
        assert!(!detector.is_user_created_file(&git_head));
        // .gitignore should be detected as user-created
        assert!(detector.is_user_created_file(&gitignore));
    }

    #[test]
    fn test_detect_various_languages() {
        let project = TestProject::new("test_project").unwrap();
        let detector = FileDetector::new().with_root(project.path());

        // Test various programming languages
        let rust_file = project.create_file("main.rs", "fn main() {}").unwrap();
        let python_file = project.create_file("script.py", "def main():").unwrap();
        let js_file = project.create_file("app.js", "function main() {}").unwrap();
        let ts_file = project
            .create_file("app.ts", "function main(): void {}")
            .unwrap();
        let java_file = project
            .create_file("Main.java", "public class Main {}")
            .unwrap();
        let cpp_file = project.create_file("main.cpp", "int main() {}").unwrap();
        let c_file = project.create_file("main.c", "int main() {}").unwrap();
        let go_file = project.create_file("main.go", "func main() {}").unwrap();
        let ruby_file = project.create_file("script.rb", "def main").unwrap();
        let php_file = project
            .create_file("index.php", "<?php echo 'hi'; ?>")
            .unwrap();

        assert!(detector.is_user_created_file(&rust_file));
        assert!(detector.is_user_created_file(&python_file));
        assert!(detector.is_user_created_file(&js_file));
        assert!(detector.is_user_created_file(&ts_file));
        assert!(detector.is_user_created_file(&java_file));
        assert!(detector.is_user_created_file(&cpp_file));
        assert!(detector.is_user_created_file(&c_file));
        assert!(detector.is_user_created_file(&go_file));
        assert!(detector.is_user_created_file(&ruby_file));
        assert!(detector.is_user_created_file(&php_file));
    }

    #[test]
    fn test_detect_config_files() {
        let project = TestProject::new("test_project").unwrap();
        let detector = FileDetector::new().with_root(project.path());

        // Test various configuration files
        let cargo_toml = project
            .create_file("Cargo.toml", "[package]\nname = \"test\"")
            .unwrap();
        let package_json = project
            .create_file("package.json", "{\"name\": \"test\"}")
            .unwrap();
        let dockerfile = project.create_file("Dockerfile", "FROM alpine").unwrap();
        let makefile = project
            .create_file("Makefile", "all:\n\techo hello")
            .unwrap();
        let yaml_config = project.create_file("config.yaml", "key: value").unwrap();
        let json_config = project
            .create_file("config.json", "{\"debug\": true}")
            .unwrap();

        assert!(detector.is_user_created_file(&cargo_toml));
        assert!(detector.is_user_created_file(&package_json));
        assert!(detector.is_user_created_file(&dockerfile));
        assert!(detector.is_user_created_file(&makefile));
        assert!(detector.is_user_created_file(&yaml_config));
        assert!(detector.is_user_created_file(&json_config));
    }

    #[test]
    fn test_detect_web_files() {
        let project = TestProject::new("test_project").unwrap();
        let detector = FileDetector::new().with_root(project.path());

        // Test web-related files
        let html_file = project.create_file("index.html", "<html></html>").unwrap();
        let css_file = project
            .create_file("styles.css", "body { margin: 0; }")
            .unwrap();
        let scss_file = project.create_file("styles.scss", "$color: blue;").unwrap();
        let vue_file = project
            .create_file("App.vue", "<template></template>")
            .unwrap();
        let jsx_file = project
            .create_file("Component.jsx", "export default () => <div/>")
            .unwrap();
        let tsx_file = project
            .create_file("Component.tsx", "export default (): JSX.Element => <div/>")
            .unwrap();

        assert!(detector.is_user_created_file(&html_file));
        assert!(detector.is_user_created_file(&css_file));
        assert!(detector.is_user_created_file(&scss_file));
        assert!(detector.is_user_created_file(&vue_file));
        assert!(detector.is_user_created_file(&jsx_file));
        assert!(detector.is_user_created_file(&tsx_file));
    }

    #[test]
    fn test_detect_documentation_files() {
        let project = TestProject::new("test_project").unwrap();
        let detector = FileDetector::new().with_root(project.path());

        // Test documentation files
        let readme = project.create_file("README.md", "# Project").unwrap();
        let changelog = project.create_file("CHANGELOG.md", "## v1.0.0").unwrap();
        let license = project.create_file("LICENSE", "MIT License").unwrap();
        let docs_file = project.create_file("docs/guide.md", "# Guide").unwrap();

        assert!(detector.is_user_created_file(&readme));
        assert!(detector.is_user_created_file(&changelog));
        assert!(detector.is_user_created_file(&docs_file));

        // A licence is boilerplate nobody in the project wrote, so it is not
        // counted -- otherwise every project gains a few hundred lines of
        // "documentation" it never authored.
        assert!(!detector.is_user_created_file(&license));
    }

    #[test]
    fn test_script_files_without_extension() {
        let project = TestProject::new("test_project").unwrap();
        let detector = FileDetector::new().with_root(project.path());

        // Test script files without extensions
        let makefile = project
            .create_file("Makefile", "all:\n\techo hello")
            .unwrap();
        let dockerfile = project.create_file("Dockerfile", "FROM alpine").unwrap();
        let jenkinsfile = project.create_file("Jenkinsfile", "pipeline {}").unwrap();
        let vagrantfile = project
            .create_file("Vagrantfile", "Vagrant.configure(2)")
            .unwrap();

        assert!(detector.is_user_created_file(&makefile));
        assert!(detector.is_user_created_file(&dockerfile));
        assert!(detector.is_user_created_file(&jenkinsfile));
        assert!(detector.is_user_created_file(&vagrantfile));
    }

    #[test]
    fn test_ignore_lock_files() {
        let project = TestProject::new("test_project").unwrap();
        let detector = FileDetector::new().with_root(project.path());

        // Test lock files (should be ignored)
        let cargo_lock = project.create_file("Cargo.lock", "# Lock file").unwrap();
        let yarn_lock = project.create_file("yarn.lock", "# yarn lockfile").unwrap();
        let package_lock = project.create_file("package-lock.json", "{}").unwrap();
        let composer_lock = project.create_file("composer.lock", "{}").unwrap();

        assert!(!detector.is_user_created_file(&cargo_lock));
        assert!(!detector.is_user_created_file(&yarn_lock));
        assert!(!detector.is_user_created_file(&package_lock));
        assert!(!detector.is_user_created_file(&composer_lock));
    }

    #[test]
    fn test_ignore_log_files() {
        let project = TestProject::new("test_project").unwrap();
        let detector = FileDetector::new().with_root(project.path());

        // Test log files
        let error_log = project.create_file("error.log", "Error occurred").unwrap();
        let access_log = project.create_file("access.log", "GET /").unwrap();
        let npm_log = project.create_file("npm-debug.log", "debug info").unwrap();

        assert!(!detector.is_user_created_file(&error_log));
        assert!(!detector.is_user_created_file(&access_log));
        assert!(!detector.is_user_created_file(&npm_log));
    }

    #[test]
    fn test_ignore_generated_files() {
        let project = TestProject::new("test_project").unwrap();
        let detector = FileDetector::new().with_root(project.path());

        // Test generated files
        let proto_gen = project
            .create_file("api.pb.go", "// Generated by protoc")
            .unwrap();
        let graphql_gen = project
            .create_file("types.generated.ts", "// Generated")
            .unwrap();
        let webpack_bundle = project
            .create_file("bundle.js", "// webpack bundle")
            .unwrap();
        let min_js = project.create_file("app.min.js", "// minified").unwrap();

        assert!(!detector.is_user_created_file(&proto_gen));
        assert!(!detector.is_user_created_file(&graphql_gen));
        assert!(!detector.is_user_created_file(&webpack_bundle));
        assert!(!detector.is_user_created_file(&min_js));
    }

    #[test]
    fn test_language_specific_patterns() {
        let project = TestProject::new("test_project").unwrap();
        let detector = FileDetector::new().with_root(project.path());

        // Node.js specific ignores
        project.create_node_modules().unwrap();
        let eslint_cache = project.create_file(".eslintcache", "cache").unwrap();
        let nyc_output = project
            .create_file(".nyc_output/coverage.json", "{}")
            .unwrap();

        // Python specific ignores
        let pycache = project
            .create_file("__pycache__/test.pyc", "bytecode")
            .unwrap();
        let pytest_cache = project.create_file(".pytest_cache/test", "cache").unwrap();

        // Java specific ignores
        let maven_target = project
            .create_file("target/classes/Main.class", "bytecode")
            .unwrap();
        let gradle_build = project.create_file("build/libs/app.jar", "jar").unwrap();

        // Go specific ignores
        let go_vendor = project
            .create_file("vendor/github.com/pkg/test.go", "package pkg")
            .unwrap();

        assert!(!detector.is_user_created_file(&eslint_cache));
        assert!(!detector.is_user_created_file(&nyc_output));
        assert!(!detector.is_user_created_file(&pycache));
        assert!(!detector.is_user_created_file(&pytest_cache));
        assert!(!detector.is_user_created_file(&maven_target));
        assert!(!detector.is_user_created_file(&gradle_build));
        assert!(!detector.is_user_created_file(&go_vendor));
    }

    /// Build and dependency locations are recognised from a project-relative
    /// path, and ordinary source is not.
    #[test]
    fn build_locations_are_recognised_and_source_is_not() {
        let patterns = PatternMatcher::new();

        for external in [
            "node_modules/express/index.js",
            "target/debug/myapp",
            "__pycache__/module.pyc",
            "vendor/package/file.go",
            "build/classes/Main.class",
            "dist/bundle.js",
        ] {
            assert!(
                patterns.matches_build_cache_pattern(external),
                "{external:?} should be recognised as build output"
            );
        }

        for source in ["src/main.rs", "scripts/deploy.py", "components/App.vue"] {
            assert!(
                !patterns.matches_build_cache_pattern(source),
                "{source:?} should not be treated as build output"
            );
        }
    }

    /// The extension table decides what gets counted when the optional external
    /// detector is unavailable, so it has to cover the mainstream languages and
    /// exclude anything binary.
    #[test]
    fn code_extensions_cover_languages_and_exclude_binaries() {
        for ext in [
            "rs", "py", "js", "ts", "java", "cpp", "go", "rb", "php", "json", "yaml", "toml", "md",
        ] {
            assert!(
                FileDetector::is_code_extension(ext),
                "{ext:?} should be counted as source"
            );
        }

        for ext in ["exe", "dll", "so", "class"] {
            assert!(
                !FileDetector::is_code_extension(ext),
                "{ext:?} is binary and must not be counted"
            );
        }
    }

    #[test]
    fn test_complex_directory_structure() {
        let project = TestProject::new("test_project").unwrap();
        let detector = FileDetector::new().with_root(project.path());

        // Create a complex project structure
        project.create_directory("src/components").unwrap();
        project.create_directory("src/utils").unwrap();
        project.create_directory("tests").unwrap();
        project.create_directory("docs").unwrap();

        // User files
        let main_rs = project.create_file("src/main.rs", "fn main() {}").unwrap();
        let component = project
            .create_file("src/components/Button.tsx", "export default Button")
            .unwrap();
        let util = project
            .create_file("src/utils/helpers.ts", "export function help() {}")
            .unwrap();
        let test_file = project
            .create_file("tests/integration.rs", "#[test] fn test() {}")
            .unwrap();
        let doc_file = project
            .create_file("docs/README.md", "# Documentation")
            .unwrap();

        // Generated/ignored files
        project.create_node_modules().unwrap();
        project.create_target_dir().unwrap();
        let node_file = project
            .create_file("node_modules/react/index.js", "module.exports = React")
            .unwrap();
        let target_file = project.create_file("target/debug/myapp", "binary").unwrap();

        // Test user files are detected
        assert!(detector.is_user_created_file(&main_rs));
        assert!(detector.is_user_created_file(&component));
        assert!(detector.is_user_created_file(&util));
        assert!(detector.is_user_created_file(&test_file));
        assert!(detector.is_user_created_file(&doc_file));

        // Test ignored files are not detected
        assert!(!detector.is_user_created_file(&node_file));
        assert!(!detector.is_user_created_file(&target_file));
    }

    #[test]
    fn test_edge_cases() {
        let project = TestProject::new("test_project").unwrap();
        let detector = FileDetector::new().with_root(project.path());

        // File with no extension
        let no_ext = project
            .create_file("script", "#!/bin/bash\necho hello")
            .unwrap();
        assert!(detector.is_user_created_file(&no_ext));

        // A shebang is what makes an extension-less file source, so a plain
        // key/value environment file is not counted: it holds secrets, it is
        // conventionally git-ignored, and it is not code.
        let env_file = project.create_file(".env", "DEBUG=true").unwrap();
        assert!(!detector.is_user_created_file(&env_file));

        // Hidden files that *are* hand-written config do count.
        let hidden_config = project.create_file(".editorconfig", "root = true").unwrap();
        assert!(detector.is_user_created_file(&hidden_config));

        // Empty file
        let empty_file = project.create_file("empty.rs", "").unwrap();
        assert!(detector.is_user_created_file(&empty_file));

        // File with unusual but valid extension
        let unusual_ext = project
            .create_file("config.toml", "key = \"value\"")
            .unwrap();
        assert!(detector.is_user_created_file(&unusual_ext));
    }

    #[test]
    fn test_case_sensitivity() {
        let project = TestProject::new("test_project").unwrap();
        let detector = FileDetector::new().with_root(project.path());

        // Test case variations
        let makefile_lower = project
            .create_file("makefile", "all:\n\techo test")
            .unwrap();
        let makefile_upper = project
            .create_file("Makefile", "all:\n\techo test")
            .unwrap();
        let dockerfile_lower = project.create_file("dockerfile", "FROM alpine").unwrap();
        let dockerfile_upper = project.create_file("Dockerfile", "FROM alpine").unwrap();

        assert!(detector.is_user_created_file(&makefile_lower));
        assert!(detector.is_user_created_file(&makefile_upper));
        assert!(detector.is_user_created_file(&dockerfile_lower));
        assert!(detector.is_user_created_file(&dockerfile_upper));
    }

    #[cfg(unix)]
    #[test]
    fn test_symlinks() {
        let project = TestProject::new("test_project").unwrap();
        let detector = FileDetector::new().with_root(project.path());

        // Create a real file
        let real_file = project.create_file("real.rs", "fn main() {}").unwrap();

        // Create a symlink (if supported on the platform)
        let symlink_path = project.root.join("link.rs");
        if std::os::unix::fs::symlink(&real_file, &symlink_path).is_ok() {
            assert!(detector.is_user_created_file(&symlink_path));
        }
    }

    #[test]
    fn test_deeply_nested_ignored_directories() {
        let project = TestProject::new("test_project").unwrap();
        let detector = FileDetector::new().with_root(project.path());

        // Create deeply nested ignored structure
        project
            .create_directory("node_modules/package/dist/subpackage")
            .unwrap();
        let deep_file = project
            .create_file(
                "node_modules/package/dist/subpackage/deeply-nested.js",
                "// deeply nested file",
            )
            .unwrap();

        assert!(!detector.is_user_created_file(&deep_file));
    }

    #[test]
    fn test_multiple_language_project() {
        let project = TestProject::new("test_project").unwrap();
        let detector = FileDetector::new().with_root(project.path());

        // Create a polyglot project
        let rust_main = project.create_file("src/main.rs", "fn main() {}").unwrap();
        let python_script = project
            .create_file("scripts/deploy.py", "#!/usr/bin/env python3")
            .unwrap();
        let js_frontend = project
            .create_file("frontend/app.js", "console.log('hello')")
            .unwrap();
        let go_backend = project
            .create_file("backend/server.go", "package main")
            .unwrap();
        let docker_config = project.create_file("Dockerfile", "FROM alpine").unwrap();
        let k8s_config = project
            .create_file("k8s/deployment.yaml", "apiVersion: apps/v1")
            .unwrap();

        // All should be detected as user files
        assert!(detector.is_user_created_file(&rust_main));
        assert!(detector.is_user_created_file(&python_script));
        assert!(detector.is_user_created_file(&js_frontend));
        assert!(detector.is_user_created_file(&go_backend));
        assert!(detector.is_user_created_file(&docker_config));
        assert!(detector.is_user_created_file(&k8s_config));

        // But ignore the generated/dependency files in the same project
        project.create_node_modules().unwrap();
        project.create_target_dir().unwrap();
        let go_vendor = project
            .create_file("vendor/github.com/pkg/errors/errors.go", "package errors")
            .unwrap();

        assert!(!detector.is_user_created_file(&go_vendor));
    }

    #[test]
    fn test_performance_with_many_files() {
        let project = TestProject::new("test_project").unwrap();
        let detector = FileDetector::new().with_root(project.path());

        // Create many files to test performance
        let mut files = Vec::new();
        for i in 0..100 {
            let file_path = project
                .create_file(&format!("file_{}.rs", i), "fn main() {}")
                .unwrap();
            files.push(file_path);
        }

        let start = std::time::Instant::now();
        for file_path in &files {
            detector.is_user_created_file(file_path);
        }
        let duration = start.elapsed();

        // Should be fast (less than 100ms for 100 files)
        assert!(duration.as_millis() < 100);
    }

    #[test]
    fn test_concurrent_detection() {
        use std::sync::Arc;
        use std::thread;

        let project = TestProject::new("test_project").unwrap();
        let detector = Arc::new(FileDetector::new().with_root(project.path()));

        // Create files to test
        let files: Vec<_> = (0..20)
            .map(|i| {
                project
                    .create_file(&format!("test_{}.rs", i), "fn main() {}")
                    .unwrap()
            })
            .collect();

        let handles: Vec<_> = files
            .into_iter()
            .map(|file_path| {
                let detector = Arc::clone(&detector);
                thread::spawn(move || detector.is_user_created_file(&file_path))
            })
            .collect();

        let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();

        // All should be detected as user files
        for result in results {
            assert!(result);
        }
    }
}

/// The contract that keeps an optional external detector from changing results.
///
/// `Rejected` and `Unknown` used to be one case -- "not recognized" -- and both
/// fell through to Sherlock. Installing Sherlock therefore re-admitted files the
/// built-in rules deliberately exclude, so the same commit measured differently
/// on two machines. These tests pin the boundary.
#[cfg(test)]
mod classification {
    use super::*;
    use crate::core::detector::{
        Classification, FileDetector, SherlockLanguage, SherlockResult, SherlockSummary,
    };

    /// A detection result that claims every given path is source.
    fn detected(root: &Path, relative: &[&str]) -> SherlockResult {
        let files: Vec<String> = relative
            .iter()
            .map(|r| root.join(r).to_string_lossy().to_string())
            .collect();
        SherlockResult {
            summary: SherlockSummary {
                languages_detected: 1,
                total_bytes: 0,
                total_files: files.len(),
            },
            languages: vec![SherlockLanguage {
                name: "Fictional".to_string(),
                color: "#000000".to_string(),
                file_count: files.len(),
                files,
                percentage: 100.0,
                total_bytes: 0,
            }],
            unknown_files: Vec::new(),
        }
    }

    #[test]
    fn recognized_source_is_source_without_any_detector() {
        let project = TestProject::new("classify_source").unwrap();
        let detector = FileDetector::new().with_root(project.path());
        for name in ["src/main.rs", "app.py", "Makefile", "index.ts", "notes.md"] {
            let path = project.create_file(name, "x\n").unwrap();
            assert_eq!(
                detector.classify(&path),
                Classification::Source,
                "{name} should classify as source on its own"
            );
        }
    }

    #[test]
    fn build_output_generated_and_boilerplate_are_rejected() {
        let project = TestProject::new("classify_rejected").unwrap();
        let detector = FileDetector::new().with_root(project.path());
        for name in [
            "target/debug/build.rs",
            "node_modules/dep/index.js",
            "api.pb.go",
            "schema.generated.rs",
            "LICENSE",
            "LICENSE.md",
            "COPYING",
            "NOTICE.txt",
            "AUTHORS",
        ] {
            let path = project.create_file(name, "x\n").unwrap();
            assert_eq!(
                detector.classify(&path),
                Classification::Rejected,
                "{name} should be rejected outright"
            );
        }
    }

    #[test]
    fn an_unrecognized_extension_is_unknown_not_rejected() {
        let project = TestProject::new("classify_unknown").unwrap();
        let detector = FileDetector::new().with_root(project.path());
        let path = project.create_file("payload.qqq", "data\n").unwrap();
        assert_eq!(detector.classify(&path), Classification::Unknown);
        // With nothing to consult, unknown means not counted.
        assert!(!detector.is_user_created_file(&path));
    }

    /// The load-bearing one: detection may resolve `Unknown`, never `Rejected`.
    #[test]
    fn detection_resolves_unknown_but_cannot_override_a_rejection() {
        let project = TestProject::new("classify_override").unwrap();
        let unknown = project.create_file("payload.qqq", "data\n").unwrap();
        let licence = project.create_file("LICENSE", "MIT License\n").unwrap();
        let vendored = project
            .create_file("node_modules/dep/index.js", "x\n")
            .unwrap();

        let claims = detected(
            project.path(),
            &["payload.qqq", "LICENSE", "node_modules/dep/index.js"],
        );
        let detector = FileDetector::new()
            .with_root(project.path())
            .with_sherlock_result(claims);

        assert!(
            detector.is_user_created_file(&unknown),
            "detection should settle a file nothing else recognizes"
        );
        assert!(
            !detector.is_user_created_file(&licence),
            "detection re-admitted boilerplate the built-in rules reject"
        );
        assert!(
            !detector.is_user_created_file(&vendored),
            "detection re-admitted a vendored dependency"
        );
    }

    /// Boilerplate is matched on the stem, so every spelling agrees. A `.rs`
    /// file merely *called* `license` is somebody's code and must survive.
    #[test]
    fn boilerplate_matching_is_by_stem_and_prose_extension() {
        for name in ["LICENSE", "license.md", "Licence.TXT", "COPYING", "NOTICE"] {
            assert!(
                PatternMatcher::is_boilerplate_file(name),
                "{name} should be treated as boilerplate"
            );
        }
        for name in ["license.rs", "licenses.py", "notice_test.go", "authors.sql"] {
            assert!(
                !PatternMatcher::is_boilerplate_file(name),
                "{name} is source that happens to be named after boilerplate"
            );
        }
    }
}
