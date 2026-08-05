//! Setting one key in a settings file without rewriting the rest of it.
//!
//! Editor settings are JSON with comments, and they belong to the user. Parsing
//! one into a map and serializing it back would silently delete every comment,
//! reorder every key, and reformat every line -- a diff nobody asked for in a
//! file they hand-maintain. So this edits the text: it finds the span the value
//! occupies and replaces exactly that, leaving every other byte alone.

use std::ops::Range;

/// Advance past whitespace and comments.
fn trivia(bytes: &[u8], mut index: usize) -> usize {
    loop {
        while bytes.get(index).is_some_and(u8::is_ascii_whitespace) {
            index += 1;
        }

        if bytes[index..].starts_with(b"//") {
            index += bytes[index..]
                .iter()
                .position(|b| *b == b'\n')
                .unwrap_or(bytes.len() - index);
        } else if bytes[index..].starts_with(b"/*") {
            let rest = &bytes[index + 2..];
            index = match rest.windows(2).position(|w| w == b"*/") {
                Some(offset) => index + 2 + offset + 2,
                None => bytes.len(),
            };
        } else {
            return index;
        }
    }
}

/// The index just past the string literal starting at `index`.
fn string_end(bytes: &[u8], index: usize) -> usize {
    let mut cursor = index + 1;
    while cursor < bytes.len() {
        match bytes[cursor] {
            b'\\' => cursor += 2,
            b'"' => return cursor + 1,
            _ => cursor += 1,
        }
    }
    bytes.len()
}

/// The index just past the value starting at `start`.
///
/// A value ends where its own nesting does: at the comma or closing brace that
/// belongs to the object holding it, never at one inside a nested object, an
/// array, a string, or a comment.
fn value_end(bytes: &[u8], start: usize) -> usize {
    let mut depth = 0usize;
    let mut index = start;

    while index < bytes.len() {
        match bytes[index] {
            b'"' => {
                index = string_end(bytes, index);
                continue;
            }
            b'/' => {
                let skipped = trivia(bytes, index);
                if skipped != index {
                    index = skipped;
                    continue;
                }
            }
            b'{' | b'[' => depth += 1,
            b'}' | b']' => match depth.checked_sub(1) {
                Some(outer) => depth = outer,
                None => break,
            },
            b',' if depth == 0 => break,
            _ => {}
        }
        index += 1;
    }

    // Trailing whitespace before the comma belongs to the layout, not the value.
    while index > start && bytes[index - 1].is_ascii_whitespace() {
        index -= 1;
    }
    index
}

/// One `"key": value` pair, as spans into the source.
struct Member {
    name: Range<usize>,
    value: Range<usize>,
}

/// The offset of the document's opening brace.
fn object_start(bytes: &[u8]) -> Option<usize> {
    let index = trivia(bytes, 0);
    (bytes.get(index) == Some(&b'{')).then_some(index)
}

/// Every top-level pair, in the order they appear.
///
/// Scanning stops at the first thing that is not a pair rather than guessing:
/// a settings file we cannot read in full is one we have no business rewriting.
fn members(bytes: &[u8], brace: usize) -> Vec<Member> {
    let mut found = Vec::new();
    let mut index = trivia(bytes, brace + 1);

    while index < bytes.len() {
        match bytes[index] {
            b'}' => break,
            b',' => {
                index = trivia(bytes, index + 1);
                continue;
            }
            b'"' => {}
            _ => break,
        }

        let name = index..string_end(bytes, index);
        index = trivia(bytes, name.end);
        if bytes.get(index) != Some(&b':') {
            break;
        }

        index = trivia(bytes, index + 1);
        let value = index..value_end(bytes, index);
        index = trivia(bytes, value.end);
        found.push(Member { name, value });
    }

    found
}

/// The indentation the file already uses for its top-level keys.
fn indent_of(source: &str, first_key: usize) -> &str {
    let line_start = source[..first_key].rfind('\n').map_or(0, |at| at + 1);
    let indent = &source[line_start..first_key];
    if indent.chars().all(char::is_whitespace) && !indent.is_empty() {
        indent
    } else {
        "  "
    }
}

/// The current value of top-level `key`, as it is written in the file.
pub fn get<'a>(source: &'a str, key: &str) -> Option<&'a str> {
    let bytes = source.as_bytes();
    let brace = object_start(bytes)?;
    members(bytes, brace)
        .into_iter()
        .find(|member| source[member.name.clone()].trim_matches('"') == key)
        .map(|member| &source[member.value])
}

