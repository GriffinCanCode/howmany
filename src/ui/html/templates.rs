use crate::core::types::{CodeStats, FileStats};
use crate::core::stats::basic::BasicStats;
use crate::core::stats::complexity::ComplexityStatsCalculator;
use crate::core::stats::aggregation::AggregatedStats;
use super::utils::FileUtils;

use std::fmt::Write;

pub struct TemplateGenerator {
    file_utils: FileUtils,
}

impl TemplateGenerator {
    pub fn new() -> Self {
        Self {
            file_utils: FileUtils::new(),
        }
    }
    
    pub fn generate_extension_rows(&self, stats: &CodeStats) -> String {
        let mut rows = String::with_capacity(stats.stats_by_extension.len() * 200); // Pre-allocate
        let mut extensions: Vec<_> = stats.stats_by_extension.iter().collect();
        extensions.sort_by(|a, b| b.1.1.total_lines.cmp(&a.1.1.total_lines));
        
        for (ext, (file_count, ext_stats)) in extensions {
            let complexity_class = self.get_complexity_class_for_extension(ext);
            let complexity_score = self.estimate_complexity_for_extension(ext, ext_stats);
            
            write!(rows, 
                r#"<tr>
                    <td>{} {}</td>
                    <td>{}</td>
                    <td>{}</td>
                    <td>{}</td>
                    <td>{}</td>
                    <td>{}</td>
                    <td>{}</td>
                    <td><span class="complexity-badge {}">{:.1}</span></td>
                    <td>{}</td>
                </tr>"#,
                self.file_utils.get_file_emoji(ext),
                ext,
                file_count,
                ext_stats.total_lines,
                ext_stats.code_lines,
                ext_stats.comment_lines,
                ext_stats.doc_lines,
                self.estimate_functions_for_extension(ext, ext_stats),
                complexity_class,
                complexity_score,
                self.file_utils.format_size(ext_stats.file_size)
            ).unwrap_or_else(|_| eprintln!("Failed to write extension row"));
        }
        
        rows
    }
    
    /// Generate extension rows using real complexity analysis from AggregatedStats
    pub fn generate_extension_rows_with_real_analysis(&self, aggregated_stats: &AggregatedStats) -> String {
        let extensions_count = aggregated_stats.basic.stats_by_extension.len();
        let mut rows = String::with_capacity(extensions_count * 300); // Better pre-allocation
        
        let mut extensions: Vec<_> = aggregated_stats.basic.stats_by_extension.iter().collect();
        extensions.sort_by(|a, b| b.1.total_lines.cmp(&a.1.total_lines));
        
        for (ext, ext_stats) in extensions {
            let complexity_data = aggregated_stats.complexity.complexity_by_extension.get(ext);
            let complexity_score = complexity_data.map(|c| c.cyclomatic_complexity).unwrap_or(0.0);
            let function_count = complexity_data.map(|c| c.function_count).unwrap_or(0);
            let complexity_class = self.get_complexity_class_for_score(complexity_score);
            
            // Use format! directly instead of write! for better performance in this case
            rows.push_str(&format!(
                "<tr><td>{} {}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td><span class=\"complexity-badge {}\">{:.1}</span></td><td>{}</td></tr>",
                self.file_utils.get_file_emoji(ext),
                ext,
                ext_stats.file_count,
                ext_stats.total_lines,
                ext_stats.code_lines,
                ext_stats.comment_lines,
                ext_stats.doc_lines,
                function_count,
                complexity_class,
                complexity_score,
                self.file_utils.format_size(ext_stats.total_size)
            ));
        }
        
        rows
    }
    
