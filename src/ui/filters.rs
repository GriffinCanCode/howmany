use crate::core::stats::aggregation::AggregatedStats;
use crate::core::stats::basic::ExtensionStats;
use crate::core::types::FileStats;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Filter options for CLI output
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FilterOptions {
    // Size filters
    pub min_lines: Option<usize>,
    pub max_lines: Option<usize>,
    pub min_size_bytes: Option<u64>,
    pub max_size_bytes: Option<u64>,

    // Complexity filters
    pub min_complexity: Option<f64>,
    pub max_complexity: Option<f64>,
    pub min_functions: Option<usize>,
    pub max_functions: Option<usize>,

    // Quality filters
    pub min_quality_score: Option<f64>,
    pub max_quality_score: Option<f64>,
    pub min_doc_ratio: Option<f64>,
    pub max_doc_ratio: Option<f64>,

    // Language/extension filters
    pub include_languages: Vec<String>,
    pub exclude_languages: Vec<String>,

    // Output customization
    pub show_complexity: bool,
    pub show_quality: bool,
    pub show_ratios: bool,
    pub show_size_info: bool,
    pub compact_output: bool,
}

/// One file's complexity, taken from the analysis the run already performed.
///
/// `--min-complexity` and `--min-functions` need this; nothing else does, so it
/// is derived from the function details rather than measured a second time.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct FileComplexity {
    pub function_count: usize,
    pub max_cyclomatic: f64,
    pub average_cyclomatic: f64,
}

impl FileComplexity {
    /// Group per-function details by the file they were found in.
    pub fn index(
        details: &[crate::core::stats::complexity::FunctionComplexityDetail],
    ) -> BTreeMap<String, FileComplexity> {
        let mut totals: BTreeMap<String, (usize, f64, f64)> = BTreeMap::new();
        for detail in details {
            let entry = totals
                .entry(detail.file_path.clone())
                .or_insert((0, 0.0, 0.0));
            let complexity = detail.cyclomatic_complexity as f64;
            entry.0 += 1;
            entry.1 += complexity;
            entry.2 = entry.2.max(complexity);
        }

        totals
            .into_iter()
            .map(|(path, (count, sum, max))| {
                (
                    path,
                    FileComplexity {
                        function_count: count,
                        max_cyclomatic: max,
                        average_cyclomatic: if count == 0 { 0.0 } else { sum / count as f64 },
                    },
                )
            })
            .collect()
    }
}

/// Filter for individual files
pub struct FileFilter {
    options: FilterOptions,
}

impl FileFilter {
    pub fn new(options: FilterOptions) -> Self {
        Self { options }
    }

    /// True when some option can only be decided per file.
    ///
    /// When nothing here is set, every file passes, and the aggregate totals the
    /// engine already computed are the answer -- retaining and re-summing one
    /// record per file would be pure overhead. On a 60k-file tree that is the
    /// difference between allocating 60k records and allocating none.
    pub fn needs_per_file_stats(options: &FilterOptions) -> bool {
        options.min_lines.is_some()
            || options.max_lines.is_some()
            || options.min_size_bytes.is_some()
            || options.max_size_bytes.is_some()
            || options.min_doc_ratio.is_some()
            || options.max_doc_ratio.is_some()
            || options.min_quality_score.is_some()
            || options.max_quality_score.is_some()
            || !options.include_languages.is_empty()
            || !options.exclude_languages.is_empty()
            || Self::needs_complexity(options)
    }

    /// True when a filter can only be decided from complexity analysis.
    ///
    /// Complexity costs a parse per file, so it is computed only when one of
    /// these options is actually set.
    pub fn needs_complexity(options: &FilterOptions) -> bool {
        options.min_complexity.is_some()
            || options.max_complexity.is_some()
            || options.min_functions.is_some()
            || options.max_functions.is_some()
    }

