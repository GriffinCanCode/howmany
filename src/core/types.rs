use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Statistics for a single file
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct FileStats {
    pub total_lines: usize,
    pub code_lines: usize,
    pub comment_lines: usize,
    pub blank_lines: usize,
    pub file_size: u64,
    pub doc_lines: usize, // Documentation content
}

impl FileStats {
    /// True when the line categories partition `total_lines`.
    ///
    /// Every line is classified into exactly one category, so this holds for any
    /// file the counter produces. Ratio and quality metrics divide by
    /// `total_lines`, so a violation would silently corrupt every derived
    /// figure -- which is why it is asserted rather than assumed.
    pub fn is_consistent(&self) -> bool {
        self.code_lines + self.comment_lines + self.doc_lines + self.blank_lines == self.total_lines
    }

    /// Add another file's statistics into this one.
    pub fn add(&mut self, other: &FileStats) {
        self.total_lines += other.total_lines;
        self.code_lines += other.code_lines;
        self.comment_lines += other.comment_lines;
        self.blank_lines += other.blank_lines;
        self.file_size += other.file_size;
        self.doc_lines += other.doc_lines;
    }
}

/// Aggregated statistics for a project
///
/// The per-extension breakdown is a `BTreeMap` rather than a `HashMap` so that
/// serialised reports come out in a fixed order. With hash ordering, two
/// identical runs produced JSON that differed only in key order, which made
/// reports impossible to diff and defeated caching them in CI.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct CodeStats {
    pub total_files: usize,
    pub total_lines: usize,
    pub total_code_lines: usize,
    pub total_comment_lines: usize,
    pub total_blank_lines: usize,
    pub total_size: u64,
    pub total_doc_lines: usize, // Documentation content
    pub stats_by_extension: BTreeMap<String, (usize, FileStats)>, // (file_count, aggregated_stats)
}

impl CodeStats {
    /// True when project totals partition consistently and agree with the
    /// per-extension breakdown.
    ///
    /// Aggregation is a plain sum, so the totals must equal the sum of the
    /// buckets no matter how many threads produced them. This is the property
    /// that makes parallel counting safe to trust.
    pub fn is_consistent(&self) -> bool {
        if self.total_code_lines
            + self.total_comment_lines
            + self.total_doc_lines
            + self.total_blank_lines
            != self.total_lines
        {
            return false;
        }

        let mut files = 0;
        let mut rolled = FileStats::default();
        for (count, stats) in self.stats_by_extension.values() {
            files += count;
            rolled.add(stats);
        }

        files == self.total_files
            && rolled.total_lines == self.total_lines
            && rolled.code_lines == self.total_code_lines
            && rolled.comment_lines == self.total_comment_lines
            && rolled.blank_lines == self.total_blank_lines
            && rolled.doc_lines == self.total_doc_lines
            && rolled.file_size == self.total_size
    }
}
