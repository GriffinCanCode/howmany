use crate::core::types::{CodeStats, FileStats};

use crossterm::event::KeyCode;
use ratatui::widgets::{ListState, TableState};
use std::time::Instant;
use std::fs;
use std::path::Path;
use crate::ui::html::HtmlReporter;
use crate::utils::errors::Result;
use serde_json;

#[derive(Debug, Clone, PartialEq)]
pub enum AppMode {
    Overview,
    Languages,
    Export,
    Help,
    Search,
}





#[derive(Debug, Clone)]
pub enum ExportFormat {
    Text,
    Json,
    Csv,
    Html,
}

#[derive(Debug, Clone)]
pub struct ExportState {
    pub selected_format: ExportFormat,
    pub export_status: String,
    pub last_export_path: Option<String>,
}

impl Default for ExportState {
    fn default() -> Self {
        Self {
            selected_format: ExportFormat::Html,
            export_status: "Ready to export".to_string(),
            last_export_path: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SearchState {
    pub query: String,
    pub is_active: bool,
    pub results: Vec<SearchResult>,
    pub selected_result: usize,
    pub search_mode: SearchMode,
}

#[derive(Debug, Clone)]
pub enum SearchMode {
    Files,
    Extensions,
    Content,
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub file_path: String,
    pub line_count: usize,
    pub code_lines: usize,
    pub match_type: String,
    pub relevance_score: f64,
}

impl Default for SearchState {
    fn default() -> Self {
        Self {
            query: String::new(),
            is_active: false,
            results: Vec::new(),
            selected_result: 0,
            search_mode: SearchMode::Files,
        }
    }
}

#[derive(Debug)]
pub struct InteractiveApp {
    pub mode: AppMode,
    pub selected_tab: usize,
    pub table_state: TableState,
    pub list_state: ListState,

    pub stats: Option<CodeStats>,
    pub individual_files: Vec<(String, FileStats)>,

    pub should_quit: bool,
    pub show_help: bool,
    pub animation_frame: usize,
    pub last_animation_update: Instant,

    pub export_state: ExportState,
    pub search_state: SearchState,
    pub filtered_files: Vec<(String, FileStats)>,
    pub filtered_extensions: Vec<String>,
    pub language_stats: std::collections::HashMap<String, (crate::ui::interactive::utils::LanguageInfo, usize, FileStats)>,
    pub show_code_health: bool,
}

impl Default for InteractiveApp {
    fn default() -> Self {
        Self {
            mode: AppMode::Overview,
            selected_tab: 0,
            table_state: TableState::default(),
            list_state: ListState::default(),

            stats: None,
            individual_files: Vec::new(),

            should_quit: false,
            show_help: false,
            animation_frame: 0,
            last_animation_update: Instant::now(),

            export_state: ExportState::default(),
            search_state: SearchState::default(),
            filtered_files: Vec::new(),
            filtered_extensions: Vec::new(),
            language_stats: std::collections::HashMap::new(),
            show_code_health: false,
        }
    }
}

impl InteractiveApp {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_data(&mut self, stats: CodeStats, individual_files: Vec<(String, FileStats)>) {
        self.stats = Some(stats.clone());
        self.individual_files = individual_files.clone();
        self.filtered_files = individual_files.clone();

        self.update_filtered_extensions();
        self.update_language_stats(&stats);
    }
    












    pub fn toggle_search(&mut self) {
        self.search_state.is_active = !self.search_state.is_active;
        if self.search_state.is_active {
            self.mode = AppMode::Search;
        } else {
            self.search_state.query.clear();
            self.search_state.results.clear();
            self.update_mode();
        }
    }

    pub fn handle_search_input(&mut self, c: char) {
        if self.search_state.is_active {
            self.search_state.query.push(c);
            self.perform_search();
        }
    }

    pub fn handle_search_backspace(&mut self) {
        if self.search_state.is_active {
            self.search_state.query.pop();
            self.perform_search();
        }
    }

    fn perform_search(&mut self) {
        if self.search_state.query.is_empty() {
            self.search_state.results.clear();
            self.filtered_files = self.individual_files.clone();
            self.update_filtered_extensions();
            return;
        }

        let query = self.search_state.query.to_lowercase();
        let mut results = Vec::new();

        match self.search_state.search_mode {
            SearchMode::Files => {
                for (file_path, file_stats) in &self.individual_files {
                    if file_path.to_lowercase().contains(&query) {
                        let relevance = self.calculate_file_relevance(file_path, &query);
                        results.push(SearchResult {
                            file_path: file_path.clone(),
                            line_count: file_stats.total_lines,
                            code_lines: file_stats.code_lines,
                            match_type: "File Name".to_string(),
                            relevance_score: relevance,
                        });
                    }
                }
            }
            SearchMode::Extensions => {
                if let Some(ref stats) = self.stats {
                    for (ext, _) in &stats.stats_by_extension {
                        if ext.to_lowercase().contains(&query) {
                            // Find files with this extension
                            for (file_path, file_stats) in &self.individual_files {
                                if file_path.ends_with(&format!(".{}", ext)) {
                                    results.push(SearchResult {
                                        file_path: file_path.clone(),
                                        line_count: file_stats.total_lines,
                                        code_lines: file_stats.code_lines,
                                        match_type: format!("Extension: {}", ext),
                                        relevance_score: 0.8,
                                    });
                                }
                            }
                        }
                    }
                }
            }
            SearchMode::Content => {
                // Simple content search based on file types and patterns
                for (file_path, file_stats) in &self.individual_files {
                    let file_content_match = self.estimate_content_match(file_path, &query);
                    if file_content_match > 0.0 {
                        results.push(SearchResult {
                            file_path: file_path.clone(),
                            line_count: file_stats.total_lines,
                            code_lines: file_stats.code_lines,
                            match_type: "Content Match".to_string(),
                            relevance_score: file_content_match,
                        });
                    }
                }
            }
        }

        // Sort by relevance
        results.sort_by(|a, b| b.relevance_score.partial_cmp(&a.relevance_score).unwrap());
        
        self.search_state.results = results;
        self.search_state.selected_result = 0;
        
        // Update filtered files
        self.filtered_files = self.search_state.results.iter()
            .map(|result| {
                let file_stats = self.individual_files.iter()
                    .find(|(path, _)| path == &result.file_path)
                    .map(|(_, stats)| stats.clone())
                    .unwrap_or_default();
                (result.file_path.clone(), file_stats)
            })
            .collect();
        
        self.update_filtered_extensions();
    }

    fn calculate_file_relevance(&self, file_path: &str, query: &str) -> f64 {
        let file_name = file_path.split('/').last().unwrap_or(file_path);
        let file_lower = file_name.to_lowercase();
        
        // Exact match gets highest score
        if file_lower == query {
            return 1.0;
        }
        
        // Starts with query gets high score
        if file_lower.starts_with(query) {
            return 0.9;
        }
        
        // Contains query gets medium score
        if file_lower.contains(query) {
            return 0.7;
        }
        
        // Fuzzy match gets lower score
        let similarity = self.fuzzy_match(&file_lower, query);
        similarity * 0.5
    }

    fn estimate_content_match(&self, file_path: &str, query: &str) -> f64 {
        // Simple heuristic based on file type and query
        let extension = file_path.split('.').last().unwrap_or("");
        
        // Programming language keywords
        let keywords = match extension {
            "rs" => vec!["fn", "struct", "impl", "trait", "enum", "mod", "use", "pub", "let", "mut"],
            "py" => vec!["def", "class", "import", "from", "if", "else", "for", "while", "try", "except"],
            "js" | "ts" => vec!["function", "class", "const", "let", "var", "if", "else", "for", "while", "try", "catch"],
            "java" => vec!["public", "private", "class", "interface", "extends", "implements", "import", "package"],
            "cpp" | "cc" | "cxx" => vec!["class", "struct", "namespace", "template", "public", "private", "protected"],
            _ => vec![],
        };
        
        if keywords.contains(&query) {
            return 0.8;
        }
        
        // Check if query might be a common programming concept
        let common_terms = vec!["main", "init", "config", "util", "helper", "test", "spec", "mock"];
        if common_terms.iter().any(|term| file_path.to_lowercase().contains(term) && query.contains(term)) {
            return 0.6;
        }
        
        0.0
    }

    fn fuzzy_match(&self, text: &str, pattern: &str) -> f64 {
        if pattern.is_empty() {
            return 0.0;
        }
        
        let text_chars: Vec<char> = text.chars().collect();
        let pattern_chars: Vec<char> = pattern.chars().collect();
        
        let mut matches = 0;
        let mut pattern_idx = 0;
        
        for &ch in &text_chars {
            if pattern_idx < pattern_chars.len() && ch == pattern_chars[pattern_idx] {
                matches += 1;
                pattern_idx += 1;
            }
        }
        
        matches as f64 / pattern_chars.len() as f64
    }

    fn update_filtered_extensions(&mut self) {
        if let Some(ref stats) = self.stats {
            self.filtered_extensions = stats.stats_by_extension.keys()
                .filter(|ext| {
                    self.filtered_files.iter().any(|(path, _)| path.ends_with(&format!(".{}", ext)))
                })
                .cloned()
                .collect();
        }
    }

    fn update_language_stats(&mut self, stats: &CodeStats) {
        self.language_stats = crate::ui::interactive::utils::group_extensions_by_language(&stats.stats_by_extension);
    }

    pub fn cycle_search_mode(&mut self) {
        self.search_state.search_mode = match self.search_state.search_mode {
            SearchMode::Files => SearchMode::Extensions,
            SearchMode::Extensions => SearchMode::Content,
            SearchMode::Content => SearchMode::Files,
        };
        self.perform_search();
    }

    pub fn handle_key_event(&mut self, key: KeyCode) {
        // Handle search mode first with high priority
        if self.search_state.is_active {
            match key {
                KeyCode::Esc => self.toggle_search(),
                KeyCode::Enter => {
                    if !self.search_state.results.is_empty() {
                        // Jump to selected result
                        self.toggle_search();
                        self.switch_to_tab(2); // Individual files tab
                    }
                }
                KeyCode::Tab => self.cycle_search_mode(),
                KeyCode::Up => {
                    if self.search_state.selected_result > 0 {
                        self.search_state.selected_result -= 1;
                    }
                }
                KeyCode::Down => {
                    if self.search_state.selected_result < self.search_state.results.len().saturating_sub(1) {
                        self.search_state.selected_result += 1;
                    }
                }
                KeyCode::Backspace => self.handle_search_backspace(),
                KeyCode::Char(c) => self.handle_search_input(c),
                _ => {}
            }
            return;
        }

        // Handle global keys with immediate response
        match key {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.should_quit = true;
                return; // Immediate quit
            },
            KeyCode::Char('h') | KeyCode::F(1) => {
                self.show_help = !self.show_help;
                return; // Immediate toggle
            },
            KeyCode::Char('/') | KeyCode::Char('s') => {
                self.toggle_search();
                return; // Immediate search toggle
            },
            KeyCode::Tab => {
                self.next_tab();
                return; // Immediate tab switch
            },
            KeyCode::BackTab => {
                self.prev_tab();
                return; // Immediate tab switch
            },
            _ => {}
        }

        // Handle mode-specific keys
        match key {
            KeyCode::Char('t') => {
                // Toggle code health in languages page
                if self.mode == AppMode::Languages {
                    self.show_code_health = !self.show_code_health;
                }
            },
            KeyCode::Char('1') => {
                if self.mode == AppMode::Export {
                    self.select_export_format(ExportFormat::Text);
                } else {
                    self.switch_to_tab(0);
                }
            },
            KeyCode::Char('2') => {
                if self.mode == AppMode::Export {
                    self.select_export_format(ExportFormat::Json);
                } else {
                    self.switch_to_tab(1);
                }
            },
            KeyCode::Char('3') => {
                if self.mode == AppMode::Export {
                    self.select_export_format(ExportFormat::Csv);
                } else {
                    self.switch_to_tab(2);
                }
            },
            KeyCode::Char('4') => {
                if self.mode == AppMode::Export {
                    self.select_export_format(ExportFormat::Html);
                }
                // Tab 3 (CodeHealth) no longer exists - integrated into Languages
            },
            KeyCode::Down | KeyCode::Char('j') => self.scroll_down(),
            KeyCode::Up | KeyCode::Char('k') => self.scroll_up(),
            KeyCode::PageDown => self.page_down(),
            KeyCode::PageUp => self.page_up(),
            KeyCode::Home => self.scroll_to_top(),
            KeyCode::End => self.scroll_to_bottom(),
            KeyCode::Enter | KeyCode::Right => self.handle_enter_key(),
            KeyCode::Left => {
                // Directory tree functionality removed
            },
            _ => {}
        }
    }

    fn next_tab(&mut self) {
        self.selected_tab = (self.selected_tab + 1) % 3;
        self.update_mode();
    }

    fn prev_tab(&mut self) {
        self.selected_tab = if self.selected_tab == 0 { 2 } else { self.selected_tab - 1 };
        self.update_mode();
    }

    fn switch_to_tab(&mut self, tab: usize) {
        if tab < 3 {
            self.selected_tab = tab;
            self.update_mode();
        }
    }

    fn update_mode(&mut self) {
        self.mode = match self.selected_tab {
            0 => AppMode::Overview,
            1 => AppMode::Languages,
            2 => AppMode::Export,
            _ => AppMode::Overview,
        };
    }

    pub fn get_current_files(&self) -> &[(String, FileStats)] {
        &self.filtered_files
    }

    pub fn get_current_extensions(&self) -> &[String] {
        &self.filtered_extensions
    }







    fn scroll_down(&mut self) {
        match self.mode {
            AppMode::Languages => {
                let len = self.language_stats.len();
                if len > 0 {
                    let selected = self.table_state.selected().unwrap_or(0);
                    self.table_state.select(Some((selected + 1).min(len - 1)));
                }
            }


            AppMode::Export => {
                self.export_state.selected_format = match self.export_state.selected_format {
                    ExportFormat::Text => ExportFormat::Json,
                    ExportFormat::Json => ExportFormat::Csv,
                    ExportFormat::Csv => ExportFormat::Html,
                    ExportFormat::Html => ExportFormat::Text,
                };
            }
            _ => {}
        }
    }

    fn scroll_up(&mut self) {
        match self.mode {
            AppMode::Languages => {
                let selected = self.table_state.selected().unwrap_or(0);
                self.table_state.select(Some(selected.saturating_sub(1)));
            }


            AppMode::Export => {
                self.export_state.selected_format = match self.export_state.selected_format {
                    ExportFormat::Text => ExportFormat::Html,
                    ExportFormat::Json => ExportFormat::Text,
                    ExportFormat::Csv => ExportFormat::Json,
                    ExportFormat::Html => ExportFormat::Csv,
                };
            }
            _ => {}
        }
    }

    fn page_down(&mut self) {
        match self.mode {
            AppMode::Languages => {
                let len = self.language_stats.len();
                if len > 0 {
                    let selected = self.table_state.selected().unwrap_or(0);
                    self.table_state.select(Some((selected + 10).min(len - 1)));
                }
            }


            _ => {}
        }
    }

    fn page_up(&mut self) {
        match self.mode {
            AppMode::Languages => {
                let selected = self.table_state.selected().unwrap_or(0);
                self.table_state.select(Some(selected.saturating_sub(10)));
            }


            _ => {}
        }
    }

    fn scroll_to_top(&mut self) {
        match self.mode {
            AppMode::Languages => self.table_state.select(Some(0)),


            _ => {}
        }
    }

    fn scroll_to_bottom(&mut self) {
        match self.mode {
            AppMode::Languages => {
                let len = self.language_stats.len();
                if len > 0 {
                    self.table_state.select(Some(len - 1));
                }
            }


            _ => {}
        }
    }

    pub fn update_animation(&mut self) {
        // This method is now called only when needed from the display loop
        self.animation_frame = (self.animation_frame + 1) % 8;
        self.last_animation_update = Instant::now();
    }

    fn handle_enter_key(&mut self) {
        match self.mode {
            AppMode::Export => self.execute_export(),
            _ => {}
        }
    }

    pub fn select_export_format(&mut self, format: ExportFormat) {
        self.export_state.selected_format = format;
        self.export_state.export_status = "Ready to export".to_string();
    }

    pub fn execute_export(&mut self) {
        if self.stats.is_none() {
            self.export_state.export_status = "Error: No data to export".to_string();
            return;
        }

        let stats = self.stats.as_ref().unwrap();
        let individual_files = &self.individual_files;

        let result = match self.export_state.selected_format {
            ExportFormat::Text => self.export_text(stats, individual_files),
            ExportFormat::Json => self.export_json(stats, individual_files),
            ExportFormat::Csv => self.export_csv(stats, individual_files),
            ExportFormat::Html => self.export_html(stats, individual_files),
        };

        match result {
            Ok(filename) => {
                self.export_state.export_status = format!("✅ Success: Exported to {}", filename);
                self.export_state.last_export_path = Some(filename);
            }
            Err(e) => {
                self.export_state.export_status = format!("❌ Error: {}", e);
                self.export_state.last_export_path = None;
            }
        }
    }

    fn export_text(&self, stats: &CodeStats, individual_files: &[(String, FileStats)]) -> Result<String> {
        let filename = "howmany-report.txt";
        let mut content = String::new();
        
        content.push_str("=== HowMany Code Analysis Report ===\n\n");
        content.push_str(&format!("Total files: {}\n", stats.total_files));
        content.push_str(&format!("Total lines: {}\n", stats.total_lines));
        content.push_str(&format!("Code lines: {}\n", stats.total_code_lines));
        content.push_str(&format!("Comment lines: {}\n", stats.total_comment_lines));
        content.push_str(&format!("Documentation lines: {}\n", stats.total_doc_lines));
        content.push_str(&format!("Blank lines: {}\n", stats.total_blank_lines));
        content.push_str(&format!("Total size: {} bytes\n\n", stats.total_size));
        
        content.push_str("=== Breakdown by Extension ===\n");
        for (ext, (file_count, file_stats)) in &stats.stats_by_extension {
            content.push_str(&format!("{}: {} files, {} lines ({} code, {} docs, {} comments)\n", 
                ext, file_count, file_stats.total_lines, file_stats.code_lines, 
                file_stats.doc_lines, file_stats.comment_lines));
        }
        
        if !individual_files.is_empty() {
            content.push_str("\n=== Individual Files ===\n");
            for (file_path, file_stats) in individual_files {
                content.push_str(&format!("{}: {} lines ({} code)\n", 
                    file_path, file_stats.total_lines, file_stats.code_lines));
            }
        }
        
        fs::write(filename, content)?;
        Ok(filename.to_string())
    }

    fn export_json(&self, stats: &CodeStats, individual_files: &[(String, FileStats)]) -> Result<String> {
        let filename = "howmany-report.json";
        
        let mut json_stats = serde_json::Map::new();
        json_stats.insert("total_files".to_string(), serde_json::Value::Number(stats.total_files.into()));
        json_stats.insert("total_lines".to_string(), serde_json::Value::Number(stats.total_lines.into()));
        json_stats.insert("total_code_lines".to_string(), serde_json::Value::Number(stats.total_code_lines.into()));
        json_stats.insert("total_comment_lines".to_string(), serde_json::Value::Number(stats.total_comment_lines.into()));
        json_stats.insert("total_doc_lines".to_string(), serde_json::Value::Number(stats.total_doc_lines.into()));
        json_stats.insert("total_blank_lines".to_string(), serde_json::Value::Number(stats.total_blank_lines.into()));
        json_stats.insert("total_size".to_string(), serde_json::Value::Number(stats.total_size.into()));
        
        let mut by_extension = serde_json::Map::new();
        for (ext, (file_count, file_stats)) in &stats.stats_by_extension {
            let mut ext_data = serde_json::Map::new();
            ext_data.insert("files".to_string(), serde_json::Value::Number((*file_count).into()));
            ext_data.insert("total_lines".to_string(), serde_json::Value::Number(file_stats.total_lines.into()));
            ext_data.insert("code_lines".to_string(), serde_json::Value::Number(file_stats.code_lines.into()));
            ext_data.insert("comment_lines".to_string(), serde_json::Value::Number(file_stats.comment_lines.into()));
            ext_data.insert("doc_lines".to_string(), serde_json::Value::Number(file_stats.doc_lines.into()));
            ext_data.insert("blank_lines".to_string(), serde_json::Value::Number(file_stats.blank_lines.into()));
            ext_data.insert("file_size".to_string(), serde_json::Value::Number(file_stats.file_size.into()));
            
            by_extension.insert(ext.clone(), serde_json::Value::Object(ext_data));
        }
        json_stats.insert("by_extension".to_string(), serde_json::Value::Object(by_extension));
        
        if !individual_files.is_empty() {
            let mut files_data = serde_json::Map::new();
            for (file_path, file_stats) in individual_files {
                let mut file_data = serde_json::Map::new();
                file_data.insert("total_lines".to_string(), serde_json::Value::Number(file_stats.total_lines.into()));
                file_data.insert("code_lines".to_string(), serde_json::Value::Number(file_stats.code_lines.into()));
                file_data.insert("comment_lines".to_string(), serde_json::Value::Number(file_stats.comment_lines.into()));
                file_data.insert("doc_lines".to_string(), serde_json::Value::Number(file_stats.doc_lines.into()));
                file_data.insert("blank_lines".to_string(), serde_json::Value::Number(file_stats.blank_lines.into()));
                file_data.insert("file_size".to_string(), serde_json::Value::Number(file_stats.file_size.into()));
                
                files_data.insert(file_path.clone(), serde_json::Value::Object(file_data));
            }
            json_stats.insert("individual_files".to_string(), serde_json::Value::Object(files_data));
        }
        
        let json_output = serde_json::Value::Object(json_stats);
        let content = serde_json::to_string_pretty(&json_output)?;
        fs::write(filename, content)?;
        Ok(filename.to_string())
    }

    fn export_csv(&self, stats: &CodeStats, _individual_files: &[(String, FileStats)]) -> Result<String> {
        let filename = "howmany-report.csv";
        let mut content = String::new();
        
        content.push_str("Extension,Files,Total Lines,Code Lines,Comment Lines,Doc Lines,Blank Lines,Size (bytes)\n");
        
        for (ext, (file_count, file_stats)) in &stats.stats_by_extension {
            content.push_str(&format!("{},{},{},{},{},{},{},{}\n", 
                ext,
                file_count,
                file_stats.total_lines,
                file_stats.code_lines,
                file_stats.comment_lines,
                file_stats.doc_lines,
                file_stats.blank_lines,
                file_stats.file_size));
        }
        
        fs::write(filename, content)?;
        Ok(filename.to_string())
    }

    fn export_html(&self, stats: &CodeStats, individual_files: &[(String, FileStats)]) -> Result<String> {
        let filename = "howmany-report.html";
        let reporter = HtmlReporter::new();
        let output_path = Path::new(filename);
        
        // Try to calculate comprehensive stats for better reporting
        let stats_calculator = crate::core::stats::StatsCalculator::new();
        match stats_calculator.calculate_project_stats(stats, individual_files) {
            Ok(aggregated_stats) => {
                // Use comprehensive report with real analysis
                reporter.generate_comprehensive_report(&aggregated_stats, individual_files, output_path)?;
            }
            Err(_) => {
                // Fallback to basic report if comprehensive analysis fails
                reporter.generate_report(stats, individual_files, output_path)?;
            }
        }
        
        Ok(filename.to_string())
    }


} 