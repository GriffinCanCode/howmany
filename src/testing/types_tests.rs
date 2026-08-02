use crate::core::types::{CodeStats, FileStats};
use std::collections::BTreeMap;

#[cfg(test)]
mod types {
    use super::*;

    // ============================================================================
    // FILE STATS TESTS
    // ============================================================================

    #[test]
    fn test_file_stats_creation() {
        let stats = FileStats {
            total_lines: 100,
            code_lines: 70,
            comment_lines: 20,
            blank_lines: 10,
            file_size: 1500,
            doc_lines: 15,
        };

        assert_eq!(stats.total_lines, 100);
        assert_eq!(stats.code_lines, 70);
        assert_eq!(stats.comment_lines, 20);
        assert_eq!(stats.blank_lines, 10);
        assert_eq!(stats.file_size, 1500);
        assert_eq!(stats.doc_lines, 15);
    }

    #[test]
    fn test_file_stats_default() {
        let stats = FileStats::default();

        assert_eq!(stats.total_lines, 0);
        assert_eq!(stats.code_lines, 0);
        assert_eq!(stats.comment_lines, 0);
        assert_eq!(stats.blank_lines, 0);
        assert_eq!(stats.file_size, 0);
        assert_eq!(stats.doc_lines, 0);
    }

    #[test]
    fn test_file_stats_clone() {
        let original = FileStats {
            total_lines: 100,
            code_lines: 70,
            comment_lines: 20,
            blank_lines: 10,
            file_size: 1500,
            doc_lines: 15,
        };

        let cloned = original.clone();

        assert_eq!(original.total_lines, cloned.total_lines);
        assert_eq!(original.code_lines, cloned.code_lines);
        assert_eq!(original.comment_lines, cloned.comment_lines);
        assert_eq!(original.blank_lines, cloned.blank_lines);
        assert_eq!(original.file_size, cloned.file_size);
        assert_eq!(original.doc_lines, cloned.doc_lines);
    }

    #[test]
    fn test_file_stats_debug() {
        let stats = FileStats {
            total_lines: 100,
            code_lines: 70,
            comment_lines: 20,
            blank_lines: 10,
            file_size: 1500,
            doc_lines: 15,
        };

        let debug_str = format!("{:?}", stats);
        assert!(debug_str.contains("total_lines: 100"));
        assert!(debug_str.contains("code_lines: 70"));
        assert!(debug_str.contains("comment_lines: 20"));
        assert!(debug_str.contains("blank_lines: 10"));
        assert!(debug_str.contains("file_size: 1500"));
        assert!(debug_str.contains("doc_lines: 15"));
    }

    #[test]
    fn test_file_stats_equality() {
        let stats1 = FileStats {
            total_lines: 100,
            code_lines: 70,
            comment_lines: 20,
            blank_lines: 10,
            file_size: 1500,
            doc_lines: 15,
        };

        let stats2 = FileStats {
            total_lines: 100,
            code_lines: 70,
            comment_lines: 20,
            blank_lines: 10,
            file_size: 1500,
            doc_lines: 15,
        };

        let stats3 = FileStats {
            total_lines: 101,
            code_lines: 70,
            comment_lines: 20,
            blank_lines: 10,
            file_size: 1500,
            doc_lines: 15,
        };

        assert_eq!(stats1, stats2);
        assert_ne!(stats1, stats3);
    }

    #[test]
    fn test_file_stats_serialization() {
        let stats = FileStats {
            total_lines: 100,
            code_lines: 70,
            comment_lines: 20,
            blank_lines: 10,
            file_size: 1500,
            doc_lines: 15,
        };

        // Test JSON serialization
        let json = serde_json::to_string(&stats).unwrap();
        assert!(json.contains("\"total_lines\":100"));
        assert!(json.contains("\"code_lines\":70"));
        assert!(json.contains("\"file_size\":1500"));

        // Test JSON deserialization
        let deserialized: FileStats = serde_json::from_str(&json).unwrap();
        assert_eq!(stats, deserialized);
    }

