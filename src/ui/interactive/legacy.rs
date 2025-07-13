use crate::core::types::{CodeStats, FileStats};
use crate::ui::interactive::display::ModernInteractiveDisplay;
use indicatif::{ProgressBar, ProgressStyle};
use owo_colors::OwoColorize;
use std::{io, time::Duration};

// Legacy display for backward compatibility
pub struct InteractiveDisplay {
    modern_display: Option<ModernInteractiveDisplay>,
}

impl InteractiveDisplay {
    pub fn new() -> Self {
        Self {
            modern_display: ModernInteractiveDisplay::new().ok(),
        }
    }

    pub fn show_welcome(&mut self) -> io::Result<()> {
        if let Some(ref mut display) = self.modern_display {
            display.show_welcome().map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        } else {
            // Fallback to simple console output
            println!("{}", "🔍 HOW MANY CODE ANALYZER 🔍".bright_cyan());
            println!("{}", "Intelligent code counting with beautiful visualization".bright_blue());
            println!();
        }
        Ok(())
    }

    pub fn show_scanning_progress(&self, path: &str) -> ProgressBar {
        println!("{}", format!("📁 Analyzing directory: {}", path).bright_yellow());
        println!("{}", "🔍 Scanning for user-created code files...".bright_blue());
        println!();
        
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏")
                .template("{spinner:.cyan} {msg}")
                .unwrap()
        );
        pb.set_message("Scanning files...");
        pb.enable_steady_tick(Duration::from_millis(100));
        pb
    }

    pub fn show_results(&mut self, stats: &CodeStats, individual_files: &[(String, FileStats)]) -> io::Result<()> {
        if let Some(ref mut display) = self.modern_display {
            let individual_files_vec = individual_files.to_vec();
            display.run_interactive_mode(stats.clone(), individual_files_vec)
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        } else {
            // Fallback to simple table output
            self.show_fallback_results(stats, individual_files)?;
        }
        Ok(())
    }

    fn show_fallback_results(&self, stats: &CodeStats, individual_files: &[(String, FileStats)]) -> io::Result<()> {
        println!("{}", "📊 RESULTS".bright_green());
        println!("{}", "─".repeat(80));
        
        println!("📁 Total Files: {}", stats.total_files.to_string().bright_yellow());
        println!("📏 Total Lines: {}", stats.total_lines.to_string().bright_blue());
        println!("💻 Code Lines: {}", stats.total_code_lines.to_string().bright_green());
        println!("💬 Comment Lines: {}", stats.total_comment_lines.to_string().bright_magenta());
        println!("📚 Documentation Lines: {}", stats.total_doc_lines.to_string().bright_cyan());
        println!("⬜ Blank Lines: {}", stats.total_blank_lines.to_string().bright_black());
        println!("💾 Total Size: {}", Self::format_size_fallback(stats.total_size).bright_cyan());

        if !individual_files.is_empty() {
            println!("\n{}", "📄 INDIVIDUAL FILES".bright_green());
            println!("{}", "─".repeat(80));
            
            for (file_path, file_stats) in individual_files {
                println!("📄 {} - {} lines", file_path, file_stats.total_lines);
            }
        }

        println!("\n{}", "Press any key to exit...".bright_green());
        use std::io::Read;
        let _ = std::io::stdin().read(&mut [0u8]).unwrap();
        
        Ok(())
    }

    fn format_size_fallback(size: u64) -> String {
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
} 