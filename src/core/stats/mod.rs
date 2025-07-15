pub mod basic;
pub mod complexity;
pub mod time;
pub mod ratios;
pub mod formatting;
pub mod aggregation;
pub mod visualization;

// Re-export commonly used types
pub use basic::{BasicStats, BasicStatsCalculator};
pub use complexity::{ComplexityStats, ComplexityStatsCalculator};
pub use time::{TimeStats, TimeStatsCalculator};
pub use ratios::{RatioStats, RatioStatsCalculator};
pub use formatting::{StatFormatter, FormattingOptions, OutputFormat, SortBy};
pub use aggregation::{StatsAggregator, AggregatedStats, StatsMetadata, AnalysisDepth};
pub use visualization::{VisualizationGenerator, PieChartData, ChartConfig, ColorScheme};



use crate::core::types::{CodeStats, FileStats};
use crate::utils::errors::Result;

/// Centralized statistics calculator that coordinates all statistics calculations
pub struct StatsCalculator {
    basic_calculator: BasicStatsCalculator,
    complexity_calculator: ComplexityStatsCalculator,
    time_calculator: TimeStatsCalculator,
    ratio_calculator: RatioStatsCalculator,
    formatter: StatFormatter,
    aggregator: StatsAggregator,
    visualization_generator: VisualizationGenerator,
}

impl StatsCalculator {
    pub fn new() -> Self {
        Self {
            basic_calculator: BasicStatsCalculator::new(),
            complexity_calculator: ComplexityStatsCalculator::new(),
            time_calculator: TimeStatsCalculator::new(),
            ratio_calculator: RatioStatsCalculator::new(),
            formatter: StatFormatter::new(),
            aggregator: StatsAggregator::new(),
            visualization_generator: VisualizationGenerator::new(),
        }
    }
    
    /// Calculate comprehensive statistics for a single file
    pub fn calculate_file_stats(&self, file_stats: &FileStats, file_path: &str) -> Result<AggregatedStats> {
        let basic_stats = self.basic_calculator.calculate_basic_stats(file_stats)?;
        let complexity_stats = self.complexity_calculator.calculate_complexity_stats(file_stats, file_path)?;
        let time_stats = self.time_calculator.calculate_time_stats(file_stats)?;
        let ratio_stats = self.ratio_calculator.calculate_ratio_stats(file_stats)?;
        
        Ok(self.aggregator.aggregate_file_stats(
            basic_stats,
            complexity_stats,
            time_stats,
            ratio_stats,
        ))
    }
    
    /// Calculate comprehensive statistics for a collection of files
    pub fn calculate_project_stats(&self, code_stats: &CodeStats, individual_files: &[(String, FileStats)]) -> Result<AggregatedStats> {
        let basic_stats = self.basic_calculator.calculate_project_basic_stats(code_stats)?;
        let complexity_stats = self.complexity_calculator.calculate_project_complexity_stats(code_stats, individual_files)?;
        let time_stats = self.time_calculator.calculate_project_time_stats_with_files(code_stats, individual_files)?;
        let ratio_stats = self.ratio_calculator.calculate_project_ratio_stats(code_stats)?;
        
        Ok(self.aggregator.aggregate_project_stats(
            basic_stats,
            complexity_stats,
            time_stats,
            ratio_stats,
        ))
    }
    
    /// Get formatted statistics for display
    pub fn format_stats(&self, stats: &AggregatedStats, options: &FormattingOptions) -> Result<String> {
        self.formatter.format_stats(stats, options)
    }
    
    /// Generate language distribution pie chart data
    pub fn generate_language_distribution(&self, stats: &AggregatedStats, config: &ChartConfig) -> PieChartData {
        self.visualization_generator.generate_language_distribution(stats, config)
    }
    
    /// Generate file count distribution pie chart data
    pub fn generate_file_count_distribution(&self, stats: &AggregatedStats, config: &ChartConfig) -> PieChartData {
        self.visualization_generator.generate_file_count_distribution(stats, config)
    }
    
    /// Generate code structure distribution pie chart data
    pub fn generate_structure_distribution(&self, stats: &AggregatedStats, config: &ChartConfig) -> PieChartData {
        self.visualization_generator.generate_structure_distribution(stats, config)
    }
    
    /// Generate complexity distribution pie chart data
    pub fn generate_complexity_distribution(&self, stats: &AggregatedStats, config: &ChartConfig) -> PieChartData {
        self.visualization_generator.generate_complexity_distribution(stats, config)
    }
    
    /// Generate line type distribution pie chart data
    pub fn generate_line_type_distribution(&self, stats: &AggregatedStats, config: &ChartConfig) -> PieChartData {
        self.visualization_generator.generate_line_type_distribution(stats, config)
    }
    
    /// Convert pie chart data to Chart.js format
    pub fn to_chartjs_format(&self, data: &PieChartData, config: &ChartConfig) -> serde_json::Value {
        self.visualization_generator.to_chartjs_format(data, config)
    }
    
