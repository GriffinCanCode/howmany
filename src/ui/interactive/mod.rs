pub mod app;
pub mod display;
pub mod rendering;
pub mod charts;
pub mod utils;
pub mod legacy;

use crate::core::types::{CodeStats, FileStats};
use crate::core::stats::AggregatedStats;
use crate::utils::errors::Result;
use display::ModernInteractiveDisplay;
use legacy::InteractiveDisplay as LegacyDisplay;

pub struct InteractiveDisplay {
    modern_display: Option<ModernInteractiveDisplay>,
    legacy_display: LegacyDisplay,
}

impl InteractiveDisplay {
    pub fn new() -> Self {
        let modern_display = ModernInteractiveDisplay::new().ok();
        let legacy_display = LegacyDisplay::new();
        
        Self {
            modern_display,
            legacy_display,
        }
    }
    
    pub fn show_welcome(&mut self) -> Result<()> {
        if let Some(ref mut modern) = self.modern_display {
            modern.show_welcome().map_err(|e| crate::utils::errors::HowManyError::display(e.to_string()))
        } else {
            self.legacy_display.show_welcome().map_err(|e| crate::utils::errors::HowManyError::display(e.to_string()))
        }
    }
    
    pub fn show_scanning_progress(&mut self, path: &str) -> Result<indicatif::ProgressBar> {
        if let Some(ref mut modern) = self.modern_display {
            modern.show_scanning_progress(path).map_err(|e| crate::utils::errors::HowManyError::display(e.to_string()))
        } else {
            Ok(self.legacy_display.show_scanning_progress(path))
        }
    }
    
    pub fn show_results(&mut self, stats: &CodeStats, individual_files: &[(String, FileStats)]) -> Result<()> {
        if let Some(ref mut _modern) = self.modern_display {
            // Convert CodeStats to AggregatedStats for comprehensive display
            let stats_calculator = crate::core::stats::StatsCalculator::new();
            if let Ok(aggregated_stats) = stats_calculator.calculate_project_stats(stats, individual_files) {
                return self.show_comprehensive_results(&aggregated_stats, individual_files);
            }
        }
        self.legacy_display.show_results(stats, individual_files).map_err(|e| crate::utils::errors::HowManyError::display(e.to_string()))
    }
    
    pub fn show_comprehensive_results(&mut self, aggregated_stats: &AggregatedStats, individual_files: &[(String, FileStats)]) -> Result<()> {
        if let Some(ref mut modern) = self.modern_display {
            // Convert AggregatedStats back to CodeStats for compatibility
            let code_stats = CodeStats {
                total_files: aggregated_stats.basic.total_files,
                total_lines: aggregated_stats.basic.total_lines,
                total_code_lines: aggregated_stats.basic.code_lines,
                total_comment_lines: aggregated_stats.basic.comment_lines,
                total_blank_lines: aggregated_stats.basic.blank_lines,
                total_size: aggregated_stats.basic.total_size,
                total_doc_lines: aggregated_stats.basic.doc_lines,
                stats_by_extension: aggregated_stats.basic.stats_by_extension.iter()
                    .map(|(ext, ext_stats)| {
                        (ext.clone(), (ext_stats.file_count, crate::core::types::FileStats {
                            total_lines: ext_stats.total_lines,
                            code_lines: ext_stats.code_lines,
                            comment_lines: ext_stats.comment_lines,
                            blank_lines: ext_stats.blank_lines,
                            file_size: ext_stats.total_size,
                            doc_lines: ext_stats.doc_lines,
                        }))
                    })
                    .collect(),
            };
            
            // Run with async support for better responsiveness
            modern.run_interactive_mode(code_stats, individual_files.to_vec()).map_err(|e| crate::utils::errors::HowManyError::display(e.to_string()))
        } else {
            // Fallback to legacy display with enhanced output
            self.show_enhanced_legacy_results(aggregated_stats, individual_files)
        }
    }
    
    fn show_enhanced_legacy_results(&mut self, aggregated_stats: &AggregatedStats, individual_files: &[(String, FileStats)]) -> Result<()> {
        use owo_colors::OwoColorize;
        
        println!("{}", "📊 COMPREHENSIVE RESULTS".bright_green());
        println!("{}", "─".repeat(80));
        
        // Basic stats
        println!("📁 Total Files: {}", aggregated_stats.basic.total_files.to_string().bright_yellow());
        println!("📏 Total Lines: {}", aggregated_stats.basic.total_lines.to_string().bright_blue());
        println!("💻 Code Lines: {}", aggregated_stats.basic.code_lines.to_string().bright_green());
        println!("💬 Comment Lines: {}", aggregated_stats.basic.comment_lines.to_string().bright_magenta());
        println!("📚 Documentation Lines: {}", aggregated_stats.basic.doc_lines.to_string().bright_cyan());
        println!("⬜ Blank Lines: {}", aggregated_stats.basic.blank_lines.to_string().bright_black());
        println!("💾 Total Size: {}", self.format_size_fallback(aggregated_stats.basic.total_size).bright_cyan());
        
        // Enhanced stats
        if aggregated_stats.complexity.function_count > 0 {
            println!();
            println!("{}", "🔧 COMPLEXITY ANALYSIS".bright_green());
            println!("{}", "─".repeat(80));
            println!("⚙️  Functions: {}", aggregated_stats.complexity.function_count.to_string().bright_yellow());
            println!("📊 Average Complexity: {:.1}", aggregated_stats.complexity.cyclomatic_complexity);
            println!("🏗️  Max Nesting Depth: {}", aggregated_stats.complexity.max_nesting_depth);
        }
        
        // Quality metrics
        println!();
        println!("{}", "🏆 QUALITY METRICS".bright_green());
        println!("{}", "─".repeat(80));
        println!("🎯 Overall Quality: {:.1}/100", aggregated_stats.ratios.quality_metrics.overall_quality_score);
        println!("📖 Documentation Score: {:.1}/100", aggregated_stats.ratios.quality_metrics.documentation_score);
        println!("🔧 Maintainability Score: {:.1}/100", aggregated_stats.ratios.quality_metrics.maintainability_score);
        
        if !individual_files.is_empty() {
            println!();
            println!("{}", "📄 INDIVIDUAL FILES".bright_green());
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
    
    fn format_size_fallback(&self, size: u64) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
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

impl Default for InteractiveDisplay {
    fn default() -> Self {
        Self::new()
    }
} 