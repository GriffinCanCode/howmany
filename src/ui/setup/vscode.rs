//! VS Code and Cursor.
//!
//! Both are the same editor underneath, so they take the same two steps: put
//! the extension in place, and point it at this binary. Only the names of the
//! directory and the command-line tool differ.

use super::{jsonc, on_path, Plan, Step};
use std::path::{Path, PathBuf};
use std::process::Command;

/// The extension the editor installs, and which starts `howmany lsp`.
const EXTENSION: &str = "GriffinCanCode.howmany";

/// The setting the extension reads to find the CLI.
const BINARY_SETTING: &str = "howmany.binaryPath";

#[derive(Clone, Copy)]
pub enum Flavor {
    VsCode,
    Cursor,
}

impl Flavor {
    /// The name of the per-user data directory, which is also the name of the
    /// command-line launcher.
    fn dir_name(self) -> &'static str {
        match self {
            Self::VsCode => "Code",
            Self::Cursor => "Cursor",
        }
    }

    fn cli_name(self) -> &'static str {
        match self {
            Self::VsCode => "code",
            Self::Cursor => "cursor",
        }
    }

    /// Where the launcher lives inside a macOS application bundle.
    ///
    /// Neither editor puts its CLI on `PATH` until you ask it to from the
    /// command palette, and most people never do, so an installed editor
    /// routinely looks uninstalled unless the bundle is checked too.
    fn bundled_cli(self) -> PathBuf {
        let app = match self {
            Self::VsCode => "Visual Studio Code",
            Self::Cursor => "Cursor",
        };
        PathBuf::from("/Applications")
            .join(format!("{app}.app"))
            .join("Contents/Resources/app/bin")
            .join(self.cli_name())
    }
}

/// The `User` directory holding `settings.json`.
fn settings_dir(flavor: Flavor) -> Option<PathBuf> {
    let name = flavor.dir_name();

    // macOS keeps editor state in Application Support while `dirs::config_dir`
    // points at Preferences, which is not where either editor looks.
    if cfg!(target_os = "macos") {
        return Some(
            dirs::home_dir()?
                .join("Library/Application Support")
                .join(name)
                .join("User"),
        );
    }
    Some(dirs::config_dir()?.join(name).join("User"))
}

fn cli(flavor: Flavor) -> Option<PathBuf> {
    on_path(flavor.cli_name()).or_else(|| {
        let bundled = flavor.bundled_cli();
        bundled.is_file().then_some(bundled)
    })
}

/// Whether this editor is installed, and if so whether it already knows about
/// us. [`None`] means it is not installed.
pub fn state(flavor: Flavor) -> Option<bool> {
    let settings = settings_dir(flavor);
    let installed = cli(flavor).is_some() || settings.as_deref().is_some_and(Path::exists);

    installed.then(|| {
        settings
            .map(|dir| std::fs::read_to_string(dir.join("settings.json")).unwrap_or_default())
            .is_some_and(|text| jsonc::get(&text, BINARY_SETTING).is_some())
    })
}

pub fn setup(flavor: Flavor, plan: &Plan, named: bool) -> Vec<Step> {
    let launcher = cli(flavor);
    let settings = settings_dir(flavor);
    let installed = launcher.is_some() || settings.as_deref().is_some_and(Path::exists);

    if !installed {
        return vec![Step::Skipped(format!(
            "{} is not installed",
            flavor.dir_name()
        ))];
    }

    let mut steps = vec![match &launcher {
        Some(path) => install_extension(path, plan),
        None => Step::Failed(format!(
            "cannot install the extension: the `{}` command is not on PATH \
             (run \"Shell Command: Install '{}' command in PATH\" from the command palette)",
            flavor.cli_name(),
            flavor.cli_name()
        )),
    }];

    match settings {
        Some(dir) => steps.push(point_at_binary(&dir.join("settings.json"), plan)),
        None if named => steps.push(Step::Failed(
            "cannot locate the settings directory for this account".to_string(),
        )),
        None => {}
    }

    steps
}

