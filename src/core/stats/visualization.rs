use crate::core::stats::aggregation::AggregatedStats;
use serde::{Deserialize, Serialize};

/// Pie chart data for visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PieChartData {
    pub labels: Vec<String>,
    pub values: Vec<f64>,
    pub colors: Vec<String>,
    pub total: f64,
}

/// Chart configuration options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartConfig {
    pub title: String,
    pub show_percentages: bool,
    pub show_values: bool,
    pub color_scheme: ColorScheme,
    pub min_slice_percentage: f64, // Minimum percentage to show a slice
}

/// Color schemes for charts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ColorScheme {
    Default,
    Pastel,
    Vibrant,
    Monochrome,
    LanguageSpecific,
}

impl Default for ChartConfig {
    fn default() -> Self {
        Self {
            title: "Distribution".to_string(),
            show_percentages: true,
            show_values: false,
            color_scheme: ColorScheme::LanguageSpecific,
            min_slice_percentage: 1.0, // Only show slices >= 1%
        }
    }
}

/// Visualization generator for statistics
pub struct VisualizationGenerator;

impl VisualizationGenerator {
    pub fn new() -> Self {
        Self
    }
    
    /// Generate language distribution pie chart data
    pub fn generate_language_distribution(&self, stats: &AggregatedStats, config: &ChartConfig) -> PieChartData {
        let mut data = Vec::new();
        let total_lines = stats.basic.total_lines as f64;
        
        for (ext, ext_stats) in &stats.basic.stats_by_extension {
            let percentage = (ext_stats.total_lines as f64 / total_lines) * 100.0;
            if percentage >= config.min_slice_percentage {
                data.push((ext.clone(), ext_stats.total_lines as f64, percentage));
            }
        }
        
        // Sort by value (descending)
        data.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        
        // Group small slices into "Others" if needed
        let mut labels = Vec::new();
        let mut values = Vec::new();
        let mut others_value = 0.0;
        
        for (ext, value, percentage) in data {
            if percentage >= config.min_slice_percentage && labels.len() < 10 {
                labels.push(self.format_language_label(&ext));
                values.push(value);
            } else {
                others_value += value;
            }
        }
        
        if others_value > 0.0 {
            labels.push("Others".to_string());
            values.push(others_value);
        }
        
        let colors = self.generate_colors(&labels, &config.color_scheme);
        
        PieChartData {
            labels: labels.clone(),
            values,
            colors,
            total: total_lines,
        }
    }
    
    /// Generate file count distribution pie chart data
    pub fn generate_file_count_distribution(&self, stats: &AggregatedStats, config: &ChartConfig) -> PieChartData {
        let mut data = Vec::new();
        let total_files = stats.basic.total_files as f64;
        
        for (ext, ext_stats) in &stats.basic.stats_by_extension {
            let percentage = (ext_stats.file_count as f64 / total_files) * 100.0;
            if percentage >= config.min_slice_percentage {
                data.push((ext.clone(), ext_stats.file_count as f64, percentage));
            }
        }
        
        // Sort by value (descending)
        data.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        
        let mut labels = Vec::new();
        let mut values = Vec::new();
        let mut others_value = 0.0;
        
        for (ext, value, percentage) in data {
            if percentage >= config.min_slice_percentage && labels.len() < 10 {
                labels.push(self.format_language_label(&ext));
                values.push(value);
            } else {
                others_value += value;
            }
        }
        
        if others_value > 0.0 {
            labels.push("Others".to_string());
            values.push(others_value);
        }
        
        let colors = self.generate_colors(&labels, &config.color_scheme);
        
        PieChartData {
            labels: labels.clone(),
            values,
            colors,
            total: total_files,
        }
    }
    
