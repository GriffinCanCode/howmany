use crate::core::types::{CodeStats, FileStats};
use crate::core::stats::AggregatedStats;
use crate::utils::errors::Result;
use std::fs;
use std::path::Path;

use super::standard_report::StandardReportGenerator;
use super::time_wasted_report::TimeWastedReportGenerator;

pub struct HtmlReporter {
    standard_generator: StandardReportGenerator,
    time_wasted_generator: TimeWastedReportGenerator,
}

impl HtmlReporter {
    pub fn new() -> Self {
        Self {
            standard_generator: StandardReportGenerator::new(),
            time_wasted_generator: TimeWastedReportGenerator::new(),
        }
    }
    
    /// Generate report from basic CodeStats (backward compatibility)
    pub fn generate_report(&self, stats: &CodeStats, individual_files: &[(String, FileStats)], output_path: &Path) -> Result<()> {
        let html_content = self.standard_generator.create_html_content(stats, individual_files)?;
        fs::write(output_path, html_content)?;
        Ok(())
    }
    
    /// Generate comprehensive report from AggregatedStats
    pub fn generate_comprehensive_report(&self, aggregated_stats: &AggregatedStats, individual_files: &[(String, FileStats)], output_path: &Path) -> Result<()> {
        let html_content = self.standard_generator.create_comprehensive_html_content(aggregated_stats, individual_files)?;
        fs::write(output_path, html_content)?;
        Ok(())
    }
    
    /// Generate time wasted report from basic CodeStats (backward compatibility)
    pub fn generate_time_wasted_report(&self, stats: &CodeStats, individual_files: &[(String, FileStats)], output_path: &Path) -> Result<()> {
        let html_content = self.time_wasted_generator.create_time_wasted_content(stats, individual_files)?;
        fs::write(output_path, html_content)?;
        Ok(())
    }
    
    /// Generate comprehensive time wasted report from AggregatedStats
    pub fn generate_comprehensive_time_wasted_report(&self, aggregated_stats: &AggregatedStats, individual_files: &[(String, FileStats)], output_path: &Path) -> Result<()> {
        let html_content = self.time_wasted_generator.create_comprehensive_time_wasted_content(aggregated_stats, individual_files)?;
        fs::write(output_path, html_content)?;
        Ok(())
    }
    
    /// Auto-detect and generate the best possible report
    pub fn generate_auto_report(&self, stats: Option<&CodeStats>, aggregated_stats: Option<&AggregatedStats>, individual_files: &[(String, FileStats)], output_path: &Path) -> Result<()> {
        match (stats, aggregated_stats) {
            (_, Some(agg_stats)) => self.generate_comprehensive_report(agg_stats, individual_files, output_path),
            (Some(basic_stats), None) => self.generate_report(basic_stats, individual_files, output_path),
            (None, None) => Err(crate::utils::errors::HowManyError::invalid_config("No statistics provided for report generation".to_string())),
        }
    }
} 