    /// True when `complexity` satisfies the complexity and function filters.
    ///
    /// A file with no analysable functions has no complexity to compare, so it
    /// fails any lower bound and passes any upper one.
    pub fn passes_complexity_filter(&self, complexity: Option<&FileComplexity>) -> bool {
        let measured = complexity.copied().unwrap_or_default();

        let within = |value: f64, min: Option<f64>, max: Option<f64>| {
            min.is_none_or(|m| value >= m) && max.is_none_or(|m| value <= m)
        };

        within(
            measured.max_cyclomatic,
            self.options.min_complexity,
            self.options.max_complexity,
        ) && self
            .options
            .min_functions
            .is_none_or(|m| measured.function_count >= m)
            && self
                .options
                .max_functions
                .is_none_or(|m| measured.function_count <= m)
    }

    pub fn passes_filter(&self, file_path: &str, file_stats: &FileStats) -> bool {
        // Size filters
        if let Some(min_lines) = self.options.min_lines {
            if file_stats.total_lines < min_lines {
                return false;
            }
        }

        if let Some(max_lines) = self.options.max_lines {
            if file_stats.total_lines > max_lines {
                return false;
            }
        }

        if let Some(min_size) = self.options.min_size_bytes {
            if file_stats.file_size < min_size {
                return false;
            }
        }

        if let Some(max_size) = self.options.max_size_bytes {
            if file_stats.file_size > max_size {
                return false;
            }
        }

        // Language/extension filters
        let extension = std::path::Path::new(file_path)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("no_ext")
            .to_lowercase();

        if !self.options.include_languages.is_empty()
            && !self
                .options
                .include_languages
                .iter()
                .any(|lang| lang.to_lowercase() == extension)
        {
            return false;
        }

        if !self.options.exclude_languages.is_empty()
            && self
                .options
                .exclude_languages
                .iter()
                .any(|lang| lang.to_lowercase() == extension)
        {
            return false;
        }

        // Documentation ratio filter
        if let Some(min_doc_ratio) = self.options.min_doc_ratio {
            let doc_ratio = if file_stats.total_lines > 0 {
                file_stats.doc_lines as f64 / file_stats.total_lines as f64
            } else {
                0.0
            };
            if doc_ratio < min_doc_ratio {
                return false;
            }
        }

        if let Some(max_doc_ratio) = self.options.max_doc_ratio {
            let doc_ratio = if file_stats.total_lines > 0 {
                file_stats.doc_lines as f64 / file_stats.total_lines as f64
            } else {
                0.0
            };
            if doc_ratio > max_doc_ratio {
                return false;
            }
        }

        // `--min-quality` / `--max-quality` used to be accepted and then
        // ignored. The score is the same one the project report shows, computed
        // for this file alone: pure arithmetic over line counts, no extra IO.
        if self.options.min_quality_score.is_some() || self.options.max_quality_score.is_some() {
            let score = Self::quality_score(file_stats);
            if self
                .options
                .min_quality_score
                .is_some_and(|min| score < min)
                || self
                    .options
                    .max_quality_score
                    .is_some_and(|max| score > max)
            {
                return false;
            }
        }

        true
    }

    /// The overall quality score for a single file, on the report's 0-100 scale.
    fn quality_score(file_stats: &FileStats) -> f64 {
        crate::core::stats::ratios::RatioStatsCalculator::new()
            .calculate_ratio_stats(file_stats)
            .map(|ratios| ratios.quality_metrics.overall_quality_score)
            .unwrap_or(0.0)
    }
}

/// Project-level filter for aggregated stats
pub struct ProjectFilter {
    options: FilterOptions,
}

impl ProjectFilter {
    pub fn new(options: FilterOptions) -> Self {
        Self { options }
    }

