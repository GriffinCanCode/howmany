use crate::core::stats::aggregation::AggregatedStats;
use crate::utils::errors::Result;
use serde::{Deserialize, Serialize};


/// Formatting options for different output formats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormattingOptions {
    pub format: OutputFormat,
    pub show_percentages: bool,
    pub show_ratios: bool,
    pub decimal_places: usize,
    pub use_emojis: bool,
    pub color_output: bool,
    pub compact_mode: bool,
    pub sort_by: SortBy,
    pub sort_descending: bool,
    pub max_items: Option<usize>,
}

/// Output format types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OutputFormat {
    Text,
    Json,
    Csv,
    Html,
    Markdown,
    Table,
    Summary,
}

/// Sort criteria
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SortBy {
    Name,
    Files,
    Lines,
    Code,
    Comments,
    Docs,
    Size,
    Complexity,
    Time,
    Quality,
}

impl Default for FormattingOptions {
    fn default() -> Self {
        Self {
            format: OutputFormat::Text,
            show_percentages: true,
            show_ratios: false,
            decimal_places: 1,
            use_emojis: true,
            color_output: true,
            compact_mode: false,
            sort_by: SortBy::Lines,
            sort_descending: true,
            max_items: None,
        }
    }
}

/// Formatter for statistics display
pub struct StatFormatter {
    size_units: Vec<&'static str>,
}

impl StatFormatter {
    pub fn new() -> Self {
        Self {
            size_units: vec!["B", "KB", "MB", "GB", "TB"],
        }
    }
    
    /// Format statistics according to options
    pub fn format_stats(&self, stats: &AggregatedStats, options: &FormattingOptions) -> Result<String> {
        match options.format {
            OutputFormat::Text => self.format_text(stats, options),
            OutputFormat::Json => self.format_json(stats, options),
            OutputFormat::Csv => self.format_csv(stats, options),
            OutputFormat::Html => self.format_html(stats, options),
            OutputFormat::Markdown => self.format_markdown(stats, options),
            OutputFormat::Table => self.format_table(stats, options),
            OutputFormat::Summary => self.format_summary(stats, options),
        }
    }
    
    /// Format as plain text
    fn format_text(&self, stats: &AggregatedStats, options: &FormattingOptions) -> Result<String> {
        let mut output = String::new();
        
        // Header
        if options.use_emojis {
            output.push_str("📊 CODE STATISTICS\n");
        } else {
            output.push_str("CODE STATISTICS\n");
        }
        output.push_str(&"═".repeat(50));
        output.push('\n');
        
        // Basic stats
        output.push_str(&format!("Total Files: {}\n", self.format_number(stats.basic.total_files)));
        output.push_str(&format!("Total Lines: {}\n", self.format_number(stats.basic.total_lines)));
        output.push_str(&format!("Code Lines: {}", self.format_number(stats.basic.code_lines)));
        
        if options.show_percentages {
            let code_pct = (stats.basic.code_lines as f64 / stats.basic.total_lines as f64) * 100.0;
            output.push_str(&format!(" ({:.1}%)", code_pct));
        }
        output.push('\n');
        
        output.push_str(&format!("Comment Lines: {}", self.format_number(stats.basic.comment_lines)));
        if options.show_percentages {
            let comment_pct = (stats.basic.comment_lines as f64 / stats.basic.total_lines as f64) * 100.0;
            output.push_str(&format!(" ({:.1}%)", comment_pct));
        }
        output.push('\n');
        
        output.push_str(&format!("Documentation Lines: {}", self.format_number(stats.basic.doc_lines)));
        if options.show_percentages {
            let doc_pct = (stats.basic.doc_lines as f64 / stats.basic.total_lines as f64) * 100.0;
            output.push_str(&format!(" ({:.1}%)", doc_pct));
        }
        output.push('\n');
        
        output.push_str(&format!("Blank Lines: {}", self.format_number(stats.basic.blank_lines)));
        if options.show_percentages {
            let blank_pct = (stats.basic.blank_lines as f64 / stats.basic.total_lines as f64) * 100.0;
            output.push_str(&format!(" ({:.1}%)", blank_pct));
        }
        output.push('\n');
        
        output.push_str(&format!("Total Size: {}\n", self.format_size(stats.basic.total_size)));
        
        // Complexity stats
        if stats.complexity.function_count > 0 {
            output.push('\n');
            output.push_str(&format!("Functions: {}\n", self.format_number(stats.complexity.function_count)));
            output.push_str(&format!("Avg Complexity: {:.1}\n", stats.complexity.cyclomatic_complexity));
            output.push_str(&format!("Max Nesting: {}\n", stats.complexity.max_nesting_depth));
        }
        
        // Time stats
        if !options.compact_mode {
            output.push('\n');
            if options.use_emojis {
                output.push_str("⏱️  TIME ESTIMATES\n");
            } else {
                output.push_str("TIME ESTIMATES\n");
            }
            output.push_str(&"─".repeat(30));
            output.push('\n');
            output.push_str(&format!("Total Time: {}\n", stats.time.total_time_formatted));
            output.push_str(&format!("Development Days: {:.1}\n", stats.time.productivity_metrics.estimated_development_days));
        }
        
        // Quality scores
        if !options.compact_mode {
            output.push('\n');
            if options.use_emojis {
                output.push_str("🏆 QUALITY SCORES\n");
            } else {
                output.push_str("QUALITY SCORES\n");
            }
            output.push_str(&"─".repeat(30));
            output.push('\n');
            output.push_str(&format!("Overall Quality: {:.1}/100\n", stats.ratios.quality_metrics.overall_quality_score));
            output.push_str(&format!("Documentation: {:.1}/100\n", stats.ratios.quality_metrics.documentation_score));
            output.push_str(&format!("Maintainability: {:.1}/100\n", stats.ratios.quality_metrics.maintainability_score));
            output.push_str(&format!("Readability: {:.1}/100\n", stats.ratios.quality_metrics.readability_score));
        }
        
        Ok(output)
    }
    
