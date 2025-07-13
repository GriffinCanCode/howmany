use crate::core::types::CodeStats;
use crate::core::stats::techstack::detection::TechStackInventory;
use crate::ui::interactive::app::{AppMode, InteractiveApp, DisplayMode, ExportFormat, SearchMode};
use crate::ui::interactive::utils::{centered_rect, format_size, get_extension_icon, get_file_icon, shorten_path};
use crate::ui::interactive::charts::{render_enhanced_overview, render_quality_metrics, render_complexity_summary, render_function_complexity_table};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, Borders, Cell, Gauge, List, ListItem, ListState, Paragraph, Row, Table, Tabs, Wrap,
    },
};

// Standalone rendering functions to avoid borrow checker issues
pub fn render_header(f: &mut ratatui::Frame, area: Rect, app: &InteractiveApp) {
    let titles = vec!["Overview", "File Types", "Individual Files", "Stack View", "Export", "Quality Analysis"];
    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title(" Navigation "))
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .select(app.selected_tab);
    
    f.render_widget(tabs, area);
}

pub fn render_main_content(f: &mut ratatui::Frame, area: Rect, app: &mut InteractiveApp) {
    if app.search_state.is_active {
        render_search(f, area, app);
    } else {
        match app.mode {
            AppMode::Overview => render_overview(f, area, app),
            AppMode::FileTypes => render_file_types(f, area, app),
            AppMode::IndividualFiles => render_individual_files(f, area, app),
            AppMode::StackView => render_stack_view(f, area, app),
            AppMode::Export => render_export(f, area, app),
            AppMode::QualityAnalysis => render_quality_analysis(f, area, app),
            AppMode::Help => render_help(f, area),
            AppMode::Search => render_search(f, area, app),
        }
    }
}

pub fn render_search(f: &mut ratatui::Frame, area: Rect, app: &InteractiveApp) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Search input
            Constraint::Length(3),  // Search mode and stats
            Constraint::Min(0),     // Results
        ])
        .split(area);
    
    // Search input
    let search_mode_text = match app.search_state.search_mode {
        SearchMode::Files => "Files",
        SearchMode::Extensions => "Extensions",
        SearchMode::Content => "Content",
    };
    
    let search_input = Paragraph::new(format!("🔍 {} Search: {}", search_mode_text, app.search_state.query))
        .block(Block::default().borders(Borders::ALL).title(" Search "))
        .style(Style::default().fg(Color::White));
    f.render_widget(search_input, chunks[0]);
    
    // Search stats
    let results_count = app.search_state.results.len();
    let total_files = app.individual_files.len();
    let stats_text = format!("Found {} results out of {} files | Tab: cycle mode | Esc: exit | Enter: go to result", 
                            results_count, total_files);
    
    let stats_para = Paragraph::new(stats_text)
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(Color::Gray));
    f.render_widget(stats_para, chunks[1]);
    
    // Search results
    if app.search_state.results.is_empty() {
        let no_results = if app.search_state.query.is_empty() {
            "Start typing to search..."
        } else {
            "No results found"
        };
        
        let no_results_para = Paragraph::new(no_results)
            .block(Block::default().borders(Borders::ALL).title(" Results "))
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center);
        f.render_widget(no_results_para, chunks[2]);
    } else {
        let items: Vec<ListItem> = app.search_state.results
            .iter()
            .enumerate()
            .map(|(i, result)| {
                let is_selected = i == app.search_state.selected_result;
                let relevance_bar = "█".repeat((result.relevance_score * 10.0) as usize);
                
                let style = if is_selected {
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };
                
                ListItem::new(vec![
                    Line::from(vec![
                        Span::styled(get_file_icon(&result.file_path), Style::default().fg(Color::Blue)),
                        Span::styled(format!(" {}", shorten_path(&result.file_path, 60)), style),
                    ]),
                    Line::from(vec![
                        Span::styled(format!("  {} | {} lines | {} code | Relevance: {}", 
                                            result.match_type, 
                                            result.line_count, 
                                            result.code_lines,
                                            relevance_bar), 
                                    Style::default().fg(Color::Gray)),
                    ]),
                ])
            })
            .collect();
        
        let results_list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title(" Search Results "))
            .style(Style::default().fg(Color::White));
        
        f.render_widget(results_list, chunks[2]);
    }
}

pub fn render_overview(f: &mut ratatui::Frame, area: Rect, app: &InteractiveApp) {
    if let Some(ref stats) = app.stats {
        // Create aggregated stats from the basic stats
        let aggregated_stats = create_aggregated_stats_from_basic(stats);
        render_enhanced_overview(f, area, &aggregated_stats);
    } else {
        let no_data = Paragraph::new("No data available")
            .block(Block::default().borders(Borders::ALL).title(" Overview "))
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center);
        f.render_widget(no_data, area);
    }
}

