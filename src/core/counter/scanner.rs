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
#[cfg(test)]
fn contains_sub(haystack: &[u8], needle: &[u8]) -> bool {
    find_sub(haystack, needle).is_some()
}

fn starts_with_any(line: &[u8], prefixes: &[&str]) -> bool {
    prefixes
        .iter()
        .any(|prefix| line.starts_with(prefix.as_bytes()))
}

/// A block-comment opener with its matching terminator, prepared once per file.
struct BlockOpener {
    start: &'static str,
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
                    start: opener,
                    opener_len: opener.len(),
                    end,
                    symmetric: end == *opener,
                }
            })
            .collect()
    }
}

/// Literal rules that cannot be expressed by comment delimiters alone.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Dialect {
    Ordinary,
    Python,
    Rust,
    Go,
    Sql,
}

impl Dialect {
    fn for_pattern(pattern: CommentPattern) -> Self {
        if pattern.multi_line_start.contains(&"\"\"\"") {
            Self::Python
        } else if pattern.doc_patterns.contains(&"//!")
            && pattern.doc_patterns.contains(&"///")
            && pattern.multi_line_start.contains(&"/*")
        {
            Self::Rust
        } else if pattern.doc_patterns == ["//"] {
            Self::Go
        } else if pattern.single_line == ["--"] && pattern.multi_line_start == ["/*"] {
            Self::Sql
        } else {
            Self::Ordinary
        }
    }
}

enum Literal {
    Quoted {
        quote: u8,
        triple: bool,
        escapes: bool,
        documentation: bool,
    },
    Raw(Vec<u8>),
}

impl Literal {
    fn documentation(&self) -> bool {
        matches!(
            self,
            Self::Quoted {
                documentation: true,
                ..
            }
        )
    }
}

/// Python documentation is the first string expression of a module, class or
/// function suite. Assigned strings and later expression statements are code.
struct PythonContext {
    suites: Vec<(usize, bool)>,
    pending_suite: Option<bool>,
    logical_start: bool,
    suite_documentation: bool,
    brackets: usize,
    last_token: Option<u8>,
}

impl PythonContext {
    fn new() -> Self {
        Self {
            suites: vec![(0, true)],
            pending_suite: None,
            logical_start: true,
            suite_documentation: false,
            brackets: 0,
            last_token: None,
        }
    }

    fn begin(&mut self, raw: &[u8], line: &[u8]) -> bool {
        if self.logical_start {
            let indentation = raw
                .iter()
                .take_while(|byte| matches!(byte, b' ' | b'\t'))
                .fold(0, |column, byte| {
                    if *byte == b'\t' {
                        (column / 8 + 1) * 8
                    } else {
                        column + 1
                    }
                });
            while self.suites.len() > 1
                && self
                    .suites
                    .last()
                    .is_some_and(|suite| suite.0 > indentation)
            {
                self.suites.pop();
            }
            if let Some(documentation) = self.pending_suite.take() {
                if self
                    .suites
                    .last()
                    .is_some_and(|suite| suite.0 < indentation)
                {
                    self.suites.push((indentation, documentation));
                }
            }
            self.suite_documentation = line.starts_with(b"def ")
                || line.starts_with(b"async def ")
                || line.starts_with(b"class ");
        }
        self.last_token = None;
        let allowed = self.logical_start && self.suites.last().is_some_and(|suite| suite.1);
        if let Some(suite) = self.suites.last_mut() {
            suite.1 = false;
        }
        allowed
    }

    fn token(&mut self, byte: u8) {
        match byte {
            b'(' | b'[' | b'{' => self.brackets += 1,
            b')' | b']' | b'}' => self.brackets = self.brackets.saturating_sub(1),
            _ => {}
        }
        self.last_token = Some(byte);
    }

    fn finish(&mut self, inside_literal: bool) {
        self.logical_start =
            self.brackets == 0 && self.last_token != Some(b'\\') && !inside_literal;
        if self.logical_start && self.last_token == Some(b':') {
            self.pending_suite = Some(self.suite_documentation);
        }
    }
}