    /// Generate code structure distribution pie chart data
    pub fn generate_structure_distribution(&self, stats: &AggregatedStats, config: &ChartConfig) -> PieChartData {
        let mut data = Vec::new();
        let total_structures = stats.complexity.total_structures as f64;
        
        if total_structures == 0.0 {
            return PieChartData {
                labels: vec!["No structures found".to_string()],
                values: vec![1.0],
                colors: vec!["#cccccc".to_string()],
                total: 1.0,
            };
        }
        
        if stats.complexity.class_count > 0 {
            data.push(("Classes".to_string(), stats.complexity.class_count as f64));
        }
        if stats.complexity.interface_count > 0 {
            data.push(("Interfaces".to_string(), stats.complexity.interface_count as f64));
        }
        if stats.complexity.trait_count > 0 {
            data.push(("Traits".to_string(), stats.complexity.trait_count as f64));
        }
        if stats.complexity.enum_count > 0 {
            data.push(("Enums".to_string(), stats.complexity.enum_count as f64));
        }
        if stats.complexity.struct_count > 0 {
            data.push(("Structs".to_string(), stats.complexity.struct_count as f64));
        }
        if stats.complexity.module_count > 0 {
            data.push(("Modules".to_string(), stats.complexity.module_count as f64));
        }
        
        // Sort by value (descending)
        data.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        
        let labels: Vec<String> = data.iter().map(|(label, _)| label.clone()).collect();
        let values: Vec<f64> = data.iter().map(|(_, value)| *value).collect();
        let colors = self.generate_colors(&labels, &config.color_scheme);
        
        PieChartData {
            labels: labels.clone(),
            values,
            colors: colors[..labels.len()].to_vec(),
            total: total_structures,
        }
    }
    
    /// Generate complexity distribution pie chart data
    pub fn generate_complexity_distribution(&self, stats: &AggregatedStats, _config: &ChartConfig) -> PieChartData {
        let dist = &stats.complexity.complexity_distribution;
        let total = (dist.low_complexity + dist.medium_complexity + dist.high_complexity + dist.very_high_complexity) as f64;
        
        if total == 0.0 {
            return PieChartData {
                labels: vec!["No functions found".to_string()],
                values: vec![1.0],
                colors: vec!["#cccccc".to_string()],
                total: 1.0,
            };
        }
        
        let mut labels = Vec::new();
        let mut values = Vec::new();
        
        if dist.low_complexity > 0 {
            labels.push("Low Complexity (1-10)".to_string());
            values.push(dist.low_complexity as f64);
        }
        if dist.medium_complexity > 0 {
            labels.push("Medium Complexity (11-20)".to_string());
            values.push(dist.medium_complexity as f64);
        }
        if dist.high_complexity > 0 {
            labels.push("High Complexity (21-50)".to_string());
            values.push(dist.high_complexity as f64);
        }
        if dist.very_high_complexity > 0 {
            labels.push("Very High Complexity (50+)".to_string());
            values.push(dist.very_high_complexity as f64);
        }
        
        let colors = vec![
            "#28a745".to_string(), // Green for low
            "#ffc107".to_string(), // Yellow for medium
            "#fd7e14".to_string(), // Orange for high
            "#dc3545".to_string(), // Red for very high
        ];
        
        PieChartData {
            labels: labels.clone(),
            values,
            colors: colors[..labels.len()].to_vec(),
            total,
        }
    }
    
    /// Generate line type distribution pie chart data
    pub fn generate_line_type_distribution(&self, stats: &AggregatedStats, _config: &ChartConfig) -> PieChartData {
        let total_lines = stats.basic.total_lines as f64;
        
        let mut labels = Vec::new();
        let mut values = Vec::new();
        
        if stats.basic.code_lines > 0 {
            labels.push("Code Lines".to_string());
            values.push(stats.basic.code_lines as f64);
        }
        if stats.basic.comment_lines > 0 {
            labels.push("Comment Lines".to_string());
            values.push(stats.basic.comment_lines as f64);
        }
        if stats.basic.doc_lines > 0 {
            labels.push("Documentation Lines".to_string());
            values.push(stats.basic.doc_lines as f64);
        }
        if stats.basic.blank_lines > 0 {
            labels.push("Blank Lines".to_string());
            values.push(stats.basic.blank_lines as f64);
        }
        
        let colors = vec![
            "#007bff".to_string(), // Blue for code
            "#6c757d".to_string(), // Gray for comments
            "#17a2b8".to_string(), // Teal for docs
            "#f8f9fa".to_string(), // Light gray for blanks
        ];
        
        PieChartData {
            labels: labels.clone(),
            values,
            colors: colors[..labels.len()].to_vec(),
            total: total_lines,
        }
    }
    