    #[test]
    fn test_file_stats_edge_cases() {
        // Test with zero values
        let zero_stats = FileStats {
            total_lines: 0,
            code_lines: 0,
            comment_lines: 0,
            blank_lines: 0,
            file_size: 0,
            doc_lines: 0,
        };

        assert_eq!(zero_stats.total_lines, 0);

        // Test with maximum values
        let max_stats = FileStats {
            total_lines: usize::MAX,
            code_lines: usize::MAX,
            comment_lines: usize::MAX,
            blank_lines: usize::MAX,
            file_size: u64::MAX,
            doc_lines: usize::MAX,
        };

        assert_eq!(max_stats.total_lines, usize::MAX);
        assert_eq!(max_stats.file_size, u64::MAX);
    }

    #[test]
    fn test_file_stats_consistency() {
        let stats = FileStats {
            total_lines: 100,
            code_lines: 70,
            comment_lines: 20,
            blank_lines: 10,
            file_size: 1500,
            doc_lines: 15,
        };

        // Verify that the sum makes sense (doc_lines is counted within comment_lines)
        assert_eq!(
            stats.code_lines + stats.comment_lines + stats.blank_lines,
            stats.total_lines
        );
        assert!(stats.doc_lines <= stats.comment_lines);
    }

    // ============================================================================
    // CODE STATS TESTS
    // ============================================================================

    #[test]
    fn test_code_stats_creation() {
        let mut stats_by_extension = BTreeMap::new();
        stats_by_extension.insert(
            "rs".to_string(),
            (
                2,
                FileStats {
                    total_lines: 150,
                    code_lines: 100,
                    comment_lines: 30,
                    blank_lines: 20,
                    file_size: 2000,
                    doc_lines: 25,
                },
            ),
        );

        let stats = CodeStats {
            total_files: 2,
            total_lines: 150,
            total_code_lines: 100,
            total_comment_lines: 30,
            total_blank_lines: 20,
            total_size: 2000,
            total_doc_lines: 25,
            stats_by_extension,
        };

        assert_eq!(stats.total_files, 2);
        assert_eq!(stats.total_lines, 150);
        assert_eq!(stats.total_code_lines, 100);
        assert_eq!(stats.total_comment_lines, 30);
        assert_eq!(stats.total_blank_lines, 20);
        assert_eq!(stats.total_size, 2000);
        assert_eq!(stats.total_doc_lines, 25);
        assert_eq!(stats.stats_by_extension.len(), 1);
    }

    #[test]
    fn test_code_stats_default() {
        let stats = CodeStats::default();

        assert_eq!(stats.total_files, 0);
        assert_eq!(stats.total_lines, 0);
        assert_eq!(stats.total_code_lines, 0);
        assert_eq!(stats.total_comment_lines, 0);
        assert_eq!(stats.total_blank_lines, 0);
        assert_eq!(stats.total_size, 0);
        assert_eq!(stats.total_doc_lines, 0);
        assert!(stats.stats_by_extension.is_empty());
    }

    #[test]
    fn test_code_stats_clone() {
        let mut stats_by_extension = BTreeMap::new();
        stats_by_extension.insert("rs".to_string(), (2, FileStats::default()));
        stats_by_extension.insert("py".to_string(), (1, FileStats::default()));

        let original = CodeStats {
            total_files: 3,
            total_lines: 200,
            total_code_lines: 140,
            total_comment_lines: 40,
            total_blank_lines: 20,
            total_size: 3000,
            total_doc_lines: 30,
            stats_by_extension,
        };

        let cloned = original.clone();

        assert_eq!(original.total_files, cloned.total_files);
        assert_eq!(original.total_lines, cloned.total_lines);
        assert_eq!(
            original.stats_by_extension.len(),
            cloned.stats_by_extension.len()
        );
        assert!(cloned.stats_by_extension.contains_key("rs"));
        assert!(cloned.stats_by_extension.contains_key("py"));
    }