pub fn render_quality_analysis(f: &mut ratatui::Frame, area: Rect, app: &InteractiveApp) {
    if let Some(ref stats) = app.stats {
        let aggregated_stats = create_aggregated_stats_from_basic(stats);
        
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(6),  // Complexity summary
                Constraint::Length(12), // Quality metrics
                Constraint::Min(10),    // Function complexity table
            ])
            .split(area);
        
        render_complexity_summary(f, chunks[0], &aggregated_stats);
        render_quality_metrics(f, chunks[1], &aggregated_stats);
        render_function_complexity_table(f, chunks[2], &aggregated_stats);
    } else {
        let no_data = Paragraph::new("No data available")
            .block(Block::default().borders(Borders::ALL).title(" Quality Analysis "))
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center);
        f.render_widget(no_data, area);
    }
}

// Helper function to create aggregated stats from basic stats
fn create_aggregated_stats_from_basic(stats: &CodeStats) -> crate::core::stats::aggregation::AggregatedStats {
    use crate::core::stats::aggregation::AggregatedStats;
    use crate::core::stats::basic::BasicStats;
    use crate::core::stats::complexity::{ComplexityStats, ComplexityDistribution, StructureDistribution, QualityMetrics};
    use crate::core::stats::time::TimeStats;
    use crate::core::stats::ratios::RatioStats;
    use crate::core::stats::aggregation::StatsMetadata;
    use std::collections::HashMap;
    
    // Create basic stats
    let basic_stats = BasicStats {
        total_files: stats.total_files,
        total_lines: stats.total_lines,
        code_lines: stats.total_code_lines,
        comment_lines: stats.total_comment_lines,
        doc_lines: stats.total_doc_lines,
        blank_lines: stats.total_blank_lines,
        total_size: stats.total_size,
        average_file_size: if stats.total_files > 0 { stats.total_size as f64 / stats.total_files as f64 } else { 0.0 },
        average_lines_per_file: if stats.total_files > 0 { stats.total_lines as f64 / stats.total_files as f64 } else { 0.0 },
        largest_file_size: stats.total_size, // Placeholder
        smallest_file_size: if stats.total_size > 0 { stats.total_size } else { 0 },
        stats_by_extension: convert_to_extension_stats(&stats.stats_by_extension),
    };
    
    // Create placeholder complexity stats (these would normally come from the complexity calculator)
    let complexity_stats = ComplexityStats {
        function_count: 0,
        class_count: 0,
        interface_count: 0,
        trait_count: 0,
        enum_count: 0,
        struct_count: 0,
        module_count: 0,
        total_structures: 0,
        cyclomatic_complexity: 0.0,
        cognitive_complexity: 0.0,
        maintainability_index: 85.0,
        average_function_length: 15.0,
        max_function_length: 0,
        min_function_length: 0,
        max_nesting_depth: 0,
        average_nesting_depth: 0.0,
        methods_per_class: 0.0,
        average_parameters_per_function: 2.5,
        max_parameters_per_function: 0,
        complexity_by_extension: HashMap::new(),
        complexity_distribution: ComplexityDistribution {
            very_low_complexity: 0,
            low_complexity: 0,
            medium_complexity: 0,
            high_complexity: 0,
            very_high_complexity: 0,
        },
        structure_distribution: StructureDistribution {
            classes: 0,
            interfaces: 0,
            traits: 0,
            enums: 0,
            structs: 0,
            modules: 0,
        },
        function_complexity_details: Vec::new(),
        quality_metrics: QualityMetrics {
            overall_quality_score: 75.0,
            maintainability_score: 80.0,
            readability_score: 70.0,
            testability_score: 65.0,
            code_duplication_ratio: 10.0,
            comment_coverage_ratio: 25.0,
            function_size_score: 85.0,
            complexity_score: 70.0,
        },
    };
    
    // Create placeholder time stats
    let time_stats = TimeStats {
        total_time_minutes: 0,
        code_time_minutes: 0,
        doc_time_minutes: 0,
        comment_time_minutes: 0,
        total_time_formatted: "0h 0m".to_string(),
        code_time_formatted: "0h 0m".to_string(),
        doc_time_formatted: "0h 0m".to_string(),
        comment_time_formatted: "0h 0m".to_string(),
        time_by_extension: HashMap::new(),
        productivity_metrics: crate::core::stats::time::ProductivityMetrics {
            lines_per_hour: 0.0,
            code_lines_per_hour: 0.0,
            files_per_hour: 0.0,
            estimated_development_days: 0.0,
            estimated_development_hours: 0.0,
        },
    };
    
    // Create placeholder ratio stats
    let ratio_stats = RatioStats {
        code_ratio: if stats.total_lines > 0 { stats.total_code_lines as f64 / stats.total_lines as f64 } else { 0.0 },
        comment_ratio: if stats.total_lines > 0 { stats.total_comment_lines as f64 / stats.total_lines as f64 } else { 0.0 },
        doc_ratio: if stats.total_lines > 0 { stats.total_doc_lines as f64 / stats.total_lines as f64 } else { 0.0 },
        blank_ratio: if stats.total_lines > 0 { stats.total_blank_lines as f64 / stats.total_lines as f64 } else { 0.0 },
        comment_to_code_ratio: if stats.total_code_lines > 0 { stats.total_comment_lines as f64 / stats.total_code_lines as f64 } else { 0.0 },
        doc_to_code_ratio: if stats.total_code_lines > 0 { stats.total_doc_lines as f64 / stats.total_code_lines as f64 } else { 0.0 },
        ratios_by_extension: HashMap::new(),
        language_distribution: HashMap::new(),
        file_distribution: HashMap::new(),
        size_distribution: HashMap::new(),
        quality_metrics: crate::core::stats::ratios::QualityMetrics {
            documentation_score: 75.0,
            maintainability_score: 80.0,
            readability_score: 70.0,
            consistency_score: 85.0,
            overall_quality_score: 77.5,
        },
    };
    
    // Create metadata
    let metadata = StatsMetadata {
        calculation_time_ms: 0,
        version: "0.3.0".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        file_count_analyzed: stats.total_files,
        total_bytes_analyzed: stats.total_size,
        languages_detected: stats.stats_by_extension.keys().cloned().collect(),
        analysis_depth: crate::core::stats::aggregation::AnalysisDepth::Advanced,
    };
    
    AggregatedStats {
        basic: basic_stats,
        complexity: complexity_stats,
        time: time_stats,
        ratios: ratio_stats,
        metadata,
    }
}