/// Set top-level `key` to `value`, which must already be valid JSON.
///
/// An existing key keeps its position and its surrounding comments; a new one
/// is added at the top, where it is visible and where it needs no guesswork
/// about the trailing comma of whatever used to be last.
pub fn upsert(source: &str, key: &str, value: &str) -> String {
    let bytes = source.as_bytes();

    let Some(brace) = object_start(bytes) else {
        // No object to edit -- an empty file, or something we cannot parse.
        // Replacing an unreadable file is not this function's call to make, so
        // the caller only ever hands it an empty one.
        return format!("{{\n  \"{key}\": {value}\n}}\n");
    };

    let existing = members(bytes, brace);

    if let Some(member) = existing
        .iter()
        .find(|member| source[member.name.clone()].trim_matches('"') == key)
    {
        return format!(
            "{}{value}{}",
            &source[..member.value.start],
            &source[member.value.end..]
        );
    }

    match existing.first() {
        Some(first) => {
            let indent = indent_of(source, first.name.start);
            format!(
                "{}\n{indent}\"{key}\": {value},{}",
                &source[..brace + 1],
                &source[brace + 1..]
            )
        }
        None => {
            let close = source[brace..]
                .rfind('}')
                .map_or(source.len(), |at| brace + at);
            format!(
                "{}\n  \"{key}\": {value}\n{}",
                &source[..brace + 1],
                &source[close..]
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The point of the whole module: an edit must not disturb a comment.
    #[test]
    fn comments_and_neighbours_survive_an_edit() {
        let before = r#"{
  // my favourite setting
  "editor.fontSize": 14,
  "howmany.binaryPath": "/old/howmany",
  /* block */
  "files.autoSave": "off"
}"#;
        let after = upsert(before, "howmany.binaryPath", "\"/new/howmany\"");

        assert!(after.contains("// my favourite setting"));
        assert!(after.contains("/* block */"));
        assert!(after.contains("\"editor.fontSize\": 14"));
        assert!(after.contains("\"howmany.binaryPath\": \"/new/howmany\""));
        assert!(!after.contains("/old/howmany"));
    }

    #[test]
    fn a_new_key_is_added_without_breaking_the_document() {
        let after = upsert("{\n  \"a\": 1\n}", "howmany.binaryPath", "\"/bin/howmany\"");
        let parsed: serde_json::Value = serde_json::from_str(&after).expect("valid JSON");
        assert_eq!(parsed["a"], 1);
        assert_eq!(parsed["howmany.binaryPath"], "/bin/howmany");
    }

    #[test]
    fn an_empty_object_gains_its_first_key() {
        let after = upsert("{}", "k", "true");
        let parsed: serde_json::Value = serde_json::from_str(&after).unwrap();
        assert_eq!(parsed["k"], true);
    }

    #[test]
    fn an_empty_file_becomes_a_document() {
        let after = upsert("", "k", "1");
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&after).unwrap()["k"],
            1
        );
    }

    /// A brace inside a nested value must not be mistaken for the end of it.
    #[test]
    fn nested_values_are_replaced_whole() {
        let before = r#"{"a": {"deep": [1, 2, {"x": "}"}]}, "b": 2}"#;
        let after = upsert(before, "a", "false");
        let parsed: serde_json::Value = serde_json::from_str(&after).unwrap();
        assert_eq!(parsed["a"], false);
        assert_eq!(parsed["b"], 2);
    }

    /// A brace or a comment marker inside a string is data, not syntax.
    #[test]
    fn punctuation_inside_strings_is_not_syntax() {
        let before = r#"{"path": "C:\\a//b, {x}", "k": 1}"#;
        let after = upsert(before, "k", "2");
        let parsed: serde_json::Value = serde_json::from_str(&after).unwrap();
        assert_eq!(parsed["path"], "C:\\a//b, {x}");
        assert_eq!(parsed["k"], 2);
    }

    #[test]
    fn a_trailing_comma_does_not_confuse_the_scan() {
        let before = "{\n  \"a\": 1,\n}";
        assert_eq!(get(before, "a"), Some("1"));
        assert!(upsert(before, "a", "9").contains("\"a\": 9"));
    }

    #[test]
    fn reading_back_a_written_value_gives_the_same_text() {
        let written = upsert("{}", "howmany.binaryPath", "\"/usr/local/bin/howmany\"");
        assert_eq!(
            get(&written, "howmany.binaryPath"),
            Some("\"/usr/local/bin/howmany\"")
        );
    }

    #[test]
    fn a_missing_key_reads_as_absent() {
        assert_eq!(get(r#"{"a": 1}"#, "b"), None);
    }

    /// Writing the same value twice must not keep appending copies.
    #[test]
    fn upsert_is_idempotent() {
        let once = upsert("{\n  \"a\": 1\n}", "k", "\"v\"");
        assert_eq!(once, upsert(&once, "k", "\"v\""));
    }

    #[test]
    fn the_file_s_own_indentation_is_matched() {
        let before = "{\n    \"a\": 1\n}";
        assert!(
            upsert(before, "k", "2").contains("\n    \"k\": 2,"),
            "four-space file should get a four-space key"
        );
    }
}