    #[test]
    fn test_code_stats_debug() {
        let mut stats_by_extension = BTreeMap::new();
        stats_by_extension.insert("rs".to_string(), (2, FileStats::default()));

        let stats = CodeStats {
            total_files: 2,
            total_lines: 150,
            total_code_lines: 100,
            total_comment_lines: 30,
            total_blank_lines: 20,
            total_size: 2000,
            total_doc_lines: 25,
            stats_by_extension,
        };

        let debug_str = format!("{:?}", stats);
        assert!(debug_str.contains("total_files: 2"));
        assert!(debug_str.contains("total_lines: 150"));
        assert!(debug_str.contains("total_code_lines: 100"));
        assert!(debug_str.contains("stats_by_extension"));
    }

    #[test]
    fn test_code_stats_equality() {
        let mut stats_by_extension1 = BTreeMap::new();
        stats_by_extension1.insert("rs".to_string(), (2, FileStats::default()));

        let mut stats_by_extension2 = BTreeMap::new();
        stats_by_extension2.insert("rs".to_string(), (2, FileStats::default()));

        let mut stats_by_extension3 = BTreeMap::new();
        stats_by_extension3.insert("py".to_string(), (2, FileStats::default()));

        let stats1 = CodeStats {
            total_files: 2,
            total_lines: 150,
            total_code_lines: 100,
            total_comment_lines: 30,
            total_blank_lines: 20,
            total_size: 2000,
            total_doc_lines: 25,
            stats_by_extension: stats_by_extension1,
        };

        let stats2 = CodeStats {
            total_files: 2,
            total_lines: 150,
            total_code_lines: 100,
            total_comment_lines: 30,
            total_blank_lines: 20,
            total_size: 2000,
            total_doc_lines: 25,
            stats_by_extension: stats_by_extension2,
        };

        let stats3 = CodeStats {
            total_files: 2,
            total_lines: 150,
            total_code_lines: 100,
            total_comment_lines: 30,
            total_blank_lines: 20,
            total_size: 2000,
            total_doc_lines: 25,
            stats_by_extension: stats_by_extension3,
        };

        assert_eq!(stats1, stats2);
        assert_ne!(stats1, stats3);
    }

    #[test]
    fn test_code_stats_serialization() {
        let mut stats_by_extension = BTreeMap::new();
        stats_by_extension.insert(
            "rs".to_string(),
            (
                2,
                FileStats {
                    total_lines: 100,
                    code_lines: 70,
                    comment_lines: 20,
                    blank_lines: 10,
                    file_size: 1500,
                    doc_lines: 15,
                },
            ),
        );
        stats_by_extension.insert(
            "py".to_string(),
            (
                1,
                FileStats {
                    total_lines: 50,
                    code_lines: 35,
                    comment_lines: 10,
                    blank_lines: 5,
                    file_size: 750,
                    doc_lines: 8,
                },
            ),
        );

        let stats = CodeStats {
            total_files: 3,
            total_lines: 150,
            total_code_lines: 105,
            total_comment_lines: 30,
            total_blank_lines: 15,
            total_size: 2250,
            total_doc_lines: 23,
            stats_by_extension,
        };

        // Test JSON serialization
        let json = serde_json::to_string(&stats).unwrap();
        assert!(json.contains("\"total_files\":3"));
        assert!(json.contains("\"total_lines\":150"));
        assert!(json.contains("\"stats_by_extension\""));
        assert!(json.contains("\"rs\""));
        assert!(json.contains("\"py\""));

        // Test JSON deserialization
        let deserialized: CodeStats = serde_json::from_str(&json).unwrap();
        assert_eq!(stats, deserialized);
        assert_eq!(deserialized.stats_by_extension.len(), 2);
        assert!(deserialized.stats_by_extension.contains_key("rs"));
        assert!(deserialized.stats_by_extension.contains_key("py"));
    }