    pub fn generate_individual_files_section(&self, individual_files: &[(String, FileStats)]) -> String {
        if individual_files.is_empty() {
            return String::new();
        }
        
        let mut section = String::from(r#"
        <h2>📄 Individual Files Analysis</h2>
        <div class="function-details">
        "#);
        
        // Sort files by real complexity analysis
        let mut sorted_files: Vec<_> = individual_files.iter().collect();
        sorted_files.sort_by(|a, b| {
            let complexity_a = self.calculate_real_file_complexity(&a.0, &a.1);
            let complexity_b = self.calculate_real_file_complexity(&b.0, &b.1);
            complexity_b.partial_cmp(&complexity_a).unwrap_or(std::cmp::Ordering::Equal)
        });
        
        for (file_path, file_stats) in sorted_files.iter().take(50) {
            let complexity = self.calculate_real_file_complexity(file_path, file_stats);
            let complexity_class = self.get_complexity_class_for_score(complexity);
            let functions = self.calculate_real_function_count(file_path, file_stats);
            
            section.push_str(&format!(
                r#"<div class="function-item">
                    <div class="function-name">{}</div>
                    <div class="function-metrics">
                        <span class="function-metric">Lines: {}</span>
                        <span class="function-metric">Code: {}</span>
                        <span class="function-metric">Functions: {}</span>
                        <span class="function-metric complexity-badge {}">Complexity: {:.1}</span>
                    </div>
                </div>"#,
                self.shorten_path(file_path),
                file_stats.total_lines,
                file_stats.code_lines,
                functions,
                complexity_class,
                complexity
            ));
        }
        
        section.push_str("</div>");
        section
    }
    
    /// Generate optimized individual files section with performance improvements
    pub fn generate_optimized_individual_files_section(&self, individual_files: &[(String, FileStats)]) -> String {
        if individual_files.is_empty() {
            return String::new();
        }
        
        // Limit to top 20 files to prevent performance issues
        let mut sorted_files: Vec<_> = individual_files.iter().collect();
        sorted_files.sort_by(|a, b| {
            // Use simpler sorting criteria for better performance
            let score_a = a.1.total_lines as f64 + (a.1.code_lines as f64 * 0.5);
            let score_b = b.1.total_lines as f64 + (b.1.code_lines as f64 * 0.5);
            score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
        });
        
        let file_count = sorted_files.len().min(20);
        let mut section = String::with_capacity(file_count * 200 + 200); // Better pre-allocation
        
        section.push_str("<h2>📄 Individual Files Analysis</h2><div class=\"function-details\">");
        
        for (file_path, file_stats) in sorted_files.iter().take(20) {
            // Use estimated complexity for better performance
            let complexity = self.estimate_file_complexity(file_stats);
            let complexity_class = self.get_complexity_class_for_score(complexity);
            let functions = self.estimate_functions_for_file(file_path, file_stats);
            
            // Build string more efficiently
            section.push_str(&format!(
                "<div class=\"function-item\"><div class=\"function-name\">{}</div><div class=\"function-metrics\"><span class=\"function-metric\">Lines: {}</span><span class=\"function-metric\">Code: {}</span><span class=\"function-metric\">Functions: {}</span><span class=\"function-metric complexity-badge {}\">Complexity: {:.1}</span></div></div>",
                self.shorten_path(file_path),
                file_stats.total_lines,
                file_stats.code_lines,
                functions,
                complexity_class,
                complexity
            ));
        }
        
        section.push_str("</div>");
        section
    }
    
    pub fn generate_complexity_labels(&self, stats: &CodeStats) -> String {
        let mut labels: Vec<String> = stats.stats_by_extension
            .keys()
            .map(|ext| format!("'{}'", ext))
            .collect();
        labels.sort();
        labels.join(", ")
    }
    
    pub fn generate_complexity_data(&self, stats: &CodeStats) -> String {
        let mut data: Vec<String> = Vec::new();
        let mut extensions: Vec<_> = stats.stats_by_extension.iter().collect();
        extensions.sort_by_key(|(ext, _)| ext.as_str());
        
        for (ext, (_, ext_stats)) in extensions {
            let complexity = self.estimate_complexity_for_extension(ext, ext_stats);
            data.push(complexity.to_string());
        }
        
        data.join(", ")
    }
    
    /// Generate complexity data using real analysis from AggregatedStats
    pub fn generate_complexity_data_with_real_analysis(&self, aggregated_stats: &AggregatedStats) -> String {
        let mut data: Vec<String> = Vec::new();
        let mut extensions: Vec<_> = aggregated_stats.basic.stats_by_extension.iter().collect();
        extensions.sort_by_key(|(ext, _)| ext.as_str());
        
        for (ext, _) in extensions {
            let complexity = aggregated_stats.complexity.complexity_by_extension
                .get(ext)
                .map(|c| c.cyclomatic_complexity)
                .unwrap_or(0.0);
            data.push(complexity.to_string());
        }
        
        data.join(", ")
    }
    
    pub fn generate_complexity_insights(&self, stats: &BasicStats) -> String {
        let mut insights = Vec::new();
        
        // Analyze overall complexity
        let total_lines = stats.total_lines;
        let code_ratio = stats.code_lines as f64 / total_lines as f64;
        let _comment_ratio = stats.comment_lines as f64 / total_lines as f64;
        let doc_ratio = stats.doc_lines as f64 / total_lines as f64;
        
        // Code quality insights
        if code_ratio > 0.8 {
            insights.push("🔴 High code density detected. Consider adding more comments and documentation.".to_string());
        } else if code_ratio > 0.6 {
            insights.push("🟡 Moderate code density. Good balance but could benefit from more documentation.".to_string());
        } else {
            insights.push("🟢 Good code-to-comment ratio. Well-documented codebase.".to_string());
        }
        
        // Documentation insights
        if doc_ratio > 0.15 {
            insights.push("📚 Excellent documentation coverage. Your future self will thank you!".to_string());
        } else if doc_ratio > 0.08 {
            insights.push("📖 Good documentation coverage. Consider adding more for complex functions.".to_string());
        } else if doc_ratio > 0.03 {
            insights.push("📝 Basic documentation present. Consider expanding for better maintainability.".to_string());
        } else {
            insights.push("⚠️ Low documentation coverage. Adding docs will improve code maintainability.".to_string());
        }
        
        // Language-specific insights
        let mut lang_insights = self.generate_language_insights(stats);
        insights.append(&mut lang_insights);
        
        // File size insights
        let avg_file_size = stats.total_size as f64 / stats.total_files as f64;
        if avg_file_size > 50000.0 {
            insights.push("📏 Large average file size detected. Consider breaking down large files.".to_string());
        } else if avg_file_size > 20000.0 {
            insights.push("📐 Moderate file sizes. Monitor for files that might need refactoring.".to_string());
        } else {
            insights.push("📋 Good file size distribution. Easy to navigate and maintain.".to_string());
        }
        
        insights.join("\n")
    }
    
    /// Generate real complexity insights using AggregatedStats
    pub fn generate_real_complexity_insights(&self, aggregated_stats: &AggregatedStats) -> String {
        let mut insights = Vec::new();
        let complexity_stats = &aggregated_stats.complexity;
        
        // Function complexity insights
        if complexity_stats.function_count > 0 {
            let avg_complexity = complexity_stats.cyclomatic_complexity;
            if avg_complexity > 15.0 {
                insights.push("🔴 High average complexity detected. Consider refactoring complex functions.".to_string());
            } else if avg_complexity > 10.0 {
                insights.push("🟡 Moderate complexity. Monitor for functions that might need simplification.".to_string());
            } else {
                insights.push("🟢 Good complexity levels. Functions are well-structured and maintainable.".to_string());
            }
            
            // Nesting depth insights
            if complexity_stats.max_nesting_depth > 6 {
                insights.push("📐 Deep nesting detected. Consider extracting nested logic into separate functions.".to_string());
            } else if complexity_stats.max_nesting_depth > 4 {
                insights.push("📏 Moderate nesting levels. Keep an eye on deeply nested code.".to_string());
            } else {
                insights.push("📋 Good nesting levels. Code structure is clean and readable.".to_string());
            }
            
            // Function size insights
            if complexity_stats.average_function_length > 50.0 {
                insights.push("📏 Large average function size. Consider breaking down large functions.".to_string());
            } else if complexity_stats.average_function_length > 30.0 {
                insights.push("📐 Moderate function sizes. Monitor for functions that might need refactoring.".to_string());
            } else {
                insights.push("📋 Good function size distribution. Functions are focused and manageable.".to_string());
            }
        }
        
        // Quality metrics insights
        let quality = &complexity_stats.quality_metrics;
        if quality.code_health_score > 80.0 {
            insights.push("⭐ Excellent code quality! Your codebase is well-structured and maintainable.".to_string());
        } else if quality.code_health_score > 60.0 {
            insights.push("👍 Good code quality. Some areas could benefit from improvement.".to_string());
        } else {
            insights.push("⚠️ Code quality needs attention. Consider refactoring and adding documentation.".to_string());
        }
        
        insights.join("\n")
    }
    
    pub fn generate_quality_recommendations(&self, stats: &CodeStats) -> String {
        let mut recommendations = Vec::new();
        
        let comment_ratio = stats.total_comment_lines as f64 / stats.total_lines as f64;
        let doc_ratio = stats.total_doc_lines as f64 / stats.total_lines as f64;
        
        // Comment recommendations
        if comment_ratio < 0.1 {
            recommendations.push("💬 Add more inline comments to explain complex logic and business rules.".to_string());
        }
        
        // Documentation recommendations
        if doc_ratio < 0.05 {
            recommendations.push("📚 Add function and class documentation to improve code understanding.".to_string());
        }
        
        // File organization recommendations
        if stats.total_files > 1000 {
            recommendations.push("📁 Consider organizing files into modules or packages for better structure.".to_string());
        }
        
        // Testing recommendations
        let has_test_files = stats.stats_by_extension.keys().any(|ext| {
            ext.contains("test") || ext.contains("spec")
        });
        
        if !has_test_files {
            recommendations.push("🧪 Add unit tests to improve code reliability and maintainability.".to_string());
        }
        
        recommendations.join("\n")
    }
    
    /// Generate quality recommendations using real analysis
    pub fn generate_real_quality_recommendations(&self, aggregated_stats: &AggregatedStats) -> String {
        let mut recommendations = Vec::new();
        let complexity_stats = &aggregated_stats.complexity;
        
        // Complexity-based recommendations
        if complexity_stats.cyclomatic_complexity > 15.0 {
            recommendations.push("🔧 Refactor high-complexity functions to improve maintainability.".to_string());
        }
        
        if complexity_stats.max_nesting_depth > 6 {
            recommendations.push("📐 Reduce nesting depth by extracting logic into separate functions.".to_string());
        }
        
        if complexity_stats.average_function_length > 50.0 {
            recommendations.push("✂️ Break down large functions into smaller, focused units.".to_string());
        }
        
        if complexity_stats.average_parameters_per_function > 5.0 {
            recommendations.push("📝 Consider using objects or structs to group related parameters.".to_string());
        }
        
        // Quality-based recommendations
        let quality = &complexity_stats.quality_metrics;
        if quality.maintainability_index < 70.0 {
            recommendations.push("🔧 Focus on improving maintainability through better structure and documentation.".to_string());
        }
        
        if quality.function_size_health < 70.0 {
            recommendations.push("📖 Improve code readability by breaking down large functions.".to_string());
        }
        
        if quality.nesting_depth_health < 70.0 {
            recommendations.push("🧪 Reduce nesting depth to improve testability and readability.".to_string());
        }
        
        if quality.documentation_coverage < 10.0 {
            recommendations.push("💬 Add more comments to explain complex logic and business rules.".to_string());
        }
        
        recommendations.join("\n")
    }
    
    /// Generate enhanced insights with better analysis
    pub fn generate_enhanced_insights(&self, aggregated_stats: &AggregatedStats) -> String {
        let mut insights = Vec::new();
        let complexity_stats = &aggregated_stats.complexity;
        let basic_stats = &aggregated_stats.basic;
        
        // Code structure insights
        if complexity_stats.function_count > 0 {
            let avg_complexity = complexity_stats.cyclomatic_complexity;
            if avg_complexity > 15.0 {
                insights.push("🔴 High complexity detected - consider refactoring for better maintainability".to_string());
            } else if avg_complexity > 10.0 {
                insights.push("🟡 Moderate complexity - monitor for potential simplification opportunities".to_string());
            } else {
                insights.push("🟢 Good complexity levels - well-structured and maintainable code".to_string());
            }
        }
        
        // Documentation insights
        let doc_ratio = basic_stats.doc_lines as f64 / basic_stats.code_lines as f64;
        if doc_ratio > 0.2 {
            insights.push("📚 Excellent documentation coverage - future developers will appreciate this".to_string());
        } else if doc_ratio > 0.1 {
            insights.push("📖 Good documentation coverage - consider expanding for complex areas".to_string());
        } else {
            insights.push("📝 Limited documentation - adding docs will improve maintainability".to_string());
        }
        
        // Size insights
        if basic_stats.total_lines > 10000 {
            insights.push("📁 Large codebase - consider modular organization strategies".to_string());
        } else if basic_stats.total_lines > 1000 {
            insights.push("📂 Well-sized project - good balance of organization and complexity".to_string());
        } else {
            insights.push("📄 Compact codebase - easy to navigate and understand".to_string());
        }
        
        insights.join("\n")
    }
    
    /// Generate enhanced recommendations with actionable advice
    pub fn generate_enhanced_recommendations(&self, aggregated_stats: &AggregatedStats) -> String {
        let mut recommendations = Vec::new();
        let complexity_stats = &aggregated_stats.complexity;
        let basic_stats = &aggregated_stats.basic;
        let ratios = &aggregated_stats.ratios;
        
        // Priority recommendations based on quality metrics
        let quality = &complexity_stats.quality_metrics;
        
        if quality.code_health_score < 60.0 {
            recommendations.push("🚨 URGENT: Code health needs immediate attention - focus on refactoring and testing".to_string());
        } else if quality.code_health_score < 80.0 {
            recommendations.push("⚠️ Code health could be improved - consider incremental refactoring".to_string());
        }
        
        // Specific actionable recommendations
        if complexity_stats.cyclomatic_complexity > 10.0 {
            recommendations.push("🔧 Reduce cyclomatic complexity by extracting methods and simplifying conditionals".to_string());
        }
        
        if complexity_stats.max_nesting_depth > 4 {
            recommendations.push("📐 Reduce nesting depth using early returns and guard clauses".to_string());
        }
        
        if ratios.comment_ratio < 0.1 {
            recommendations.push("💬 Add inline comments to explain business logic and complex algorithms".to_string());
        }
        
        if ratios.doc_ratio < 0.05 {
            recommendations.push("📚 Add API documentation for public functions and classes".to_string());
        }
        
        if basic_stats.average_lines_per_file > 500.0 {
            recommendations.push("📄 Break down large files into smaller, focused modules".to_string());
        }
        
        // Testing recommendations
        let has_tests = basic_stats.stats_by_extension.keys()
            .any(|ext| ext.contains("test") || ext.contains("spec"));
        
        if !has_tests {
            recommendations.push("🧪 Add unit tests to improve code reliability and enable safe refactoring".to_string());
        }
        
        recommendations.join("\n")
    }
    
    /// Generate enhanced individual files section
    pub fn generate_enhanced_individual_files_section(&self, individual_files: &[(String, FileStats)]) -> String {
        if individual_files.is_empty() {
            return String::new();
        }
        
        let mut section = String::from(r#"
        <div class="section">
            <h2 class="section-title">📄 Individual Files Analysis</h2>
            <div class="file-analysis">
                <p>Top files by complexity and size - these may benefit from refactoring:</p>
                <div class="file-list">
        "#);
        
        // Sort files by complexity and size
        let mut sorted_files: Vec<_> = individual_files.iter().collect();
        sorted_files.sort_by(|a, b| {
            let complexity_a = self.calculate_real_file_complexity(&a.0, &a.1);
            let complexity_b = self.calculate_real_file_complexity(&b.0, &b.1);
            complexity_b.partial_cmp(&complexity_a).unwrap_or(std::cmp::Ordering::Equal)
        });
        
        for (file_path, file_stats) in sorted_files.iter().take(20) {
            let complexity = self.calculate_real_file_complexity(file_path, file_stats);
            let complexity_class = self.get_complexity_class_for_score(complexity);
            let functions = self.calculate_real_function_count(file_path, file_stats);
            let risk_level = if complexity > 15.0 { "HIGH" } else if complexity > 10.0 { "MEDIUM" } else { "LOW" };
            
            section.push_str(&format!(
                r#"<div class="file-item">
                    <div class="file-name">{}</div>
                    <div class="file-metrics">
                        <span class="file-metric">Lines: {}</span>
                        <span class="file-metric">Code: {}</span>
                        <span class="file-metric">Functions: {}</span>
                        <span class="file-metric complexity-badge {}">Risk: {}</span>
                    </div>
                </div>"#,
                self.shorten_path(file_path),
                file_stats.total_lines,
                file_stats.code_lines,
                functions,
                complexity_class,
                risk_level
            ));
        }
        
        section.push_str("</div></div></div>");
        section
    }
    
    /// Calculate real file complexity using complexity analysis
    fn calculate_real_file_complexity(&self, file_path: &str, file_stats: &FileStats) -> f64 {
        let complexity_calculator = ComplexityStatsCalculator::new();
        match complexity_calculator.calculate_complexity_stats(file_stats, file_path) {
            Ok(complexity_stats) => complexity_stats.cyclomatic_complexity,
            Err(_) => self.estimate_file_complexity(file_stats), // Fallback to estimation
        }
    }
    
    /// Calculate real function count using complexity analysis
    fn calculate_real_function_count(&self, file_path: &str, file_stats: &FileStats) -> usize {
        let complexity_calculator = ComplexityStatsCalculator::new();
        match complexity_calculator.calculate_complexity_stats(file_stats, file_path) {
            Ok(complexity_stats) => complexity_stats.function_count,
            Err(_) => self.estimate_functions_for_file(file_path, file_stats), // Fallback to estimation
        }
    }
    
    fn generate_language_insights(&self, stats: &BasicStats) -> Vec<String> {
        let mut insights = Vec::new();
        
        // Find the most used language
        if let Some((most_used_ext, most_used_stats)) = stats.stats_by_extension
            .iter()
            .max_by_key(|(_, ext_stats)| ext_stats.total_lines) {
            
            let percentage = (most_used_stats.total_lines as f64 / stats.total_lines as f64) * 100.0;
            
            if percentage > 80.0 {
                insights.push(format!("🎯 Primarily {} codebase ({:.1}%). Consider language-specific best practices.", 
                    self.get_language_name(most_used_ext), percentage));
            } else if percentage > 60.0 {
                insights.push(format!("🌟 Mainly {} with some diversity ({:.1}%). Good balance of technologies.", 
                    self.get_language_name(most_used_ext), percentage));
            }
        }
        
        // Check for polyglot projects
        let language_count = stats.stats_by_extension.len();
        if language_count > 10 {
            insights.push("🌐 Highly polyglot project. Ensure consistent coding standards across languages.".to_string());
        } else if language_count > 5 {
            insights.push("🔄 Multi-language project. Consider documentation for language-specific conventions.".to_string());
        }
        
        insights
    }
    
    fn estimate_complexity_for_extension(&self, ext: &str, ext_stats: &FileStats) -> f64 {
        let base_complexity = match ext {
            "rs" => 3.0,  // Rust tends to be more complex but safer
            "cpp" | "cc" | "cxx" | "c" => 4.0,  // C++ can be very complex
            "java" => 3.5,  // Java is moderately complex
            "py" => 2.5,  // Python is generally simpler
            "js" | "ts" => 3.0,  // JavaScript complexity varies
            "go" => 2.0,  // Go is designed to be simple
            "rb" => 2.5,  // Ruby is generally readable
            "php" => 3.5,  // PHP can be complex
            "cs" => 3.0,  // C# is moderately complex
            "swift" => 3.0,  // Swift is moderately complex
            "kt" => 2.5,  // Kotlin is cleaner than Java
            "html" | "css" | "scss" => 1.0,  // Markup/styling is simpler
            "json" | "yaml" | "toml" => 0.5,  // Config files are simple
            "md" => 0.2,  // Markdown is very simple
            _ => 2.0,  // Default complexity
        };
        
        // Adjust based on file characteristics
        let mut complexity: f64 = base_complexity;
        
        // Larger files tend to be more complex
        if ext_stats.total_lines > 1000 {
            complexity += 2.0;
        } else if ext_stats.total_lines > 500 {
            complexity += 1.0;
        }
        
        // Low comment ratio indicates potential complexity
        let comment_ratio = ext_stats.comment_lines as f64 / ext_stats.total_lines.max(1) as f64;
        if comment_ratio < 0.1 {
            complexity += 1.0;
        }
        
        complexity.min(10.0)
    }
    
    fn estimate_functions_for_extension(&self, ext: &str, ext_stats: &FileStats) -> usize {
        let function_density =         match ext {
            "rs" => 0.05,  // Rust functions tend to be well-sized
            "py" => 0.08,  // Python functions are often smaller
            "js" | "ts" => 0.06,  // JavaScript functions vary
            "java" => 0.04,  // Java methods in classes
            "cpp" | "cc" | "cxx" | "c" => 0.03,  // C++ functions can be larger
            "go" => 0.06,  // Go functions are typically small
            "rb" => 0.07,  // Ruby methods are often small
            "php" => 0.05,  // PHP functions vary
            "cs" => 0.04,  // C# methods in classes
            "swift" => 0.05,  // Swift functions
            "kt" => 0.05,  // Kotlin functions
            "dart" => 0.06,  // Dart functions
            "erl" | "hrl" => 0.04,  // Erlang functions
            "pl" | "pm" => 0.05,  // Perl subroutines
            "r" | "R" => 0.07,  // R functions are often small
            "m" | "mlx" => 0.04,  // MATLAB functions
            _ => 0.05,  // Default density
        };
        
        (ext_stats.code_lines as f64 * function_density) as usize
    }
    
    fn estimate_file_complexity(&self, file_stats: &FileStats) -> f64 {
        let mut complexity = 1.0;
        
        // Base complexity from file size
        complexity += (file_stats.total_lines as f64 / 100.0).min(5.0);
        
        // Comment ratio affects complexity perception
        let comment_ratio = file_stats.comment_lines as f64 / file_stats.total_lines.max(1) as f64;
        if comment_ratio < 0.1 {
            complexity += 1.0;
        }
        
        // Code density
        let code_ratio = file_stats.code_lines as f64 / file_stats.total_lines.max(1) as f64;
        if code_ratio > 0.8 {
            complexity += 1.0;
        }
        
        complexity.min(10.0)
    }
    
    fn estimate_functions_for_file(&self, file_path: &str, file_stats: &FileStats) -> usize {
        let ext = std::path::Path::new(file_path)
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");
        
        self.estimate_functions_for_extension(ext, file_stats)
    }
    
    fn get_complexity_class_for_extension(&self, ext: &str) -> &'static str {
        match ext {
            "cpp" | "cc" | "cxx" | "c" => "complexity-high",
            "java" | "cs" | "php" => "complexity-medium",
            "rs" | "js" | "ts" | "swift" => "complexity-medium",
            "py" | "rb" | "go" | "kt" => "complexity-low",
            "html" | "css" | "scss" => "complexity-very-low",
            "json" | "yaml" | "toml" | "md" => "complexity-very-low",
            _ => "complexity-low",
        }
    }
    
    fn get_complexity_class_for_score(&self, score: f64) -> &'static str {
        if score <= 2.0 {
            "complexity-very-low"
        } else if score <= 4.0 {
            "complexity-low"
        } else if score <= 6.0 {
            "complexity-medium"
        } else if score <= 8.0 {
            "complexity-high"
        } else {
            "complexity-very-high"
        }
    }
    
    fn get_language_name(&self, ext: &str) -> &'static str {
        match ext {
            "rs" => "Rust",
            "py" => "Python",
            "js" => "JavaScript",
            "ts" => "TypeScript",
            "java" => "Java",
            "cpp" | "cc" | "cxx" => "C++",
            "c" => "C",
            "go" => "Go",
            "rb" => "Ruby",
            "php" => "PHP",
            "cs" => "C#",
            "swift" => "Swift",
            "kt" => "Kotlin",
            _ => "Unknown",
        }
    }
    
    fn shorten_path(&self, path: &str) -> String {
        if path.len() > 50 {
            let parts: Vec<&str> = path.split('/').collect();
            if parts.len() > 2 {
                format!(".../{}/{}", parts[parts.len() - 2], parts[parts.len() - 1])
            } else {
                path.chars().take(47).collect::<String>() + "..."
            }
        } else {
            path.to_string()
        }
    }
    

} 