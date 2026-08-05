//! `howmany init` -- connecting the language server to the editors you have.
//!
//! Installing a CLI and then hand-editing three config files to get anything
//! out of it is the step where most people stop. So this does the editing: it
//! looks for the editors that are actually on the machine, wires each one to
//! the binary that is running right now, and says exactly what it changed.
//!
//! Two rules it will not break. It never touches an editor that is not
//! installed, and it never overwrites a setting a user has already pointed
//! somewhere else without being asked with `--force`.

pub mod jsonc;
pub mod neovim;
pub mod vscode;

use crate::ui::cli::InitArgs;
use crate::utils::errors::Result;
use clap::ValueEnum;
use std::fmt;
use std::path::{Path, PathBuf};

/// An editor `init` knows how to set up.
///
/// Deliberately short. Zed and Helix cannot register an arbitrary language
/// server from their settings -- Zed needs a compiled extension, Helix needs
/// the server named under every language individually -- so claiming to
/// support them would mean writing files that quietly do nothing.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, ValueEnum)]
pub enum EditorId {
    #[value(name = "vscode", alias = "code")]
    VsCode,
    Cursor,
    #[value(name = "neovim", alias = "nvim")]
    Neovim,
}

impl EditorId {
    pub const ALL: [Self; 3] = [Self::VsCode, Self::Cursor, Self::Neovim];

    pub fn label(self) -> &'static str {
        match self {
            Self::VsCode => "VS Code",
            Self::Cursor => "Cursor",
            Self::Neovim => "Neovim",
        }
    }
}

impl fmt::Display for EditorId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

/// What happened to one editor.
#[derive(Debug, PartialEq, Eq)]
pub enum Step {
    /// Something on disk changed.
    Changed(String),
    /// Already correct, so nothing was written.
    Unchanged(String),
    /// Deliberately left alone, and why.
    Skipped(String),
    Failed(String),
}

impl Step {
    fn mark(&self) -> &'static str {
        match self {
            Self::Changed(_) => "+",
            Self::Unchanged(_) => "=",
            Self::Skipped(_) => "-",
            Self::Failed(_) => "!",
        }
    }

    fn detail(&self) -> &str {
        let (Self::Changed(text)
        | Self::Unchanged(text)
        | Self::Skipped(text)
        | Self::Failed(text)) = self;
        text
    }
}

/// Everything one editor's setup needs to know.
pub struct Plan<'a> {
    /// Absolute path to the running binary.
    ///
    /// Absolute rather than the bare name because a GUI editor launched from
    /// the dock does not inherit a shell's `PATH`; the commonest failure of
    /// every hand-written integration is an editor that cannot find a binary
    /// the terminal finds instantly.
    pub binary: &'a Path,
    pub dry_run: bool,
    pub force: bool,
}

impl Plan<'_> {
    /// Write `contents` to `path`, creating parents, unless this is a rehearsal.
    pub fn write(&self, path: &Path, contents: &str) -> std::io::Result<()> {
        if self.dry_run {
            return Ok(());
        }
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, contents)
    }
}

/// Set up every editor asked for, or every editor found.
pub fn run(args: &InitArgs) -> Result<()> {
    let binary = std::env::current_exe()?;
    let binary = binary.canonicalize().unwrap_or(binary);
    let plan = Plan {
        binary: &binary,
        dry_run: args.dry_run,
        force: args.force,
    };

    let wanted: Vec<EditorId> = if args.editors.is_empty() {
        EditorId::ALL.to_vec()
    } else {
        args.editors.clone()
    };
    // An editor named outright is one the user knows they have, so a detection
    // miss must not silently drop it.
    let named = !args.editors.is_empty();

    println!("howmany {}", env!("CARGO_PKG_VERSION"));
    println!("using {}\n", binary.display());

    let mut reached = 0;
    for editor in wanted {
        let steps = match editor {
            EditorId::VsCode => vscode::setup(vscode::Flavor::VsCode, &plan, named),
            EditorId::Cursor => vscode::setup(vscode::Flavor::Cursor, &plan, named),
            EditorId::Neovim => neovim::setup(&plan, named),
        };

        // An editor that is simply not installed is not news.
        if !named && steps.iter().all(|step| matches!(step, Step::Skipped(_))) {
            continue;
        }

        reached += 1;
        println!("{editor}");
        for step in &steps {
            println!("  {} {}", step.mark(), step.detail());
        }
        println!();
    }

    if reached == 0 {
        println!("No supported editor found. Install one, then run `howmany init` again.");
    } else if args.dry_run {
        println!("Nothing was written. Run without --dry-run to apply.");
    } else {
        println!("Open a file in your editor to see the line breakdown above it.");
    }

    Ok(())
}

/// Editors that are installed but not yet wired up.
pub fn pending() -> Vec<EditorId> {
    EditorId::ALL
        .into_iter()
        .filter(|editor| {
            let state = match editor {
                EditorId::VsCode => vscode::state(vscode::Flavor::VsCode),
                EditorId::Cursor => vscode::state(vscode::Flavor::Cursor),
                EditorId::Neovim => neovim::state(),
            };
            state == Some(false)
        })
        .collect()
}