    #[test]
    fn test_code_stats_extension_operations() {
        let mut stats = CodeStats::default();

        // Add first extension
        let rust_stats = FileStats {
            total_lines: 100,
            code_lines: 70,
            comment_lines: 20,
            blank_lines: 10,
            file_size: 1500,
            doc_lines: 15,
        };
        stats
            .stats_by_extension
            .insert("rs".to_string(), (2, rust_stats.clone()));

        // Add second extension
        let python_stats = FileStats {
            total_lines: 50,
            code_lines: 35,
            comment_lines: 10,
            blank_lines: 5,
            file_size: 750,
            doc_lines: 8,
        };
        stats
            .stats_by_extension
            .insert("py".to_string(), (1, python_stats.clone()));

        // Test retrieval
        assert_eq!(stats.stats_by_extension.len(), 2);
        assert!(stats.stats_by_extension.contains_key("rs"));
        assert!(stats.stats_by_extension.contains_key("py"));

        let (rust_count, rust_file_stats) = &stats.stats_by_extension["rs"];
        assert_eq!(*rust_count, 2);
        assert_eq!(rust_file_stats.total_lines, 100);

        let (python_count, python_file_stats) = &stats.stats_by_extension["py"];
        assert_eq!(*python_count, 1);
        assert_eq!(python_file_stats.total_lines, 50);

        // Test iteration
        let mut extensions: Vec<String> = stats.stats_by_extension.keys().cloned().collect();
        extensions.sort();
        assert_eq!(extensions, vec!["py", "rs"]);
    }

    #[test]
    fn test_code_stats_consistency() {
        let mut stats_by_extension = BTreeMap::new();
        stats_by_extension.insert(
            "rs".to_string(),
            (
                2,
                FileStats {
                    total_lines: 100,
                    code_lines: 70,
                    comment_lines: 20,
                    blank_lines: 10,
                    file_size: 1500,
                    doc_lines: 15,
                },
            ),
        );
        stats_by_extension.insert(
            "py".to_string(),
            (
                1,
                FileStats {
                    total_lines: 50,
                    code_lines: 35,
                    comment_lines: 10,
                    blank_lines: 5,
                    file_size: 750,
                    doc_lines: 8,
                },
            ),
        );

        let stats = CodeStats {
            total_files: 3,
            total_lines: 150,
            total_code_lines: 105,
            total_comment_lines: 30,
            total_blank_lines: 15,
            total_size: 2250,
            total_doc_lines: 23,
            stats_by_extension,
        };

        // Verify consistency between totals and per-extension data
        let extension_file_count: usize = stats
            .stats_by_extension
            .values()
            .map(|(count, _)| count)
            .sum();
        assert_eq!(stats.total_files, extension_file_count);

        let extension_total_lines: usize = stats
            .stats_by_extension
            .values()
            .map(|(_, file_stats)| file_stats.total_lines)
            .sum();
        assert_eq!(stats.total_lines, extension_total_lines);

        let extension_code_lines: usize = stats
            .stats_by_extension
            .values()
            .map(|(_, file_stats)| file_stats.code_lines)
            .sum();
        assert_eq!(stats.total_code_lines, extension_code_lines);

        let extension_total_size: u64 = stats
            .stats_by_extension
            .values()
            .map(|(_, file_stats)| file_stats.file_size)
            .sum();
        assert_eq!(stats.total_size, extension_total_size);

        // Verify line count consistency
        assert_eq!(
            stats.total_code_lines + stats.total_comment_lines + stats.total_blank_lines,
            stats.total_lines
        );

        // Verify doc lines are subset of comment lines
        assert!(stats.total_doc_lines <= stats.total_comment_lines);
    }