    /// Format as JSON
    fn format_json(&self, stats: &AggregatedStats, _options: &FormattingOptions) -> Result<String> {
        let json = serde_json::to_string_pretty(stats)?;
        Ok(json)
    }
    
    /// Format as CSV
    fn format_csv(&self, stats: &AggregatedStats, options: &FormattingOptions) -> Result<String> {
        let mut output = String::new();
        
        // Header
        output.push_str("Extension,Files,Lines,Code,Comments,Docs,Blank,Size,Functions,Complexity,Time\n");
        
        // Sort extensions
        let mut extensions: Vec<_> = stats.basic.stats_by_extension.iter().collect();
        self.sort_extensions(&mut extensions, options);
        
        // Data rows
        for (ext, ext_stats) in extensions {
            let complexity = stats.complexity.complexity_by_extension.get(ext)
                .map(|c| c.cyclomatic_complexity)
                .unwrap_or(0.0);
            
            let time = stats.time.time_by_extension.get(ext)
                .map(|t| t.total_time_minutes)
                .unwrap_or(0);
            
            output.push_str(&format!(
                "{},{},{},{},{},{},{},{},{},{:.1},{}\n",
                ext,
                ext_stats.file_count,
                ext_stats.total_lines,
                ext_stats.code_lines,
                ext_stats.comment_lines,
                ext_stats.doc_lines,
                ext_stats.blank_lines,
                ext_stats.total_size,
                stats.complexity.complexity_by_extension.get(ext)
                    .map(|c| c.function_count)
                    .unwrap_or(0),
                complexity,
                time
            ));
        }
        
        Ok(output)
    }
    
    /// Format as HTML
    fn format_html(&self, stats: &AggregatedStats, options: &FormattingOptions) -> Result<String> {
        let mut html = String::new();
        
        html.push_str("<!DOCTYPE html>\n");
        html.push_str("<html>\n<head>\n");
        html.push_str("<title>Code Statistics</title>\n");
        html.push_str("<style>\n");
        html.push_str(self.get_html_styles());
        html.push_str("</style>\n");
        html.push_str("</head>\n<body>\n");
        
        html.push_str("<h1>📊 Code Statistics</h1>\n");
        
        // Summary table
        html.push_str("<table class='summary-table'>\n");
        html.push_str("<tr><th>Metric</th><th>Value</th></tr>\n");
        html.push_str(&format!("<tr><td>Total Files</td><td>{}</td></tr>\n", self.format_number(stats.basic.total_files)));
        html.push_str(&format!("<tr><td>Total Lines</td><td>{}</td></tr>\n", self.format_number(stats.basic.total_lines)));
        html.push_str(&format!("<tr><td>Code Lines</td><td>{}</td></tr>\n", self.format_number(stats.basic.code_lines)));
        html.push_str(&format!("<tr><td>Functions</td><td>{}</td></tr>\n", self.format_number(stats.complexity.function_count)));
        html.push_str(&format!("<tr><td>Avg Complexity</td><td>{:.1}</td></tr>\n", stats.complexity.cyclomatic_complexity));
        html.push_str(&format!("<tr><td>Total Size</td><td>{}</td></tr>\n", self.format_size(stats.basic.total_size)));
        html.push_str("</table>\n");
        
        // Extension breakdown
        html.push_str("<h2>📋 File Type Breakdown</h2>\n");
        html.push_str("<table class='breakdown-table'>\n");
        html.push_str("<tr><th>Extension</th><th>Files</th><th>Lines</th><th>Code</th><th>Comments</th><th>Docs</th><th>Size</th></tr>\n");
        
        let mut extensions: Vec<_> = stats.basic.stats_by_extension.iter().collect();
        self.sort_extensions(&mut extensions, options);
        
        for (ext, ext_stats) in extensions {
            html.push_str(&format!(
                "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>\n",
                ext,
                self.format_number(ext_stats.file_count),
                self.format_number(ext_stats.total_lines),
                self.format_number(ext_stats.code_lines),
                self.format_number(ext_stats.comment_lines),
                self.format_number(ext_stats.doc_lines),
                self.format_size(ext_stats.total_size)
            ));
        }
        
        html.push_str("</table>\n");
        html.push_str("</body>\n</html>\n");
        
        Ok(html)
    }
    
