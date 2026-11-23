//! Repository-owned evidence for which files are authored here.
//!
//! Git attributes supply generated/vendor declarations beyond filename rules.
//! Queries are batched over the exact candidate set; a broken query is an error
//! rather than an empty exclusion set.

use crate::core::detector::SkipReason;
use crate::utils::errors::{HowManyError, Result};
use std::collections::HashMap;
use std::ffi::OsString;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

pub(crate) struct Repository {
    directory: PathBuf,
}

impl Repository {
    pub(crate) fn discover(root: &Path) -> Result<Option<Self>> {
        let directory = if root.is_file() {
            root.parent().unwrap_or(root)
        } else {
            root
        };
        let status = Command::new("git")
            .args(["--no-optional-locks", "-C"])
            .arg(directory)
            .args(["rev-parse", "--is-inside-work-tree"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
        match status {
            Ok(status) if status.success() => Ok(Some(Self {
                directory: directory.to_path_buf(),
            })),
            Ok(_) => Ok(None),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(error.into()),
        }
    }

    fn command(&self) -> Command {
        let mut command = Command::new("git");
        command
            .args(["--no-optional-locks", "-C"])
            .arg(&self.directory);
        command
    }

    /// Tracked working-tree paths, including tracked hidden and ignored files.
    pub(crate) fn tracked_paths(&self, root: &Path) -> Result<Vec<PathBuf>> {
        let output = self
            .command()
            .args(["ls-files", "-z", "--cached"])
            .output()?;
        if !output.status.success() {
            return Err(HowManyError::filter(format!(
                "git ls-files failed: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            )));
        }
        let mut paths = output
            .stdout
            .split(|byte| *byte == 0)
            .filter(|path| !path.is_empty())
            .map(|path| decode_path(path).map(|path| self.directory.join(path)))
            .collect::<Result<Vec<_>>>()?;
        if root.is_file() {
            paths.retain(|path| path == root);
        }
        paths.sort_unstable();
        paths.dedup();
        Ok(paths)
    }

    /// Read nested `.gitattributes` using Git's own parser and precedence rules.
    pub(crate) fn exclusions(&self, paths: &[PathBuf]) -> Result<HashMap<PathBuf, SkipReason>> {
        if paths.is_empty() {
            return Ok(HashMap::new());
        }
        let mut input = Vec::new();
        for path in paths {
            let relative = path.strip_prefix(&self.directory).map_err(|_| {
                HowManyError::filter("attribute candidate lies outside the analysis directory")
            })?;
            encode_path(relative, &mut input)?;
            input.push(0);
        }
        let mut child = self
            .command()
            .args([
                "check-attr",
                "-z",
                "--stdin",
                "linguist-generated",
                "linguist-vendored",
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        let Some(mut stdin) = child.stdin.take() else {
            let _ = child.kill();
            let _ = child.wait();
            return Err(HowManyError::filter("git check-attr has no input pipe"));
        };
        // Git emits results while reading. Drain output concurrently with the
        // input writer so a repository larger than either pipe cannot deadlock.
        let writer = match std::thread::Builder::new()
            .name("howmany-git-attributes".into())
            .spawn(move || stdin.write_all(&input))
        {
            Ok(writer) => writer,
            Err(error) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(error.into());
            }
        };
        let output = child.wait_with_output();
        let written = writer
            .join()
            .map_err(|_| HowManyError::filter("git attribute writer stopped unexpectedly"))?;
        let output = output?;
        if !output.status.success() {
            return Err(HowManyError::filter(format!(
                "git check-attr failed: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            )));
        }
        written?;
        parse_attributes(
            &self.directory,
            &output.stdout,
            paths.len().saturating_mul(2),
        )
    }
}

#[cfg(unix)]
fn decode_path(bytes: &[u8]) -> Result<OsString> {
    use std::os::unix::ffi::OsStringExt;
    Ok(OsString::from_vec(bytes.to_vec()))
}

#[cfg(not(unix))]
fn decode_path(bytes: &[u8]) -> Result<OsString> {
    std::str::from_utf8(bytes)
        .map(OsString::from)
        .map_err(|_| HowManyError::filter("Git returned a path that is not valid UTF-8"))
}

fn encode_path(path: &Path, output: &mut Vec<u8>) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt;
        output.extend_from_slice(path.as_os_str().as_bytes());
    }
    #[cfg(not(unix))]
    {
        let text = path
            .to_str()
            .ok_or_else(|| HowManyError::filter("Git candidate is not valid UTF-8"))?;
        output.extend_from_slice(text.replace('\\', "/").as_bytes());
    }
    Ok(())
}

fn parse_attributes(
    root: &Path,
    bytes: &[u8],
    expected_rows: usize,
) -> Result<HashMap<PathBuf, SkipReason>> {
    if bytes.is_empty() {
        return if expected_rows == 0 {
            Ok(HashMap::new())
        } else {
            Err(HowManyError::filter("empty git check-attr response"))
        };
    }
    let Some(body) = bytes.strip_suffix(b"\0") else {
        return Err(HowManyError::filter("unterminated git check-attr response"));
    };
    let fields: Vec<&[u8]> = body.split(|byte| *byte == 0).collect();
    if fields.len() != expected_rows.saturating_mul(3) {
        return Err(HowManyError::filter("incomplete git check-attr response"));
    }
    let mut excluded = HashMap::new();
    for row in fields.as_chunks::<3>().0 {
        if matches!(row[2], b"unspecified" | b"unset" | b"false") {
            continue;
        }
        let reason = match row[1] {
            b"linguist-generated" => SkipReason::Generated,
            b"linguist-vendored" => SkipReason::NotAuthoredHere,
            _ => return Err(HowManyError::filter("unexpected Git provenance attribute")),
        };
        let path = PathBuf::from(decode_path(row[0])?);
        if path.is_absolute()
            || path
                .components()
                .any(|part| part == std::path::Component::ParentDir)
        {
            return Err(HowManyError::filter(
                "Git attribute path escapes the analysis directory",
            ));
        }
        excluded.insert(root.join(path), reason);
    }
    Ok(excluded)
}

/// Explicit generator annotations in the leading comments, allowing import
/// declarations and an HTML doctype before the banner. Inspect a bounded
/// header, not arbitrary strings or examples in the executable body.
pub(crate) fn has_generated_banner(path: &Path) -> Result<bool> {
    let mut head = [0; 8192];
    let read = std::fs::File::open(path)?.read(&mut head)?;
    let header = String::from_utf8_lossy(&head[..read]);
    let mut block_end = None;
    let mut fence = None;
    let mut comments = String::new();
    for line in header.lines().take(32) {
        let line = line.trim().trim_start_matches('\u{feff}');
        if line.is_empty() || line.starts_with("#!") {
            continue;
        }
        let comment = if block_end.is_some() {
            line.trim_start_matches('*').trim()
        } else if let Some((prefix, end)) = [
            ("<!--", "-->"),
            ("/*", "*/"),
            ("/-", "-/"),
            ("\"\"\"", "\"\"\""),
            ("'''", "'''"),
        ]
        .into_iter()
        .find(|(prefix, _)| line.starts_with(prefix))
        {
            block_end = Some(end);
            line[prefix.len()..]
                .trim_start_matches(['*', '!', '-'])
                .trim()
        } else if let Some(comment) = ["//", "#", "--"]
            .into_iter()
            .find_map(|prefix| line.strip_prefix(prefix))
        {
            comment.trim_start_matches(['/', '!']).trim()
        } else if is_header_preamble(line) {
            comments.clear();
            continue;
        } else {
            break;
        };

        let mut ends_header = false;
        let comment = if let Some(end) = block_end {
            if let Some((text, suffix)) = comment.split_once(end) {
                block_end = None;
                ends_header = !suffix.trim().is_empty();
                text
            } else {
                comment
            }
        } else {
            comment
        };
        let comment = comment.trim();
        if let Some(marker) = fence {
            if comment.starts_with(marker) {
                fence = None;
            }
            if ends_header {
                break;
            }
            continue;
        }
        if let Some(marker) = ["```", "~~~"]
            .into_iter()
            .find(|marker| comment.starts_with(marker))
        {
            fence = Some(marker);
            comments.clear();
            continue;
        }
        if !comments.is_empty() {
            comments.push(' ');
        }
        comments.push_str(&comment.to_ascii_lowercase());
        if is_generator_annotation(&comment.to_ascii_lowercase())
            || is_generator_annotation(&comments)
        {
            return Ok(true);
        }
        if ends_header {
            break;
        }
    }
    Ok(false)
}

fn is_header_preamble(line: &str) -> bool {
    if line.len() >= 10
        && line
            .get(..9)
            .is_some_and(|prefix| prefix.eq_ignore_ascii_case("<!doctype"))
        && line.as_bytes()[9].is_ascii_whitespace()
        && line.ends_with('>')
    {
        return true;
    }
    let import = line
        .strip_prefix("@testable ")
        .or_else(|| line.strip_prefix("@preconcurrency "))
        .or_else(|| line.strip_prefix("public "))
        .unwrap_or(line)
        .strip_prefix("import ");
    import.is_some_and(|module| {
        let module = module.trim_end_matches(';').trim();
        !module.is_empty()
            && module
                .chars()
                .all(|c| c.is_alphanumeric() || matches!(c, '_' | '.'))
    })
}

fn is_generator_annotation(comment: &str) -> bool {
    comment.starts_with("code generated ")
        || comment.starts_with("generated by ")
        || comment.starts_with("automatically generated ")
        || comment.starts_with("auto-generated ")
        || comment == "@generated"
        || comment.starts_with("@generated ")
        || comment == "lean compiler output"
        || comment.starts_with("this file was automatically generated ")
        || comment.starts_with("this file is automatically generated ")
        || comment.starts_with("this is an automatically generated file")
        || (comment.starts_with("generated from ") && forbids_editing(comment))
        || (comment.starts_with("this file ")
            && comment.contains("generated")
            && forbids_editing(comment))
        || (comment.contains("generated by ") && forbids_editing(comment))
}

fn forbids_editing(comment: &str) -> bool {
    ["do not edit", "never hand-edit"]
        .into_iter()
        .any(|directive| {
            comment.match_indices(directive).any(|(at, _)| {
                let rest = comment[at + directive.len()..].trim_start();
                rest.is_empty()
                    || rest.starts_with(['.', '!', ':', ';'])
                    || ["this file", "by hand", "manually"]
                        .into_iter()
                        .any(|object| {
                            rest.strip_prefix(object).is_some_and(|suffix| {
                                suffix.is_empty() || suffix.starts_with(['.', '!', ':', ';', ' '])
                            })
                        })
            })
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn attributes_respect_explicit_false_and_reject_partial_responses() {
        let root = Path::new("project");
        let excluded = parse_attributes(
            root,
            b"generated.go\0linguist-generated\0set\0source.go\0linguist-generated\0false\0vendor.go\0linguist-vendored\0true\0",
            3,
        )
        .unwrap();
        assert_eq!(excluded.len(), 2);
        assert_eq!(excluded[&root.join("generated.go")], SkipReason::Generated);
        assert_eq!(
            excluded[&root.join("vendor.go")],
            SkipReason::NotAuthoredHere
        );
        assert!(parse_attributes(root, b"file\0linguist-generated\0", 1).is_err());
        assert!(parse_attributes(root, b"../outside\0linguist-generated\0set\0", 1).is_err());
        assert!(parse_attributes(root, b"", 1).is_err());
    }

    #[test]
    fn banners_do_not_confuse_generated_dependencies_with_generated_files() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("source.go");
        for (text, generated) in [
            (
                "// Code generated by sqlc. DO NOT EDIT.\npackage data\n",
                true,
            ),
            ("# Generated by schema compiler.\nvalue = 1\n", true),
            (
                "\"\"\"Auto-generated whitelist. Do not edit.\"\"\"\nvalue = 1\n",
                true,
            ),
            (
                "\"\"\"Uniform banners for generated files.\"\"\"\nvalue = 1\n",
                false,
            ),
            (
                "// The types are generated from a registry.\npackage source\n",
                false,
            ),
            (
                "// Generated from every type token in theme.css.\nconst x = 1;\n",
                false,
            ),
            (
                "const x = 1;\n// Code generated by the example. DO NOT EDIT.\n",
                false,
            ),
        ] {
            std::fs::write(&path, text).unwrap();
            assert_eq!(has_generated_banner(&path).unwrap(), generated, "{text}");
        }
    }

    #[test]
    fn generator_annotations_support_documentation_comments_and_preambles() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("source");
        for text in [
            "//! A table generated by `make_table` from a registry.\n//! Do not edit: change the registry.\nconst TABLE: &[u8] = &[];\n",
            "/// Code generated by the schema compiler.\npub struct Model;\n",
            "/*\n * This file was generated from a schema.\n * DO NOT EDIT!\n */\nint value;\n",
            "<!-- This is an automatically generated file.\n     It will be overwritten.\n     DO NOT EDIT! -->\n<p>value</p>\n",
            "<!DOCTYPE html>\n<!-- This file is generated.\nDO NOT EDIT! -->\n<p>value</p>\n",
            "import AppKit\nimport Carbon\n// Code generated by catalog. DO NOT EDIT.\nenum Command {}\n",
            "// Copyright Example\n// Generated by a schema compiler.\nstruct Model;\n",
            "-- THIS FILE WAS AUTOMATICALLY GENERATED BY AENEAS\nmodule\n",
            "/-! This file is automatically generated by a model extractor. -/\nmodule\n",
            "/-- @generated -/\ndef value := 1\n",
            "// Lean compiler output\n#include <lean/lean.h>\n",
            "/** @generated */\nexport const value = 1;\n",
        ] {
            std::fs::write(&path, text).unwrap();
            assert!(has_generated_banner(&path).unwrap(), "{text}");
        }
    }

    #[test]
    fn descriptions_strings_and_body_examples_are_not_generator_annotations() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("source");
        for text in [
            "// Generated inventory; keep the list in sync.\nenum Command {}\n",
            "// The types are generated by a dependency.\nfn main() {}\n",
            "// A test of the @generated syntax.\nfn main() {}\n",
            "const banner = \"// Code generated by tool. DO NOT EDIT.\";\n",
            "import AppKit; run()\n// Code generated by the example. DO NOT EDIT.\n",
            "<!DOCTYPE html>\n<script>\n// Code generated by the example. DO NOT EDIT.\n</script>\n",
            "αβγδεζηθικλ := 1\n-- Code generated by the example. DO NOT EDIT.\n",
            "/* Copyright Example */ fn main() {\n// Code generated by the example. DO NOT EDIT.\n}\n",
            "//! An example of generated code:\n//! ```rust\n//! // Code generated by tool. DO NOT EDIT.\n//! ```\nfn main() {}\n",
            "//! Tests values generated by a dependency.\n//! Do not edit the dependency; edit this adapter instead.\nfn main() {}\n",
        ] {
            std::fs::write(&path, text).unwrap();
            assert!(!has_generated_banner(&path).unwrap(), "{text}");
        }
    }
}