    #[test]
    fn test_code_stats_edge_cases() {
        // Test with no extensions
        let empty_stats = CodeStats::default();
        assert_eq!(empty_stats.total_files, 0);
        assert!(empty_stats.stats_by_extension.is_empty());

        // Test with single extension
        let mut single_extension = BTreeMap::new();
        single_extension.insert(
            "rs".to_string(),
            (
                1,
                FileStats {
                    total_lines: 10,
                    code_lines: 7,
                    comment_lines: 2,
                    blank_lines: 1,
                    file_size: 150,
                    doc_lines: 1,
                },
            ),
        );

        let single_stats = CodeStats {
            total_files: 1,
            total_lines: 10,
            total_code_lines: 7,
            total_comment_lines: 2,
            total_blank_lines: 1,
            total_size: 150,
            total_doc_lines: 1,
            stats_by_extension: single_extension,
        };

        assert_eq!(single_stats.total_files, 1);
        assert_eq!(single_stats.stats_by_extension.len(), 1);

        // Test with many extensions
        let mut many_extensions = BTreeMap::new();
        for i in 0..100 {
            let ext = format!("ext{}", i);
            many_extensions.insert(
                ext,
                (
                    1,
                    FileStats {
                        total_lines: 10,
                        code_lines: 7,
                        comment_lines: 2,
                        blank_lines: 1,
                        file_size: 150,
                        doc_lines: 1,
                    },
                ),
            );
        }

        let many_stats = CodeStats {
            total_files: 100,
            total_lines: 1000,
            total_code_lines: 700,
            total_comment_lines: 200,
            total_blank_lines: 100,
            total_size: 15000,
            total_doc_lines: 100,
            stats_by_extension: many_extensions,
        };

        assert_eq!(many_stats.total_files, 100);
        assert_eq!(many_stats.stats_by_extension.len(), 100);
    }

    #[test]
    fn test_code_stats_large_values() {
        // Test with large values to ensure no overflow
        let mut large_extension = BTreeMap::new();
        large_extension.insert(
            "rs".to_string(),
            (
                1000000,
                FileStats {
                    total_lines: 1000000,
                    code_lines: 700000,
                    comment_lines: 200000,
                    blank_lines: 100000,
                    file_size: 15000000,
                    doc_lines: 150000,
                },
            ),
        );

        let large_stats = CodeStats {
            total_files: 1000000,
            total_lines: 1000000,
            total_code_lines: 700000,
            total_comment_lines: 200000,
            total_blank_lines: 100000,
            total_size: 15000000,
            total_doc_lines: 150000,
            stats_by_extension: large_extension,
        };

        assert_eq!(large_stats.total_files, 1000000);
        assert_eq!(large_stats.total_lines, 1000000);
        assert_eq!(large_stats.total_size, 15000000);

        // Test serialization with large values
        let json = serde_json::to_string(&large_stats).unwrap();
        let deserialized: CodeStats = serde_json::from_str(&json).unwrap();
        assert_eq!(large_stats, deserialized);
    }

    #[test]
    fn test_code_stats_unicode_extensions() {
        // Test with unicode extension names
        let mut unicode_extensions = BTreeMap::new();
        unicode_extensions.insert("ру́сский".to_string(), (1, FileStats::default()));
        unicode_extensions.insert("中文".to_string(), (1, FileStats::default()));
        unicode_extensions.insert("العربية".to_string(), (1, FileStats::default()));
        unicode_extensions.insert("🦀".to_string(), (1, FileStats::default())); // Emoji extension

        let unicode_stats = CodeStats {
            total_files: 4,
            total_lines: 0,
            total_code_lines: 0,
            total_comment_lines: 0,
            total_blank_lines: 0,
            total_size: 0,
            total_doc_lines: 0,
            stats_by_extension: unicode_extensions,
        };

        assert_eq!(unicode_stats.stats_by_extension.len(), 4);
        assert!(unicode_stats.stats_by_extension.contains_key("ру́сский"));
        assert!(unicode_stats.stats_by_extension.contains_key("中文"));
        assert!(unicode_stats.stats_by_extension.contains_key("العربية"));
        assert!(unicode_stats.stats_by_extension.contains_key("🦀"));

        // Test serialization with unicode
        let json = serde_json::to_string(&unicode_stats).unwrap();
        let deserialized: CodeStats = serde_json::from_str(&json).unwrap();
        assert_eq!(unicode_stats, deserialized);
    }

