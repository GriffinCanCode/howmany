//! Source formats must be recognized, named and counted without an external detector.

use howmany::{AnalysisOptions, Engine};

#[test]
fn lean_and_cpp_modules_contribute_to_source_totals_without_detection() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    std::fs::write(
        root.join("Main.lean"),
        "-- ordinary comment\n/-- Declaration documentation -/\ndef value' := 1\n",
    )
    .unwrap();
    for extension in ["cppm", "ixx", "ccm", "cxxm"] {
        std::fs::write(
            root.join(format!("api.{extension}")),
            "export module sample;\n// ordinary comment\nexport int value() { return 1; }\n",
        )
        .unwrap();
    }
    let analysis = Engine::new()
        .analyze(
            root,
            &AnalysisOptions {
                source_only: true,
                collect_individual_files: true,
                compute_complexity: false,
                ..AnalysisOptions::reproducible()
            },
        )
        .unwrap();
    assert_eq!(analysis.basic.total_files, 5);
    assert_eq!(analysis.basic.total_lines, 15);
    assert_eq!(analysis.basic.total_code_lines, 9);
    assert_eq!(analysis.basic.total_comment_lines, 5);
    assert_eq!(analysis.basic.total_doc_lines, 1);
    let breakdown = analysis.stats.breakdown();
    let rows = &breakdown.rows;
    assert_eq!(
        rows.iter()
            .find(|row| row.language == "Lean")
            .unwrap()
            .file_count,
        1
    );
    assert_eq!(
        rows.iter()
            .find(|row| row.language == "C++")
            .unwrap()
            .file_count,
        4
    );
    assert!(analysis.basic.is_consistent());
}
