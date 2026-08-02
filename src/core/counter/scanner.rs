//! Line classification.
//!
//! Scanning works on bytes rather than `String`s for two reasons.
//!
//! *Throughput*: `BufRead::lines()` allocates and UTF-8 validates a fresh
//! `String` for every line in the repository. Reading into one reused byte
//! buffer removes both costs from the innermost loop.
//!
//! *Robustness*: `lines()` fails the moment a file is not valid UTF-8, and the
//! caller could only respond by dropping the file. Latin-1 sources, files with
//! a stray `0x80`, and anything saved in a legacy encoding were therefore
//! silently missing from the totals. Comment markers are all ASCII, so byte
//! comparison classifies such files correctly instead of discarding them.

use super::comment_patterns::CommentPattern;
use crate::core::types::FileStats;
use std::io::{self, BufRead};

/// Reused line buffers above this size are released rather than retained, so a
/// single minified or generated line cannot pin megabytes for a whole run.
const MAX_RETAINED_LINE_CAPACITY: usize = 1 << 20;

/// How the lines of one file were classified.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct LineTally {
    pub total_lines: usize,
    pub code_lines: usize,
    pub comment_lines: usize,
    pub doc_lines: usize,
    pub blank_lines: usize,
}

impl LineTally {
    /// Attach a file size to produce the public statistics type.
    pub fn into_file_stats(self, file_size: u64) -> FileStats {
        FileStats {
            total_lines: self.total_lines,
            code_lines: self.code_lines,
            comment_lines: self.comment_lines,
            blank_lines: self.blank_lines,
            file_size,
            doc_lines: self.doc_lines,
        }
    }

    /// True when the categories add up to the total.
    ///
    /// Every line takes exactly one branch of the classifier, so this must hold
    /// for any input; it is asserted directly in the tests and relied upon by
    /// the aggregate invariant checks.
    pub fn is_partitioned(&self) -> bool {
        self.code_lines + self.comment_lines + self.doc_lines + self.blank_lines == self.total_lines
    }
}

/// Strip leading and trailing ASCII whitespace, including the line terminator.
fn trim_ascii(mut bytes: &[u8]) -> &[u8] {
    while let [first, rest @ ..] = bytes {
        if first.is_ascii_whitespace() {
            bytes = rest;
        } else {
            break;
        }
    }
    while let [rest @ .., last] = bytes {
        if last.is_ascii_whitespace() {
            bytes = rest;
        } else {
            break;
        }
    }
    bytes
}

/// Editors and Windows tooling prefix files with a UTF-8 byte-order mark.
const BOM: &[u8] = b"\xef\xbb\xbf";

/// Strip a leading byte-order mark so the first line classifies like any other.
///
/// Without this the mark sits in front of the comment marker, nothing matches,
/// and the first line of every BOM-prefixed file is counted as code.
fn strip_bom(bytes: &[u8]) -> &[u8] {
    bytes.strip_prefix(BOM).unwrap_or(bytes)
}

/// True when `line` is the interpreter directive of a script.
///
/// A shebang is executable configuration -- the kernel acts on it -- so it is
/// counted as code even though it opens with a comment marker in every language
/// that uses one.
fn is_shebang(line: &[u8]) -> bool {
    line.starts_with(b"#!")
}

/// Offset of the first occurrence of `needle` in `haystack`.
fn find_sub(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    memchr::memmem::find(haystack, needle)
}

/// True when `needle` occurs anywhere in `haystack`.
fn contains_sub(haystack: &[u8], needle: &[u8]) -> bool {
    find_sub(haystack, needle).is_some()
}

fn starts_with_any(line: &[u8], prefixes: &[&str]) -> bool {
    prefixes
        .iter()
        .any(|prefix| line.starts_with(prefix.as_bytes()))
}

fn contains_any(line: &[u8], needles: &[&str]) -> bool {
    needles
        .iter()
        .any(|needle| contains_sub(line, needle.as_bytes()))
}

