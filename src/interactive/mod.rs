use crate::counter::{CodeStats, FileStats};
use colored::*;
use comfy_table::{Table, Row, Cell, presets::UTF8_FULL, ContentArrangement, Color};
use console::{Term, style};
use indicatif::{ProgressBar, ProgressStyle};
use std::collections::HashMap;
use std::io::{self, Write};
use std::time::Duration;

pub struct InteractiveDisplay {
    term: Term,
}

impl InteractiveDisplay {
    pub fn new() -> Self {
        Self {
            term: Term::stdout(),
        }
    }

    pub fn show_welcome(&self) -> io::Result<()> {
        self.term.clear_screen()?;
        
        let title = r#"
╔═══════════════════════════════════════════════════════════════════════════════╗
║                                                                               ║
║                          🔍 HOW MANY CODE ANALYZER 🔍                        ║
║                                                                               ║
║              Intelligent code counting with beautiful visualization           ║
║                                                                               ║
╚═══════════════════════════════════════════════════════════════════════════════╝
        "#;
        
        println!("{}", title.bright_cyan());
        println!();
        Ok(())
    }

    pub fn show_scanning_progress(&self, path: &str) -> ProgressBar {
        println!("{}", format!("📁 Analyzing directory: {}", path).bright_yellow());
        println!("{}", "🔍 Scanning for user-created code files...".bright_blue());
        println!();
        
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
                .template("{spinner:.cyan} {msg}")
                .unwrap()
        );
        pb.set_message("Scanning files...");
        pb.enable_steady_tick(Duration::from_millis(100));
        pb
    }

    pub fn show_results(&self, stats: &CodeStats, individual_files: &[(String, FileStats)]) -> io::Result<()> {
        self.term.clear_screen()?;
        self.show_welcome()?;
        
        // Main statistics overview
        self.show_overview(stats)?;
        
        // Detailed breakdown by file type
        self.show_breakdown_by_type(stats)?;
        
        // Individual files if requested
        if !individual_files.is_empty() {
            self.show_individual_files(individual_files)?;
        }
        
        // Summary footer
        self.show_footer()?;
        
        Ok(())
    }

    fn show_overview(&self, stats: &CodeStats) -> io::Result<()> {
        println!("{}", "📊 OVERVIEW".bright_green().bold());
        println!("{}", "─".repeat(80).bright_black());
        
        let mut table = Table::new();
        table.load_preset(UTF8_FULL)
             .set_content_arrangement(ContentArrangement::Dynamic);
        
        // Add header
        table.add_row(Row::from(vec![
            Cell::new("Metric").fg(Color::Cyan).add_attribute(comfy_table::Attribute::Bold),
            Cell::new("Count").fg(Color::Cyan).add_attribute(comfy_table::Attribute::Bold),
            Cell::new("Percentage").fg(Color::Cyan).add_attribute(comfy_table::Attribute::Bold),
            Cell::new("Visual").fg(Color::Cyan).add_attribute(comfy_table::Attribute::Bold),
        ]));
        
        // Calculate percentages
        let total_lines = stats.total_lines as f64;
        let code_pct = if total_lines > 0.0 { (stats.total_code_lines as f64 / total_lines) * 100.0 } else { 0.0 };
        let comment_pct = if total_lines > 0.0 { (stats.total_comment_lines as f64 / total_lines) * 100.0 } else { 0.0 };
        let blank_pct = if total_lines > 0.0 { (stats.total_blank_lines as f64 / total_lines) * 100.0 } else { 0.0 };
        
        // Add data rows
        table.add_row(Row::from(vec![
            Cell::new("📁 Total Files").fg(Color::Yellow),
            Cell::new(format!("{}", stats.total_files)).fg(Color::White),
            Cell::new("-").fg(Color::DarkGrey),
            Cell::new("📁".repeat(std::cmp::min(stats.total_files, 20))).fg(Color::Yellow),
        ]));
        
        table.add_row(Row::from(vec![
            Cell::new("📏 Total Lines").fg(Color::Blue),
            Cell::new(format!("{}", stats.total_lines)).fg(Color::White),
            Cell::new("100.0%").fg(Color::Green),
            Cell::new(self.create_bar(100.0, "█")).fg(Color::Blue),
        ]));
        
        table.add_row(Row::from(vec![
            Cell::new("💻 Code Lines").fg(Color::Green),
            Cell::new(format!("{}", stats.total_code_lines)).fg(Color::White),
            Cell::new(format!("{:.1}%", code_pct)).fg(Color::Green),
            Cell::new(self.create_bar(code_pct, "█")).fg(Color::Green),
        ]));
        
        table.add_row(Row::from(vec![
            Cell::new("💬 Comment Lines").fg(Color::Magenta),
            Cell::new(format!("{}", stats.total_comment_lines)).fg(Color::White),
            Cell::new(format!("{:.1}%", comment_pct)).fg(Color::Magenta),
            Cell::new(self.create_bar(comment_pct, "█")).fg(Color::Magenta),
        ]));
        
        table.add_row(Row::from(vec![
            Cell::new("⬜ Blank Lines").fg(Color::DarkGrey),
            Cell::new(format!("{}", stats.total_blank_lines)).fg(Color::White),
            Cell::new(format!("{:.1}%", blank_pct)).fg(Color::DarkGrey),
            Cell::new(self.create_bar(blank_pct, "░")).fg(Color::DarkGrey),
        ]));
        
        table.add_row(Row::from(vec![
            Cell::new("💾 Total Size").fg(Color::Cyan),
            Cell::new(self.format_size(stats.total_size)).fg(Color::White),
            Cell::new("-").fg(Color::DarkGrey),
            Cell::new("💾".repeat(std::cmp::min((stats.total_size / 1024) as usize, 20))).fg(Color::Cyan),
        ]));
        
        println!("{}", table);
        println!();
        
        Ok(())
    }

    fn show_breakdown_by_type(&self, stats: &CodeStats) -> io::Result<()> {
        if stats.stats_by_extension.is_empty() {
            return Ok(());
        }
        
        println!("{}", "🔍 BREAKDOWN BY FILE TYPE".bright_green().bold());
        println!("{}", "─".repeat(80).bright_black());
        
        let mut table = Table::new();
        table.load_preset(UTF8_FULL)
             .set_content_arrangement(ContentArrangement::Dynamic);
        
        // Add header
        table.add_row(Row::from(vec![
            Cell::new("Extension").fg(Color::Cyan).add_attribute(comfy_table::Attribute::Bold),
            Cell::new("Files").fg(Color::Cyan).add_attribute(comfy_table::Attribute::Bold),
            Cell::new("Lines").fg(Color::Cyan).add_attribute(comfy_table::Attribute::Bold),
            Cell::new("Code").fg(Color::Cyan).add_attribute(comfy_table::Attribute::Bold),
            Cell::new("Comments").fg(Color::Cyan).add_attribute(comfy_table::Attribute::Bold),
            Cell::new("Blanks").fg(Color::Cyan).add_attribute(comfy_table::Attribute::Bold),
            Cell::new("Size").fg(Color::Cyan).add_attribute(comfy_table::Attribute::Bold),
            Cell::new("Distribution").fg(Color::Cyan).add_attribute(comfy_table::Attribute::Bold),
        ]));
        
        // Sort by total lines descending
        let mut ext_stats: Vec<_> = stats.stats_by_extension.iter().collect();
        ext_stats.sort_by(|a, b| b.1.1.total_lines.cmp(&a.1.1.total_lines));
        
        for (ext, (file_count, file_stats)) in ext_stats {
            let ext_icon = self.get_extension_icon(ext);
            let percentage = if stats.total_lines > 0 {
                (file_stats.total_lines as f64 / stats.total_lines as f64) * 100.0
            } else {
                0.0
            };
            
            table.add_row(Row::from(vec![
                Cell::new(format!("{} {}", ext_icon, ext)).fg(Color::Yellow),
                Cell::new(format!("{}", file_count)).fg(Color::White),
                Cell::new(format!("{}", file_stats.total_lines)).fg(Color::Blue),
                Cell::new(format!("{}", file_stats.code_lines)).fg(Color::Green),
                Cell::new(format!("{}", file_stats.comment_lines)).fg(Color::Magenta),
                Cell::new(format!("{}", file_stats.blank_lines)).fg(Color::DarkGrey),
                Cell::new(self.format_size(file_stats.file_size)).fg(Color::Cyan),
                Cell::new(format!("{} {:.1}%", self.create_bar(percentage, "▓"), percentage)).fg(Color::Yellow),
            ]));
        }
        
        println!("{}", table);
        println!();
        
        Ok(())
    }

    fn show_individual_files(&self, individual_files: &[(String, FileStats)]) -> io::Result<()> {
        println!("{}", "📄 INDIVIDUAL FILES".bright_green().bold());
        println!("{}", "─".repeat(80).bright_black());
        
        let mut table = Table::new();
        table.load_preset(UTF8_FULL)
             .set_content_arrangement(ContentArrangement::Dynamic);
        
        // Add header
        table.add_row(Row::from(vec![
            Cell::new("File").fg(Color::Cyan).add_attribute(comfy_table::Attribute::Bold),
            Cell::new("Lines").fg(Color::Cyan).add_attribute(comfy_table::Attribute::Bold),
            Cell::new("Code").fg(Color::Cyan).add_attribute(comfy_table::Attribute::Bold),
            Cell::new("Comments").fg(Color::Cyan).add_attribute(comfy_table::Attribute::Bold),
            Cell::new("Blanks").fg(Color::Cyan).add_attribute(comfy_table::Attribute::Bold),
            Cell::new("Size").fg(Color::Cyan).add_attribute(comfy_table::Attribute::Bold),
        ]));
        
        for (file_path, file_stats) in individual_files {
            let file_icon = self.get_file_icon(file_path);
            let shortened_path = self.shorten_path(file_path, 40);
            
            table.add_row(Row::from(vec![
                Cell::new(format!("{} {}", file_icon, shortened_path)).fg(Color::Yellow),
                Cell::new(format!("{}", file_stats.total_lines)).fg(Color::Blue),
                Cell::new(format!("{}", file_stats.code_lines)).fg(Color::Green),
                Cell::new(format!("{}", file_stats.comment_lines)).fg(Color::Magenta),
                Cell::new(format!("{}", file_stats.blank_lines)).fg(Color::DarkGrey),
                Cell::new(self.format_size(file_stats.file_size)).fg(Color::Cyan),
            ]));
        }
        
        println!("{}", table);
        println!();
        
        Ok(())
    }

    fn show_footer(&self) -> io::Result<()> {
        println!("{}", "─".repeat(80).bright_black());
        println!("{}", "✨ Analysis complete! Press any key to exit...".bright_green());
        
        // Wait for user input
        self.term.read_key()?;
        
        Ok(())
    }

    fn create_bar(&self, percentage: f64, char: &str) -> String {
        let width = 20;
        let filled = ((percentage / 100.0) * width as f64) as usize;
        let filled = std::cmp::min(filled, width);
        char.repeat(filled)
    }

    fn format_size(&self, size: u64) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
        let mut size = size as f64;
        let mut unit_index = 0;
        
        while size >= 1024.0 && unit_index < UNITS.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }
        
        if unit_index == 0 {
            format!("{} {}", size as u64, UNITS[unit_index])
        } else {
            format!("{:.1} {}", size, UNITS[unit_index])
        }
    }

    fn get_extension_icon(&self, ext: &str) -> &'static str {
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

    fn get_file_icon(&self, file_path: &str) -> &'static str {
        if file_path.ends_with(".rs") {
            "🦀"
        } else if file_path.ends_with(".py") {
            "🐍"
        } else if file_path.ends_with(".js") || file_path.ends_with(".jsx") {
            "📜"
        } else if file_path.ends_with(".ts") || file_path.ends_with(".tsx") {
            "📘"
        } else if file_path.ends_with(".toml") {
            "🔧"
        } else if file_path.ends_with(".json") {
            "📋"
        } else if file_path.ends_with(".md") {
            "📝"
        } else {
            "📄"
        }
    }

    fn shorten_path(&self, path: &str, max_length: usize) -> String {
        if path.len() <= max_length {
            path.to_string()
        } else {
            let parts: Vec<&str> = path.split('/').collect();
            if parts.len() > 2 {
                format!(".../{}", parts[parts.len()-1])
            } else {
                format!("...{}", &path[path.len()-max_length+3..])
            }
        }
    }
} 