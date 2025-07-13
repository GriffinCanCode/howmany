use crate::core::stats::basic::BasicStats;
use crate::core::stats::complexity::ComplexityStats;
use crate::core::stats::time::TimeStats;
use crate::core::stats::ratios::RatioStats;
use crate::core::types::{CodeStats, FileStats};
use crate::utils::errors::{Result, HowManyError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Aggregated statistics containing all types of statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedStats {
    pub basic: BasicStats,
    pub complexity: ComplexityStats,
    pub time: TimeStats,
    pub ratios: RatioStats,
    pub metadata: StatsMetadata,
}

/// Metadata about the statistics calculation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatsMetadata {
    pub calculation_time_ms: u64,
    pub version: String,
    pub timestamp: String,
    pub file_count_analyzed: usize,
    pub total_bytes_analyzed: u64,
    pub languages_detected: Vec<String>,
    pub analysis_depth: AnalysisDepth,
}

/// Depth of analysis performed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnalysisDepth {
    Basic,      // Only basic line counting
    Standard,   // Basic + ratios + time estimates
    Advanced,   // Standard + complexity analysis
    Complete,   // Advanced + all insights and quality metrics
}

/// Aggregator for combining different types of statistics
pub struct StatsAggregator {
    version: String,
}

impl StatsAggregator {
    pub fn new() -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
    