/// A block-comment opener with its matching terminator, prepared once per file.
///
/// Searching for openers used to rebuild the substring search for every line of
/// every file. A [`Finder`](memchr::memmem::Finder) analyses its needle once and
/// then scans with SIMD, which matters because this search runs over every line
/// of source in the repository -- including the majority that contain no comment
/// at all and must be rejected as cheaply as possible.
struct BlockOpener {
    finder: memchr::memmem::Finder<'static>,
    opener_len: usize,
    end: &'static str,
    /// True when opener and terminator are the same token, which makes it a
    /// string delimiter as well and so only counts at the start of a line.
    symmetric: bool,
}

impl BlockOpener {
    fn prepare(pattern: CommentPattern) -> Vec<Self> {
        pattern
            .multi_line_start
            .iter()
            .enumerate()
            .map(|(index, opener)| {
                let end = pattern.multi_line_end.get(index).copied().unwrap_or(opener);
                Self {
                    finder: memchr::memmem::Finder::new(opener.as_bytes()),
                    opener_len: opener.len(),
                    end,
                    symmetric: end == *opener,
                }
            })
            .collect()
    }
}

/// Read one line into `buf`, returning `false` at end of input.
///
/// `buf` is cleared first and reused across calls; oversized buffers are
/// released so peak memory tracks the typical line, not the worst one.
fn next_line<R: BufRead>(reader: &mut R, buf: &mut Vec<u8>) -> io::Result<bool> {
    if buf.capacity() > MAX_RETAINED_LINE_CAPACITY {
        *buf = Vec::with_capacity(256);
    } else {
        buf.clear();
    }
    Ok(reader.read_until(b'\n', buf)? != 0)
}

/// Split `bytes` into lines, each keeping its terminator.
///
/// Matches `BufRead::read_until(b'\n')` exactly, including the case of a final
/// line with no newline, so the two drivers below classify identically.
fn split_lines(bytes: &[u8]) -> impl Iterator<Item = &[u8]> {
    let mut rest = bytes;
    std::iter::from_fn(move || {
        if rest.is_empty() {
            return None;
        }
        let line = match memchr::memchr(b'\n', rest) {
            Some(at) => {
                let (line, tail) = rest.split_at(at + 1);
                rest = tail;
                line
            }
            None => std::mem::take(&mut rest),
        };
        Some(line)
    })
}

/// The line classifier, as a state machine fed one raw line at a time.
///
/// Both entry points below drive this one implementation, so a file read whole
/// into memory and a file streamed through a reader cannot disagree about how
/// its lines are classified.
struct Classifier {
    pattern: CommentPattern,
    openers: Vec<BlockOpener>,
    /// Go documents declarations with the same `//` it uses for ordinary
    /// comments. Where a language does that, treating every comment as
    /// documentation would report zero comment lines for the whole project, so
    /// indentation decides: gofmt keeps declaration docs at column zero and
    /// indents everything inside a body.
    doc_marker_is_ambiguous: bool,
    in_block: bool,
    in_doc_block: bool,
    block_end: &'static str,
    tally: LineTally,
}

impl Classifier {
    fn new(pattern: CommentPattern) -> Self {
        Self {
            pattern,
            openers: BlockOpener::prepare(pattern),
            doc_marker_is_ambiguous: pattern
                .doc_patterns
                .iter()
                .any(|doc| pattern.single_line.contains(doc)),
            in_block: false,
            in_doc_block: false,
            block_end: "",
            tally: LineTally::default(),
        }
    }

