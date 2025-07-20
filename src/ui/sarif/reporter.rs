use crate::core::types::{CodeStats, FileStats};
use crate::core::stats::AggregatedStats;
use crate::utils::errors::Result;
use super::converter::SarifConverter;
use std::fs;
use std::path::Path;

pub struct SarifReporter {
    converter: SarifConverter,
}

impl SarifReporter {
    pub fn new() -> Self {
        Self {
            converter: SarifConverter::new(),
        }
    }

    /// Generate SARIF report from basic CodeStats
    pub fn generate_report(
        &self,
        stats: &CodeStats,
        individual_files: &[(String, FileStats)],
        output_path: &Path,
    ) -> Result<()> {
        let sarif_log = self.converter.convert_basic_analysis(stats, individual_files)?;
        let sarif_content = serde_json::to_string_pretty(&sarif_log)
            .map_err(|e| crate::utils::errors::HowManyError::display(format!("SARIF serialization failed: {}", e)))?;
        
        fs::write(output_path, sarif_content)
            .map_err(|e| crate::utils::errors::HowManyError::file_processing(format!("Failed to write SARIF file: {}", e)))?;
        
        Ok(())
    }

    /// Generate comprehensive SARIF report from AggregatedStats
    pub fn generate_comprehensive_report(
        &self,
        aggregated_stats: &AggregatedStats,
        individual_files: &[(String, FileStats)],
        output_path: &Path,
    ) -> Result<()> {
        let sarif_log = self.converter.convert_comprehensive_analysis(aggregated_stats, individual_files)?;
        let sarif_content = serde_json::to_string_pretty(&sarif_log)
            .map_err(|e| crate::utils::errors::HowManyError::display(format!("SARIF serialization failed: {}", e)))?;
        
        fs::write(output_path, sarif_content)
            .map_err(|e| crate::utils::errors::HowManyError::file_processing(format!("Failed to write SARIF file: {}", e)))?;
        
        Ok(())
    }

    /// Auto-detect and generate the best possible SARIF report
    pub fn generate_auto_report(
        &self,
        stats: Option<&CodeStats>,
        aggregated_stats: Option<&AggregatedStats>,
        individual_files: &[(String, FileStats)],
        output_path: &Path,
    ) -> Result<()> {
        match (stats, aggregated_stats) {
            (_, Some(agg_stats)) => {
                self.generate_comprehensive_report(agg_stats, individual_files, output_path)
            }
            (Some(basic_stats), None) => {
                self.generate_report(basic_stats, individual_files, output_path)
            }
            (None, None) => {
                Err(crate::utils::errors::HowManyError::invalid_config(
                    "No statistics provided for SARIF report generation".to_string()
                ))
            }
        }
    }

    /// Generate SARIF content as string without writing to file
    pub fn generate_sarif_string(
        &self,
        stats: Option<&CodeStats>,
        aggregated_stats: Option<&AggregatedStats>,
        individual_files: &[(String, FileStats)],
    ) -> Result<String> {
        let sarif_log = match (stats, aggregated_stats) {
            (_, Some(agg_stats)) => {
                self.converter.convert_comprehensive_analysis(agg_stats, individual_files)?
            }
            (Some(basic_stats), None) => {
                self.converter.convert_basic_analysis(basic_stats, individual_files)?
            }
            (None, None) => {
                return Err(crate::utils::errors::HowManyError::invalid_config(
                    "No statistics provided for SARIF generation".to_string()
                ));
            }
        };

        serde_json::to_string_pretty(&sarif_log)
            .map_err(|e| crate::utils::errors::HowManyError::display(format!("SARIF serialization failed: {}", e)))
    }

    /// Validate that the generated SARIF is well-formed
    pub fn validate_sarif_output(&self, sarif_content: &str) -> Result<()> {
        // Try to parse the SARIF content back to ensure it's valid JSON
        let _: serde_json::Value = serde_json::from_str(sarif_content)
            .map_err(|e| crate::utils::errors::HowManyError::invalid_config(
                format!("Generated SARIF is not valid JSON: {}", e)
            ))?;

        // Additional SARIF-specific validation could be added here
        // For now, we rely on the serde-sarif library to ensure structure compliance
        
        Ok(())
    }

    /// Get the recommended file extension for SARIF files
    pub fn get_file_extension() -> &'static str {
        "sarif"
    }

    /// Get the MIME type for SARIF files
    pub fn get_mime_type() -> &'static str {
        "application/sarif+json"
    }

    /// Create a default output filename for SARIF reports
    pub fn default_filename() -> String {
        use chrono::Utc;
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        format!("howmany-report-{}.sarif", timestamp)
    }
}

impl Default for SarifReporter {
    fn default() -> Self {
        Self::new()
    }
} 