    #[test]
    fn test_types_interoperability() {
        // Test that FileStats and CodeStats work well together
        let file_stats1 = FileStats {
            total_lines: 50,
            code_lines: 35,
            comment_lines: 10,
            blank_lines: 5,
            file_size: 750,
            doc_lines: 8,
        };

        let file_stats2 = FileStats {
            total_lines: 100,
            code_lines: 70,
            comment_lines: 20,
            blank_lines: 10,
            file_size: 1500,
            doc_lines: 15,
        };

        // Create CodeStats from FileStats
        let mut stats_by_extension = BTreeMap::new();
        stats_by_extension.insert("py".to_string(), (1, file_stats1.clone()));
        stats_by_extension.insert("rs".to_string(), (1, file_stats2.clone()));

        let code_stats = CodeStats {
            total_files: 2,
            total_lines: file_stats1.total_lines + file_stats2.total_lines,
            total_code_lines: file_stats1.code_lines + file_stats2.code_lines,
            total_comment_lines: file_stats1.comment_lines + file_stats2.comment_lines,
            total_blank_lines: file_stats1.blank_lines + file_stats2.blank_lines,
            total_size: file_stats1.file_size + file_stats2.file_size,
            total_doc_lines: file_stats1.doc_lines + file_stats2.doc_lines,
            stats_by_extension,
        };

        assert_eq!(code_stats.total_files, 2);
        assert_eq!(code_stats.total_lines, 150);
        assert_eq!(code_stats.total_code_lines, 105);
        assert_eq!(code_stats.total_size, 2250);

        // Verify that individual FileStats are preserved
        let (_, retrieved_file_stats1) = &code_stats.stats_by_extension["py"];
        let (_, retrieved_file_stats2) = &code_stats.stats_by_extension["rs"];

        assert_eq!(*retrieved_file_stats1, file_stats1);
        assert_eq!(*retrieved_file_stats2, file_stats2);
    }

    #[test]
    fn test_types_thread_safety() {
        use std::sync::Arc;
        use std::thread;

        let stats = Arc::new(CodeStats {
            total_files: 100,
            total_lines: 1000,
            total_code_lines: 700,
            total_comment_lines: 200,
            total_blank_lines: 100,
            total_size: 15000,
            total_doc_lines: 150,
            stats_by_extension: BTreeMap::new(),
        });

        let handles: Vec<_> = (0..10)
            .map(|_| {
                let stats = Arc::clone(&stats);
                thread::spawn(move || {
                    // Read operations should be thread-safe
                    let _files = stats.total_files;
                    let _lines = stats.total_lines;
                    let _size = stats.total_size;
                    let _extensions = stats.stats_by_extension.len();
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }
    }

    #[test]
    fn test_types_performance() {
        // Test performance with large datasets
        let start = std::time::Instant::now();

        // Create large CodeStats
        let mut large_extensions = BTreeMap::new();
        for i in 0..1000 {
            let ext = format!("ext{}", i);
            large_extensions.insert(
                ext,
                (
                    100,
                    FileStats {
                        total_lines: 1000,
                        code_lines: 700,
                        comment_lines: 200,
                        blank_lines: 100,
                        file_size: 15000,
                        doc_lines: 150,
                    },
                ),
            );
        }

        let large_stats = CodeStats {
            total_files: 100000,
            total_lines: 1000000,
            total_code_lines: 700000,
            total_comment_lines: 200000,
            total_blank_lines: 100000,
            total_size: 15000000,
            total_doc_lines: 150000,
            stats_by_extension: large_extensions,
        };

        // Clone should be fast
        let _cloned = large_stats.clone();

        // Serialization should complete in reasonable time
        let _json = serde_json::to_string(&large_stats).unwrap();

        let duration = start.elapsed();
        assert!(duration.as_secs() < 1); // Should complete in less than 1 second
    }
}