    fn feed(&mut self, bytes: &[u8]) {
        self.tally.total_lines += 1;
        let first_line = self.tally.total_lines == 1;
        let raw: &[u8] = if first_line { strip_bom(bytes) } else { bytes };
        let indented = raw.first().is_some_and(u8::is_ascii_whitespace);
        let line = trim_ascii(raw);

        if line.is_empty() {
            self.tally.blank_lines += 1;
            return;
        }

        if first_line && is_shebang(line) {
            self.tally.code_lines += 1;
            return;
        }

        if self.in_block {
            let is_doc = self.in_doc_block;
            if contains_sub(line, self.block_end.as_bytes()) {
                self.in_block = false;
                self.in_doc_block = false;
            }
            self.count_comment(is_doc);
            return;
        }

        // The earliest opener on the line is the one that takes effect: with
        // `# /* x */` in a language that has both, the `#` came first.
        //
        // A symmetric delimiter is also a string delimiter in most languages:
        // `"""x"""` opens a Python docstring only when it begins the line.
        // `code = '''text` is an assignment, and its body is code.
        let opened = self
            .openers
            .iter()
            .filter_map(|opener| {
                let at = opener.finder.find(line)?;
                (!opener.symmetric || at == 0).then_some((opener, at))
            })
            .min_by_key(|&(_, at)| at);

        if let Some((opener, at)) = opened {
            let is_doc = contains_any(line, self.pattern.doc_patterns);

            // The terminator has to be searched *after* the opener. Python
            // docstrings and Ruby heredocs open and close with the same token,
            // so looking at the whole line would close every block on the line
            // that opened it.
            if !contains_sub(&line[at + opener.opener_len..], opener.end.as_bytes()) {
                self.in_block = true;
                self.in_doc_block = is_doc;
                self.block_end = opener.end;
            }

            self.count_comment(is_doc);
        } else if starts_with_any(line, self.pattern.single_line) {
            let is_doc = starts_with_any(line, self.pattern.doc_patterns)
                && !(self.doc_marker_is_ambiguous && indented);
            self.count_comment(is_doc);
        } else {
            self.tally.code_lines += 1;
        }
    }

    fn count_comment(&mut self, is_doc: bool) {
        if is_doc {
            self.tally.doc_lines += 1;
        } else {
            self.tally.comment_lines += 1;
        }
    }

    fn finish(self) -> LineTally {
        debug_assert!(self.tally.is_partitioned());
        self.tally
    }
}

/// Classify every line of `reader` using `pattern`.
pub fn classify<R: BufRead>(reader: &mut R, pattern: CommentPattern) -> io::Result<LineTally> {
    let mut classifier = Classifier::new(pattern);
    let mut buf = Vec::with_capacity(256);
    while next_line(reader, &mut buf)? {
        classifier.feed(&buf);
    }
    Ok(classifier.finish())
}

/// Classify a file already held in memory.
///
/// The allocation-free path. Streaming a file through a `BufReader` costs a
/// fresh 64 KiB buffer per file plus a copy of every line into a second
/// buffer; with the bytes already to hand, lines are borrowed from them
/// directly. That removed the allocator contention which made counting on all
/// sixteen threads of this machine slower than counting on four.
pub fn classify_bytes(bytes: &[u8], pattern: CommentPattern) -> LineTally {
    let mut classifier = Classifier::new(pattern);
    for line in split_lines(bytes) {
        classifier.feed(line);
    }
    classifier.finish()
}

/// Classify a Markdown file.
///
/// Prose is documentation, fenced or indented blocks are code, and HTML
/// comments are comments.
pub fn classify_markdown<R: BufRead>(reader: &mut R) -> io::Result<LineTally> {
    let mut classifier = MarkdownClassifier::default();
    let mut buf = Vec::with_capacity(256);
    while next_line(reader, &mut buf)? {
        classifier.feed(&buf);
    }
    Ok(classifier.finish())
}

/// Classify a Markdown file already held in memory.
pub fn classify_markdown_bytes(bytes: &[u8]) -> LineTally {
    let mut classifier = MarkdownClassifier::default();
    for line in split_lines(bytes) {
        classifier.feed(line);
    }
    classifier.finish()
}

/// The Markdown classifier, as a state machine fed one raw line at a time.
#[derive(Default)]
struct MarkdownClassifier {
    in_code_block: bool,
    in_html_comment: bool,
    tally: LineTally,
}