    /// Format as Markdown
    fn format_markdown(&self, stats: &AggregatedStats, options: &FormattingOptions) -> Result<String> {
        let mut md = String::new();
        
        md.push_str("# 📊 Code Statistics\n\n");
        
        // Summary
        md.push_str("## Summary\n\n");
        md.push_str("| Metric | Value |\n");
        md.push_str("|--------|-------|\n");
        md.push_str(&format!("| Total Files | {} |\n", self.format_number(stats.basic.total_files)));
        md.push_str(&format!("| Total Lines | {} |\n", self.format_number(stats.basic.total_lines)));
        md.push_str(&format!("| Code Lines | {} |\n", self.format_number(stats.basic.code_lines)));
        md.push_str(&format!("| Functions | {} |\n", self.format_number(stats.complexity.function_count)));
        md.push_str(&format!("| Avg Complexity | {:.1} |\n", stats.complexity.cyclomatic_complexity));
        md.push_str(&format!("| Total Size | {} |\n", self.format_size(stats.basic.total_size)));
        md.push_str("\n");
        
        // Extension breakdown
        md.push_str("## 📋 File Type Breakdown\n\n");
        md.push_str("| Extension | Files | Lines | Code | Comments | Docs | Size |\n");
        md.push_str("|-----------|-------|-------|------|----------|------|------|\n");
        
        let mut extensions: Vec<_> = stats.basic.stats_by_extension.iter().collect();
        self.sort_extensions(&mut extensions, options);
        
        for (ext, ext_stats) in extensions {
            md.push_str(&format!(
                "| {} | {} | {} | {} | {} | {} | {} |\n",
                ext,
                self.format_number(ext_stats.file_count),
                self.format_number(ext_stats.total_lines),
                self.format_number(ext_stats.code_lines),
                self.format_number(ext_stats.comment_lines),
                self.format_number(ext_stats.doc_lines),
                self.format_size(ext_stats.total_size)
            ));
        }
        
        Ok(md)
    }
    
    /// Format as table
    fn format_table(&self, stats: &AggregatedStats, options: &FormattingOptions) -> Result<String> {
        let mut output = String::new();
        
        // Header
        let header = format!(
            "{:<12} {:>8} {:>10} {:>10} {:>12} {:>10} {:>12}",
            "Extension", "Files", "Lines", "Code", "Comments", "Docs", "Size"
        );
        output.push_str(&header);
        output.push('\n');
        output.push_str(&"─".repeat(header.len()));
        output.push('\n');
        
        // Sort extensions
        let mut extensions: Vec<_> = stats.basic.stats_by_extension.iter().collect();
        self.sort_extensions(&mut extensions, options);
        
        // Apply max items limit
        if let Some(max) = options.max_items {
            extensions.truncate(max);
        }
        
        // Data rows
        for (ext, ext_stats) in extensions {
            output.push_str(&format!(
                "{:<12} {:>8} {:>10} {:>10} {:>12} {:>10} {:>12}\n",
                ext,
                self.format_number(ext_stats.file_count),
                self.format_number(ext_stats.total_lines),
                self.format_number(ext_stats.code_lines),
                self.format_number(ext_stats.comment_lines),
                self.format_number(ext_stats.doc_lines),
                self.format_size(ext_stats.total_size)
            ));
        }
        
        Ok(output)
    }
    
    /// Format as summary
    fn format_summary(&self, stats: &AggregatedStats, options: &FormattingOptions) -> Result<String> {
        let mut output = String::new();
        
        if options.use_emojis {
            output.push_str("📊 ");
        }
        
        output.push_str(&format!(
            "{} files, {} lines ({} code), {} functions, {:.1} avg complexity, {}",
            self.format_number(stats.basic.total_files),
            self.format_number(stats.basic.total_lines),
            self.format_number(stats.basic.code_lines),
            self.format_number(stats.complexity.function_count),
            stats.complexity.cyclomatic_complexity,
            self.format_size(stats.basic.total_size)
        ));
        
        Ok(output)
    }
    
