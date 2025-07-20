use crate::core::types::CodeStats;

use crate::ui::interactive::app::{AppMode, InteractiveApp, ExportFormat, SearchMode};
use crate::ui::interactive::utils::{centered_rect, format_size, get_file_icon, shorten_path};
use crate::ui::interactive::charts::{render_enhanced_overview, render_advanced_language_visualizer};
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
    let titles = vec!["Overview", "Languages", "Export"];
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
            AppMode::Languages => render_languages(f, area, app),
            AppMode::Export => render_export(f, area, app),
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
        // Create comprehensive aggregated stats with real-time tracking
        let stats_calculator = crate::core::stats::StatsCalculator::new();
        let aggregated_stats = stats_calculator.calculate_project_stats(stats, &app.individual_files)
            .unwrap_or_else(|_| {
                // Fallback to basic aggregated stats if comprehensive calculation fails
                create_aggregated_stats_from_basic(stats)
            });
        
        render_enhanced_overview(f, area, &aggregated_stats);
    } else {
        let no_data = Paragraph::new("No data available")
            .block(Block::default().borders(Borders::ALL).title(" Overview "))
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center);
        f.render_widget(no_data, area);
    }
}

pub fn render_code_health(f: &mut ratatui::Frame, area: Rect, app: &InteractiveApp) {
    if let Some(ref stats) = app.stats {
        // Create comprehensive aggregated stats with real-time tracking
        let stats_calculator = crate::core::stats::StatsCalculator::new();
        let aggregated_stats = stats_calculator.calculate_project_stats(stats, &app.individual_files)
            .unwrap_or_else(|_| {
                // Fallback to basic aggregated stats if comprehensive calculation fails
                create_aggregated_stats_from_basic(stats)
            });
        
        // Use the new advanced language visualizer instead of the old code health sections
        render_advanced_language_visualizer(f, area, &aggregated_stats);
    } else {
        let no_data = Paragraph::new("No data available for language analysis")
            .block(Block::default().borders(Borders::ALL).title(" Language Analysis "))
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
        largest_file_size: calculate_largest_file_size(stats),
        smallest_file_size: calculate_smallest_file_size(stats),
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
            code_health_score: if stats.total_lines > 0 {
                let comment_ratio = stats.total_comment_lines as f64 / stats.total_lines as f64;
                let code_ratio = stats.total_code_lines as f64 / stats.total_lines as f64;
                ((comment_ratio * 30.0) + (code_ratio * 50.0) + 20.0).min(100.0)
            } else { 0.0 },
            maintainability_index: if stats.total_code_lines > 0 {
                let doc_ratio = (stats.total_doc_lines + stats.total_comment_lines) as f64 / stats.total_code_lines as f64;
                (doc_ratio * 100.0).min(100.0)
            } else { 0.0 },
            documentation_coverage: if stats.total_code_lines > 0 {
                let doc_lines = stats.total_doc_lines + stats.total_comment_lines;
                (doc_lines as f64 / stats.total_code_lines as f64 * 100.0).min(100.0)
            } else { 0.0 },
            avg_complexity: if stats.total_files > 0 {
                // Estimate average complexity based on file structure
                let avg_lines_per_file = stats.total_lines as f64 / stats.total_files as f64;
                (avg_lines_per_file / 20.0).min(10.0) // Rough estimate
            } else { 0.0 },
            function_size_health: if stats.total_files > 0 {
                let avg_lines_per_file = stats.total_lines as f64 / stats.total_files as f64;
                // Smaller files generally indicate better function sizes
                (100.0 - (avg_lines_per_file / 10.0)).max(0.0).min(100.0)
            } else { 0.0 },
            nesting_depth_health: if stats.total_files > 0 {
                // Estimate nesting health based on file structure
                let avg_lines_per_file = stats.total_lines as f64 / stats.total_files as f64;
                (100.0 - (avg_lines_per_file / 15.0)).max(0.0).min(100.0)
            } else { 0.0 },
            code_duplication_ratio: 5.0, // Conservative estimate
            technical_debt_ratio: if stats.total_files > 0 {
                // Estimate technical debt based on various factors
                let avg_lines_per_file = stats.total_lines as f64 / stats.total_files as f64;
                let large_file_penalty: f64 = if avg_lines_per_file > 100.0 { 20.0 } else { 0.0 };
                let low_comment_penalty: f64 = if stats.total_code_lines > 0 && (stats.total_comment_lines as f64 / stats.total_code_lines as f64) < 0.1 { 15.0 } else { 0.0 };
                (large_file_penalty + low_comment_penalty).min(100.0)
            } else { 0.0 },
        },
    };
    
    // Create realistic time stats using actual calculations
    let time_calculator = crate::core::stats::time::TimeStatsCalculator::new();
    let time_stats = time_calculator.calculate_project_time_stats(stats).unwrap_or_else(|_| {
        // Fallback to basic calculation if advanced calculation fails
        let code_minutes = (stats.total_code_lines as f64 * 0.2) as usize; // 0.2 minutes per line of code (realistic for modern dev)
        let doc_minutes = (stats.total_doc_lines as f64 * 0.5) as usize; // 0.5 minutes per line of docs
        let comment_minutes = (stats.total_comment_lines as f64 * 0.1) as usize; // 0.1 minutes per line of comments
        let total_minutes = code_minutes + doc_minutes + comment_minutes;
        
        let total_hours = total_minutes as f64 / 60.0;
        let productivity_metrics = crate::core::stats::time::ProductivityMetrics {
            lines_per_hour: if total_hours > 0.0 { stats.total_lines as f64 / total_hours } else { 0.0 },
            code_lines_per_hour: if total_hours > 0.0 { stats.total_code_lines as f64 / total_hours } else { 0.0 },
            files_per_hour: if total_hours > 0.0 { stats.total_files as f64 / total_hours } else { 0.0 },
            estimated_development_days: total_hours / 8.0, // 8 hours per day
            estimated_development_hours: total_hours,
        };
        
        // Simple time formatting function
        let format_time = |minutes: usize| -> String {
            if minutes < 60 {
                format!("{}min", minutes)
            } else if minutes < 1440 { // less than 24 hours
                let hours = minutes / 60;
                let mins = minutes % 60;
                if mins > 0 {
                    format!("{}h {}min", hours, mins)
                } else {
                    format!("{}h", hours)
                }
            } else {
                let days = minutes / 1440;
                let hours = (minutes % 1440) / 60;
                if hours > 0 {
                    format!("{} days {}h", days, hours)
                } else {
                    format!("{} days", days)
                }
            }
        };
        
        TimeStats {
            total_time_minutes: total_minutes,
            code_time_minutes: code_minutes,
            doc_time_minutes: doc_minutes,
            comment_time_minutes: comment_minutes,
            total_time_formatted: format_time(total_minutes),
            code_time_formatted: format_time(code_minutes),
            doc_time_formatted: format_time(doc_minutes),
            comment_time_formatted: format_time(comment_minutes),
            time_by_extension: HashMap::new(),
            productivity_metrics,
        }
    });
    
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
        quality_metrics: {
            let doc_score = if stats.total_code_lines > 0 {
                ((stats.total_doc_lines + stats.total_comment_lines) as f64 / stats.total_code_lines as f64 * 100.0).min(100.0)
            } else { 0.0 };
            
            let maintainability_score = if stats.total_lines > 0 {
                let comment_ratio = stats.total_comment_lines as f64 / stats.total_lines as f64;
                let code_ratio = stats.total_code_lines as f64 / stats.total_lines as f64;
                ((comment_ratio * 30.0) + (code_ratio * 50.0) + 20.0).min(100.0)
            } else { 0.0 };
            
            let readability_score = if stats.total_lines > 0 {
                let comment_ratio = stats.total_comment_lines as f64 / stats.total_lines as f64;
                let blank_ratio = stats.total_blank_lines as f64 / stats.total_lines as f64;
                
                // Comment contribution (0-70 points)
                let comment_score = if comment_ratio >= 0.15 {
                    70.0
                } else {
                    (comment_ratio / 0.15) * 70.0
                };
                
                // Blank lines contribution (0-30 points) - ideal is 15%
                let blank_score = if blank_ratio <= 0.15 {
                    (blank_ratio / 0.15) * 30.0
                } else {
                    let penalty = (blank_ratio - 0.15) * 60.0;
                    (30.0 - penalty).max(0.0)
                };
                
                (comment_score + blank_score).min(100.0)
            } else { 0.0 };
            
            let consistency_score = if stats.stats_by_extension.len() > 0 {
                // Calculate consistency based on how evenly distributed the code is
                let avg_lines_per_ext = stats.total_lines as f64 / stats.stats_by_extension.len() as f64;
                let variance = stats.stats_by_extension.values()
                    .map(|(_, file_stats)| {
                        let diff = file_stats.total_lines as f64 - avg_lines_per_ext;
                        diff * diff
                    })
                    .sum::<f64>() / stats.stats_by_extension.len() as f64;
                (100.0 - (variance.sqrt() / avg_lines_per_ext * 100.0)).max(0.0).min(100.0)
            } else { 0.0 };
            
            let overall_score = (doc_score + maintainability_score + readability_score + consistency_score) / 4.0;
            
            crate::core::stats::ratios::QualityMetrics {
                documentation_score: doc_score,
                maintainability_score: maintainability_score,
                readability_score: readability_score,
                consistency_score: consistency_score,
                overall_quality_score: overall_score,
            }
        },
    };
    
    // Create metadata
    let metadata = StatsMetadata {
        calculation_time_ms: 0,
                    version: "0.3.2".to_string(),
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

pub fn render_languages(f: &mut ratatui::Frame, area: Rect, app: &mut InteractiveApp) {
    if app.show_code_health {
        // Show code health integrated into languages page
        render_languages_with_code_health(f, area, app);
    } else {
        // Show regular language analysis
        render_languages_regular(f, area, app);
    }
}

fn render_languages_regular(f: &mut ratatui::Frame, area: Rect, app: &mut InteractiveApp) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(12), // Language overview chart
            Constraint::Min(0),     // Language details table
        ])
        .split(area);

    // Render language overview chart (without quick stats)
    render_language_overview_chart_no_stats(f, chunks[0], app);
    
    // Render language details table
    render_language_details_table(f, chunks[1], app);
}