/// Where the fact that we have already asked is recorded.
fn asked_marker() -> Option<PathBuf> {
    Some(dirs::config_dir()?.join("howmany").join("editors-offered"))
}

/// Ask once, the first time howmany is run by hand, whether to set up editors.
///
/// Neither `cargo install` nor a Homebrew bottle can run anything after itself
/// that is allowed to touch a user's editor configuration, so the offer has to
/// come from the tool. It comes exactly once, only when a person is watching,
/// and only when there is an editor to offer -- and taking no for an answer is
/// recorded before the question is asked, so an interrupted run cannot turn
/// into a prompt that reappears forever.
pub fn offer_once() {
    use std::io::{BufRead, IsTerminal, Write};

    if std::env::var_os("HOWMANY_NO_SETUP").is_some() {
        return;
    }
    if !std::io::stdout().is_terminal() || !std::io::stdin().is_terminal() {
        return;
    }

    let Some(marker) = asked_marker() else { return };
    if marker.exists() {
        return;
    }

    let waiting = pending();
    if waiting.is_empty() {
        return;
    }

    if let Some(parent) = marker.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    if std::fs::write(&marker, "").is_err() {
        // Unable to remember the answer, so asking would mean asking forever.
        return;
    }

    let names: Vec<_> = waiting.iter().copied().map(EditorId::label).collect();
    println!(
        "\nhowmany can show these counts in {} as you type.",
        names.join(" and ")
    );
    print!("Set that up now? [Y/n] ");
    std::io::stdout().flush().ok();

    let mut answer = String::new();
    if std::io::stdin().lock().read_line(&mut answer).is_err() {
        return;
    }

    if matches!(answer.trim().to_lowercase().as_str(), "" | "y" | "yes") {
        println!();
        if let Err(err) = run(&InitArgs::default()) {
            eprintln!("Setup failed: {err}. Run `howmany init` to try again.");
        }
    } else {
        println!("Run `howmany init` whenever you want it.");
    }
}

/// The first directory on `PATH` holding an executable called `name`.
///
/// A dependency for this would be reasonable, but it is ten lines and the
/// tool's install footprint is a selling point.
pub fn on_path(name: &str) -> Option<PathBuf> {
    std::env::split_paths(&std::env::var_os("PATH")?)
        .map(|dir| dir.join(name))
        .find(|candidate| is_executable(candidate))
}

fn is_executable(path: &Path) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::metadata(path)
            .is_ok_and(|meta| meta.is_file() && meta.permissions().mode() & 0o111 != 0)
    }
    #[cfg(not(unix))]
    {
        path.is_file()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_editor_has_a_name_a_user_would_recognize() {
        for editor in EditorId::ALL {
            assert!(!editor.to_string().is_empty());
        }
    }

    /// The flag spellings are part of the interface; renaming one silently
    /// breaks every script and README that uses it.
    #[test]
    fn editors_are_selectable_by_the_names_we_document() {
        for (spelling, expected) in [
            ("vscode", EditorId::VsCode),
            ("code", EditorId::VsCode),
            ("cursor", EditorId::Cursor),
            ("neovim", EditorId::Neovim),
            ("nvim", EditorId::Neovim),
        ] {
            assert_eq!(
                EditorId::from_str(spelling, true).ok(),
                Some(expected),
                "--editor {spelling} should select {expected}"
            );
        }
    }

    #[test]
    fn a_rehearsal_writes_nothing() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("nested").join("settings.json");
        let binary = PathBuf::from("/usr/local/bin/howmany");

        Plan {
            binary: &binary,
            dry_run: true,
            force: false,
        }
        .write(&target, "{}")
        .unwrap();

        assert!(!target.exists(), "--dry-run must not create files");
    }

    #[test]
    fn writing_creates_missing_directories() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("a").join("b").join("settings.json");
        let binary = PathBuf::from("/usr/local/bin/howmany");

        Plan {
            binary: &binary,
            dry_run: false,
            force: false,
        }
        .write(&target, "{}")
        .unwrap();

        assert_eq!(std::fs::read_to_string(&target).unwrap(), "{}");
    }

    #[test]
    fn a_binary_that_is_on_path_is_found() {
        let dir = tempfile::tempdir().unwrap();
        let name = "howmany-path-probe";
        let target = dir.path().join(name);
        std::fs::write(&target, "#!/bin/sh\n").unwrap();

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&target, std::fs::Permissions::from_mode(0o755)).unwrap();
        }

        let restore = std::env::var_os("PATH");
        // SAFETY: single-threaded test, and the previous value is restored.
        unsafe { std::env::set_var("PATH", dir.path()) };
        let found = on_path(name);
        unsafe {
            match restore {
                Some(value) => std::env::set_var("PATH", value),
                None => std::env::remove_var("PATH"),
            }
        }

        assert_eq!(found, Some(target));
    }

    #[test]
    fn a_binary_that_is_not_on_path_is_not_found() {
        assert_eq!(on_path("howmany-definitely-not-installed-xyz"), None);
    }
}