    /// Format a number with thousand separators
    pub fn format_number(&self, num: usize) -> String {
        let num_str = num.to_string();
        let mut result = String::new();
        let chars: Vec<char> = num_str.chars().collect();
        
        for (i, ch) in chars.iter().enumerate() {
            if i > 0 && (chars.len() - i) % 3 == 0 {
                result.push(',');
            }
            result.push(*ch);
        }
        
        result
    }
    
    /// Format file size in human-readable format
    pub fn format_size(&self, size: u64) -> String {
        let mut size = size as f64;
        let mut unit_index = 0;
        
        while size >= 1024.0 && unit_index < self.size_units.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }
        
        if unit_index == 0 {
            format!("{} {}", size as u64, self.size_units[unit_index])
        } else {
            format!("{:.1} {}", size, self.size_units[unit_index])
        }
    }
    
    /// Format percentage
    pub fn format_percentage(&self, ratio: f64, decimal_places: usize) -> String {
        format!("{:.prec$}%", ratio * 100.0, prec = decimal_places)
    }
    
    /// Get file extension emoji
    pub fn get_extension_emoji(&self, ext: &str) -> &'static str {
        match ext {
            "rs" => "🦀",
            "py" => "🐍",
            "js" | "jsx" => "📜",
            "ts" | "tsx" => "📘",
            "html" => "🌐",
            "css" | "scss" | "sass" => "🎨",
            "json" => "📋",
            "xml" => "📄",
            "yaml" | "yml" => "⚙️",
            "toml" => "🔧",
            "md" => "📝",
            "txt" => "📄",
            "java" => "☕",
            "c" | "cpp" | "cc" | "cxx" => "⚡",
            "h" | "hpp" => "📎",
            "go" => "🐹",
            "php" => "🐘",
            "rb" => "💎",
            "swift" => "🍎",
            "kt" => "🎯",
            "scala" => "🎭",
            "sh" | "bash" | "zsh" => "🐚",
            _ => "📄",
        }
    }
    
    /// Sort extensions according to options
    fn sort_extensions(&self, extensions: &mut Vec<(&String, &crate::core::stats::basic::ExtensionStats)>, options: &FormattingOptions) {
        extensions.sort_by(|a, b| {
            let comparison = match options.sort_by {
                SortBy::Name => a.0.cmp(b.0),
                SortBy::Files => a.1.file_count.cmp(&b.1.file_count),
                SortBy::Lines => a.1.total_lines.cmp(&b.1.total_lines),
                SortBy::Code => a.1.code_lines.cmp(&b.1.code_lines),
                SortBy::Comments => a.1.comment_lines.cmp(&b.1.comment_lines),
                SortBy::Docs => a.1.doc_lines.cmp(&b.1.doc_lines),
                SortBy::Size => a.1.total_size.cmp(&b.1.total_size),
                _ => a.1.total_lines.cmp(&b.1.total_lines), // Default to lines
            };
            
            if options.sort_descending {
                comparison.reverse()
            } else {
                comparison
            }
        });
    }
    
    /// Get HTML styles
    fn get_html_styles(&self) -> &'static str {
        r#"
        body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; margin: 20px; }
        h1, h2 { color: #2c3e50; }
        table { border-collapse: collapse; width: 100%; margin: 20px 0; }
        th, td { border: 1px solid #ddd; padding: 8px; text-align: left; }
        th { background-color: #f2f2f2; font-weight: bold; }
        .summary-table { max-width: 400px; }
        .breakdown-table th:not(:first-child), .breakdown-table td:not(:first-child) { text-align: right; }
        tr:nth-child(even) { background-color: #f9f9f9; }
        "#
    }
    
    /// Create formatting options for different presets
    pub fn create_preset_options(preset: &str) -> FormattingOptions {
        match preset {
            "compact" => FormattingOptions {
                compact_mode: true,
                show_percentages: false,
                use_emojis: false,
                max_items: Some(10),
                ..Default::default()
            },
            "detailed" => FormattingOptions {
                show_percentages: true,
                show_ratios: true,
                decimal_places: 2,
                compact_mode: false,
                ..Default::default()
            },
            "json" => FormattingOptions {
                format: OutputFormat::Json,
                ..Default::default()
            },
            "csv" => FormattingOptions {
                format: OutputFormat::Csv,
                use_emojis: false,
                color_output: false,
                ..Default::default()
            },
            "html" => FormattingOptions {
                format: OutputFormat::Html,
                show_percentages: true,
                ..Default::default()
            },
            "markdown" => FormattingOptions {
                format: OutputFormat::Markdown,
                use_emojis: true,
                ..Default::default()
            },
            _ => FormattingOptions::default(),
        }
    }
}

impl Default for StatFormatter {
    fn default() -> Self {
        Self::new()
    }
} 