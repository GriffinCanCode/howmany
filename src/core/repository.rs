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
    for row in fields.chunks_exact(3) {
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

/// An explicit generator banner before the first line of executable source.
/// Mentions of generated dependencies in ordinary comments are not banners.
pub(crate) fn has_generated_banner(path: &Path) -> Result<bool> {
    let mut head = [0; 8192];
    let read = std::fs::File::open(path)?.read(&mut head)?;
    let header = String::from_utf8_lossy(&head[..read]);
    for line in header.lines().take(32) {
        let line = line.trim().trim_start_matches('\u{feff}');
        if line.is_empty() || line.starts_with("#!") {
            continue;
        }
        let Some(comment) = ["//", "#", "/*", "*", "--", "<!--"]
            .iter()
            .find_map(|prefix| line.strip_prefix(prefix))
        else {
            break;
        };
        let comment = comment.trim().to_ascii_lowercase();
        if comment.starts_with("code generated ")
            || comment.starts_with("generated by ")
            || comment.starts_with("automatically generated ")
            || comment.starts_with("auto-generated ")
            || (comment.starts_with("generated from ")
                && (comment.contains("do not edit") || comment.contains("never hand-edit")))
            || (comment.starts_with("this file ")
                && comment.contains("generated")
                && comment.contains("do not edit"))
        {
            return Ok(true);
        }
    }
    Ok(false)
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
}