/// A Python literal's prefix and opening quote. Prefix recognition is lexical:
/// an identifier containing `r` or `f` must not be mistaken for a string prefix.
fn python_quote(line: &[u8], at: usize) -> Option<(usize, bool, bool)> {
    let mut quote = at;
    while line
        .get(quote)
        .is_some_and(|byte| matches!(byte, b'r' | b'R' | b'u' | b'U' | b'f' | b'F' | b'b' | b'B'))
    {
        quote += 1;
    }
    if !matches!(line.get(quote), Some(b'\'' | b'"')) || quote - at > 2 {
        return None;
    }
    let prefix = &line[at..quote];
    Some((
        quote,
        // Raw Python literals retain backslashes, but an escaped quote still
        // does not terminate the literal.
        true,
        !prefix
            .iter()
            .any(|byte| matches!(byte, b'f' | b'F' | b'b' | b'B')),
    ))
}

/// Rust's raw string terminator contains exactly as many hashes as its opener.
fn rust_raw_quote(line: &[u8], at: usize) -> Option<Vec<u8>> {
    let hashes = line[..at]
        .iter()
        .rev()
        .take_while(|byte| **byte == b'#')
        .count();
    if hashes > 255 || at <= hashes || line[at - hashes - 1] != b'r' {
        return None;
    }
    let mut end = Vec::with_capacity(hashes + 1);
    end.push(b'"');
    end.resize(hashes + 1, b'#');
    Some(end)
}

/// PostgreSQL dollar quoting protects its entire payload, including comment
/// markers, until the same case-sensitive tag closes it.
fn sql_dollar(line: &[u8], at: usize) -> Option<Vec<u8>> {
    let rest = line.get(at..)?.strip_prefix(b"$")?;
    let length = rest.iter().position(|byte| *byte == b'$')?;
    let tag = &rest[..length];
    if !tag.is_empty()
        && (!tag[0].is_ascii_alphabetic() && tag[0] != b'_'
            || !tag
                .iter()
                .all(|byte| byte.is_ascii_alphanumeric() || *byte == b'_'))
    {
        return None;
    }
    Some(line[at..at + length + 2].to_vec())
}

fn rust_character(line: &[u8]) -> bool {
    let Some(&first) = line.get(1) else {
        return false;
    };
    let width = if first == b'\\' {
        match line.get(2) {
            Some(b'x') => 4,
            Some(b'u') if line.get(3) == Some(&b'{') => {
                let Some(end) = line.iter().position(|byte| *byte == b'}') else {
                    return false;
                };
                end
            }
            Some(_) => 2,
            None => return false,
        }
    } else if first.is_ascii() {
        1
    } else if first & 0xe0 == 0xc0 {
        2
    } else if first & 0xf0 == 0xe0 {
        3
    } else if first & 0xf8 == 0xf0 {
        4
    } else {
        return false;
    };
    line.get(width + 1) == Some(&b'\'')
}

fn cpp_raw(line: &[u8], at: usize) -> Option<(usize, Vec<u8>)> {
    let rest = line.get(at..)?.strip_prefix(b"R\"")?;
    let delimiter = rest.iter().position(|byte| *byte == b'(')?;
    if delimiter > 16
        || rest[..delimiter]
            .iter()
            .any(|byte| byte.is_ascii_whitespace() || matches!(byte, b')' | b'\\'))
    {
        return None;
    }
    let mut end = Vec::with_capacity(delimiter + 2);
    end.push(b')');
    end.extend_from_slice(&rest[..delimiter]);
    end.push(b'"');
    Some((delimiter + 3, end))
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
    dialect: Dialect,
    python: Option<PythonContext>,
    literal: Option<Literal>,
    block_depth: usize,
    block_start: &'static str,
    comment_initials: Vec<u8>,
    in_doc_block: bool,
    block_end: &'static str,
    tally: LineTally,
}