fn render_languages_with_code_health(f: &mut ratatui::Frame, area: Rect, app: &mut InteractiveApp) {
    if let Some(ref stats) = app.stats {
        // Create comprehensive aggregated stats
        let stats_calculator = crate::core::stats::StatsCalculator::new();
        let aggregated_stats = stats_calculator.calculate_project_stats(stats, &app.individual_files)
            .unwrap_or_else(|_| {
                // Fallback to basic aggregated stats if comprehensive calculation fails
                create_aggregated_stats_from_basic(stats)
            });
        
        // Use the new advanced language visualizer instead of the old code health sections
        render_advanced_language_visualizer(f, area, &aggregated_stats);
    } else {
        let no_data = Paragraph::new("No data available for language analysis")
            .block(Block::default().borders(Borders::ALL).title(" Language Analysis "))
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center);
        f.render_widget(no_data, area);
    }
}

fn render_language_overview_chart(f: &mut ratatui::Frame, area: Rect, app: &InteractiveApp) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(60), // Bar chart
            Constraint::Percentage(40), // Stats summary
        ])
        .split(area);

    render_language_bar_chart(f, chunks[0], app);
    render_language_stats_summary(f, chunks[1], app);
}

fn render_language_overview_chart_no_stats(f: &mut ratatui::Frame, area: Rect, app: &InteractiveApp) {
    // Show enhanced language chart without quick stats
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(70), // Expanded bar chart
            Constraint::Percentage(30), // Toggle hint and instructions
        ])
        .split(area);

    render_language_bar_chart(f, chunks[0], app);
    render_language_toggle_hint(f, chunks[1], app);
}

