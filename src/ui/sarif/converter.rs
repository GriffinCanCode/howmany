use crate::core::types::{CodeStats, FileStats};
use crate::core::stats::AggregatedStats;
use crate::utils::errors::Result;
use serde_sarif::sarif::{
    Sarif, Run, Tool, ToolComponent, Result as SarifResult, 
    Location, PhysicalLocation, ArtifactLocation, Region,
    Message, ReportingDescriptor, MultiformatMessageString,
    RunAutomationDetails, ResultKind, ResultLevel
};
use serde_json::Value;
use chrono::Utc;

pub struct SarifConverter {
    tool_name: String,
    tool_version: String,
}

impl SarifConverter {
    pub fn new() -> Self {
        Self {
            tool_name: "howmany".to_string(),
            tool_version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }

    /// Get the tool name
    pub fn tool_name(&self) -> &str {
        &self.tool_name
    }

    /// Get the tool version
    pub fn tool_version(&self) -> &str {
        &self.tool_version
    }

    /// Convert CodeStats and file analysis to SARIF format
    pub fn convert_basic_analysis(
        &self,
        stats: &CodeStats,
        individual_files: &[(String, FileStats)],
    ) -> Result<Sarif> {
        let mut results = Vec::new();

        // Create results for each file with potential issues
        for (file_path, file_stats) in individual_files {
            // Generate code quality results
            if let Some(quality_results) = self.analyze_file_quality(file_path, file_stats) {
                results.extend(quality_results);
            }
        }

        // Add project-level summary results
        results.extend(self.create_project_summary_results(stats));

        self.create_sarif_log(results)
    }

    /// Convert comprehensive AggregatedStats to SARIF format
    pub fn convert_comprehensive_analysis(
        &self,
        aggregated_stats: &AggregatedStats,
        individual_files: &[(String, FileStats)],
    ) -> Result<Sarif> {
        let mut results = Vec::new();

        // Generate detailed complexity and quality results
        for (file_path, file_stats) in individual_files {
            // Complexity analysis results
            if let Some(complexity_results) = self.analyze_file_complexity(file_path, file_stats, aggregated_stats) {
                results.extend(complexity_results);
            }

            // Quality metrics results
            if let Some(quality_results) = self.analyze_comprehensive_quality(file_path, file_stats, aggregated_stats) {
                results.extend(quality_results);
            }
        }

        // Add comprehensive project-level results
        results.extend(self.create_comprehensive_project_results(aggregated_stats));

        self.create_sarif_log(results)
    }

    /// Analyze individual file for basic quality issues
    fn analyze_file_quality(&self, file_path: &str, file_stats: &FileStats) -> Option<Vec<SarifResult>> {
        let mut results = Vec::new();

        // Large file warning
        if file_stats.total_lines > 1000 {
            results.push(self.create_result(
                "HM001",
                "Large File",
                &format!("File has {} lines, consider breaking it into smaller modules", file_stats.total_lines),
                "warning",
                file_path,
                None,
            ));
        }

        // Low documentation ratio
        let doc_ratio = if file_stats.total_lines > 0 {
            (file_stats.doc_lines + file_stats.comment_lines) as f64 / file_stats.total_lines as f64
        } else {
            0.0
        };

        if doc_ratio < 0.1 && file_stats.code_lines > 50 {
            results.push(self.create_result(
                "HM002",
                "Low Documentation",
                &format!("Documentation ratio is {:.1}%, consider adding more comments and documentation", doc_ratio * 100.0),
                "info",
                file_path,
                None,
            ));
        }

        // Empty or very small files
        if file_stats.code_lines == 0 {
            results.push(self.create_result(
                "HM003",
                "Empty File",
                "File contains no code lines, consider removing if unused",
                "note",
                file_path,
                None,
            ));
        }

        if results.is_empty() { None } else { Some(results) }
    }

    /// Analyze file complexity using AggregatedStats
    fn analyze_file_complexity(
        &self,
        file_path: &str,
        _file_stats: &FileStats,
        aggregated_stats: &AggregatedStats,
    ) -> Option<Vec<SarifResult>> {
        let mut results = Vec::new();

        // Get file extension for complexity analysis
        let extension = std::path::Path::new(file_path)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("unknown");

        // Check if we have complexity data for this extension
        if let Some(complexity_data) = aggregated_stats.complexity.complexity_by_extension.get(extension) {
            // High complexity warning
            if complexity_data.cyclomatic_complexity > 15.0 {
                results.push(self.create_result(
                    "HM101",
                    "High Complexity",
                    &format!("Average cyclomatic complexity is {:.1}, consider refactoring", complexity_data.cyclomatic_complexity),
                    "warning",
                    file_path,
                    None,
                ));
            }

            // Cognitive complexity issues
            if complexity_data.cognitive_complexity > 20.0 {
                results.push(self.create_result(
                    "HM102",
                    "High Cognitive Complexity",
                    &format!("Cognitive complexity is {:.1}, may be difficult to understand", complexity_data.cognitive_complexity),
                    "warning",
                    file_path,
                    None,
                ));
            }

            // Deep nesting warning
            if complexity_data.max_nesting_depth > 5 {
                results.push(self.create_result(
                    "HM103",
                    "Deep Nesting",
                    &format!("Maximum nesting depth is {}, consider extracting nested logic", complexity_data.max_nesting_depth),
                    "info",
                    file_path,
                    None,
                ));
            }
        }

        if results.is_empty() { None } else { Some(results) }
    }

    /// Analyze comprehensive quality metrics
    fn analyze_comprehensive_quality(
        &self,
        file_path: &str,
        _file_stats: &FileStats,
        aggregated_stats: &AggregatedStats,
    ) -> Option<Vec<SarifResult>> {
        let mut results = Vec::new();

        // Maintainability issues
        if aggregated_stats.complexity.quality_metrics.maintainability_index < 50.0 {
            results.push(self.create_result(
                "HM201",
                "Low Maintainability",
                &format!("Maintainability index is {:.1}/100, consider refactoring", 
                    aggregated_stats.complexity.quality_metrics.maintainability_index),
                "warning",
                file_path,
                None,
            ));
        }

        // Code health issues
        if aggregated_stats.complexity.quality_metrics.code_health_score < 60.0 {
            results.push(self.create_result(
                "HM202",
                "Poor Code Health",
                &format!("Code health score is {:.1}/100, review code quality practices", 
                    aggregated_stats.complexity.quality_metrics.code_health_score),
                "info",
                file_path,
                None,
            ));
        }

        if results.is_empty() { None } else { Some(results) }
    }

    /// Create project-level summary results
    fn create_project_summary_results(&self, stats: &CodeStats) -> Vec<SarifResult> {
        let mut results = Vec::new();

        // Project size summary
        results.push(self.create_result(
            "HM000",
            "Project Summary",
            &format!("Project contains {} files with {} total lines ({} code, {} comments)", 
                stats.total_files, stats.total_lines, stats.total_code_lines, stats.total_comment_lines),
            "note",
            "project://summary",
            None,
        ));

        // Large project warning
        if stats.total_files > 1000 {
            results.push(self.create_result(
                "HM301",
                "Large Project",
                &format!("Project has {} files, consider modularization strategies", stats.total_files),
                "info",
                "project://summary",
                None,
            ));
        }

        results
    }

    /// Create comprehensive project-level results
    fn create_comprehensive_project_results(&self, aggregated_stats: &AggregatedStats) -> Vec<SarifResult> {
        let mut results = Vec::new();

        // Comprehensive project summary
        results.push(self.create_result(
            "HM000",
            "Comprehensive Project Summary",
            &format!("Project: {} files, {} functions, {:.1} avg complexity, {:.1}/100 quality score", 
                aggregated_stats.basic.total_files,
                aggregated_stats.complexity.function_count,
                aggregated_stats.complexity.cyclomatic_complexity,
                aggregated_stats.complexity.quality_metrics.code_health_score),
            "note",
            "project://summary",
            None,
        ));

        // Technical debt indicators
        if aggregated_stats.complexity.quality_metrics.code_health_score < 70.0 {
            results.push(self.create_result(
                "HM401",
                "Technical Debt Alert",
                &format!("Overall code health is {:.1}/100. Consider prioritizing refactoring efforts", 
                    aggregated_stats.complexity.quality_metrics.code_health_score),
                "warning",
                "project://technical-debt",
                None,
            ));
        }

        // Development time insights
        results.push(self.create_result(
            "HM402",
            "Development Time Analysis",
            &format!("Estimated development time: {} (Code: {}, Docs: {})", 
                aggregated_stats.time.total_time_formatted,
                aggregated_stats.time.code_time_formatted,
                aggregated_stats.time.doc_time_formatted),
            "note",
            "project://time-analysis",
            None,
        ));

        results
    }

    /// Create a SARIF result object
    fn create_result(
        &self,
        rule_id: &str,
        _rule_name: &str,
        message_text: &str,
        level: &str,
        file_path: &str,
        region: Option<Region>,
    ) -> SarifResult {
        let result_level = match level {
            "error" => ResultLevel::Error,
            "warning" => ResultLevel::Warning,
            "note" => ResultLevel::Note,
            "info" => ResultLevel::None, // Use None as fallback since Info doesn't exist
            _ => ResultLevel::None,
        };

        let result = SarifResult {
            rule_id: Some(rule_id.to_string()),
            rule_index: None,
            kind: Some(ResultKind::Review),
            level: Some(result_level),
            message: Message {
                text: Some(message_text.to_string()),
                markdown: None,
                id: None,
                arguments: None,
                properties: None,
            },
            locations: None,
            analysis_target: None,
            web_request: None,
            web_response: None,
            fingerprints: None,
            partial_fingerprints: None,
            code_flows: None,
            graphs: None,
            graph_traversals: None,
            stacks: None,
            related_locations: None,
            suppressions: None,
            baseline_state: None,
            attachments: None,
            work_item_uris: None,
            provenance: None,
            fixes: None,
            taxa: None,
            properties: None,
            // Add missing required fields
            correlation_guid: None,
            guid: None,
            hosted_viewer_uri: None,
            occurrence_count: None,
            rank: None,
            rule: None,
        };

        // Set location if it's a file-based result
        if !file_path.starts_with("project://") {
            let location = Location {
                id: None,
                physical_location: Some(PhysicalLocation {
                    artifact_location: Some(ArtifactLocation {
                        uri: Some(self.normalize_file_path(file_path)),
                        uri_base_id: None,
                        index: None,
                        description: None,
                        properties: None,
                    }),
                    region,
                    context_region: None,
                    address: None,
                    properties: None,
                }),
                logical_locations: None,
                message: None,
                annotations: None,
                relationships: None,
                properties: None,
            };
            
            let mut result_with_location = result;
            result_with_location.locations = Some(vec![location]);
            result_with_location
        } else {
            result
        }
    }

    /// Create the complete SARIF log structure
    fn create_sarif_log(&self, results: Vec<SarifResult>) -> Result<Sarif> {
        let sarif_log = Sarif {
            version: Value::String("2.1.0".to_string()),
            schema: Some("https://docs.oasis-open.org/sarif/sarif/v2.1.0/errata01/os/schemas/sarif-schema-2.1.0.json".to_string()),
            runs: vec![self.create_run(results)],
            inline_external_properties: None,
            properties: None,
        };

        Ok(sarif_log)
    }

    /// Create a SARIF run
    fn create_run(&self, results: Vec<SarifResult>) -> Run {
        let driver = ToolComponent {
            guid: None,
            name: self.tool_name.clone(),
            organization: None,
            product: None,
            product_suite: None,
            short_description: None,
            full_description: None,
            full_name: Some("HowMany Code Analysis Tool".to_string()),
            version: Some(self.tool_version.clone()),
            semantic_version: Some(self.tool_version.clone()),
            dotted_quad_file_version: None,
            release_date_utc: None,
            download_uri: None,
            information_uri: Some("https://github.com/GriffinCanCode/howmany".to_string()),
            global_message_strings: None,
            notifications: None,
            rules: Some(self.create_rule_definitions()),
            taxa: None,
            locations: None,
            language: None,
            contents: None,
            is_comprehensive: None,
            localized_data_semantic_version: None,
            minimum_required_localized_data_semantic_version: None,
            associated_component: None,
            translation_metadata: None,
            supported_taxonomies: None,
            properties: None,
        };

        let tool = Tool {
            driver,
            extensions: None,
            properties: None,
        };

        let start_time = Utc::now();
        let automation_details = RunAutomationDetails {
            description: Some(Message {
                text: Some("HowMany code analysis run".to_string()),
                markdown: None,
                id: None,
                arguments: None,
                properties: None,
            }),
            id: Some(format!("howmany-{}", start_time.format("%Y%m%d-%H%M%S"))),
            guid: None,
            correlation_guid: None,
            properties: None,
        };

        Run {
            tool,
            invocations: None,
            conversion: None,
            language: None,
            version_control_provenance: None,
            original_uri_base_ids: None,
            results: Some(results),
            automation_details: Some(automation_details),
            run_aggregates: None,
            baseline_guid: None,
            redaction_tokens: None,
            default_encoding: None,
            default_source_language: None,
            newline_sequences: None,
            column_kind: None,
            external_property_file_references: None,
            thread_flow_locations: None,
            taxonomies: None,
            addresses: None,
            translations: None,
            policies: None,
            web_requests: None,
            web_responses: None,
            properties: None,
            // Add missing required fields for Run
            artifacts: None,
            graphs: None,
            logical_locations: None,
            special_locations: None,
        }
    }

    /// Create rule definitions for all howmany rules
    pub fn create_rule_definitions(&self) -> Vec<ReportingDescriptor> {
        vec![
            self.create_rule("HM000", "Project Summary", "Provides an overview of project statistics"),
            self.create_rule("HM001", "Large File", "Detects files that may be too large and should be split"),
            self.create_rule("HM002", "Low Documentation", "Identifies files with insufficient documentation"),
            self.create_rule("HM003", "Empty File", "Detects files with no code content"),
            self.create_rule("HM101", "High Complexity", "Identifies functions or files with high cyclomatic complexity"),
            self.create_rule("HM102", "High Cognitive Complexity", "Detects code that may be difficult to understand"),
            self.create_rule("HM103", "Deep Nesting", "Identifies deeply nested code structures"),
            self.create_rule("HM201", "Low Maintainability", "Detects code with low maintainability scores"),
            self.create_rule("HM202", "Poor Code Health", "Identifies overall code health issues"),
            self.create_rule("HM301", "Large Project", "Warns about projects that may benefit from modularization"),
            self.create_rule("HM401", "Technical Debt Alert", "Highlights significant technical debt indicators"),
            self.create_rule("HM402", "Development Time Analysis", "Provides development time estimates and insights"),
        ]
    }

    /// Create a single rule definition
    fn create_rule(&self, id: &str, name: &str, description: &str) -> ReportingDescriptor {
        ReportingDescriptor {
            id: id.to_string(),
            deprecated_ids: None,
            deprecated_names: None,
            deprecated_guids: None,
            guid: None,
            name: Some(name.to_string()),
            short_description: Some(MultiformatMessageString {
                text: description.to_string(),
                markdown: None,
                properties: None,
            }),
            full_description: Some(MultiformatMessageString {
                text: self.get_rule_help_text(id),
                markdown: None,
                properties: None,
            }),
            message_strings: None,
            default_configuration: None,
            help_uri: Some(format!("https://github.com/GriffinCanCode/howmany/blob/main/docs/rules/{}.md", id)),
            help: None,
            relationships: None,
            properties: None,
        }
    }

    /// Get detailed help text for rules
    fn get_rule_help_text(&self, rule_id: &str) -> String {
        match rule_id {
            "HM001" => "Large files can be difficult to maintain and understand. Consider breaking them into smaller, more focused modules.".to_string(),
            "HM002" => "Well-documented code is easier to maintain. Consider adding comments explaining complex logic and public APIs.".to_string(),
            "HM003" => "Empty files may indicate incomplete implementation or files that can be removed to clean up the codebase.".to_string(),
            "HM101" => "High cyclomatic complexity indicates code that may be difficult to test and maintain. Consider refactoring into smaller functions.".to_string(),
            "HM102" => "High cognitive complexity makes code harder to understand. Consider simplifying control flow and reducing nested conditions.".to_string(),
            "HM103" => "Deeply nested code is harder to read and maintain. Consider extracting nested logic into separate functions.".to_string(),
            "HM201" => "Low maintainability scores indicate code that may be expensive to modify. Focus on improving code structure and reducing complexity.".to_string(),
            "HM202" => "Poor code health affects long-term project sustainability. Review coding standards and consider refactoring efforts.".to_string(),
            "HM301" => "Large projects benefit from modular architecture. Consider organizing code into logical modules or packages.".to_string(),
            "HM401" => "Technical debt accumulation can slow development. Prioritize refactoring efforts to improve code quality.".to_string(),
            "HM402" => "Development time analysis helps with project planning and resource allocation.".to_string(),
            _ => format!("Rule {} provides code quality insights from HowMany analysis.", rule_id),
        }
    }

    /// Normalize file paths for SARIF output
    pub fn normalize_file_path(&self, file_path: &str) -> String {
        // Convert to forward slashes and ensure proper URI format
        let normalized = file_path.replace('\\', "/");
        
        // If it's already a URI, return as-is
        if normalized.starts_with("http://") || normalized.starts_with("https://") || normalized.starts_with("file://") {
            normalized
        } else if std::path::Path::new(&normalized).is_absolute() {
            // Convert absolute paths to file URIs
            format!("file://{}", normalized)
        } else {
            // Relative paths are kept as-is
            normalized
        }
    }
}

impl Default for SarifConverter {
    fn default() -> Self {
        Self::new()
    }
} 