    /// Aggregate statistics for a single file
    pub fn aggregate_file_stats(
        &self,
        basic: BasicStats,
        complexity: ComplexityStats,
        time: TimeStats,
        ratios: RatioStats,
    ) -> AggregatedStats {
        let metadata = StatsMetadata {
            calculation_time_ms: 0, // Will be set by caller
            version: self.version.clone(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            file_count_analyzed: 1,
            total_bytes_analyzed: basic.total_size,
            languages_detected: vec!["unknown".to_string()], // Will be updated by caller
            analysis_depth: AnalysisDepth::Complete,
        };
        
        AggregatedStats {
            basic,
            complexity,
            time,
            ratios,
            metadata,
        }
    }
    
    /// Aggregate statistics for a project
    pub fn aggregate_project_stats(
        &self,
        basic: BasicStats,
        complexity: ComplexityStats,
        time: TimeStats,
        ratios: RatioStats,
    ) -> AggregatedStats {
        let languages_detected: Vec<String> = basic.stats_by_extension.keys().cloned().collect();
        
        let metadata = StatsMetadata {
            calculation_time_ms: 0, // Will be set by caller
            version: self.version.clone(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            file_count_analyzed: basic.total_files,
            total_bytes_analyzed: basic.total_size,
            languages_detected,
            analysis_depth: AnalysisDepth::Complete,
        };
        
        AggregatedStats {
            basic,
            complexity,
            time,
            ratios,
            metadata,
        }
    }
    
    /// Merge multiple aggregated statistics
    pub fn merge_stats(&self, stats_list: Vec<AggregatedStats>) -> Result<AggregatedStats> {
        if stats_list.is_empty() {
            return Err(HowManyError::invalid_config("Cannot merge empty statistics list".to_string()));
        }
        
        if stats_list.len() == 1 {
            return Ok(stats_list.into_iter().next().unwrap());
        }
        
        // Merge basic stats
        let mut merged_basic = self.merge_basic_stats(&stats_list)?;
        
        // Merge complexity stats
        let merged_complexity = self.merge_complexity_stats(&stats_list)?;
        
        // Merge time stats
        let merged_time = self.merge_time_stats(&stats_list)?;
        
        // Merge ratio stats
        let merged_ratios = self.merge_ratio_stats(&stats_list)?;
        
        // Create merged metadata
        let merged_metadata = self.merge_metadata(&stats_list)?;
        
        Ok(AggregatedStats {
            basic: merged_basic,
            complexity: merged_complexity,
            time: merged_time,
            ratios: merged_ratios,
            metadata: merged_metadata,
        })
    }
    
    /// Merge basic statistics
    fn merge_basic_stats(&self, stats_list: &[AggregatedStats]) -> Result<BasicStats> {
        let mut total_files = 0;
        let mut total_lines = 0;
        let mut code_lines = 0;
        let mut comment_lines = 0;
        let mut doc_lines = 0;
        let mut blank_lines = 0;
        let mut total_size = 0;
        let mut merged_extensions = HashMap::new();
        let mut all_file_sizes = Vec::new();
        
        for stats in stats_list {
            total_files += stats.basic.total_files;
            total_lines += stats.basic.total_lines;
            code_lines += stats.basic.code_lines;
            comment_lines += stats.basic.comment_lines;
            doc_lines += stats.basic.doc_lines;
            blank_lines += stats.basic.blank_lines;
            total_size += stats.basic.total_size;
            
            // Merge extension stats
            for (ext, ext_stats) in &stats.basic.stats_by_extension {
                let entry = merged_extensions.entry(ext.clone()).or_insert_with(|| {
                    crate::core::stats::basic::ExtensionStats {
                        file_count: 0,
                        total_lines: 0,
                        code_lines: 0,
                        comment_lines: 0,
                        doc_lines: 0,
                        blank_lines: 0,
                        total_size: 0,
                        average_lines_per_file: 0.0,
                        average_size_per_file: 0.0,
                    }
                });
                
                entry.file_count += ext_stats.file_count;
                entry.total_lines += ext_stats.total_lines;
                entry.code_lines += ext_stats.code_lines;
                entry.comment_lines += ext_stats.comment_lines;
                entry.doc_lines += ext_stats.doc_lines;
                entry.blank_lines += ext_stats.blank_lines;
                entry.total_size += ext_stats.total_size;
            }
            
            all_file_sizes.push(stats.basic.largest_file_size);
            all_file_sizes.push(stats.basic.smallest_file_size);
        }
        
        // Recalculate averages for merged extensions
        for ext_stats in merged_extensions.values_mut() {
            ext_stats.average_lines_per_file = if ext_stats.file_count > 0 {
                ext_stats.total_lines as f64 / ext_stats.file_count as f64
            } else {
                0.0
            };
            
            ext_stats.average_size_per_file = if ext_stats.file_count > 0 {
                ext_stats.total_size as f64 / ext_stats.file_count as f64
            } else {
                0.0
            };
        }
        
        let largest_file_size = all_file_sizes.iter().max().copied().unwrap_or(0);
        let smallest_file_size = all_file_sizes.iter().min().copied().unwrap_or(0);
        
        Ok(BasicStats {
            total_files,
            total_lines,
            code_lines,
            comment_lines,
            doc_lines,
            blank_lines,
            total_size,
            average_file_size: if total_files > 0 { total_size as f64 / total_files as f64 } else { 0.0 },
            average_lines_per_file: if total_files > 0 { total_lines as f64 / total_files as f64 } else { 0.0 },
            largest_file_size,
            smallest_file_size,
            stats_by_extension: merged_extensions,
        })
    }
    
    /// Merge complexity statistics
    fn merge_complexity_stats(&self, stats_list: &[AggregatedStats]) -> Result<ComplexityStats> {
        let mut total_functions = 0;
        let mut total_complexity = 0.0;
        let mut total_cognitive_complexity = 0.0;
        let mut total_maintainability = 0.0;
        let mut total_function_lines = 0;
        let mut max_function_length = 0;
        let mut min_function_length = usize::MAX;
        let mut max_nesting_depth = 0;
        let mut total_nesting_depth = 0.0;
        let mut total_parameters = 0;
        let mut max_parameters = 0;
        let mut merged_complexity_by_extension = HashMap::new();
        
        // Merge complexity distribution
        let mut merged_distribution = crate::core::stats::complexity::ComplexityDistribution {
            very_low_complexity: 0,
            low_complexity: 0,
            medium_complexity: 0,
            high_complexity: 0,
            very_high_complexity: 0,
        };
        
        for stats in stats_list {
            total_functions += stats.complexity.function_count;
            total_complexity += stats.complexity.cyclomatic_complexity * stats.complexity.function_count as f64;
            total_cognitive_complexity += stats.complexity.cognitive_complexity * stats.complexity.function_count as f64;
            total_maintainability += stats.complexity.maintainability_index * stats.complexity.function_count as f64;
            total_function_lines += (stats.complexity.average_function_length * stats.complexity.function_count as f64) as usize;
            max_function_length = max_function_length.max(stats.complexity.max_function_length);
            min_function_length = min_function_length.min(stats.complexity.min_function_length);
            max_nesting_depth = max_nesting_depth.max(stats.complexity.max_nesting_depth);
            total_nesting_depth += stats.complexity.average_nesting_depth * stats.complexity.function_count as f64;
            total_parameters += (stats.complexity.average_parameters_per_function * stats.complexity.function_count as f64) as usize;
            max_parameters = max_parameters.max(stats.complexity.max_parameters_per_function);
            
            // Merge complexity distribution
            merged_distribution.very_low_complexity += stats.complexity.complexity_distribution.very_low_complexity;
            merged_distribution.low_complexity += stats.complexity.complexity_distribution.low_complexity;
            merged_distribution.medium_complexity += stats.complexity.complexity_distribution.medium_complexity;
            merged_distribution.high_complexity += stats.complexity.complexity_distribution.high_complexity;
            merged_distribution.very_high_complexity += stats.complexity.complexity_distribution.very_high_complexity;
            
            // Merge extension complexity
            for (ext, ext_complexity) in &stats.complexity.complexity_by_extension {
                let entry = merged_complexity_by_extension.entry(ext.clone()).or_insert_with(|| {
                    crate::core::stats::complexity::ExtensionComplexity {
                        function_count: 0,
                        class_count: 0,
                        interface_count: 0,
                        trait_count: 0,
                        enum_count: 0,
                        struct_count: 0,
                        total_structures: 0,
                        cyclomatic_complexity: 0.0,
                        cognitive_complexity: 0.0,
                        maintainability_index: 0.0,
                        average_function_length: 0.0,
                        max_nesting_depth: 0,
                        average_nesting_depth: 0.0,
                        methods_per_class: 0.0,
                        average_parameters_per_function: 0.0,
                        quality_score: 0.0,
                    }
                });
                
                let old_count = entry.function_count;
                entry.function_count += ext_complexity.function_count;
                
                // Weighted average for complexity
                entry.cyclomatic_complexity = if entry.function_count > 0 {
                    (entry.cyclomatic_complexity * old_count as f64 + ext_complexity.cyclomatic_complexity * ext_complexity.function_count as f64) / entry.function_count as f64
                } else {
                    0.0
                };
                
                // Weighted average for cognitive complexity
                entry.cognitive_complexity = if entry.function_count > 0 {
                    (entry.cognitive_complexity * old_count as f64 + ext_complexity.cognitive_complexity * ext_complexity.function_count as f64) / entry.function_count as f64
                } else {
                    0.0
                };
                
                // Weighted average for maintainability
                entry.maintainability_index = if entry.function_count > 0 {
                    (entry.maintainability_index * old_count as f64 + ext_complexity.maintainability_index * ext_complexity.function_count as f64) / entry.function_count as f64
                } else {
                    0.0
                };
                
                // Weighted average for function length
                entry.average_function_length = if entry.function_count > 0 {
                    (entry.average_function_length * old_count as f64 + ext_complexity.average_function_length * ext_complexity.function_count as f64) / entry.function_count as f64
                } else {
                    0.0
                };
                
                entry.max_nesting_depth = entry.max_nesting_depth.max(ext_complexity.max_nesting_depth);
                
                // Weighted average for nesting depth
                entry.average_nesting_depth = if entry.function_count > 0 {
                    (entry.average_nesting_depth * old_count as f64 + ext_complexity.average_nesting_depth * ext_complexity.function_count as f64) / entry.function_count as f64
                } else {
                    0.0
                };
                
                // Weighted average for parameters
                entry.average_parameters_per_function = if entry.function_count > 0 {
                    (entry.average_parameters_per_function * old_count as f64 + ext_complexity.average_parameters_per_function * ext_complexity.function_count as f64) / entry.function_count as f64
                } else {
                    0.0
                };
                
                // Weighted average for quality score
                entry.quality_score = if entry.function_count > 0 {
                    (entry.quality_score * old_count as f64 + ext_complexity.quality_score * ext_complexity.function_count as f64) / entry.function_count as f64
                } else {
                    0.0
                };
            }
        }
        
        // Merge quality metrics
        let mut merged_quality_metrics = crate::core::stats::complexity::QualityMetrics {
            overall_quality_score: 0.0,
            maintainability_score: 0.0,
            readability_score: 0.0,
            testability_score: 0.0,
            code_duplication_ratio: 0.0,
            comment_coverage_ratio: 0.0,
            function_size_score: 0.0,
            complexity_score: 0.0,
        };
        
        if !stats_list.is_empty() {
            for stats in stats_list {
                merged_quality_metrics.overall_quality_score += stats.complexity.quality_metrics.overall_quality_score;
                merged_quality_metrics.maintainability_score += stats.complexity.quality_metrics.maintainability_score;
                merged_quality_metrics.readability_score += stats.complexity.quality_metrics.readability_score;
                merged_quality_metrics.testability_score += stats.complexity.quality_metrics.testability_score;
                merged_quality_metrics.code_duplication_ratio += stats.complexity.quality_metrics.code_duplication_ratio;
                merged_quality_metrics.comment_coverage_ratio += stats.complexity.quality_metrics.comment_coverage_ratio;
                merged_quality_metrics.function_size_score += stats.complexity.quality_metrics.function_size_score;
                merged_quality_metrics.complexity_score += stats.complexity.quality_metrics.complexity_score;
            }
            
            let stats_count = stats_list.len() as f64;
            merged_quality_metrics.overall_quality_score /= stats_count;
            merged_quality_metrics.maintainability_score /= stats_count;
            merged_quality_metrics.readability_score /= stats_count;
            merged_quality_metrics.testability_score /= stats_count;
            merged_quality_metrics.code_duplication_ratio /= stats_count;
            merged_quality_metrics.comment_coverage_ratio /= stats_count;
            merged_quality_metrics.function_size_score /= stats_count;
            merged_quality_metrics.complexity_score /= stats_count;
        }
        
        Ok(ComplexityStats {
            function_count: total_functions,
            class_count: 0,
            interface_count: 0,
            trait_count: 0,
            enum_count: 0,
            struct_count: 0,
            module_count: 0,
            total_structures: 0,
            cyclomatic_complexity: if total_functions > 0 { total_complexity / total_functions as f64 } else { 0.0 },
            cognitive_complexity: if total_functions > 0 { total_cognitive_complexity / total_functions as f64 } else { 0.0 },
            maintainability_index: if total_functions > 0 { total_maintainability / total_functions as f64 } else { 100.0 },
            average_function_length: if total_functions > 0 { total_function_lines as f64 / total_functions as f64 } else { 0.0 },
            max_function_length,
            min_function_length: if min_function_length == usize::MAX { 0 } else { min_function_length },
            max_nesting_depth,
            average_nesting_depth: if total_functions > 0 { total_nesting_depth / total_functions as f64 } else { 0.0 },
            methods_per_class: 0.0,
            average_parameters_per_function: if total_functions > 0 { total_parameters as f64 / total_functions as f64 } else { 0.0 },
            max_parameters_per_function: max_parameters,
            complexity_by_extension: merged_complexity_by_extension,
            complexity_distribution: merged_distribution,
            structure_distribution: crate::core::stats::complexity::StructureDistribution {
                classes: 0,
                interfaces: 0,
                traits: 0,
                enums: 0,
                structs: 0,
                modules: 0,
            },
            function_complexity_details: Vec::new(),
            quality_metrics: merged_quality_metrics,
        })
    }
    
    /// Merge time statistics
    fn merge_time_stats(&self, stats_list: &[AggregatedStats]) -> Result<TimeStats> {
        let mut total_time_minutes = 0;
        let mut code_time_minutes = 0;
        let mut doc_time_minutes = 0;
        let mut comment_time_minutes = 0;
        let mut merged_time_by_extension = HashMap::new();
        
        for stats in stats_list {
            total_time_minutes += stats.time.total_time_minutes;
            code_time_minutes += stats.time.code_time_minutes;
            doc_time_minutes += stats.time.doc_time_minutes;
            comment_time_minutes += stats.time.comment_time_minutes;
            
            // Merge extension time stats
            for (ext, ext_time) in &stats.time.time_by_extension {
                let entry = merged_time_by_extension.entry(ext.clone()).or_insert_with(|| {
                    crate::core::stats::time::ExtensionTimeStats {
                        total_time_minutes: 0,
                        code_time_minutes: 0,
                        doc_time_minutes: 0,
                        comment_time_minutes: 0,
                        formatted_time: String::new(),
                        average_time_per_file: 0.0,
                    }
                });
                
                entry.total_time_minutes += ext_time.total_time_minutes;
                entry.code_time_minutes += ext_time.code_time_minutes;
                entry.doc_time_minutes += ext_time.doc_time_minutes;
                entry.comment_time_minutes += ext_time.comment_time_minutes;
            }
        }
        
        // Recalculate formatted times and averages
        let time_calculator = crate::core::stats::time::TimeStatsCalculator::new();
        for ext_time in merged_time_by_extension.values_mut() {
            ext_time.formatted_time = time_calculator.format_time_human(ext_time.total_time_minutes);
            // Note: average_time_per_file would need file count from basic stats to calculate properly
        }
        
        // Calculate productivity metrics
        let total_files: usize = stats_list.iter().map(|s| s.basic.total_files).sum();
        let total_lines: usize = stats_list.iter().map(|s| s.basic.total_lines).sum();
        let total_code_lines: usize = stats_list.iter().map(|s| s.basic.code_lines).sum();
        
        let total_hours = total_time_minutes as f64 / 60.0;
        let productivity_metrics = crate::core::stats::time::ProductivityMetrics {
            lines_per_hour: if total_hours > 0.0 { total_lines as f64 / total_hours } else { 0.0 },
            code_lines_per_hour: if total_hours > 0.0 { total_code_lines as f64 / total_hours } else { 0.0 },
            files_per_hour: if total_hours > 0.0 { total_files as f64 / total_hours } else { 0.0 },
            estimated_development_hours: total_hours,
            estimated_development_days: total_hours / 8.0,
        };
        
        Ok(TimeStats {
            total_time_minutes,
            code_time_minutes,
            doc_time_minutes,
            comment_time_minutes,
            total_time_formatted: time_calculator.format_time_human(total_time_minutes),
            code_time_formatted: time_calculator.format_time_human(code_time_minutes),
            doc_time_formatted: time_calculator.format_time_human(doc_time_minutes),
            comment_time_formatted: time_calculator.format_time_human(comment_time_minutes),
            time_by_extension: merged_time_by_extension,
            productivity_metrics,
        })
    }
    
    /// Merge ratio statistics
    fn merge_ratio_stats(&self, stats_list: &[AggregatedStats]) -> Result<RatioStats> {
        // Calculate overall ratios from merged basic stats
        let total_lines: usize = stats_list.iter().map(|s| s.basic.total_lines).sum();
        let total_code_lines: usize = stats_list.iter().map(|s| s.basic.code_lines).sum();
        let comment_lines = stats_list.iter().map(|s| s.basic.comment_lines).sum::<usize>();
        let doc_lines = stats_list.iter().map(|s| s.basic.doc_lines).sum::<usize>();
        let blank_lines = stats_list.iter().map(|s| s.basic.blank_lines).sum::<usize>();
        
        let total_lines_f64 = total_lines as f64;
        let code_ratio = if total_lines_f64 > 0.0 { total_code_lines as f64 / total_lines_f64 } else { 0.0 };
        let comment_ratio = if total_lines_f64 > 0.0 { comment_lines as f64 / total_lines_f64 } else { 0.0 };
        let doc_ratio = if total_lines_f64 > 0.0 { doc_lines as f64 / total_lines_f64 } else { 0.0 };
        let blank_ratio = if total_lines_f64 > 0.0 { blank_lines as f64 / total_lines_f64 } else { 0.0 };
        
        let comment_to_code_ratio = if total_code_lines > 0 { comment_lines as f64 / total_code_lines as f64 } else { 0.0 };
        let doc_to_code_ratio = if total_code_lines > 0 { doc_lines as f64 / total_code_lines as f64 } else { 0.0 };
        
        // Use the first stats' ratio calculator to recalculate everything
        let ratio_calculator = crate::core::stats::ratios::RatioStatsCalculator::new();
        
        // Create a temporary CodeStats for recalculation
        let mut temp_stats_by_extension = HashMap::new();
        for stats in stats_list {
            for (ext, ext_stats) in &stats.basic.stats_by_extension {
                let entry = temp_stats_by_extension.entry(ext.clone()).or_insert((0, FileStats {
                    total_lines: 0,
                    code_lines: 0,
                    comment_lines: 0,
                    blank_lines: 0,
                    file_size: 0,
                    doc_lines: 0,
                }));
                
                entry.0 += ext_stats.file_count;
                entry.1.total_lines += ext_stats.total_lines;
                entry.1.code_lines += ext_stats.code_lines;
                entry.1.comment_lines += ext_stats.comment_lines;
                entry.1.doc_lines += ext_stats.doc_lines;
                entry.1.blank_lines += ext_stats.blank_lines;
                entry.1.file_size += ext_stats.total_size;
            }
        }
        
        let temp_code_stats = CodeStats {
            total_files: stats_list.iter().map(|s| s.basic.total_files).sum(),
            total_lines,
            total_code_lines: total_code_lines,
            total_comment_lines: comment_lines,
            total_blank_lines: blank_lines,
            total_size: stats_list.iter().map(|s| s.basic.total_size).sum(),
            total_doc_lines: doc_lines,
            stats_by_extension: temp_stats_by_extension,
        };
        
        ratio_calculator.calculate_project_ratio_stats(&temp_code_stats)
    }
    
    /// Merge metadata
    fn merge_metadata(&self, stats_list: &[AggregatedStats]) -> Result<StatsMetadata> {
        let total_calculation_time = stats_list.iter().map(|s| s.metadata.calculation_time_ms).sum();
        let total_files = stats_list.iter().map(|s| s.metadata.file_count_analyzed).sum();
        let total_bytes = stats_list.iter().map(|s| s.metadata.total_bytes_analyzed).sum();
        
        let mut all_languages = std::collections::HashSet::new();
        for stats in stats_list {
            for lang in &stats.metadata.languages_detected {
                all_languages.insert(lang.clone());
            }
        }
        
        let mut languages_detected: Vec<String> = all_languages.into_iter().collect();
        languages_detected.sort();
        
        Ok(StatsMetadata {
            calculation_time_ms: total_calculation_time,
            version: self.version.clone(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            file_count_analyzed: total_files,
            total_bytes_analyzed: total_bytes,
            languages_detected,
            analysis_depth: AnalysisDepth::Complete,
        })
    }
    
    /// Get summary statistics
    pub fn get_summary(&self, stats: &AggregatedStats) -> HashMap<String, String> {
        let mut summary = HashMap::new();
        
        summary.insert("total_files".to_string(), stats.basic.total_files.to_string());
        summary.insert("total_lines".to_string(), stats.basic.total_lines.to_string());
        summary.insert("code_lines".to_string(), stats.basic.code_lines.to_string());
        summary.insert("functions".to_string(), stats.complexity.function_count.to_string());
        summary.insert("avg_complexity".to_string(), format!("{:.1}", stats.complexity.cyclomatic_complexity));
        summary.insert("total_time".to_string(), stats.time.total_time_formatted.clone());
        summary.insert("quality_score".to_string(), format!("{:.1}", stats.ratios.quality_metrics.overall_quality_score));
        summary.insert("languages".to_string(), stats.metadata.languages_detected.len().to_string());
        
        summary
    }
    
    /// Update metadata with timing information
    pub fn update_timing(stats: &mut AggregatedStats, start_time: std::time::Instant) {
        stats.metadata.calculation_time_ms = start_time.elapsed().as_millis() as u64;
    }
}

impl Default for StatsAggregator {
    fn default() -> Self {
        Self::new()
    }
} 