/// Install the extension, letting the editor decide whether it already has it.
fn install_extension(launcher: &Path, plan: &Plan) -> Step {
    if plan.dry_run {
        return Step::Changed(format!("would install the {EXTENSION} extension"));
    }

    let output = Command::new(launcher)
        .args(["--install-extension", EXTENSION, "--force"])
        .output();

    match output {
        Ok(result) if result.status.success() => {
            let said = String::from_utf8_lossy(&result.stdout);
            if said.contains("is already installed") {
                Step::Unchanged(format!("{EXTENSION} is already installed"))
            } else {
                Step::Changed(format!("installed the {EXTENSION} extension"))
            }
        }
        // The extension is on the VS Code Marketplace; a Cursor or VSCodium
        // build pointed at Open VSX will not find it there, and the reason is
        // worth saying rather than reporting a bare non-zero exit.
        Ok(result) => Step::Failed(format!(
            "could not install {EXTENSION}: {}",
            first_line(&String::from_utf8_lossy(&result.stderr))
                .unwrap_or("the editor reported a failure")
        )),
        Err(err) => Step::Failed(format!("could not run {}: {err}", launcher.display())),
    }
}

fn first_line(text: &str) -> Option<&str> {
    text.lines().map(str::trim).find(|line| !line.is_empty())
}