    /// Get comprehensive summary of statistics
    pub fn get_comprehensive_summary(&self, stats: &AggregatedStats) -> std::collections::HashMap<String, String> {
        let mut summary = self.aggregator.get_summary(stats);
        
        // Add additional insights
        summary.insert("analysis_depth".to_string(), format!("{:?}", stats.metadata.analysis_depth));
        summary.insert("calculation_time".to_string(), format!("{}ms", stats.metadata.calculation_time_ms));
        summary.insert("version".to_string(), stats.metadata.version.clone());
        
        // Quality insights
        if stats.ratios.quality_metrics.overall_quality_score >= 80.0 {
            summary.insert("quality_level".to_string(), "Excellent".to_string());
        } else if stats.ratios.quality_metrics.overall_quality_score >= 60.0 {
            summary.insert("quality_level".to_string(), "Good".to_string());
        } else {
            summary.insert("quality_level".to_string(), "Needs Improvement".to_string());
        }
        
        // Complexity insights
        if stats.complexity.cyclomatic_complexity <= 5.0 {
            summary.insert("complexity_level".to_string(), "Low".to_string());
        } else if stats.complexity.cyclomatic_complexity <= 10.0 {
            summary.insert("complexity_level".to_string(), "Medium".to_string());
        } else {
            summary.insert("complexity_level".to_string(), "High".to_string());
        }
        
        summary
    }
    
    /// Convert AggregatedStats to CodeStats for backward compatibility
    pub fn to_code_stats(&self, aggregated_stats: &AggregatedStats) -> CodeStats {
        CodeStats {
            total_files: aggregated_stats.basic.total_files,
            total_lines: aggregated_stats.basic.total_lines,
            total_code_lines: aggregated_stats.basic.code_lines,
            total_comment_lines: aggregated_stats.basic.comment_lines,
            total_blank_lines: aggregated_stats.basic.blank_lines,
            total_size: aggregated_stats.basic.total_size,
            total_doc_lines: aggregated_stats.basic.doc_lines,
            stats_by_extension: aggregated_stats.basic.stats_by_extension.iter()
                .map(|(ext, ext_stats)| {
                    (ext.clone(), (ext_stats.file_count, FileStats {
                        total_lines: ext_stats.total_lines,
                        code_lines: ext_stats.code_lines,
                        comment_lines: ext_stats.comment_lines,
                        blank_lines: ext_stats.blank_lines,
                        file_size: ext_stats.total_size,
                        doc_lines: ext_stats.doc_lines,
                    }))
                })
                .collect(),
        }
    }
    
    /// Get the basic calculator for direct access
    pub fn basic_calculator(&self) -> &BasicStatsCalculator {
        &self.basic_calculator
    }
    
    /// Get the complexity calculator for direct access
    pub fn complexity_calculator(&self) -> &ComplexityStatsCalculator {
        &self.complexity_calculator
    }
    
    /// Get the time calculator for direct access
    pub fn time_calculator(&self) -> &TimeStatsCalculator {
        &self.time_calculator
    }
    
    /// Get the ratio calculator for direct access
    pub fn ratio_calculator(&self) -> &RatioStatsCalculator {
        &self.ratio_calculator
    }
    
    /// Get the formatter for direct access
    pub fn formatter(&self) -> &StatFormatter {
        &self.formatter
    }
    
    /// Get the aggregator for direct access
    pub fn aggregator(&self) -> &StatsAggregator {
        &self.aggregator
    }
    
    /// Get the visualization generator for direct access
    pub fn visualization_generator(&self) -> &VisualizationGenerator {
        &self.visualization_generator
    }
}

impl Default for StatsCalculator {
    fn default() -> Self {
        Self::new()
    }
}

/// Utility functions for stats integration across the codebase
pub mod integration {
    use super::*;
    
    /// Create a comprehensive stats pipeline for any analysis
    pub fn create_comprehensive_pipeline() -> StatsCalculator {
        StatsCalculator::new()
    }
    
    /// Quick stats calculation with sensible defaults
    pub fn quick_project_analysis(code_stats: &CodeStats, individual_files: &[(String, FileStats)]) -> Result<AggregatedStats> {
        let calculator = StatsCalculator::new();
        calculator.calculate_project_stats(code_stats, individual_files)
    }
    
    /// Get formatted output with default options
    pub fn format_for_display(stats: &AggregatedStats, format: OutputFormat) -> Result<String> {
        let calculator = StatsCalculator::new();
        let options = FormattingOptions {
            format,
            show_percentages: true,
            show_ratios: true,
            decimal_places: 1,
            use_emojis: true,
            color_output: true,
            compact_mode: false,
            sort_by: SortBy::Lines,
            sort_descending: true,
            max_items: None,
        };
        calculator.format_stats(stats, &options)
    }
    
    /// Generate visualization data for web interfaces
    pub fn generate_web_charts(stats: &AggregatedStats) -> Result<serde_json::Value> {
        let calculator = StatsCalculator::new();
        let config = ChartConfig::default();
        
        let mut charts = serde_json::Map::new();
        
        let language_dist = calculator.generate_language_distribution(stats, &config);
        charts.insert("language_distribution".to_string(), calculator.to_chartjs_format(&language_dist, &config));
        
        let complexity_dist = calculator.generate_complexity_distribution(stats, &config);
        charts.insert("complexity_distribution".to_string(), calculator.to_chartjs_format(&complexity_dist, &config));
        
        let line_type_dist = calculator.generate_line_type_distribution(stats, &config);
        charts.insert("line_type_distribution".to_string(), calculator.to_chartjs_format(&line_type_dist, &config));
        
        Ok(serde_json::Value::Object(charts))
    }
} 