fn render_language_toggle_hint(f: &mut ratatui::Frame, area: Rect, app: &InteractiveApp) {
    let toggle_text = if app.show_code_health {
        "Press 't' to view\nlanguage breakdown"
    } else {
        "Press 't' to view\ncode health"
    };
    
    let hint_lines = vec![
        Line::from(vec![
            Span::styled("💡 Toggle View", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(toggle_text, Style::default().fg(Color::Cyan)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("📊 Current Mode:", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled(
                if app.show_code_health { "Code Health" } else { "Language Stats" },
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("🔍 Navigation:", Style::default().fg(Color::Gray)),
        ]),
        Line::from(vec![
            Span::styled("↑/↓ - Navigate", Style::default().fg(Color::Gray)),
        ]),
        Line::from(vec![
            Span::styled("Tab - Switch tabs", Style::default().fg(Color::Gray)),
        ]),
    ];
    
    let hint_paragraph = Paragraph::new(hint_lines)
        .alignment(Alignment::Left)
        .block(Block::default().borders(Borders::ALL).title(" Controls "))
        .wrap(Wrap { trim: true });
    
    f.render_widget(hint_paragraph, area);
}

fn render_language_bar_chart(f: &mut ratatui::Frame, area: Rect, app: &InteractiveApp) {
    let mut chart_data = Vec::new();
    let mut total_lines = 0;
    
    // Collect data for the chart
    for (language_name, (language_info, _file_count, file_stats)) in &app.language_stats {
        chart_data.push((language_name.clone(), language_info.clone(), file_stats.total_lines));
        total_lines += file_stats.total_lines;
    }
    
    // Sort by lines descending
    chart_data.sort_by(|a, b| b.2.cmp(&a.2));
    
    let chart_lines = if chart_data.is_empty() {
        vec![
            Line::from(vec![
                Span::styled("No languages detected", Style::default().fg(Color::Gray)),
            ]),
        ]
    } else {
        let mut lines = vec![
            Line::from(vec![
                Span::styled("📊 Language Distribution", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(""),
        ];
        
        for (language_name, language_info, line_count) in chart_data.iter().take(8) {
            let percentage = if total_lines > 0 {
                (*line_count as f64 / total_lines as f64) * 100.0
            } else {
                0.0
            };
            
            // Create a visual bar using Unicode blocks with gradient effect
            let bar_width = 35;
            let filled_width = ((percentage / 100.0) * bar_width as f64) as usize;
            
            // Create gradient bar with different block characters
            let mut bar = String::new();
            for i in 0..bar_width {
                if i < filled_width {
                    let intensity = (i as f64 / filled_width as f64) * 0.8 + 0.2;
                    if intensity > 0.8 {
                        bar.push('█');
                    } else if intensity > 0.6 {
                        bar.push('▉');
                    } else if intensity > 0.4 {
                        bar.push('▊');
                    } else if intensity > 0.2 {
                        bar.push('▋');
                    } else {
                        bar.push('▌');
                    }
                } else {
                    bar.push('░');
                }
            }
            
            // Enhanced color mapping with RGB values
            let color = parse_hex_color(&language_info.color);
            
            lines.push(Line::from(vec![
                Span::styled(format!("{} ", language_info.icon), Style::default().fg(color)),
                Span::styled(format!("{:<12}", language_name), Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                Span::styled(bar, Style::default().fg(color)),
                Span::styled(format!(" {:.1}%", percentage), Style::default().fg(Color::Gray)),
            ]));
        }
        
        if chart_data.len() > 8 {
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::styled(format!("... and {} more languages", chart_data.len() - 8), Style::default().fg(Color::DarkGray)),
            ]));
        }
        
        lines
    };
    
    let chart_paragraph = Paragraph::new(chart_lines)
        .alignment(Alignment::Left)
        .block(Block::default().borders(Borders::ALL).title(" Language Distribution "))
        .wrap(Wrap { trim: true });
    
    f.render_widget(chart_paragraph, area);
}

fn render_language_stats_summary(f: &mut ratatui::Frame, area: Rect, app: &InteractiveApp) {
    let mut total_files = 0;
    let mut total_lines = 0;
    let mut total_code_lines = 0;
    let language_count = app.language_stats.len();
    
    // Calculate totals
    for (_, (_, file_count, file_stats)) in &app.language_stats {
        total_files += file_count;
        total_lines += file_stats.total_lines;
        total_code_lines += file_stats.code_lines;
    }
    
    // Find dominant language
    let dominant_language = app.language_stats.iter()
        .max_by_key(|(_, (_, _, file_stats))| file_stats.total_lines)
        .map(|(name, (info, _, _))| (name.clone(), info.clone()));
    
    let mut summary_lines = vec![
        Line::from(vec![
            Span::styled("📈 Project Summary", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
    ];
    
    // Language count with icon
    summary_lines.push(Line::from(vec![
        Span::styled("🌐 Languages: ", Style::default().fg(Color::Cyan)),
        Span::styled(language_count.to_string(), Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
    ]));
    
    // Total files with icon
    summary_lines.push(Line::from(vec![
        Span::styled("📁 Files: ", Style::default().fg(Color::Blue)),
        Span::styled(total_files.to_string(), Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
    ]));
    
    // Total lines with icon
    summary_lines.push(Line::from(vec![
        Span::styled("📏 Lines: ", Style::default().fg(Color::Green)),
        Span::styled(total_lines.to_string(), Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
    ]));
    
    // Code lines with icon
    summary_lines.push(Line::from(vec![
        Span::styled("⚡ Code: ", Style::default().fg(Color::Magenta)),
        Span::styled(total_code_lines.to_string(), Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
    ]));
    
    summary_lines.push(Line::from(""));
    
    // Dominant language
    if let Some((lang_name, lang_info)) = dominant_language {
        summary_lines.push(Line::from(vec![
            Span::styled("👑 Primary: ", Style::default().fg(Color::Yellow)),
        ]));
        summary_lines.push(Line::from(vec![
            Span::styled(format!("{} {}", lang_info.icon, lang_name), 
                Style::default().fg(parse_hex_color(&lang_info.color)).add_modifier(Modifier::BOLD)),
        ]));
    }
    
    // Code quality indicator
    let code_ratio = if total_lines > 0 {
        (total_code_lines as f64 / total_lines as f64) * 100.0
    } else {
        0.0
    };
    
    summary_lines.push(Line::from(""));
    summary_lines.push(Line::from(vec![
        Span::styled("📊 Code Ratio: ", Style::default().fg(Color::Cyan)),
        Span::styled(format!("{:.1}%", code_ratio), 
            Style::default().fg(if code_ratio > 70.0 { Color::Green } else if code_ratio > 50.0 { Color::Yellow } else { Color::Red })),
    ]));
    
    let summary_paragraph = Paragraph::new(summary_lines)
        .alignment(Alignment::Left)
        .block(Block::default().borders(Borders::ALL).title(" Quick Stats "))
        .wrap(Wrap { trim: true });
    
    f.render_widget(summary_paragraph, area);
}

fn parse_hex_color(hex: &str) -> Color {
    if hex.starts_with('#') && hex.len() == 7 {
        if let (Ok(r), Ok(g), Ok(b)) = (
            u8::from_str_radix(&hex[1..3], 16),
            u8::from_str_radix(&hex[3..5], 16),
            u8::from_str_radix(&hex[5..7], 16),
        ) {
            return Color::Rgb(r, g, b);
        }
    }
    Color::White
}

fn render_language_details_table(f: &mut ratatui::Frame, area: Rect, app: &mut InteractiveApp) {
    let header = Row::new(vec![
        Cell::from("Language"),
        Cell::from("Files"),
        Cell::from("Lines"),
        Cell::from("Code"),
        Cell::from("Comments"),
        Cell::from("Docs"),
        Cell::from("Blank"),
        Cell::from("Size"),
        Cell::from("Extensions"),
    ]);

    let mut rows = Vec::new();
    let mut language_data: Vec<_> = app.language_stats.iter().collect();
    
    // Sort by total lines descending
    language_data.sort_by(|a, b| b.1.2.total_lines.cmp(&a.1.2.total_lines));
    
    for (language_name, (language_info, file_count, file_stats)) in language_data {
        let extensions_str = language_info.extensions.join(", ");
        let row = Row::new(vec![
            Cell::from(format!("{} {}", language_info.icon, language_name)),
            Cell::from(file_count.to_string()),
            Cell::from(file_stats.total_lines.to_string()),
            Cell::from(file_stats.code_lines.to_string()),
            Cell::from(file_stats.comment_lines.to_string()),
            Cell::from(file_stats.doc_lines.to_string()),
            Cell::from(file_stats.blank_lines.to_string()),
            Cell::from(format_size(file_stats.file_size)),
            Cell::from(extensions_str),
        ]);
        rows.push(row);
    }

    let table = Table::new(rows, &[
        Constraint::Length(15),
        Constraint::Length(8),
        Constraint::Length(8),
        Constraint::Length(8),
        Constraint::Length(10),
        Constraint::Length(8),
        Constraint::Length(8),
        Constraint::Length(10),
        Constraint::Length(15),
    ])
    .header(header)
    .block(Block::default().borders(Borders::ALL).title(" Language Details "))
    .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED))
    .highlight_symbol(">> ");

    f.render_stateful_widget(table, area, &mut app.table_state);
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
        Line::from("  1, 2, 3           - Jump to specific tab"),
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
        Line::from(""),
        Line::from(vec![
            Span::styled("Export:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]),
        Line::from("  1-4               - Select export format"),
        Line::from("  Enter             - Export to selected format"),
        Line::from("  ↑/↓ or j/k        - Navigate formats"),
        Line::from(""),
        Line::from(vec![
            Span::styled("Tabs:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]),
        Line::from("  Overview          - Summary statistics with charts"),
        Line::from("  Languages         - Programming language breakdown with code health (press 't' to toggle)"),
        Line::from("  Export            - Export results to various formats"),
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
        Span::styled("Tab", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::styled(" to switch tabs, ", Style::default().fg(Color::White)),
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
            AppMode::Export => {
                footer_spans.extend(vec![
                    Span::styled(", ", Style::default().fg(Color::White)),
                    Span::styled("1-4", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                    Span::styled(" to select format, ", Style::default().fg(Color::White)),
                    Span::styled("Enter", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                    Span::styled(" to export", Style::default().fg(Color::White)),
                ]);
            }
            AppMode::Languages => {
                footer_spans.extend(vec![
                    Span::styled(", ", Style::default().fg(Color::White)),
                    Span::styled("t", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                    Span::styled(" to toggle code health", Style::default().fg(Color::White)),
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
                Span::styled("🔍 SARIF Report", Style::default().fg(Color::White)),
                Span::styled(" - Static Analysis Results Interchange Format", Style::default().fg(Color::Gray)),
            ]),
        ]),

    ];

    let selected_index = match app.export_state.selected_format {
        ExportFormat::Text => 0,
        ExportFormat::Json => 1,
        ExportFormat::Csv => 2,
        ExportFormat::Html => 3,
        ExportFormat::Sarif => 4,
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

// Helper functions for realistic file size calculations
fn calculate_largest_file_size(stats: &CodeStats) -> u64 {
    if stats.stats_by_extension.is_empty() {
        return 0;
    }
    
    // Estimate largest file size based on extension with most lines
    let max_lines_per_ext = stats.stats_by_extension.values()
        .map(|(_, file_stats)| file_stats.total_lines)
        .max()
        .unwrap_or(0);
    
    // Estimate bytes per line (average ~50 bytes per line)
    (max_lines_per_ext as u64 * 50).max(1)
}

fn calculate_smallest_file_size(stats: &CodeStats) -> u64 {
    if stats.stats_by_extension.is_empty() {
        return 0;
    }
    
    // Estimate smallest file size based on extension with fewest lines
    let min_lines_per_ext = stats.stats_by_extension.values()
        .map(|(_, file_stats)| file_stats.total_lines)
        .min()
        .unwrap_or(0);
    
    // Estimate bytes per line (average ~50 bytes per line)
    (min_lines_per_ext as u64 * 50).max(1)
} 