// Helper function to convert CodeStats extension stats to BasicStats extension stats
fn convert_to_extension_stats(stats_by_extension: &std::collections::HashMap<String, (usize, crate::core::types::FileStats)>) -> std::collections::HashMap<String, crate::core::stats::basic::ExtensionStats> {
    use crate::core::stats::basic::ExtensionStats;
    
    stats_by_extension.iter().map(|(ext, (file_count, file_stats))| {
        let extension_stats = ExtensionStats {
            file_count: *file_count,
            total_lines: file_stats.total_lines,
            code_lines: file_stats.code_lines,
            comment_lines: file_stats.comment_lines,
            doc_lines: file_stats.doc_lines,
            blank_lines: file_stats.blank_lines,
            total_size: file_stats.file_size,
            average_lines_per_file: if *file_count > 0 { file_stats.total_lines as f64 / *file_count as f64 } else { 0.0 },
            average_size_per_file: if *file_count > 0 { file_stats.file_size as f64 / *file_count as f64 } else { 0.0 },
        };
        (ext.clone(), extension_stats)
    }).collect()
}

pub fn render_main_stats(f: &mut ratatui::Frame, area: Rect, stats: &CodeStats) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .split(area);

    // Files count with enhanced styling
    let files_text = vec![
        Line::from(vec![
            Span::styled("📁 ", Style::default().fg(Color::Yellow)),
            Span::styled("Files", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(format!("{}", stats.total_files), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("tracked", Style::default().fg(Color::DarkGray)),
        ]),
    ];
    let files_block = Paragraph::new(files_text)
        .alignment(Alignment::Center)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow))
            .title(" 📊 Files ")
            .title_alignment(Alignment::Center)
        );
    f.render_widget(files_block, chunks[0]);

    // Total lines with enhanced styling
    let lines_text = vec![
        Line::from(vec![
            Span::styled("📏 ", Style::default().fg(Color::Blue)),
            Span::styled("Total Lines", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(format!("{}", stats.total_lines), Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("all content", Style::default().fg(Color::DarkGray)),
        ]),
    ];
    let lines_block = Paragraph::new(lines_text)
        .alignment(Alignment::Center)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Blue))
            .title(" 📐 Lines ")
            .title_alignment(Alignment::Center)
        );
    f.render_widget(lines_block, chunks[1]);

    // Code lines with enhanced styling
    let code_percentage = if stats.total_lines > 0 {
        (stats.total_code_lines as f64 / stats.total_lines as f64 * 100.0) as u16
    } else {
        0
    };
    let code_text = vec![
        Line::from(vec![
            Span::styled("💻 ", Style::default().fg(Color::Green)),
            Span::styled("Code Lines", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(format!("{}", stats.total_code_lines), Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(format!("{}% of total", code_percentage), Style::default().fg(Color::DarkGray)),
        ]),
    ];
    let code_block = Paragraph::new(code_text)
        .alignment(Alignment::Center)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Green))
            .title(" 🔧 Code ")
            .title_alignment(Alignment::Center)
        );
    f.render_widget(code_block, chunks[2]);

    // Size with enhanced styling
    let size_text = vec![
        Line::from(vec![
            Span::styled("💾 ", Style::default().fg(Color::Cyan)),
            Span::styled("Total Size", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(format_size(stats.total_size), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("on disk", Style::default().fg(Color::DarkGray)),
        ]),
    ];
    let size_block = Paragraph::new(size_text)
        .alignment(Alignment::Center)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .title(" 💿 Size ")
            .title_alignment(Alignment::Center)
        );
    f.render_widget(size_block, chunks[3]);
}