impl MarkdownClassifier {
    fn feed(&mut self, bytes: &[u8]) {
        self.tally.total_lines += 1;
        // Indented code blocks are significant, so measure indentation before
        // trimming and classify on the trimmed form.
        let bytes: &[u8] = if self.tally.total_lines == 1 {
            strip_bom(bytes)
        } else {
            bytes
        };
        let raw = trim_ascii_end(bytes);
        let line = trim_ascii(bytes);

        if line.is_empty() {
            self.tally.blank_lines += 1;
            return;
        }

        if line.starts_with(b"<!--") {
            self.in_html_comment = true;
        }

        if self.in_html_comment {
            self.tally.comment_lines += 1;
            if line.ends_with(b"-->") {
                self.in_html_comment = false;
            }
            return;
        }

        if line.starts_with(b"```") || line.starts_with(b"~~~") {
            self.in_code_block = !self.in_code_block;
            self.tally.code_lines += 1;
            return;
        }

        if self.in_code_block || raw.starts_with(b"    ") || raw.starts_with(b"\t") {
            self.tally.code_lines += 1;
        } else {
            self.tally.doc_lines += 1;
        }
    }

    fn finish(self) -> LineTally {
        debug_assert!(self.tally.is_partitioned());
        self.tally
    }
}

/// Strip only the trailing line terminator, preserving leading indentation.
fn trim_ascii_end(mut bytes: &[u8]) -> &[u8] {
    while let [rest @ .., last] = bytes {
        if *last == b'\n' || *last == b'\r' {
            bytes = rest;
        } else {
            break;
        }
    }
    bytes
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::counter::comment_patterns;

    fn scan(source: &str, ext: &str) -> LineTally {
        scan_bytes(source.as_bytes(), ext)
    }

    /// Classify through both drivers and assert they agree.
    ///
    /// There are two entry points -- one streaming a reader, one over bytes
    /// already in memory -- because the in-memory path avoids an allocation per
    /// file. They must be indistinguishable, so every test in this module
    /// exercises both rather than trusting whichever one it happened to call.
    fn scan_bytes(source: &[u8], ext: &str) -> LineTally {
        let pattern = comment_patterns::lookup_or_empty(ext);
        let streamed = classify(&mut &source[..], pattern).unwrap();
        let in_memory = classify_bytes(source, pattern);
        assert_eq!(
            streamed,
            in_memory,
            "the streaming and in-memory classifiers disagreed on {:?}",
            String::from_utf8_lossy(source)
        );
        streamed
    }

    #[test]
    fn trim_ascii_strips_both_ends_and_terminators() {
        assert_eq!(trim_ascii(b"  hi \t\r\n"), b"hi");
        assert_eq!(trim_ascii(b"\r\n"), b"");
        assert_eq!(trim_ascii(b""), b"");
        assert_eq!(trim_ascii(b"x"), b"x");
    }

    #[test]
    fn contains_sub_matches_std_semantics() {
        let cases: &[(&str, &str)] = &[
            ("hello world", "world"),
            ("hello world", "hello"),
            ("hello", "hello"),
            ("hello", "helloo"),
            ("aaa", "aa"),
            ("abc", "d"),
            ("", "a"),
            ("abc", ""),
            ("/* doc */", "*/"),
        ];
        for (hay, needle) in cases {
            assert_eq!(
                contains_sub(hay.as_bytes(), needle.as_bytes()),
                hay.contains(needle),
                "contains_sub disagreed with str::contains for {hay:?} / {needle:?}"
            );
        }
    }

    #[test]
    fn empty_input_yields_zero_lines() {
        assert_eq!(scan("", "rs"), LineTally::default());
    }

    /// Line accounting must match `BufRead::lines()` exactly for the trailing
    /// newline cases, which is where off-by-one errors hide.
    #[test]
    fn line_counts_match_bufread_lines() {
        for source in [
            "",
            "a",
            "a\n",
            "a\nb",
            "a\nb\n",
            "\n",
            "\n\n",
            "a\n\nb\n",
            "a\r\nb\r\n",
        ] {
            let expected = source.as_bytes().lines().count();
            let tally = scan(source, "rs");
            assert_eq!(
                tally.total_lines, expected,
                "total_lines disagreed with BufRead::lines for {source:?}"
            );
        }
    }

    #[test]
    fn blank_lines_are_counted_separately() {
        let tally = scan("\n\n\n\n", "rs");
        assert_eq!(tally.total_lines, 4);
        assert_eq!(tally.blank_lines, 4);
        assert_eq!(tally.code_lines, 0);
    }

    #[test]
    fn whitespace_only_lines_are_blank() {
        let tally = scan("   \n\t\n  \t  \n", "rs");
        assert_eq!(tally.blank_lines, 3);
        assert_eq!(tally.code_lines, 0);
    }

    #[test]
    fn rust_comments_docs_and_code_are_distinguished() {
        let tally = scan(
            "// plain comment\n/// doc comment\n//! inner doc\nfn main() {}\n\n",
            "rs",
        );
        assert_eq!(tally.total_lines, 5);
        assert_eq!(tally.comment_lines, 1);
        assert_eq!(tally.doc_lines, 2);
        assert_eq!(tally.code_lines, 1);
        assert_eq!(tally.blank_lines, 1);
    }

    #[test]
    fn block_comments_span_lines_and_terminate() {
        let tally = scan("/*\nstill comment\n*/\nfn main() {}\n", "c");
        assert_eq!(tally.comment_lines, 3);
        assert_eq!(tally.code_lines, 1);
    }

    /// An unterminated block comment must consume the remaining lines rather
    /// than resetting, and must never lose lines from the total.
    #[test]
    fn unterminated_block_comment_consumes_the_rest() {
        let tally = scan("/*\na\nb\nc\n", "c");
        assert_eq!(tally.total_lines, 4);
        assert_eq!(tally.comment_lines, 4);
        assert_eq!(tally.code_lines, 0);
        assert!(tally.is_partitioned());
    }

    #[test]
    fn crlf_is_handled_identically_to_lf() {
        let lf = scan("// c\nfn main() {}\n\n", "rs");
        let crlf = scan("// c\r\nfn main() {}\r\n\r\n", "rs");
        assert_eq!(lf, crlf, "CRLF input produced a different tally than LF");
    }

    /// Files that are not valid UTF-8 used to fail outright and vanish from the
    /// totals; they must now be counted.
    #[test]
    fn invalid_utf8_is_counted_not_rejected() {
        let source = b"fn main() {}\n\xff\xfe not utf8 \x80\n// comment\n";
        let tally = scan_bytes(source, "rs");
        assert_eq!(tally.total_lines, 3);
        assert_eq!(tally.comment_lines, 1);
        assert_eq!(tally.code_lines, 2);
        assert!(tally.is_partitioned());
    }

    #[test]
    fn a_line_with_no_terminator_still_counts() {
        let tally = scan("fn a() {}\nfn b() {}", "rs");
        assert_eq!(tally.total_lines, 2);
        assert_eq!(tally.code_lines, 2);
    }

    #[test]
    fn very_long_lines_are_classified_correctly() {
        let long = format!("// {}\n", "x".repeat(200_000));
        let tally = scan(&long, "rs");
        assert_eq!(tally.total_lines, 1);
        assert_eq!(tally.comment_lines, 1);
    }

    /// A single enormous line must not leave a large buffer retained for the
    /// rest of the file.
    #[test]
    fn oversized_line_buffer_is_released() {
        let mut buf = Vec::with_capacity(MAX_RETAINED_LINE_CAPACITY * 2);
        let mut reader = &b"short\n"[..];
        next_line(&mut reader, &mut buf).unwrap();
        assert!(
            buf.capacity() < MAX_RETAINED_LINE_CAPACITY,
            "oversized line buffer was retained"
        );
    }

    #[test]
    fn python_docstrings_count_as_documentation() {
        let tally = scan(
            "def f():\n    \"\"\"Doc line\n    more doc\n    \"\"\"\n    # comment\n    return 1\n",
            "py",
        );
        assert!(tally.doc_lines >= 3, "docstring lines: {tally:?}");
        assert_eq!(tally.comment_lines, 1);
        assert!(tally.code_lines >= 2);
    }

    /// Go's `//` serves as both comment and doc marker. Splitting on
    /// indentation follows gofmt: declaration docs sit at column zero, body
    /// comments are indented. Without this every Go project reported zero
    /// comment lines.
    #[test]
    fn go_doc_comments_are_distinguished_from_body_comments() {
        let tally = scan(
            "// Package p does things.\npackage p\n\nfunc f() {\n\t// step one\n\treturn\n}\n",
            "go",
        );
        assert_eq!(tally.doc_lines, 1, "{tally:?}");
        assert_eq!(tally.comment_lines, 1, "{tally:?}");
        assert_eq!(tally.code_lines, 4);
        assert!(tally.is_partitioned());
    }

    /// Languages with a distinct doc marker must not be affected by the Go
    /// rule: an indented `///` is still documentation.
    #[test]
    fn indented_rust_doc_comments_remain_documentation() {
        let tally = scan("impl T {\n    /// Doc.\n    fn f() {}\n}\n", "rs");
        assert_eq!(tally.doc_lines, 1, "{tally:?}");
        assert_eq!(tally.comment_lines, 0);
    }

    /// A triple-quoted block that opens and closes on one line is still a
    /// docstring; the closing token must be found after the opening one.
    #[test]
    fn single_line_docstring_closes_itself() {
        let tally = scan("def f():\n    \"\"\"One liner.\"\"\"\n    return 1\n", "py");
        assert_eq!(tally.doc_lines, 1);
        assert_eq!(tally.code_lines, 2);
    }

    /// The body of a multi-line string is code. Counting it as documentation
    /// would inflate the doc ratio of any file with an embedded SQL query or
    /// template.
    #[test]
    fn multiline_string_body_is_code_not_documentation() {
        let tally = scan(
            "query = '''SELECT 1\nFROM t\nWHERE x = 2'''\nrun(query)\n",
            "py",
        );
        assert_eq!(tally.doc_lines, 0, "{tally:?}");
        assert_eq!(tally.code_lines, 4);
    }

    /// An unterminated block comment must not swallow the rest of the file in
    /// the wrong category, and must still partition.
    #[test]
    fn unterminated_block_comment_stays_a_comment() {
        let tally = scan("code();\n/* opened\nstill inside\nnever closed\n", "rs");
        assert_eq!(tally.code_lines, 1);
        assert_eq!(tally.comment_lines, 3);
        assert!(tally.is_partitioned());
    }

    #[test]
    fn languages_without_comment_syntax_treat_everything_as_code() {
        let tally = scan("a\nb\n\n", "unknown-extension");
        assert_eq!(tally.code_lines, 2);
        assert_eq!(tally.blank_lines, 1);
        assert_eq!(tally.comment_lines, 0);
    }

    #[test]
    fn markdown_separates_prose_code_and_comments() {
        let source = "# Title\n\nProse here.\n\n```rust\nfn main() {}\n```\n\n<!-- note -->\n";
        let tally = classify_markdown(&mut source.as_bytes()).unwrap();
        assert_eq!(tally.total_lines, 9);
        assert_eq!(tally.blank_lines, 3);
        assert_eq!(tally.comment_lines, 1);
        assert_eq!(tally.code_lines, 3, "two fences plus one body line");
        assert_eq!(tally.doc_lines, 2);
        assert!(tally.is_partitioned());
    }

    #[test]
    fn markdown_indented_code_is_code() {
        let tally = classify_markdown(&mut "prose\n\n    indented code\n".as_bytes()).unwrap();
        assert_eq!(tally.doc_lines, 1);
        assert_eq!(tally.code_lines, 1);
        assert_eq!(tally.blank_lines, 1);
    }

    /// Whatever the input, the four categories must partition the total. This is
    /// the single invariant every downstream ratio depends on.
    #[test]
    fn categories_always_partition_the_total() {
        let sources: &[&str] = &[
            "",
            "\n",
            "// a\n/* b\nc */\nd\n",
            "/*\n*/\n/*\n*/\n",
            "\"\"\"\na\n\"\"\"\n",
            "#\n##\n###\n",
            "-- a\n{- b -}\n",
            "<!-- a -->\nb\n",
            "a\r\n\r\n\tb\n",
        ];
        for ext in ["rs", "py", "c", "hs", "html", "md", "sh", "unknown"] {
            for source in sources {
                let tally = scan(source, ext);
                assert!(
                    tally.is_partitioned(),
                    "categories did not partition total for ext={ext} source={source:?} -> {tally:?}"
                );
            }
        }
    }
}
