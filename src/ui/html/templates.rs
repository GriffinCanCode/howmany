use crate::core::types::{CodeStats, FileStats};
use crate::core::stats::basic::BasicStats;
use super::utils::FileUtils;
use super::time_utils::TimeCalculator;

pub struct TemplateGenerator {
    file_utils: FileUtils,
    time_calculator: TimeCalculator,
}

impl TemplateGenerator {
    pub fn new() -> Self {
        Self {
            file_utils: FileUtils::new(),
            time_calculator: TimeCalculator::new(),
        }
    }
    
    pub fn generate_extension_rows(&self, stats: &CodeStats) -> String {
        let mut rows = String::new();
        let mut extensions: Vec<_> = stats.stats_by_extension.iter().collect();
        extensions.sort_by(|a, b| b.1.1.total_lines.cmp(&a.1.1.total_lines));
        
        for (ext, (file_count, ext_stats)) in extensions {
            let complexity_class = self.get_complexity_class_for_extension(ext);
            let complexity_score = self.estimate_complexity_for_extension(ext, ext_stats);
            
            rows.push_str(&format!(
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
        
        // Sort files by estimated complexity
        let mut sorted_files: Vec<_> = individual_files.iter().collect();
        sorted_files.sort_by(|a, b| {
            let complexity_a = self.estimate_file_complexity(&a.1);
            let complexity_b = self.estimate_file_complexity(&b.1);
            complexity_b.partial_cmp(&complexity_a).unwrap_or(std::cmp::Ordering::Equal)
        });
        
        for (file_path, file_stats) in sorted_files.iter().take(50) {
            let complexity = self.estimate_file_complexity(file_stats);
            let complexity_class = self.get_complexity_class_for_score(complexity);
            let functions = self.estimate_functions_for_file(file_path, file_stats);
            
            section.push_str(&format!(
                r#"<div class="function-item">
                    <div class="function-name">{}</div>
                    <div class="function-metrics">
                        <span class="function-metric">Lines: {}</span>
                        <span class="function-metric">Code: {}</span>
                        <span class="function-metric">Functions: ~{}</span>
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
    
    pub fn generate_complexity_insights(&self, stats: &BasicStats) -> String {
        let mut insights = Vec::new();
        
        // Analyze overall complexity
        let total_lines = stats.total_lines;
        let code_ratio = stats.code_lines as f64 / total_lines as f64;
        let comment_ratio = stats.comment_lines as f64 / total_lines as f64;
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
        let function_density = match ext {
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
    
    pub fn generate_waste_table_rows(&self, stats: &CodeStats) -> String {
        let mut rows = String::new();
        let mut extensions: Vec<_> = stats.stats_by_extension.iter().collect();
        extensions.sort_by(|a, b| {
            let a_total = a.1.1.total_lines;
            let b_total = b.1.1.total_lines;
            b_total.cmp(&a_total)
        });
        
        for (ext, (file_count, file_stats)) in extensions.iter().take(10) {
            let time_wasted = self.time_calculator.calculate_file_type_time(file_stats);
            let regret_level = self.file_utils.get_regret_level(file_stats);
            let could_have_been = self.file_utils.get_alternative_activity(ext, file_stats);
            
            rows.push_str(&format!(
                r#"<tr>
                    <td>{} .{}</td>
                    <td>{}</td>
                    <td>{}</td>
                    <td><span class="waste-badge {}">{}</span></td>
                    <td>{}</td>
                </tr>"#,
                self.file_utils.get_file_emoji(ext),
                ext,
                file_count,
                time_wasted,
                regret_level.1,
                regret_level.0,
                could_have_been
            ));
        }
        
        rows
    }
} 