impl Classifier {
    fn new(pattern: CommentPattern) -> Self {
        let mut comment_initials = Vec::new();
        for marker in pattern.single_line.iter().chain(pattern.multi_line_start) {
            if let Some(&byte) = marker.as_bytes().first() {
                if !comment_initials.contains(&byte) {
                    comment_initials.push(byte);
                }
            }
        }
        if Dialect::for_pattern(pattern) == Dialect::Sql {
            comment_initials.push(b'$');
        }
        Self {
            pattern,
            openers: BlockOpener::prepare(pattern),
            doc_marker_is_ambiguous: pattern
                .doc_patterns
                .iter()
                .any(|doc| pattern.single_line.contains(doc)),
            dialect: Dialect::for_pattern(pattern),
            python: (Dialect::for_pattern(pattern) == Dialect::Python).then(PythonContext::new),
            literal: None,
            block_depth: 0,
            block_start: "",
            comment_initials,
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
        if self.pattern == CommentPattern::EMPTY {
            self.tally.code_lines += 1;
            return;
        }

        // A line comment consumes the rest of the line. A quoted `/*` inside
        // it cannot open a block that changes the following lines.
        if self.block_depth == 0
            && self.literal.is_none()
            && starts_with_any(line, self.pattern.single_line)
            && !self
                .openers
                .iter()
                .any(|opener| line.starts_with(opener.start.as_bytes()))
        {
            let is_doc = starts_with_any(line, self.pattern.doc_patterns)
                && !(self.doc_marker_is_ambiguous && indented);
            self.count_comment(is_doc);
            return;
        }

        let documentation_allowed = if self.literal.is_none() && self.block_depth == 0 {
            self.python
                .as_mut()
                .is_some_and(|python| python.begin(raw, line))
        } else {
            false
        };
        let mut code = false;
        let mut documentation = false;
        let mut comment = false;
        let mut at = 0;

        if let Some((quote_at, escapes, true)) = documentation_allowed
            .then(|| python_quote(line, 0))
            .flatten()
        {
            let quote = line[quote_at];
            let triple = line.get(quote_at..quote_at + 3) == Some(&[quote; 3]);
            documentation = true;
            at = quote_at + if triple { 3 } else { 1 };
            self.literal = Some(Literal::Quoted {
                quote,
                triple,
                escapes,
                documentation: true,
            });
            if let Some(python) = self.python.as_mut() {
                python.last_token = Some(quote);
            }
        }

        while at < line.len() {
            if self.block_depth > 0 {
                documentation |= self.in_doc_block;
                comment |= !self.in_doc_block;
                let rest = &line[at..];
                let close = find_sub(rest, self.block_end.as_bytes());
                let nested = (matches!(self.dialect, Dialect::Rust | Dialect::Sql)
                    || matches!(self.block_start, "#|" | "{-" | "(*"))
                .then(|| find_sub(rest, self.block_start.as_bytes()))
                .flatten();
                if let Some(open) = nested.filter(|open| close.is_none_or(|close| *open < close)) {
                    self.block_depth += 1;
                    at += open + self.block_start.len();
                } else if let Some(close) = close {
                    self.block_depth -= 1;
                    at += close + self.block_end.len();
                } else {
                    break;
                }
                continue;
            }

            if let Some(literal) = self.literal.as_ref() {
                documentation |= literal.documentation();
                code |= !literal.documentation();
                match literal {
                    Literal::Raw(end) => {
                        if let Some(close) = find_sub(&line[at..], end) {
                            at += close + end.len();
                            self.literal = None;
                        } else {
                            break;
                        }
                    }
                    Literal::Quoted {
                        quote,
                        triple,
                        escapes,
                        ..
                    } => {
                        if *escapes && line[at] == b'\\' {
                            at = (at + 2).min(line.len());
                        } else if line[at] == *quote
                            && (!triple || line.get(at..at + 3) == Some(&[*quote; 3]))
                        {
                            let width = if *triple { 3 } else { 1 };
                            // SQL doubles quote characters inside string and
                            // identifier literals instead of backslash-escaping.
                            if self.dialect == Dialect::Sql && line.get(at + 1) == Some(quote) {
                                at += 2;
                            } else {
                                at += width;
                                self.literal = None;
                            }
                        } else {
                            at += 1;
                        }
                    }
                }
                continue;
            }

            if line[at].is_ascii_whitespace() {
                at += 1;
                continue;
            }

            // Jump over ordinary tokens instead of running the complete lexer
            // dispatch once per identifier byte.
            let rest = &line[at..];
            let mut next = memchr::memchr3(b'\'', b'"', b'`', rest).unwrap_or(rest.len());
            for &initial in &self.comment_initials {
                if let Some(candidate) = memchr::memchr(initial, &rest[..next]) {
                    next = candidate;
                }
            }
            if next > 0 {
                let tokens = &rest[..next];
                code = true;
                if let Some(python) = self.python.as_mut() {
                    for &byte in tokens {
                        if !byte.is_ascii_whitespace() {
                            python.token(byte);
                        }
                    }
                }
                at += next;
                continue;
            }

            if let Some(opener) = self
                .openers
                .iter()
                .find(|opener| !opener.symmetric && line[at..].starts_with(opener.start.as_bytes()))
            {
                self.block_depth = 1;
                self.block_start = opener.start;
                self.block_end = opener.end;
                self.in_doc_block = starts_with_any(&line[at..], self.pattern.doc_patterns);
                documentation |= self.in_doc_block;
                comment |= !self.in_doc_block;
                at += opener.opener_len;
                continue;
            }

            if starts_with_any(&line[at..], self.pattern.single_line) {
                documentation |= starts_with_any(&line[at..], self.pattern.doc_patterns)
                    && !(self.doc_marker_is_ambiguous && indented);
                comment = true;
                break;
            }

            if self.dialect == Dialect::Sql
                && line[at] == b'$'
                && (at == 0
                    || !line[at - 1].is_ascii_alphanumeric()
                        && !matches!(line[at - 1], b'_' | b'$'))
            {
                if let Some(end) = sql_dollar(line, at) {
                    code = true;
                    at += end.len();
                    self.literal = Some(Literal::Raw(end));
                    continue;
                }
            }

            let quote_at = at;
            let quote = line[at];
            if matches!(quote, b'\'' | b'"' | b'`') {
                if quote == b'"' && self.dialect == Dialect::Rust {
                    if let Some(end) = rust_raw_quote(line, at) {
                        code = true;
                        at += 1;
                        self.literal = Some(Literal::Raw(end));
                        continue;
                    }
                }
                if quote == b'"' && self.dialect == Dialect::Ordinary && at > 0 {
                    if let Some((width, end)) = cpp_raw(line, at - 1) {
                        code = true;
                        at += width - 1;
                        self.literal = Some(Literal::Raw(end));
                        continue;
                    }
                }
                // A Rust lifetime has no closing quote; consuming it as a
                // character literal would hide the rest of the declaration.
                if self.dialect == Dialect::Rust
                    && quote == b'\''
                    && !rust_character(&line[quote_at..])
                {
                    code = true;
                    at += 1;
                    continue;
                }
                let triple = quote != b'`' && line.get(quote_at..quote_at + 3) == Some(&[quote; 3]);
                code = true;
                if let Some(python) = self.python.as_mut() {
                    python.last_token = Some(quote);
                }
                at = quote_at + if triple { 3 } else { 1 };
                if quote == b'`' && self.dialect == Dialect::Go {
                    self.literal = Some(Literal::Raw(vec![quote]));
                } else {
                    self.literal = Some(Literal::Quoted {
                        quote,
                        triple,
                        escapes: self.dialect != Dialect::Sql,
                        documentation: false,
                    });
                }
                continue;
            }

            code = true;
            if let Some(python) = self.python.as_mut() {
                python.token(line[at]);
            }
            at += 1;
        }

        if let Some(python) = self.python.as_mut() {
            python.finish(self.literal.is_some());
        }
        if code {
            self.tally.code_lines += 1;
        } else if documentation {
            self.tally.doc_lines += 1;
        } else if comment {
            self.tally.comment_lines += 1;
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
    fence: Option<Fence>,
    in_html_comment: bool,
    tally: LineTally,
}

#[derive(Clone, Copy)]
struct Fence {
    marker: u8,
    length: usize,
}

impl Fence {
    fn opening(line: &[u8]) -> Option<Self> {
        let &marker = line.first()?;
        if !matches!(marker, b'`' | b'~') {
            return None;
        }
        let length = line.iter().take_while(|byte| **byte == marker).count();
        (length >= 3 && (marker != b'`' || !line[length..].contains(&b'`')))
            .then_some(Self { marker, length })
    }

    fn closes(self, line: &[u8]) -> bool {
        if line.iter().take_while(|byte| **byte == b' ').count() > 3 || line.first() == Some(&b'\t')
        {
            return false;
        }
        let line = trim_ascii(line);
        let length = line.iter().take_while(|byte| **byte == self.marker).count();
        length >= self.length && line[length..].iter().all(u8::is_ascii_whitespace)
    }
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

        // Examples remain code when their payload spells an HTML comment or
        // a different fence delimiter.
        if let Some(fence) = self.fence {
            self.tally.code_lines += 1;
            if fence.closes(raw) {
                self.fence = None;
            }
            return;
        }
        if !self.in_html_comment && (raw.starts_with(b"    ") || raw.starts_with(b"\t")) {
            self.tally.code_lines += 1;
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

        if let Some(fence) = Fence::opening(line) {
            self.fence = Some(fence);
            self.tally.code_lines += 1;
            return;
        }

        self.tally.doc_lines += 1;
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

    #[test]
    fn quoted_json_globs_and_urls_cannot_open_comments() {
        let source = concat!(
            "{\n",
            "  \"workspaces\": [\"clients/**/node_modules\"],\n",
            "  \"url\": \"https://example.test/*\",\n",
            "  \"escaped\": \"\\\"/* still a string\",\n",
            "  \"enabled\": true\n",
            "}\n",
        );
        let tally = scan(source, "json");
        assert_eq!(tally.total_lines, 6);
        assert_eq!(tally.code_lines, 6);
        assert_eq!(tally.comment_lines + tally.doc_lines, 0);
    }

    #[test]
    fn specialist_language_comments_use_their_actual_delimiters() {
        for (extension, source) in [
            ("rego", "# policy comment\nallow if { true }\n"),
            (
                "cedar",
                "// policy comment\npermit(principal, action, resource);\n",
            ),
            ("metal", "/* kernel comment */\nkernel void compute() {}\n"),
            ("scm", "; Scheme comment\n(display \"hello\")\n"),
            ("j2", "{# template comment #}\n{{ value }}\n"),
            ("astro", "<!-- component comment -->\n<Component />\n"),
        ] {
            let tally = scan(source, extension);
            assert_eq!(tally.code_lines, 1, "{extension}: {tally:?}");
            assert_eq!(tally.comment_lines, 1, "{extension}: {tally:?}");
        }
    }

    #[test]
    fn cpp_raw_delimiters_and_escaped_javascript_backticks_protect_payloads() {
        let cpp = scan(
            "auto value = R\"tag(start\n/* literal \" quote\n)tag\";\n// actual\n",
            "cpp",
        );
        assert_eq!(cpp.code_lines, 3);
        assert_eq!(cpp.comment_lines, 1);
        let js = scan(
            "const value = `escaped \\`\n/* literal\n`;\n// actual\n",
            "js",
        );
        assert_eq!(js.code_lines, 3);
        assert_eq!(js.comment_lines, 1);
    }

    #[test]
    fn executable_tokens_win_over_comments_on_the_same_line() {
        let tally = scan(
            "int a = 1; /* trailing */\n/* prefix */ int b = 2;\n/* open\nclose */ int c = 3;\n// /* not a block\nint d = 4;\n",
            "c",
        );
        assert_eq!(tally.total_lines, 6);
        assert_eq!(tally.code_lines, 4);
        assert_eq!(tally.comment_lines, 2);
    }

    #[test]
    fn doc_markers_inside_an_ordinary_comment_do_not_change_its_kind() {
        let tally = scan(
            "/* A description of /** syntax. */\n// The /// marker is documented elsewhere.\n",
            "c",
        );
        assert_eq!(tally.doc_lines, 0);
        assert_eq!(tally.comment_lines, 2);
    }

    #[test]
    fn rust_nested_comments_close_only_after_the_outer_comment() {
        let tally = scan(
            "/* outer\n/* inner */\nstill outer\n*/\nfn main() {}\n",
            "rs",
        );
        assert_eq!(tally.comment_lines, 4);
        assert_eq!(tally.code_lines, 1);
    }

    #[test]
    fn rust_raw_strings_and_lifetimes_are_lexically_distinct() {
        let tally = scan(
            "fn borrow<'a>(value: &'a str) {}\nlet payload = br##\"start\n// payload\n/* payload\n\"# remains payload\n\"##;\n// actual comment\n",
            "rs",
        );
        assert_eq!(tally.total_lines, 7);
        assert_eq!(tally.code_lines, 6);
        assert_eq!(tally.comment_lines, 1);
    }

    #[test]
    fn assigned_python_string_payloads_keep_their_lexical_state() {
        let tally = scan("prompt = \"\"\"\n# payload\n'''other delimiter\n\"\"\"\n# actual comment\nprint(prompt)\n", "py");
        assert_eq!(tally.code_lines, 5);
        assert_eq!(tally.comment_lines, 1);
        assert_eq!(tally.doc_lines, 0);
    }

    #[test]
    fn python_docstrings_require_the_first_statement_of_a_documentable_suite() {
        let tally = scan(
            "\"\"\"Module.\"\"\"\nvalue = \"\"\"payload\n# literal\n\"\"\"\ndef f(\n    value: str,\n):\n    r\"\"\"Function doc.\"\"\"\n    value = 1\n    \"\"\"Later expression.\"\"\"\n    return value\n# comment\n",
            "py",
        );
        assert_eq!(tally.total_lines, 12);
        assert_eq!(tally.doc_lines, 2);
        assert_eq!(tally.comment_lines, 1);
        assert_eq!(tally.code_lines, 9);
        let conditional = scan(
            "if True:\n    \"\"\"Expression, not a docstring.\"\"\"\n",
            "py",
        );
        assert_eq!(conditional.code_lines, 2);
        assert_eq!(conditional.doc_lines, 0);
    }

    #[test]
    fn sql_comments_and_dollar_quoted_payloads_are_separate() {
        let tally = scan(
            "-- ordinary SQL comment\nSELECT '/*', '--', 'it''s quoted';\nDO $body$\n-- literal body\n/* literal body\n$body$;\n/* outer /* nested */ still outer */ SELECT 1;\n",
            "sql",
        );
        assert_eq!(tally.total_lines, 7);
        assert_eq!(tally.code_lines, 6);
        assert_eq!(tally.comment_lines, 1);
        assert_eq!(tally.doc_lines, 0);
        let identifier = scan("SELECT value$tag$ FROM values;\n-- actual comment\n", "sql");
        assert_eq!(identifier.code_lines, 1);
        assert_eq!(identifier.comment_lines, 1);
    }

    #[test]
    fn go_raw_strings_and_javascript_templates_keep_comment_markers_literal() {
        let source = "value = `payload\n// literal\n/* literal\n`;\n// actual comment\n";
        for ext in ["go", "js"] {
            let tally = scan(source, ext);
            assert_eq!(tally.code_lines, 4, "{ext}: {tally:?}");
            assert_eq!(tally.comment_lines + tally.doc_lines, 1);
        }
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

    #[test]
    fn markdown_fenced_html_examples_are_code_until_the_matching_fence() {
        for (source, code_lines) in [
            ("````html\n<!-- literal -->\n```\n~~~\n````\nProse.\n", 5),
            ("~~~\n~~~ still code\n<!-- literal -->\n~~~~\nProse.\n", 4),
            ("```\n    ```\n<!-- still literal -->\n```\nProse.\n", 4),
        ] {
            let streamed = classify_markdown(&mut source.as_bytes()).unwrap();
            let in_memory = classify_markdown_bytes(source.as_bytes());
            assert_eq!(streamed, in_memory);
            assert_eq!(streamed.code_lines, code_lines, "{source:?}: {streamed:?}");
            assert_eq!(streamed.doc_lines, 1);
            assert_eq!(streamed.comment_lines, 0);
            assert!(streamed.is_partitioned());
        }
    }

    #[test]
    fn markdown_indented_html_examples_are_code() {
        let source = "Prose.\n\n    <!-- literal -->\n";
        let tally = classify_markdown_bytes(source.as_bytes());
        assert_eq!(tally.code_lines, 1);
        assert_eq!(tally.doc_lines, 1);
        assert_eq!(tally.blank_lines, 1);
        assert_eq!(tally.comment_lines, 0);
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