    /// Format language label with emoji and proper name
    fn format_language_label(&self, ext: &str) -> String {
        let (emoji, name) = self.get_language_info(ext);
        format!("{} {}", emoji, name)
    }
    /// Get language icon and proper name
    fn get_language_info(&self, ext: &str) -> (&'static str, &'static str) {
        match ext {
            "rs" => ("●", "Rust"),
            "py" => ("●", "Python"),
            "js" => ("●", "JavaScript"),
            "jsx" => ("●", "React JSX"),
            "ts" => ("●", "TypeScript"),
            "tsx" => ("●", "React TSX"),
            "html" => ("●", "HTML"),
            "css" => ("●", "CSS"),
            "scss" => ("●", "Sass"),
            "sass" => ("●", "Sass"),
            "json" => ("●", "JSON"),
            "xml" => ("●", "XML"),
            "yaml" | "yml" => ("●", "YAML"),
            "toml" => ("●", "TOML"),
            "md" => ("●", "Markdown"),
            "txt" => ("●", "Text"),
            "java" => ("●", "Java"),
            "c" => ("●", "C"),
            "cpp" | "cc" | "cxx" => ("●", "C++"),
            "h" | "hpp" => ("●", "C/C++ Header"),
            "go" => ("●", "Go"),
            "php" => ("●", "PHP"),
            "rb" => ("●", "Ruby"),
            "swift" => ("●", "Swift"),
            "kt" => ("●", "Kotlin"),
            "scala" => ("●", "Scala"),
            "sh" | "bash" | "zsh" => ("●", "Shell"),
            "cs" => ("●", "C#"),
            "vb" | "vbs" => ("●", "Visual Basic"),
            "dart" => ("●", "Dart"),
            "r" => ("●", "R"),
            "sql" => ("●", "SQL"),
            "hs" | "lhs" | "hsc" => ("●", "Haskell"),
            "ex" | "exs" | "eex" => ("●", "Elixir"),
            "erl" | "hrl" => ("●", "Erlang"),
            "jl" => ("●", "Julia"),
            "lua" => ("●", "Lua"),
            "pl" | "pm" | "pod" => ("●", "Perl"),
            "zig" => ("●", "Zig"),
            "clj" | "cljs" | "cljc" => ("●", "Clojure"),
            "ps1" | "psm1" | "psd1" => ("●", "PowerShell"),
            "bat" | "cmd" => ("●", "Batch"),
            "mlx" => ("●", "MATLAB"),
            "rmd" | "Rmd" => ("●", "R Markdown"),
            _ => ("●", "Unknown"),
        }
    }
    
    /// Generate colors for chart based on color scheme
    fn generate_colors(&self, labels: &[String], scheme: &ColorScheme) -> Vec<String> {
        match scheme {
            ColorScheme::LanguageSpecific => self.generate_language_specific_colors(labels),
            ColorScheme::Default => self.generate_default_colors(labels.len()),
            ColorScheme::Pastel => self.generate_pastel_colors(labels.len()),
            ColorScheme::Vibrant => self.generate_vibrant_colors(labels.len()),
            ColorScheme::Monochrome => self.generate_monochrome_colors(labels.len()),
        }
    }
    
    /// Generate language-specific colors
    fn generate_language_specific_colors(&self, labels: &[String]) -> Vec<String> {
        labels.iter().map(|label| {
            // Extract extension from formatted label
            let ext = if label.contains("Rust") { "rs" }
            else if label.contains("Python") { "py" }
            else if label.contains("JavaScript") { "js" }
            else if label.contains("TypeScript") { "ts" }
            else if label.contains("HTML") { "html" }
            else if label.contains("CSS") { "css" }
            else if label.contains("Java") && !label.contains("JavaScript") { "java" }
            else if label.contains("C++") { "cpp" }
            else if label.contains("Go") { "go" }
            else if label.contains("PHP") { "php" }
            else if label.contains("Ruby") { "rb" }
            else if label.contains("Swift") { "swift" }
            else { "default" };
            
            self.get_language_color(ext).to_string()
        }).collect()
    }
    