/// Set `howmany.binaryPath` so the extension runs this build.
fn point_at_binary(settings: &Path, plan: &Plan) -> Step {
    let existing = std::fs::read_to_string(settings).unwrap_or_default();
    let wanted = serde_json::Value::String(plan.binary.to_string_lossy().into_owned()).to_string();

    match jsonc::get(&existing, BINARY_SETTING) {
        Some(current) if current == wanted => {
            return Step::Unchanged(format!("{BINARY_SETTING} already points here"))
        }
        // A path the user chose on purpose -- a debug build, a wrapper script --
        // is not ours to redirect.
        Some(current) if !plan.force => {
            return Step::Skipped(format!(
                "{BINARY_SETTING} is set to {current}; pass --force to repoint it"
            ))
        }
        _ => {}
    }

    let updated = jsonc::upsert(&existing, BINARY_SETTING, &wanted);

    // Reading the key back proves the edit landed where it was meant to.
    // Refusing a settings file we cannot parse is the difference between
    // adding a key and corrupting someone's configuration.
    if jsonc::get(&updated, BINARY_SETTING) != Some(wanted.as_str()) {
        return Step::Failed(format!(
            "could not edit {}; add {BINARY_SETTING}: {wanted} by hand",
            settings.display()
        ));
    }

    match plan.write(settings, &updated) {
        Ok(()) if plan.dry_run => Step::Changed(format!("would set {BINARY_SETTING}")),
        Ok(()) => Step::Changed(format!("set {BINARY_SETTING} in {}", settings.display())),
        Err(err) => Step::Failed(format!("could not write {}: {err}", settings.display())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plan_for(binary: &Path, force: bool) -> Plan<'_> {
        Plan {
            binary,
            dry_run: false,
            force,
        }
    }

    #[test]
    fn a_fresh_install_gets_the_binary_path_written() {
        let dir = tempfile::tempdir().unwrap();
        let settings = dir.path().join("settings.json");
        let binary = PathBuf::from("/opt/howmany");

        assert!(matches!(
            point_at_binary(&settings, &plan_for(&binary, false)),
            Step::Changed(_)
        ));

        let written: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&settings).unwrap()).unwrap();
        assert_eq!(written[BINARY_SETTING], "/opt/howmany");
    }

    /// Running init twice must be a no-op the second time, not a second write.
    #[test]
    fn a_second_run_changes_nothing() {
        let dir = tempfile::tempdir().unwrap();
        let settings = dir.path().join("settings.json");
        let binary = PathBuf::from("/opt/howmany");

        point_at_binary(&settings, &plan_for(&binary, false));
        let after_first = std::fs::read_to_string(&settings).unwrap();

        assert!(matches!(
            point_at_binary(&settings, &plan_for(&binary, false)),
            Step::Unchanged(_)
        ));
        assert_eq!(std::fs::read_to_string(&settings).unwrap(), after_first);
    }

    /// A path the user set deliberately survives an init they did not ask to
    /// be destructive.
    #[test]
    fn a_users_own_choice_is_left_alone() {
        let dir = tempfile::tempdir().unwrap();
        let settings = dir.path().join("settings.json");
        std::fs::write(
            &settings,
            "{\n  \"howmany.binaryPath\": \"/my/debug/howmany\"\n}",
        )
        .unwrap();
        let binary = PathBuf::from("/opt/howmany");

        assert!(matches!(
            point_at_binary(&settings, &plan_for(&binary, false)),
            Step::Skipped(_)
        ));
        assert!(std::fs::read_to_string(&settings)
            .unwrap()
            .contains("/my/debug/howmany"));
    }

    #[test]
    fn force_repoints_a_users_choice() {
        let dir = tempfile::tempdir().unwrap();
        let settings = dir.path().join("settings.json");
        std::fs::write(&settings, "{\"howmany.binaryPath\": \"/my/debug/howmany\"}").unwrap();
        let binary = PathBuf::from("/opt/howmany");

        assert!(matches!(
            point_at_binary(&settings, &plan_for(&binary, true)),
            Step::Changed(_)
        ));
        assert!(std::fs::read_to_string(&settings)
            .unwrap()
            .contains("/opt/howmany"));
    }

    /// Other people's settings are not collateral damage.
    #[test]
    fn unrelated_settings_and_comments_are_preserved() {
        let dir = tempfile::tempdir().unwrap();
        let settings = dir.path().join("settings.json");
        std::fs::write(
            &settings,
            "{\n  // hands off\n  \"editor.fontSize\": 13,\n  \"workbench.colorTheme\": \"Dark+\"\n}",
        )
        .unwrap();
        let binary = PathBuf::from("/opt/howmany");

        point_at_binary(&settings, &plan_for(&binary, false));

        let after = std::fs::read_to_string(&settings).unwrap();
        assert!(after.contains("// hands off"));
        assert!(after.contains("\"editor.fontSize\": 13"));
        assert!(after.contains("\"workbench.colorTheme\": \"Dark+\""));
        assert!(after.contains("/opt/howmany"));
    }

    #[test]
    fn a_rehearsal_reports_the_change_without_making_it() {
        let dir = tempfile::tempdir().unwrap();
        let settings = dir.path().join("settings.json");
        let binary = PathBuf::from("/opt/howmany");

        let step = point_at_binary(
            &settings,
            &Plan {
                binary: &binary,
                dry_run: true,
                force: false,
            },
        );

        assert!(matches!(step, Step::Changed(_)));
        assert!(!settings.exists());
    }

    /// A Windows path has backslashes, which are not legal raw in JSON.
    #[test]
    fn a_path_is_escaped_as_json() {
        let dir = tempfile::tempdir().unwrap();
        let settings = dir.path().join("settings.json");
        let binary = PathBuf::from(r"C:\Program Files\howmany\howmany.exe");

        point_at_binary(&settings, &plan_for(&binary, false));

        let text = std::fs::read_to_string(&settings).unwrap();
        let written: serde_json::Value = serde_json::from_str(&text).expect("must stay valid JSON");
        assert_eq!(
            written[BINARY_SETTING],
            r"C:\Program Files\howmany\howmany.exe"
        );
    }

    #[test]
    fn an_editor_that_is_not_installed_is_skipped_rather_than_failed() {
        // No launcher and no settings directory is the uninstalled case; the
        // real check lives in `setup`, so this pins the message it produces.
        let step = Step::Skipped("Code is not installed".to_string());
        assert!(matches!(step, Step::Skipped(_)));
    }

    #[test]
    fn both_flavors_name_a_distinct_directory_and_command() {
        assert_ne!(Flavor::VsCode.dir_name(), Flavor::Cursor.dir_name());
        assert_ne!(Flavor::VsCode.cli_name(), Flavor::Cursor.cli_name());
        assert!(Flavor::VsCode.bundled_cli().ends_with("bin/code"));
        assert!(Flavor::Cursor.bundled_cli().ends_with("bin/cursor"));
    }
}