    /// Filter extensions based on criteria
    pub fn filter_extensions(
        &self,
        stats_by_extension: &BTreeMap<String, ExtensionStats>,
    ) -> BTreeMap<String, ExtensionStats> {
        let mut filtered = BTreeMap::new();

        for (ext, stats) in stats_by_extension {
            // Language filters
            if !self.options.include_languages.is_empty()
                && !self
                    .options
                    .include_languages
                    .iter()
                    .any(|lang| lang.to_lowercase() == ext.to_lowercase())
            {
                continue;
            }

            if !self.options.exclude_languages.is_empty()
                && self
                    .options
                    .exclude_languages
                    .iter()
                    .any(|lang| lang.to_lowercase() == ext.to_lowercase())
            {
                continue;
            }

            // Size filters
            if let Some(min_lines) = self.options.min_lines {
                if stats.total_lines < min_lines {
                    continue;
                }
            }

            if let Some(max_lines) = self.options.max_lines {
                if stats.total_lines > max_lines {
                    continue;
                }
            }

            if let Some(min_size) = self.options.min_size_bytes {
                if stats.total_size < min_size {
                    continue;
                }
            }

            if let Some(max_size) = self.options.max_size_bytes {
                if stats.total_size > max_size {
                    continue;
                }
            }

            filtered.insert(ext.clone(), stats.clone());
        }

        filtered
    }
}

/// Utility functions for filter parsing
pub struct FilterParser;

impl FilterParser {
    /// Parse a size such as `1024`, `1KB`, `500mb` or `2GB` into bytes.
    ///
    /// A value that is not a size returns `None` so the caller can report it.
    /// Negative and non-finite inputs are rejected rather than saturating to
    /// zero, which silently turned `--min-size -5` into "no minimum".
    pub fn parse_size(size_str: &str) -> Option<u64> {
        const UNITS: [(&str, u64); 4] = [
            ("KB", 1024),
            ("MB", 1024 * 1024),
            ("GB", 1024 * 1024 * 1024),
            ("B", 1),
        ];

        let upper = size_str.trim().to_uppercase();
        let (number, multiplier) = UNITS
            .iter()
            .find_map(|(suffix, factor)| Some((upper.strip_suffix(suffix)?, *factor)))
            .unwrap_or((upper.as_str(), 1));

        let value: f64 = number.trim().parse().ok()?;
        (value.is_finite() && value >= 0.0).then_some((value * multiplier as f64) as u64)
    }

    /// Parse a comma-separated list of languages into lowercase keys.
    ///
    /// Filtering compares against lowercase file extensions, so `--languages
    /// Rust,Python` has to arrive here in the same case or it matches nothing.
    pub fn parse_languages(lang_str: &str) -> Vec<String> {
        lang_str
            .split(',')
            .map(|s| s.trim().to_lowercase())
            .filter(|s| !s.is_empty())
            .collect()
    }
}

/// Format filtered output with additional information
pub struct FilteredOutputFormatter;

impl FilteredOutputFormatter {
    pub fn format_enhanced_cli_output(
        aggregated_stats: &AggregatedStats,
        options: &FilterOptions,
    ) -> String {
        let mut output = String::new();

        // Basic counts
        output.push_str(&format!(
            "{} files, {} lines",
            aggregated_stats.basic.total_files, aggregated_stats.basic.total_lines
        ));

        if options.show_size_info {
            let size_mb = aggregated_stats.basic.total_size as f64 / (1024.0 * 1024.0);
            output.push_str(&format!(", {:.1} MB", size_mb));
        }

        if options.show_complexity && aggregated_stats.complexity.function_count > 0 {
            output.push_str(&format!(
                ", {:.1} avg complexity",
                aggregated_stats.complexity.cyclomatic_complexity
            ));
        }

        if options.show_quality {
            output.push_str(&format!(
                ", {:.1}/100 quality",
                aggregated_stats
                    .ratios
                    .quality_metrics
                    .overall_quality_score
            ));
        }

        if options.show_ratios {
            output.push_str(&format!(
                ", {:.1}% code",
                aggregated_stats.ratios.code_ratio * 100.0
            ));
            if aggregated_stats.ratios.comment_ratio > 0.0 {
                output.push_str(&format!(
                    ", {:.1}% comments",
                    aggregated_stats.ratios.comment_ratio * 100.0
                ));
            }
        }

        output
    }
}