pub fn render_progress_bars(f: &mut ratatui::Frame, area: Rect, stats: &CodeStats) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4),
            Constraint::Length(4),
            Constraint::Length(4),
            Constraint::Length(4),
        ])
        .split(area);

    let total_lines = stats.total_lines as f64;
    
    // Code lines percentage with enhanced styling
    let code_pct = if total_lines > 0.0 { (stats.total_code_lines as f64 / total_lines) * 100.0 } else { 0.0 };
    let code_gauge = Gauge::default()
        .block(Block::default()
            .borders(Borders::ALL)
            .title(format!(" 💻 Code Lines - {:.1}% ", code_pct))
            .title_alignment(Alignment::Center)
            .border_style(Style::default().fg(Color::Green))
        )
        .gauge_style(Style::default().fg(Color::Green).bg(Color::Black))
        .percent(code_pct as u16)
        .label(format!("{} / {} lines", stats.total_code_lines, stats.total_lines));
    f.render_widget(code_gauge, chunks[0]);

    // Comment lines percentage with enhanced styling
    let comment_pct = if total_lines > 0.0 { (stats.total_comment_lines as f64 / total_lines) * 100.0 } else { 0.0 };
    let comment_gauge = Gauge::default()
        .block(Block::default()
            .borders(Borders::ALL)
            .title(format!(" 💬 Comments - {:.1}% ", comment_pct))
            .title_alignment(Alignment::Center)
            .border_style(Style::default().fg(Color::Magenta))
        )
        .gauge_style(Style::default().fg(Color::Magenta).bg(Color::Black))
        .percent(comment_pct as u16)
        .label(format!("{} / {} lines", stats.total_comment_lines, stats.total_lines));
    f.render_widget(comment_gauge, chunks[1]);

    // Documentation lines percentage with enhanced styling
    let doc_pct = if total_lines > 0.0 { (stats.total_doc_lines as f64 / total_lines) * 100.0 } else { 0.0 };
    let doc_gauge = Gauge::default()
        .block(Block::default()
            .borders(Borders::ALL)
            .title(format!(" 📚 Documentation - {:.1}% ", doc_pct))
            .title_alignment(Alignment::Center)
            .border_style(Style::default().fg(Color::Cyan))
        )
        .gauge_style(Style::default().fg(Color::Cyan).bg(Color::Black))
        .percent(doc_pct as u16)
        .label(format!("{} / {} lines", stats.total_doc_lines, stats.total_lines));
    f.render_widget(doc_gauge, chunks[2]);

    // Blank lines percentage with enhanced styling
    let blank_pct = if total_lines > 0.0 { (stats.total_blank_lines as f64 / total_lines) * 100.0 } else { 0.0 };
    let blank_gauge = Gauge::default()
        .block(Block::default()
            .borders(Borders::ALL)
            .title(format!(" ⬜ Blank Lines - {:.1}% ", blank_pct))
            .title_alignment(Alignment::Center)
            .border_style(Style::default().fg(Color::Gray))
        )
        .gauge_style(Style::default().fg(Color::Gray).bg(Color::Black))
        .percent(blank_pct as u16)
        .label(format!("{} / {} lines", stats.total_blank_lines, stats.total_lines));
    f.render_widget(blank_gauge, chunks[3]);
}

pub fn render_file_types(f: &mut ratatui::Frame, area: Rect, app: &mut InteractiveApp) {
    if let Some(ref stats) = app.stats {
        let header = Row::new(vec![
            Cell::from("Extension"),
            Cell::from("Files"),
            Cell::from("Lines"),
            Cell::from("Code"),
            Cell::from("Comments"),
            Cell::from("Docs"),
            Cell::from("Blank"),
            Cell::from("Size"),
        ]);

        let current_extensions = app.get_current_extensions();
        let mut rows = Vec::new();
        
        for ext in current_extensions {
            if let Some((file_count, file_stats)) = stats.stats_by_extension.get(ext) {
                let icon = get_extension_icon(ext);
                let row = Row::new(vec![
                    Cell::from(format!("{} {}", icon, ext)),
                    Cell::from(file_count.to_string()),
                    Cell::from(file_stats.total_lines.to_string()),
                    Cell::from(file_stats.code_lines.to_string()),
                    Cell::from(file_stats.comment_lines.to_string()),
                    Cell::from(file_stats.doc_lines.to_string()),
                    Cell::from(file_stats.blank_lines.to_string()),
                    Cell::from(format_size(file_stats.file_size)),
                ]);
                rows.push(row);
            }
        }

        let table = Table::new(rows, &[
            Constraint::Length(12),
            Constraint::Length(8),
            Constraint::Length(8),
            Constraint::Length(8),
            Constraint::Length(10),
            Constraint::Length(8),
            Constraint::Length(8),
            Constraint::Length(10),
        ])
        .header(header)
        .block(Block::default().borders(Borders::ALL).title(" File Types "))
        .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol(">> ");

        f.render_stateful_widget(table, area, &mut app.table_state);
    }
}

