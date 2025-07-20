#[cfg(test)]
mod tests {
    use crate::core::types::{CodeStats, FileStats};
    use crate::ui::sarif::{SarifConverter, SarifReporter};
    use serde_sarif::sarif::Sarif;
    use std::collections::HashMap;
    use tempfile::NamedTempFile;

    fn create_test_stats() -> CodeStats {
        let mut stats_by_extension = HashMap::new();
        
        // Add some test data
        let rust_stats = FileStats {
            total_lines: 1000,
            code_lines: 750,
            comment_lines: 100,
            doc_lines: 50,
            blank_lines: 100,
            file_size: 25000,
        };
        stats_by_extension.insert("rs".to_string(), (5, rust_stats));

        let js_stats = FileStats {
            total_lines: 500,
            code_lines: 400,
            comment_lines: 50,
            doc_lines: 25,
            blank_lines: 25,
            file_size: 12000,
        };
        stats_by_extension.insert("js".to_string(), (3, js_stats));

        CodeStats {
            total_files: 8,
            total_lines: 1500,
            total_code_lines: 1150,
            total_comment_lines: 150,
            total_doc_lines: 75,
            total_blank_lines: 125,
            total_size: 37000,
            stats_by_extension,
        }
    }

    fn create_test_individual_files() -> Vec<(String, FileStats)> {
        vec![
            ("src/main.rs".to_string(), FileStats {
                total_lines: 200,
                code_lines: 150,
                comment_lines: 25,
                doc_lines: 15,
                blank_lines: 10,
                file_size: 5000,
            }),
            ("src/lib.rs".to_string(), FileStats {
                total_lines: 100,
                code_lines: 80,
                comment_lines: 10,
                doc_lines: 5,
                blank_lines: 5,
                file_size: 2500,
            }),
        ]
    }

    #[test]
    fn test_sarif_converter_creation() {
        let converter = SarifConverter::new();
        assert_eq!(converter.tool_name(), "howmany");
        assert!(!converter.tool_version().is_empty());
    }

    #[test]
    fn test_basic_sarif_conversion() {
        let converter = SarifConverter::new();
        let stats = create_test_stats();
        let individual_files = create_test_individual_files();

        let result = converter.convert_basic_analysis(&stats, &individual_files);
        assert!(result.is_ok());

        let sarif_log = result.unwrap();
        assert_eq!(sarif_log.version, serde_json::Value::String("2.1.0".to_string()));
        assert!(!sarif_log.runs.is_empty());

        let run = &sarif_log.runs[0];
        assert_eq!(run.tool.driver.name, "howmany");
        
        // Should have some results
        if let Some(results) = &run.results {
            assert!(!results.is_empty());
        }
    }

    #[test]
    fn test_sarif_reporter() {
        let reporter = SarifReporter::new();
        let stats = create_test_stats();
        let individual_files = create_test_individual_files();
        
        let temp_file = NamedTempFile::new().unwrap();
        let result = reporter.generate_report(&stats, &individual_files, temp_file.path());
        assert!(result.is_ok());
        
        // Verify file was created and has content
        let content = std::fs::read_to_string(temp_file.path()).unwrap();
        assert!(!content.is_empty());
        
        // Verify it's valid JSON
        let sarif_log: Sarif = serde_json::from_str(&content).unwrap();
        assert_eq!(sarif_log.version, serde_json::Value::String("2.1.0".to_string()));
        assert!(!sarif_log.runs.is_empty());
    }

    #[test]
    fn test_sarif_serialization() {
        let converter = SarifConverter::new();
        let stats = create_test_stats();
        let individual_files = create_test_individual_files();
        
        let sarif_log = converter.convert_basic_analysis(&stats, &individual_files).unwrap();
        
        // Test that it can be serialized to JSON
        let json_content = serde_json::to_string_pretty(&sarif_log).unwrap();
        assert!(!json_content.is_empty());
        
        // Test that it can be deserialized back
        let deserialized: Sarif = serde_json::from_str(&json_content).unwrap();
        assert_eq!(deserialized.version, sarif_log.version);
    }

    #[test] 
    fn test_file_path_normalization() {
        let converter = SarifConverter::new();
        
        // Test different path formats
        assert_eq!(converter.normalize_file_path("src/main.rs"), "src/main.rs");
        assert_eq!(converter.normalize_file_path("src\\main.rs"), "src/main.rs");
        assert_eq!(converter.normalize_file_path("/absolute/path/file.rs"), "file:///absolute/path/file.rs");
        assert_eq!(converter.normalize_file_path("file:///already/uri"), "file:///already/uri");
    }

    #[test]
    fn test_rule_definitions() {
        let converter = SarifConverter::new();
        let rules = converter.create_rule_definitions();
        
        assert!(!rules.is_empty());
        
        // Check that all rules have required fields
        for rule in &rules {
            assert!(!rule.id.is_empty());
            assert!(rule.name.is_some());
            assert!(rule.short_description.is_some());
            assert!(rule.full_description.is_some());
        }
        
        // Check for specific expected rules
        let rule_ids: Vec<&str> = rules.iter().map(|r| r.id.as_str()).collect();
        assert!(rule_ids.contains(&"HM001")); // Large File
        assert!(rule_ids.contains(&"HM000")); // Project Summary
    }

    #[test]
    fn test_empty_stats() {
        let converter = SarifConverter::new();
        let stats = CodeStats {
            total_files: 0,
            total_lines: 0,
            total_code_lines: 0,
            total_comment_lines: 0,
            total_doc_lines: 0,
            total_blank_lines: 0,
            total_size: 0,
            stats_by_extension: HashMap::new(),
        };
        let individual_files = vec![];

        let result = converter.convert_basic_analysis(&stats, &individual_files);
        assert!(result.is_ok());

        let sarif_log = result.unwrap();
        assert!(!sarif_log.runs.is_empty());
        
        // Should still have at least a project summary
        let run = &sarif_log.runs[0];
        if let Some(results) = &run.results {
            // Should have at least one result (project summary)
            assert!(!results.is_empty());
        }
    }
} 