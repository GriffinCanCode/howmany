//! Turning a `file:` URI back into a path.
//!
//! `lsp-types` 0.97 carries a bare [`Uri`] with no filesystem conversion, and
//! the server needs one: an editor sends `file:///src/my%20app/main.rs` and the
//! classifier has to see `main.rs`, not `main.rs` with an escaped space
//! somewhere earlier in the path.

use lsp_types::Uri;
use std::path::PathBuf;

/// Decode `%XX` escapes, leaving anything malformed exactly as it was.
///
/// A stray `%` is a legal character in a file name on every platform we run on,
/// so it is passed through rather than treated as a truncated escape.
fn percent_decode(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut index = 0;

    while index < bytes.len() {
        let hex = (index + 2 < bytes.len() && bytes[index] == b'%')
            .then(|| std::str::from_utf8(&bytes[index + 1..index + 3]).ok())
            .flatten()
            .and_then(|pair| u8::from_str_radix(pair, 16).ok());

        match hex {
            Some(byte) => {
                out.push(byte);
                index += 3;
            }
            None => {
                out.push(bytes[index]);
                index += 1;
            }
        }
    }

    String::from_utf8_lossy(&out).into_owned()
}

/// The local path `uri` names, if it names one at all.
///
/// Anything that is not a `file:` URI -- an unsaved buffer under `untitled:`,
/// a remote document -- has no path, and the caller is expected to fall back to
/// the URI's own tail for classification.
pub fn to_path(uri: &Uri) -> Option<PathBuf> {
    if uri.scheme().map(|s| s.as_str()) != Some("file") {
        return None;
    }

    let raw = uri.path().as_estr().as_str();
    let decoded = percent_decode(raw);

    // `file:///C:/src/main.rs` arrives with a leading slash before the drive
    // letter, which Windows will not open.
    let trimmed = decoded
        .strip_prefix('/')
        .filter(|rest| {
            cfg!(windows)
                && rest.as_bytes().first().is_some_and(u8::is_ascii_alphabetic)
                && rest[1..].starts_with(':')
        })
        .unwrap_or(&decoded);

    Some(PathBuf::from(trimmed))
}

/// The name at the end of `uri`, whether or not it is a file on this machine.
///
/// The classifier keys off a file's name, and an unsaved buffer still has one.
pub fn file_name(uri: &Uri) -> String {
    percent_decode(uri.path().as_estr().as_str())
        .rsplit('/')
        .next()
        .unwrap_or_default()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    fn uri(text: &str) -> Uri {
        Uri::from_str(text).expect("test URIs must parse")
    }

    #[test]
    fn a_plain_path_survives_the_round_trip() {
        assert_eq!(
            to_path(&uri("file:///home/dev/src/main.rs")),
            Some(PathBuf::from("/home/dev/src/main.rs"))
        );
    }

    /// The reason this module exists: an escaped path must be decoded before
    /// anything tries to read it or read its extension.
    #[test]
    fn escapes_are_decoded() {
        assert_eq!(
            to_path(&uri("file:///home/dev/my%20app/main%2Ers")),
            Some(PathBuf::from("/home/dev/my app/main.rs"))
        );
    }

    #[test]
    fn a_non_file_uri_has_no_path_but_still_has_a_name() {
        let buffer = uri("untitled:Untitled-1");
        assert_eq!(to_path(&buffer), None);
        assert_eq!(file_name(&buffer), "Untitled-1");
    }

    #[test]
    fn the_name_is_taken_from_the_tail() {
        assert_eq!(file_name(&uri("file:///a/b/c/report.md")), "report.md");
    }

    /// A percent that is not an escape is an ordinary character, not a reason
    /// to mangle the rest of the name.
    #[test]
    fn a_stray_percent_is_left_alone() {
        assert_eq!(percent_decode("100%"), "100%");
        assert_eq!(percent_decode("a%zz b"), "a%zz b");
        assert_eq!(percent_decode("%2Fslash"), "/slash");
    }

    #[test]
    fn decoding_is_utf8_aware() {
        assert_eq!(percent_decode("caf%C3%A9.rs"), "café.rs");
    }
}