pub fn render_individual_files(f: &mut ratatui::Frame, area: Rect, app: &mut InteractiveApp) {
    let current_files = app.get_current_files();
    let items: Vec<ListItem> = current_files
        .iter()
        .map(|(file_path, file_stats)| {
            let icon = get_file_icon(file_path);
            let shortened_path = shorten_path(file_path, 50);
            
            ListItem::new(vec![
                Line::from(vec![
                    Span::styled(format!("{} ", icon), Style::default().fg(Color::Yellow)),
                    Span::styled(shortened_path, Style::default().fg(Color::White)),
                ]),
                Line::from(vec![
                    Span::styled(format!("  Lines: {}", file_stats.total_lines), Style::default().fg(Color::Blue)),
                    Span::styled(format!(" | Code: {}", file_stats.code_lines), Style::default().fg(Color::Green)),
                    Span::styled(format!(" | Size: {}", format_size(file_stats.file_size)), Style::default().fg(Color::Cyan)),
                ]),
            ])
        })
        .collect();

    let title = if app.search_state.is_active && !app.search_state.query.is_empty() {
        format!(" Individual Files (Filtered: {}) ", current_files.len())
    } else {
        " Individual Files ".to_string()
    };

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(title))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol(">> ");

    f.render_stateful_widget(list, area, &mut app.list_state);
}

pub fn render_stack_view(f: &mut ratatui::Frame, area: Rect, app: &mut InteractiveApp) {
    // Split the area into two sections: directory tree and techstack analysis
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(60), // Directory tree
            Constraint::Percentage(40), // Techstack analysis
        ])
        .split(area);

    // Render directory tree on the left
    render_directory_tree_section(f, chunks[0], app);
    
    // Render techstack analysis on the right
    render_techstack_analysis_section(f, chunks[1], app);
}

fn render_directory_tree_section(f: &mut ratatui::Frame, area: Rect, app: &mut InteractiveApp) {
    if let Some(ref tree) = app.directory_tree {
        let flattened = app.flatten_tree_for_display(tree);
        
        let title = match app.display_mode {
            DisplayMode::Lines => " Directory Tree - Lines View ",
            DisplayMode::Files => " Directory Tree - Files View ",
        };
        
        let items: Vec<ListItem> = flattened
            .iter()
            .map(|node| {
                let indent = "  ".repeat(node.depth.saturating_sub(1));
                let icon = if node.is_directory {
                    if node.is_expanded {
                        "📂"
                    } else {
                        "📁"
                    }
                } else {
                    get_file_icon(&node.path)
                };
                
                let name = if node.name.len() > 25 {
                    format!("{}...", &node.name[..22])
                } else {
                    node.name.clone()
                };
                
                let count_display = match app.display_mode {
                    DisplayMode::Lines => {
                        if node.is_directory {
                            format!("({} lines)", node.line_count)
                        } else {
                            format!("({} lines)", node.line_count)
                        }
                    }
                    DisplayMode::Files => {
                        if node.is_directory {
                            format!("({} files)", node.file_count)
                        } else {
                            "".to_string()
                        }
                    }
                };
                
                // Create a compact single-line display
                let display_text = if node.is_directory {
                    format!("{}{} {} {}", indent, icon, name, count_display)
                } else {
                    format!("{}{} {} {}", indent, icon, name, count_display)
                };
                
                // Truncate if too long for small terminals
                let max_width = area.width.saturating_sub(4) as usize;
                let truncated = if display_text.len() > max_width {
                    format!("{}...", &display_text[..max_width.saturating_sub(3)])
                } else {
                    display_text
                };
                
                let style = if node.is_directory {
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };
                
                ListItem::new(vec![
                    Line::from(vec![
                        Span::styled(truncated, style),
                    ]),
                ])
            })
            .collect();

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title(title))
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
            .highlight_symbol(">> ");

        f.render_stateful_widget(list, area, &mut app.tree_state);
    }
}

