//! What the editor is shown for one file: a lens and any threshold warnings.
//!
//! Kept apart from the protocol plumbing in [`super`] so the thing worth
//! getting right -- what counts as too long, what counts as undocumented -- can
//! be read and tested without a language server attached.

use crate::core::languages;
use crate::core::types::FileStats;
use crate::utils::config::LspPreferences;
use lsp_types::{
    CodeLens, Command, Diagnostic, DiagnosticSeverity, NumberOrString, Position, Range,
};

/// The whole of the first line, which is where a file-level remark belongs.
fn first_line() -> Range {
    Range::new(Position::new(0, 0), Position::new(0, 0))
}

/// `n` with thousands separators.
fn grouped(n: usize) -> String {
    let digits = n.to_string();
    let mut out = String::with_capacity(digits.len() + digits.len() / 3);
    for (index, digit) in digits.char_indices() {
        if index > 0 && (digits.len() - index).is_multiple_of(3) {
            out.push(',');
        }
        out.push(digit);
    }
    out
}

/// The lens shown above the first line of `path`.
///
/// The command is deliberately empty: the lens exists to be read, and naming a
/// command every client would have to implement would leave it broken
/// everywhere but one editor.
pub fn lens(stats: &FileStats, key: &str) -> CodeLens {
    let language = languages::describe(key).0;
    let mut parts = vec![format!("{} {}", grouped(stats.total_lines), language)];
    for (count, label) in [
        (stats.code_lines, "code"),
        (stats.doc_lines, "doc"),
        (stats.comment_lines, "comment"),
        (stats.blank_lines, "blank"),
    ] {
        if count > 0 {
            parts.push(format!("{} {label}", grouped(count)));
        }
    }

    CodeLens {
        range: first_line(),
        command: Some(Command {
            title: parts.join(" · "),
            command: String::new(),
            arguments: None,
        }),
        data: None,
    }
}

/// A warning attributed to howmany, tagged so a user can silence one rule.
fn warn(code: &'static str, message: String) -> Diagnostic {
    Diagnostic {
        range: first_line(),
        severity: Some(DiagnosticSeverity::WARNING),
        code: Some(NumberOrString::String(code.to_string())),
        source: Some("howmany".to_string()),
        message,
        ..Default::default()
    }
}

/// Every threshold `stats` breaches.
///
/// Silence is the common case, so each rule states the number it objected to.
/// A warning that only says "too long" sends the reader to the settings file to
/// find out what the limit was.
pub fn diagnostics(stats: &FileStats, prefs: &LspPreferences) -> Vec<Diagnostic> {
    let mut found = Vec::new();

    if prefs.max_file_lines > 0 && stats.total_lines > prefs.max_file_lines {
        found.push(warn(
            "long-file",
            format!(
                "{} lines, over the {} this project allows",
                grouped(stats.total_lines),
                grouped(prefs.max_file_lines)
            ),
        ));
    }

    // Documentation is measured against code, not against the file: a file that
    // is mostly blank lines and comments is not undocumented, and judging it by
    // total lines made every short file look negligent.
    let documented = stats.doc_lines + stats.comment_lines;
    if prefs.min_doc_ratio > 0.0
        && stats.code_lines >= prefs.documented_from_lines
        && (documented as f64) < prefs.min_doc_ratio * stats.code_lines as f64
    {
        found.push(warn(
            "undocumented",
            format!(
                "{} lines of code with {} of documentation, under the {:.0}% this project asks for",
                grouped(stats.code_lines),
                grouped(documented),
                prefs.min_doc_ratio * 100.0
            ),
        ));
    }

    found
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stats(total: usize, code: usize, doc: usize) -> FileStats {
        FileStats {
            total_lines: total,
            code_lines: code,
            comment_lines: 0,
            blank_lines: total.saturating_sub(code + doc),
            doc_lines: doc,
            file_size: total as u64 * 20,
        }
    }

    fn prefs() -> LspPreferences {
        LspPreferences::default()
    }

    #[test]
    fn the_lens_leads_with_the_language_and_the_total() {
        let title = lens(&stats(120, 90, 10), "rs").command.unwrap().title;
        assert!(title.starts_with("120 Rust"), "{title}");
        assert!(title.contains("90 code"), "{title}");
    }

    /// A category nobody has any of is left out rather than shown as zero.
    #[test]
    fn the_lens_omits_empty_categories() {
        let title = lens(&stats(10, 10, 0), "rs").command.unwrap().title;
        assert!(!title.contains("doc"), "{title}");
        assert!(!title.contains("blank"), "{title}");
    }

    #[test]
    fn large_counts_are_grouped_for_reading() {
        let title = lens(&stats(12_345, 10_000, 2_000), "py")
            .command
            .unwrap()
            .title;
        assert!(title.contains("12,345"), "{title}");
    }

    #[test]
    fn an_ordinary_file_produces_no_warnings() {
        assert!(diagnostics(&stats(200, 150, 30), &prefs()).is_empty());
    }

    #[test]
    fn a_long_file_is_warned_about_with_both_numbers() {
        let mut p = prefs();
        p.max_file_lines = 100;
        let found = diagnostics(&stats(250, 200, 40), &p);
        let message = &found
            .iter()
            .find(|d| matches!(&d.code, Some(NumberOrString::String(c)) if c == "long-file"))
            .expect("the length rule should have fired")
            .message;
        assert!(
            message.contains("250") && message.contains("100"),
            "{message}"
        );
    }

    /// The rule exists to catch a large undocumented file, so a small one must
    /// not trip it however bare it is.
    #[test]
    fn a_short_file_is_never_called_undocumented() {
        let p = prefs();
        let just_under = stats(p.documented_from_lines - 1, p.documented_from_lines - 1, 0);
        assert!(diagnostics(&just_under, &p).is_empty());
    }

    #[test]
    fn a_large_bare_file_is_called_undocumented() {
        let p = prefs();
        let bare = stats(p.documented_from_lines * 2, p.documented_from_lines * 2, 0);
        assert!(diagnostics(&bare, &p)
            .iter()
            .any(|d| matches!(&d.code, Some(NumberOrString::String(c)) if c == "undocumented")));
    }

    /// Both rules are switched off by zero, so a project can keep the lens and
    /// decline the opinions.
    #[test]
    fn zero_disables_a_rule() {
        let p = LspPreferences {
            max_file_lines: 0,
            min_doc_ratio: 0.0,
            ..LspPreferences::default()
        };
        assert!(diagnostics(&stats(100_000, 100_000, 0), &p).is_empty());
    }

    #[test]
    fn every_diagnostic_is_attributed_and_coded() {
        let mut p = prefs();
        p.max_file_lines = 1;
        for diagnostic in diagnostics(&stats(500, 500, 0), &p) {
            assert_eq!(diagnostic.source.as_deref(), Some("howmany"));
            assert!(diagnostic.code.is_some(), "a rule must be silenceable");
        }
    }
}