    /// Get language-specific color
    fn get_language_color(&self, ext: &str) -> &'static str {
        match ext {
            "rs" => "#dea584",      // Rust orange
            "py" => "#3776ab",      // Python blue
            "js" => "#f7df1e",      // JavaScript yellow
            "ts" => "#3178c6",      // TypeScript blue
            "html" => "#e34f26",    // HTML orange-red
            "css" => "#1572b6",     // CSS blue
            "java" => "#ed8b00",    // Java orange
            "cpp" => "#00599c",     // C++ blue
            "go" => "#00add8",      // Go cyan
            "php" => "#777bb4",     // PHP purple
            "rb" => "#cc342d",      // Ruby red
            "swift" => "#fa7343",   // Swift orange
            _ => "#6c757d",         // Default gray
        }
    }
    
    /// Generate default color palette
    fn generate_default_colors(&self, count: usize) -> Vec<String> {
        let colors = [
            "#007bff", "#28a745", "#dc3545", "#ffc107", "#17a2b8",
            "#6f42c1", "#e83e8c", "#fd7e14", "#20c997", "#6c757d"
        ];
        
        (0..count).map(|i| colors[i % colors.len()].to_string()).collect()
    }
    
    /// Generate pastel color palette
    fn generate_pastel_colors(&self, count: usize) -> Vec<String> {
        let colors = [
            "#ffb3ba", "#ffdfba", "#ffffba", "#baffc9", "#bae1ff",
            "#d4baff", "#ffb3d4", "#c9baff", "#baffe1", "#ffc9ba"
        ];
        
        (0..count).map(|i| colors[i % colors.len()].to_string()).collect()
    }
    
    /// Generate vibrant color palette
    fn generate_vibrant_colors(&self, count: usize) -> Vec<String> {
        let colors = [
            "#ff6b6b", "#4ecdc4", "#45b7d1", "#96ceb4", "#feca57",
            "#ff9ff3", "#54a0ff", "#5f27cd", "#00d2d3", "#ff9f43"
        ];
        
        (0..count).map(|i| colors[i % colors.len()].to_string()).collect()
    }
    
    /// Generate monochrome color palette
    fn generate_monochrome_colors(&self, count: usize) -> Vec<String> {
        (0..count).map(|i| {
            let intensity = 0.2 + (0.6 * i as f32 / count.max(1) as f32);
            let gray_value = (255.0 * intensity) as u8;
            format!("#{:02x}{:02x}{:02x}", gray_value, gray_value, gray_value)
        }).collect()
    }
    
    /// Convert pie chart data to Chart.js format
    pub fn to_chartjs_format(&self, data: &PieChartData, config: &ChartConfig) -> serde_json::Value {
        let mut labels = data.labels.clone();
        let chart_data = data.values.clone();
        
        // Add percentages to labels if requested
        if config.show_percentages {
            labels = labels.iter().zip(chart_data.iter()).map(|(label, value)| {
                let percentage = (value / data.total) * 100.0;
                format!("{} ({:.1}%)", label, percentage)
            }).collect();
        }
        
        serde_json::json!({
            "type": "doughnut",
            "data": {
                "labels": labels,
                "datasets": [{
                    "data": chart_data,
                    "backgroundColor": data.colors,
                    "borderWidth": 2,
                    "borderColor": "#fff"
                }]
            },
            "options": {
                "responsive": true,
                "maintainAspectRatio": false,
                "plugins": {
                    "title": {
                        "display": true,
                        "text": config.title
                    },
                    "legend": {
                        "position": "bottom",
                        "labels": {
                            "padding": 20,
                            "usePointStyle": true
                        }
                    }
                }
            }
        })
    }
}

impl Default for VisualizationGenerator {
    fn default() -> Self {
        Self::new()
    }
} 