fn render_techstack_analysis_section(f: &mut ratatui::Frame, area: Rect, app: &InteractiveApp) {
    if let Some(ref inventory) = app.techstack_inventory {
        // Split techstack section into summary and technologies list
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(8),  // Summary
                Constraint::Min(0),     // Technologies list
            ])
            .split(area);

        // Render summary
        render_techstack_summary(f, chunks[0], inventory);
        
        // Render technologies list
        render_technologies_list(f, chunks[1], inventory);
    } else {
        // Show loading or error state
        let loading_text = Paragraph::new("🔍 Analyzing techstack...")
            .block(Block::default().borders(Borders::ALL).title(" Tech Stack Analysis "))
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center);
        f.render_widget(loading_text, area);
    }
}

fn render_techstack_summary(f: &mut ratatui::Frame, area: Rect, inventory: &TechStackInventory) {
    let summary = &inventory.analysis_summary;
    
    let summary_text = vec![
        Line::from(vec![
            Span::styled("🏗️ Architecture: ", Style::default().fg(Color::Cyan)),
            Span::styled(format!("{:?}", summary.architecture_type), Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("☁️ Deployment: ", Style::default().fg(Color::Blue)),
            Span::styled(format!("{:?}", summary.deployment_type), Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("🔒 Security: ", Style::default().fg(Color::Red)),
            Span::styled(format!("{:?}", summary.security_posture), Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("📊 Modernization: ", Style::default().fg(Color::Green)),
            Span::styled(format!("{:.1}%", summary.modernization_score), Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("🎯 Confidence: ", Style::default().fg(Color::Yellow)),
            Span::styled(format!("{:.1}%", inventory.overall_confidence * 100.0), Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("📦 Technologies: ", Style::default().fg(Color::Magenta)),
            Span::styled(format!("{}", inventory.technologies.len()), Style::default().fg(Color::White)),
        ]),
    ];

    let summary_block = Paragraph::new(summary_text)
        .block(Block::default().borders(Borders::ALL).title(" Tech Stack Summary "))
        .style(Style::default().fg(Color::White));
    
    f.render_widget(summary_block, area);
}

fn render_technologies_list(f: &mut ratatui::Frame, area: Rect, inventory: &TechStackInventory) {
    let items: Vec<ListItem> = inventory.technologies
        .iter()
        .take(20) // Limit to prevent overflow
        .map(|tech| {
            let confidence_bar = create_confidence_bar(tech.confidence);
            let category_emoji = get_category_emoji(&tech.category);
            
            let tech_line = Line::from(vec![
                Span::styled(format!("{} ", category_emoji), Style::default().fg(Color::Yellow)),
                Span::styled(&tech.name, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                Span::styled(
                    if let Some(ref version) = tech.version {
                        format!(" v{}", version)
                    } else {
                        "".to_string()
                    },
                    Style::default().fg(Color::Gray)
                ),
                Span::styled(format!(" {}", confidence_bar), Style::default()),
            ]);
            
            ListItem::new(vec![tech_line])
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(" Detected Technologies "))
        .style(Style::default().fg(Color::White));

    f.render_widget(list, area);
}

fn create_confidence_bar(confidence: f64) -> String {
    let width = 10;
    let filled = (confidence * width as f64) as usize;
    let empty = width - filled;
    
    format!("[{}{}] {:.0}%", 
        "█".repeat(filled), 
        "░".repeat(empty), 
        confidence * 100.0
    )
}

fn get_category_emoji(category: &crate::core::stats::techstack::TechCategory) -> &'static str {
    match category {
        crate::core::stats::techstack::TechCategory::ProgrammingLanguage => "🔤",
        crate::core::stats::techstack::TechCategory::Frontend => "🎨",
        crate::core::stats::techstack::TechCategory::Backend => "⚙️",
        crate::core::stats::techstack::TechCategory::Database => "🗄️",
        crate::core::stats::techstack::TechCategory::WebFramework => "🌐",
        crate::core::stats::techstack::TechCategory::BuildTool => "🔨",
        crate::core::stats::techstack::TechCategory::TestingFramework => "🧪",
        crate::core::stats::techstack::TechCategory::Containerization => "📦",
        crate::core::stats::techstack::TechCategory::CloudProvider => "☁️",
        crate::core::stats::techstack::TechCategory::Security => "🔒",
        crate::core::stats::techstack::TechCategory::Monitoring => "📊",
        crate::core::stats::techstack::TechCategory::Runtime => "⚡",
        _ => "🔧",
    }
}

pub fn render_help(f: &mut ratatui::Frame, area: Rect) {
    let help_text = vec![
        Line::from(vec![
            Span::styled("🔍 HOW MANY - Help", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Navigation:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]),
        Line::from("  Tab / Shift+Tab    - Switch between tabs"),
        Line::from("  1, 2, 3, 4, 5, 6  - Jump to specific tab"),
        Line::from("  ↑/↓ or j/k        - Scroll up/down"),
        Line::from("  Page Up/Down      - Scroll by page"),
        Line::from("  Home/End          - Go to top/bottom"),
        Line::from(""),
        Line::from(vec![
            Span::styled("Search:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]),
        Line::from("  / or s            - Toggle search mode"),
        Line::from("  Tab               - Cycle search mode (Files/Extensions/Content)"),
        Line::from("  Enter             - Jump to selected result"),
        Line::from("  Esc               - Exit search mode"),
        Line::from("  ↑/↓               - Navigate search results"),
        Line::from(""),
        Line::from(vec![
            Span::styled("Actions:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]),
        Line::from("  h or F1           - Toggle this help"),
        Line::from("  q or Esc          - Quit application"),
        Line::from("  t                 - Toggle lines/files view (Stack View)"),
        Line::from(""),
        Line::from(vec![
            Span::styled("Stack View:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]),
        Line::from("  Enter/Right       - Expand/collapse directory"),
        Line::from("  Left              - Collapse directory"),
        Line::from(""),
        Line::from(vec![
            Span::styled("Export:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]),
        Line::from("  1-5               - Select export format"),
        Line::from("  Enter             - Export to selected format"),
        Line::from("  ↑/↓ or j/k        - Navigate formats"),
        Line::from(""),
        Line::from(vec![
            Span::styled("Tabs:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]),
        Line::from("  Overview          - Summary statistics with charts"),
        Line::from("  File Types        - Statistics by file extension"),
        Line::from("  Individual Files  - List of all analyzed files"),
        Line::from("  Stack View        - Directory tree with tech stack analysis"),
        Line::from("  Export            - Export results to various formats"),
        Line::from("  Quality Analysis  - Code quality metrics and insights"),
        Line::from(""),
        Line::from(vec![
            Span::styled("Search Modes:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]),
        Line::from("  Files             - Search by file name and path"),
        Line::from("  Extensions        - Search by file extension"),
        Line::from("  Content           - Search by estimated content/keywords"),
    ];

    let help_paragraph = Paragraph::new(help_text)
        .alignment(Alignment::Left)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Help ")
                .title_alignment(Alignment::Center)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .wrap(Wrap { trim: true });

    f.render_widget(help_paragraph, area);
}

pub fn render_footer(f: &mut ratatui::Frame, area: Rect, app: &InteractiveApp) {
    let mut footer_spans = vec![
        Span::styled("q", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::styled(" to quit, ", Style::default().fg(Color::White)),
        Span::styled("h", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::styled(" for help, ", Style::default().fg(Color::White)),
        Span::styled("/", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::styled(" to search", Style::default().fg(Color::White)),
    ];
    
    if app.search_state.is_active {
        footer_spans = vec![
            Span::styled("Search Mode: ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::styled("Tab", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(" cycle mode, ", Style::default().fg(Color::White)),
            Span::styled("Enter", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(" select, ", Style::default().fg(Color::White)),
            Span::styled("Esc", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(" exit search", Style::default().fg(Color::White)),
        ];
    } else {
        match app.mode {
            AppMode::StackView => {
                footer_spans.extend(vec![
                    Span::styled(", ", Style::default().fg(Color::White)),
                    Span::styled("t", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                    Span::styled(" to toggle ", Style::default().fg(Color::White)),
                    Span::styled(
                        match app.display_mode {
                            DisplayMode::Lines => "[Lines]",
                            DisplayMode::Files => "[Files]",
                        },
                        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
                    ),
                ]);
            }
            AppMode::Export => {
                footer_spans.extend(vec![
                    Span::styled(", ", Style::default().fg(Color::White)),
                    Span::styled("1-5", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                    Span::styled(" to select format, ", Style::default().fg(Color::White)),
                    Span::styled("Enter", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                    Span::styled(" to export", Style::default().fg(Color::White)),
                ]);
            }
            _ => {}
        }
    }
    
    let footer_text = vec![Line::from(footer_spans)];

    let footer = Paragraph::new(footer_text)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));

    f.render_widget(footer, area);
}

pub fn render_welcome(f: &mut ratatui::Frame, area: Rect) {
    let welcome_text = vec![
        Line::from(vec![
            Span::styled("🔍 ", Style::default().fg(Color::Cyan)),
            Span::styled("HOW MANY", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(" - Modern Code Analyzer", Style::default().fg(Color::White)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("✨ ", Style::default().fg(Color::Yellow)),
            Span::styled("Intelligent code counting with beautiful visualization", Style::default().fg(Color::Gray)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("🚀 ", Style::default().fg(Color::Green)),
            Span::styled("Loading...", Style::default().fg(Color::White)),
        ]),
    ];

    let welcome_paragraph = Paragraph::new(welcome_text)
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Welcome ")
                .title_alignment(Alignment::Center)
                .border_style(Style::default().fg(Color::Cyan)),
        );

    let centered_area = centered_rect(60, 20, area);
    f.render_widget(welcome_paragraph, centered_area);
} 

pub fn render_export(f: &mut ratatui::Frame, area: Rect, app: &InteractiveApp) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8),
            Constraint::Length(12),
            Constraint::Length(6),
            Constraint::Min(0),
        ])
        .split(area);

    // Title and description
    let title_text = vec![
        Line::from(vec![
            Span::styled("📤 ", Style::default().fg(Color::Yellow)),
            Span::styled("Export Code Analysis", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Export your code analysis results to various formats", Style::default().fg(Color::Gray)),
        ]),
        Line::from(vec![
            Span::styled("Use ↑/↓ to select format, Enter to export, or press the number key", Style::default().fg(Color::Gray)),
        ]),
    ];
    
    let title_block = Paragraph::new(title_text)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).title(" Export Options "));
    f.render_widget(title_block, chunks[0]);

    // Export format selection
    let format_items = vec![
        ListItem::new(vec![
            Line::from(vec![
                Span::styled("1. ", Style::default().fg(Color::Yellow)),
                Span::styled("📄 Text Report", Style::default().fg(Color::White)),
                Span::styled(" - Simple text-based summary", Style::default().fg(Color::Gray)),
            ]),
        ]),
        ListItem::new(vec![
            Line::from(vec![
                Span::styled("2. ", Style::default().fg(Color::Yellow)),
                Span::styled("📋 JSON Export", Style::default().fg(Color::White)),
                Span::styled(" - Machine-readable data format", Style::default().fg(Color::Gray)),
            ]),
        ]),
        ListItem::new(vec![
            Line::from(vec![
                Span::styled("3. ", Style::default().fg(Color::Yellow)),
                Span::styled("📊 CSV Export", Style::default().fg(Color::White)),
                Span::styled(" - Spreadsheet-compatible format", Style::default().fg(Color::Gray)),
            ]),
        ]),
        ListItem::new(vec![
            Line::from(vec![
                Span::styled("4. ", Style::default().fg(Color::Yellow)),
                Span::styled("🌐 HTML Report", Style::default().fg(Color::White)),
                Span::styled(" - Interactive web report with charts", Style::default().fg(Color::Gray)),
            ]),
        ]),
        ListItem::new(vec![
            Line::from(vec![
                Span::styled("5. ", Style::default().fg(Color::Yellow)),
                Span::styled("⏰ Time Wasted Report", Style::default().fg(Color::White)),
                Span::styled(" - Humorous analysis of time spent", Style::default().fg(Color::Gray)),
            ]),
        ]),
    ];

    let selected_index = match app.export_state.selected_format {
        ExportFormat::Text => 0,
        ExportFormat::Json => 1,
        ExportFormat::Csv => 2,
        ExportFormat::Html => 3,
        ExportFormat::TimeWasted => 4,
    };

    let format_list = List::new(format_items)
        .block(Block::default().borders(Borders::ALL).title(" Available Formats "))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED).fg(Color::Yellow))
        .highlight_symbol(">> ");

    // Create a temporary ListState for rendering
    let mut temp_list_state = ListState::default();
    temp_list_state.select(Some(selected_index));
    f.render_stateful_widget(format_list, chunks[1], &mut temp_list_state);

    // Export status
    let status_color = if app.export_state.export_status.contains("Success") {
        Color::Green
    } else if app.export_state.export_status.contains("Error") {
        Color::Red
    } else {
        Color::Blue
    };

    let status_text = vec![
        Line::from(vec![
            Span::styled("Status: ", Style::default().fg(Color::White)),
            Span::styled(&app.export_state.export_status, Style::default().fg(status_color)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Last Export: ", Style::default().fg(Color::White)),
            Span::styled(
                app.export_state.last_export_path.as_deref().unwrap_or("None"),
                Style::default().fg(Color::Gray)
            ),
        ]),
    ];

    let status_block = Paragraph::new(status_text)
        .alignment(Alignment::Left)
        .block(Block::default().borders(Borders::ALL).title(" Export Status "));
    f.render_widget(status_block, chunks[2]);

    // Help text
    let help_text = vec![
        Line::from(vec![
            Span::styled("⌨️  Controls:", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("↑/↓ or j/k", Style::default().fg(Color::Yellow)),
            Span::styled(" - Navigate formats", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("Enter", Style::default().fg(Color::Yellow)),
            Span::styled(" - Export in selected format", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("1-5", Style::default().fg(Color::Yellow)),
            Span::styled(" - Quick select format", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("Tab", Style::default().fg(Color::Yellow)),
            Span::styled(" - Switch to other tabs", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("q/Esc", Style::default().fg(Color::Yellow)),
            Span::styled(" - Quit application", Style::default().fg(Color::White)),
        ]),
    ];

    let help_block = Paragraph::new(help_text)
        .alignment(Alignment::Left)
        .block(Block::default().borders(Borders::ALL).title(" Help "));
    f.render_widget(help_block, chunks[3]);
} 