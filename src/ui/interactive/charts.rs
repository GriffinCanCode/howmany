use crate::core::stats::aggregation::AggregatedStats;

use crate::core::stats::visualization::{PieChartData, ChartConfig};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, Borders, Cell, Gauge, List, ListItem, Paragraph, Row, Table, Wrap
    },
};

// Using PieChartData and ChartConfig from visualization module

pub struct AsciiPieChart {
    data: PieChartData,
}

impl AsciiPieChart {
    pub fn new(data: PieChartData, _config: ChartConfig) -> Self {
        Self { data }
    }

    pub fn render(&self, f: &mut ratatui::Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(area);

        self.render_chart(f, chunks[0]);
        self.render_legend(f, chunks[1]);
    }

    fn render_chart(&self, f: &mut ratatui::Frame, area: Rect) {
        let chart_area = Block::default()
            .borders(Borders::ALL)
            .title(" Chart ")
            .inner(area);
        
        // Create a circular representation using block characters
        let center_x = chart_area.width / 2;
        let center_y = chart_area.height / 2;
        let radius = std::cmp::min(center_x, center_y).saturating_sub(1);
        
        let mut chart_content = Vec::new();
        
        // Generate pie slices
        let _current_angle = 0.0;
        let total = self.data.values.iter().sum::<f64>();
        
        for row in 0..chart_area.height {
            let mut line_spans = Vec::new();
            
            for col in 0..chart_area.width {
                let dx = col as f64 - center_x as f64;
                let dy = row as f64 - center_y as f64;
                let distance = (dx * dx + dy * dy).sqrt();
                
                if distance <= radius as f64 {
                    let angle = dy.atan2(dx);
                    let normalized_angle = if angle < 0.0 { angle + 2.0 * std::f64::consts::PI } else { angle };
                    
                    let mut slice_index = 0;
                    let mut cumulative_angle = 0.0;
                    
                    for (i, &value) in self.data.values.iter().enumerate() {
                        let slice_angle = (value / total) * 2.0 * std::f64::consts::PI;
                        if normalized_angle >= cumulative_angle && normalized_angle < cumulative_angle + slice_angle {
                            slice_index = i;
                            break;
                        }
                        cumulative_angle += slice_angle;
                    }
                    
                    let color = self.get_color_for_slice(slice_index);
                    line_spans.push(Span::styled("█", Style::default().fg(color)));
                } else {
                    line_spans.push(Span::raw(" "));
                }
            }
            
            chart_content.push(Line::from(line_spans));
        }
        
        let chart_paragraph = Paragraph::new(chart_content)
            .block(Block::default().borders(Borders::ALL).title(" Distribution "));
        
        f.render_widget(chart_paragraph, area);
    }

    fn render_legend(&self, f: &mut ratatui::Frame, area: Rect) {
        let mut legend_items = Vec::new();
        
        for (i, (label, &value)) in self.data.labels.iter().zip(self.data.values.iter()).enumerate() {
            let percentage = (value / self.data.total) * 100.0;
            let color = self.get_color_for_slice(i);
            
            let legend_item = ListItem::new(Line::from(vec![
                Span::styled("██", Style::default().fg(color)),
                Span::raw(format!(" {} ({:.1}%)", label, percentage)),
            ]));
            
            legend_items.push(legend_item);
        }
        
        let legend_list = List::new(legend_items)
            .block(Block::default().borders(Borders::ALL).title(" Legend "));
        
        f.render_widget(legend_list, area);
    }

    fn get_color_for_slice(&self, index: usize) -> Color {
        let colors = [
            Color::Red, Color::Green, Color::Blue, Color::Yellow,
            Color::Magenta, Color::Cyan, Color::Gray, Color::LightRed,
            Color::LightGreen, Color::LightBlue, Color::LightYellow,
            Color::LightMagenta, Color::LightCyan,
        ];
        
        colors[index % colors.len()]
    }
}

/// Render language distribution as horizontal bars
pub fn render_language_bars(f: &mut ratatui::Frame, area: Rect, stats: &AggregatedStats) {
    let mut bars = Vec::new();
    let total_lines = stats.basic.total_lines as f64;
    
    if total_lines == 0.0 {
        let no_data = Paragraph::new("No code found")
            .block(Block::default()
                .borders(Borders::ALL)
                .title(" 🌐 Language Distribution ")
                .title_alignment(Alignment::Center)
                .border_style(Style::default().fg(Color::Gray))
            )
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center);
        f.render_widget(no_data, area);
        return;
    }
    
    // Sort extensions by line count
    let mut extensions: Vec<_> = stats.basic.stats_by_extension.iter().collect();
    extensions.sort_by(|a, b| b.1.total_lines.cmp(&a.1.total_lines));
    
    for (ext, ext_stats) in extensions.iter().take(8) {
        let percentage = (ext_stats.total_lines as f64 / total_lines) * 100.0;
        let (emoji, name) = get_language_info(ext);
        let color = get_language_color(ext);
        
        let bar = Gauge::default()
            .block(Block::default()
                .title(format!(" {} {} - {:.1}% ", emoji, name, percentage))
                .title_alignment(Alignment::Center)
                .border_style(Style::default().fg(color))
            )
            .gauge_style(Style::default().fg(color).bg(Color::Black))
            .ratio(percentage / 100.0)
            .label(format!("{} lines", ext_stats.total_lines));
        
        bars.push(bar);
    }
    
    // Split area into sections for each bar
    let constraints: Vec<Constraint> = bars.iter().map(|_| Constraint::Length(4)).collect();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(area);
    
    for (i, bar) in bars.into_iter().enumerate() {
        if i < chunks.len() {
            f.render_widget(bar, chunks[i]);
        }
    }
}

/// Render structure distribution as horizontal bars
pub fn render_structure_bars(f: &mut ratatui::Frame, area: Rect, stats: &AggregatedStats) {
    let mut bars = Vec::new();
    let total_structures = stats.complexity.total_structures as f64;
    
    if total_structures == 0.0 {
        let no_data = Paragraph::new("No code structures found")
            .block(Block::default()
                .borders(Borders::ALL)
                .title(" 🏗️ Code Structures ")
                .title_alignment(Alignment::Center)
                .border_style(Style::default().fg(Color::Gray))
            )
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center);
        f.render_widget(no_data, area);
        return;
    }
    
    let structures = vec![
        ("Classes", stats.complexity.class_count, "🏛️"),
        ("Interfaces", stats.complexity.interface_count, "🔌"),
        ("Traits", stats.complexity.trait_count, "🎭"),
        ("Enums", stats.complexity.enum_count, "📋"),
        ("Structs", stats.complexity.struct_count, "🏗️"),
        ("Modules", stats.complexity.module_count, "📦"),
    ];
    
    for (name, count, emoji) in structures {
        if count > 0 {
            let percentage = (count as f64 / total_structures) * 100.0;
            let bar = Gauge::default()
                .block(Block::default()
                    .title(format!(" {} {} - {:.1}% ", emoji, name, percentage))
                    .title_alignment(Alignment::Center)
                    .border_style(Style::default().fg(get_structure_color(name)))
                )
                .gauge_style(Style::default().fg(get_structure_color(name)).bg(Color::Black))
                .ratio(percentage / 100.0)
                .label(format!("{} items", count));
            
            bars.push(bar);
        }
    }
    
    // Split area into sections for each bar
    let constraints: Vec<Constraint> = bars.iter().map(|_| Constraint::Length(4)).collect();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(area);
    
    for (i, bar) in bars.into_iter().enumerate() {
        if i < chunks.len() {
            f.render_widget(bar, chunks[i]);
        }
    }
}

/// Render enhanced complexity distribution with more detailed metrics
pub fn render_complexity_bars(f: &mut ratatui::Frame, area: Rect, stats: &AggregatedStats) {
    let dist = &stats.complexity.complexity_distribution;
    let total = (dist.very_low_complexity + dist.low_complexity + dist.medium_complexity + dist.high_complexity + dist.very_high_complexity) as f64;
    
    if total == 0.0 {
        let no_data = Paragraph::new("No functions found")
            .block(Block::default()
                .borders(Borders::ALL)
                .title(" 🎯 Complexity Distribution ")
                .title_alignment(Alignment::Center)
                .border_style(Style::default().fg(Color::Gray))
            )
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center);
        f.render_widget(no_data, area);
        return;
    }
    
    let complexity_data = vec![
        ("Very Low (1-5)", dist.very_low_complexity, "🟢", Color::LightGreen),
        ("Low (6-10)", dist.low_complexity, "🟡", Color::Green),
        ("Medium (11-20)", dist.medium_complexity, "🟠", Color::Yellow),
        ("High (21-50)", dist.high_complexity, "🔴", Color::Red),
        ("Very High (51+)", dist.very_high_complexity, "🟣", Color::Magenta),
    ];
    
    let mut bars = Vec::new();
    
    for (name, count, emoji, color) in complexity_data {
        if count > 0 {
            let percentage = (count as f64 / total) * 100.0;
            let bar = Gauge::default()
                .block(Block::default()
                    .title(format!(" {} {} - {:.1}% ", emoji, name, percentage))
                    .title_alignment(Alignment::Center)
                    .border_style(Style::default().fg(color))
                )
                .gauge_style(Style::default().fg(color).bg(Color::Black))
                .ratio(percentage / 100.0)
                .label(format!("{} functions", count));
            
            bars.push(bar);
        }
    }
    
    // Split area into sections for each bar
    let constraints: Vec<Constraint> = bars.iter().map(|_| Constraint::Length(4)).collect();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(area);
    
    for (i, bar) in bars.into_iter().enumerate() {
        if i < chunks.len() {
            f.render_widget(bar, chunks[i]);
        }
    }
}

/// Enhanced quality metrics display with detailed insights
pub fn render_quality_metrics(f: &mut ratatui::Frame, area: Rect, stats: &AggregatedStats) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(60),
            Constraint::Percentage(40),
        ])
        .split(area);
    
    // Code health metrics table
    let quality_rows = vec![
        Row::new(vec![
            Cell::from("Code Health"),
            Cell::from(format!("{:.1}%", stats.complexity.quality_metrics.code_health_score)),
            Cell::from(get_quality_indicator(stats.complexity.quality_metrics.code_health_score)),
        ]),
        Row::new(vec![
            Cell::from("Maintainability"),
            Cell::from(format!("{:.1}%", stats.complexity.quality_metrics.maintainability_index)),
            Cell::from(get_quality_indicator(stats.complexity.quality_metrics.maintainability_index)),
        ]),
        Row::new(vec![
            Cell::from("Avg Complexity"),
            Cell::from(format!("{:.1}", stats.complexity.quality_metrics.avg_complexity)),
            Cell::from(get_complexity_indicator(stats.complexity.quality_metrics.avg_complexity)),
        ]),
        Row::new(vec![
            Cell::from("Function Size"),
            Cell::from(format!("{:.1}%", stats.complexity.quality_metrics.function_size_health)),
            Cell::from(get_quality_indicator(stats.complexity.quality_metrics.function_size_health)),
        ]),
        Row::new(vec![
            Cell::from("Documentation"),
            Cell::from(format!("{:.1}%", stats.complexity.quality_metrics.documentation_coverage)),
            Cell::from(get_quality_indicator(stats.complexity.quality_metrics.documentation_coverage)),
        ]),
        Row::new(vec![
            Cell::from("Technical Debt"),
            Cell::from(format!("{:.1}%", stats.complexity.quality_metrics.technical_debt_ratio)),
            Cell::from(get_debt_indicator(stats.complexity.quality_metrics.technical_debt_ratio)),
        ]),
    ];
    
    let quality_table = Table::new(quality_rows, &[
        Constraint::Length(15),
        Constraint::Length(10),
        Constraint::Length(10),
    ])
    .header(Row::new(vec!["Metric", "Score", "Status"]))
    .block(Block::default().borders(Borders::ALL).title(" 🎯 Code Health Metrics "))
    .style(Style::default().fg(Color::White));
    
    f.render_widget(quality_table, chunks[0]);
    
    // Quality recommendations
    let recommendations = generate_quality_recommendations(stats);
    let rec_items: Vec<ListItem> = recommendations
        .iter()
        .map(|rec| {
            let (icon, color) = match rec.priority {
                RecommendationPriority::High => ("🔴", Color::Red),
                RecommendationPriority::Medium => ("🟡", Color::Yellow),
                RecommendationPriority::Low => ("🟢", Color::Green),
            };
            
            ListItem::new(vec![
                Line::from(vec![
                    Span::styled(format!("{} ", icon), Style::default().fg(color)),
                    Span::styled(&rec.title, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                ]),
                Line::from(vec![
                    Span::styled(format!("  {}", rec.description), Style::default().fg(Color::Gray)),
                ]),
            ])
        })
        .collect();
    
    let rec_list = List::new(rec_items)
        .block(Block::default().borders(Borders::ALL).title(" 💡 Recommendations "))
        .style(Style::default().fg(Color::White));
    
    f.render_widget(rec_list, chunks[1]);
}

/// Enhanced function complexity table with detailed analysis
pub fn render_function_complexity_table(f: &mut ratatui::Frame, area: Rect, stats: &AggregatedStats) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(70),
            Constraint::Percentage(30),
        ])
        .split(area);
    
    // Language complexity breakdown
    let mut lang_complexity: Vec<_> = stats.complexity.complexity_by_extension.iter().collect();
    lang_complexity.sort_by(|a, b| b.1.cyclomatic_complexity.partial_cmp(&a.1.cyclomatic_complexity).unwrap());
    
    let lang_rows: Vec<Row> = lang_complexity
        .iter()
        .take(8) // Show top 8 languages
        .map(|(ext, complexity)| {
            let complexity_level = if complexity.cyclomatic_complexity <= 5.0 { "Low" } 
                                  else if complexity.cyclomatic_complexity <= 10.0 { "Medium" } 
                                  else { "High" };
            
            Row::new(vec![
                Cell::from(format!("{}", ext)),
                Cell::from(format!("{}", complexity.function_count)),
                Cell::from(format!("{:.1}", complexity.cyclomatic_complexity)),
                Cell::from(format!("{:.1}", complexity.average_function_length)),
                Cell::from(format!("{}", complexity.max_nesting_depth)),
                Cell::from(complexity_level),
            ])
        })
        .collect();
    
    let lang_table = Table::new(lang_rows, &[
        Constraint::Length(8),
        Constraint::Length(8),
        Constraint::Length(10),
        Constraint::Length(10),
        Constraint::Length(8),
        Constraint::Length(8),
    ])
    .header(Row::new(vec!["Lang", "Funcs", "Complexity", "Avg Length", "Nesting", "Level"]))
    .block(Block::default().borders(Borders::ALL).title(" 📊 Language Complexity Breakdown "))
    .style(Style::default().fg(Color::White));
    
    f.render_widget(lang_table, chunks[0]);
    
    // Complexity insights
    let insights = generate_complexity_insights(stats);
    let insight_items: Vec<ListItem> = insights
        .iter()
        .map(|insight| {
            ListItem::new(vec![
                Line::from(vec![
                    Span::styled(format!("{} ", insight.icon), Style::default().fg(insight.color)),
                    Span::styled(&insight.text, Style::default().fg(Color::White)),
                ]),
            ])
        })
        .collect();
    
    let insight_list = List::new(insight_items)
        .block(Block::default().borders(Borders::ALL).title(" 💡 Complexity Insights "))
        .style(Style::default().fg(Color::White));
    
    f.render_widget(insight_list, chunks[1]);
}

/// Get language info (emoji and name)
fn get_language_info(ext: &str) -> (&'static str, &'static str) {
    match ext {
        "rs" => ("🦀", "Rust"),
        "py" => ("🐍", "Python"),
        "js" => ("🟨", "JavaScript"),
        "ts" => ("🔷", "TypeScript"),
        "jsx" => ("⚛️", "React"),
        "tsx" => ("⚛️", "React TS"),
        "java" => ("☕", "Java"),
        "cpp" | "cc" | "cxx" => ("⚡", "C++"),
        "c" => ("🔧", "C"),
        "h" | "hpp" => ("📄", "Header"),
        "go" => ("🐹", "Go"),
        "cs" => ("🔷", "C#"),
        "php" => ("🐘", "PHP"),
        "rb" => ("💎", "Ruby"),
        "swift" => ("🍎", "Swift"),
        "kt" => ("🎯", "Kotlin"),
        "html" => ("🌐", "HTML"),
        "css" => ("🎨", "CSS"),
        "scss" => ("🎨", "SCSS"),
        "sql" => ("🗃️", "SQL"),
        "sh" => ("🐚", "Shell"),
        "md" => ("📝", "Markdown"),
        "json" => ("📋", "JSON"),
        "xml" => ("📄", "XML"),
        "yaml" | "yml" => ("⚙️", "YAML"),
        "toml" => ("⚙️", "TOML"),
        "hs" | "lhs" | "hsc" => ("λ", "Haskell"),
        "ex" | "exs" | "eex" => ("💧", "Elixir"),
        "erl" | "hrl" => ("📞", "Erlang"),
        "jl" => ("🔬", "Julia"),
        "lua" => ("🌙", "Lua"),
        "pl" | "pm" | "pod" => ("🐪", "Perl"),
        "zig" => ("⚡", "Zig"),
        "clj" | "cljs" | "cljc" => ("🔄", "Clojure"),
        "ps1" | "psm1" | "psd1" => ("⚡", "PowerShell"),
        "bat" | "cmd" => ("⚙️", "Batch"),
        "vb" | "vbs" => ("🔷", "Visual Basic"),
        "mlx" => ("📊", "MATLAB"),
        "rmd" | "Rmd" => ("📊", "R Markdown"),
        _ => ("📄", "Unknown"),
    }
}

fn get_language_color(ext: &str) -> Color {
    match ext {
        "rs" => Color::Red,
        "py" => Color::Blue,
        "js" => Color::Yellow,
        "ts" => Color::Blue,
        "jsx" | "tsx" => Color::Cyan,
        "java" => Color::Red,
        "cpp" | "cc" | "cxx" | "c" => Color::Blue,
        "go" => Color::Cyan,
        "cs" => Color::Blue,
        "php" => Color::Magenta,
        "rb" => Color::Red,
        "swift" => Color::Red,
        "kt" => Color::Magenta,
        "html" => Color::Red,
        "css" | "scss" => Color::Blue,
        "sql" => Color::Yellow,
        "sh" => Color::Green,
        _ => Color::Gray,
    }
}

fn get_structure_color(structure_type: &str) -> Color {
    match structure_type {
        "Classes" => Color::Blue,
        "Interfaces" => Color::Cyan,
        "Traits" => Color::Magenta,
        "Enums" => Color::Yellow,
        "Structs" => Color::Green,
        "Modules" => Color::Red,
        _ => Color::Gray,
    }
}

fn get_quality_color(score: f64) -> Color {
    if score >= 80.0 {
        Color::Green
    } else if score >= 60.0 {
        Color::Yellow
    } else if score >= 40.0 {
        Color::Red
    } else {
        Color::Magenta
    }
}

fn get_complexity_color(complexity: f64) -> Color {
    if complexity <= 5.0 {
        Color::Green
    } else if complexity <= 10.0 {
        Color::Yellow
    } else if complexity <= 20.0 {
        Color::Red
    } else {
        Color::Magenta
    }
}

fn get_parameter_color(params: f64) -> Color {
    if params <= 3.0 {
        Color::Green
    } else if params <= 5.0 {
        Color::Yellow
    } else {
        Color::Red
    }
}

/// Render enhanced overview with charts
pub fn render_enhanced_overview(f: &mut ratatui::Frame, area: Rect, stats: &AggregatedStats) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(6),  // First row of 4 metrics boxes
            Constraint::Length(6),  // Second row of 4 metrics boxes
            Constraint::Length(8),  // Code breakdown bars
            Constraint::Min(10),    // Language distribution
        ])
        .split(area);
    
    // First row of metrics boxes
    render_first_metrics_row(f, chunks[0], stats);
    
    // Second row of metrics boxes
    render_second_metrics_row(f, chunks[1], stats);
    
    // Code breakdown with progress bars
    render_code_breakdown_bars(f, chunks[2], stats);
    
    // Language distribution
    render_language_bars(f, chunks[3], stats);
}

/// Render first row of 4 metrics boxes
fn render_first_metrics_row(f: &mut ratatui::Frame, area: Rect, stats: &AggregatedStats) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .split(area);
    
    // Average Complexity
    let avg_complexity = stats.complexity.cyclomatic_complexity;
    let complexity_color = if avg_complexity <= 5.0 { Color::Green } 
                          else if avg_complexity <= 10.0 { Color::Yellow } 
                          else { Color::Red };
    
    let complexity_text = vec![
        Line::from(vec![
            Span::styled("🔄", Style::default().fg(complexity_color)),
        ]),
        Line::from(vec![
            Span::styled(format!("{:.1}", avg_complexity), Style::default().fg(complexity_color).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("Avg Complexity", Style::default().fg(Color::White)),
        ]),
    ];
    
    let complexity_block = Paragraph::new(complexity_text)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(Color::White));
    f.render_widget(complexity_block, chunks[0]);
    
    // Maintainability Index
    let maintainability = stats.complexity.maintainability_index;
    let maint_color = if maintainability >= 80.0 { Color::Green } 
                     else if maintainability >= 60.0 { Color::Yellow } 
                     else { Color::Red };
    
    let maint_text = vec![
        Line::from(vec![
            Span::styled("🔧", Style::default().fg(maint_color)),
        ]),
        Line::from(vec![
            Span::styled(format!("{:.1}", maintainability), Style::default().fg(maint_color).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("Maintainability", Style::default().fg(Color::White)),
        ]),
    ];
    
    let maint_block = Paragraph::new(maint_text)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(Color::White));
    f.render_widget(maint_block, chunks[1]);
    
    // Max Nesting Depth
    let max_nesting = stats.complexity.max_nesting_depth;
    let nesting_color = if max_nesting <= 3 { Color::Green } 
                       else if max_nesting <= 5 { Color::Yellow } 
                       else { Color::Red };
    
    let nesting_text = vec![
        Line::from(vec![
            Span::styled("🏗️", Style::default().fg(nesting_color)),
        ]),
        Line::from(vec![
            Span::styled(format!("{}", max_nesting), Style::default().fg(nesting_color).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("Max Nesting", Style::default().fg(Color::White)),
        ]),
    ];
    
    let nesting_block = Paragraph::new(nesting_text)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(Color::White));
    f.render_widget(nesting_block, chunks[2]);
    
    // Function Count
    let function_count = stats.complexity.function_count;
    let func_color = Color::Blue;
    
    let func_text = vec![
        Line::from(vec![
            Span::styled("⚙️", Style::default().fg(func_color)),
        ]),
        Line::from(vec![
            Span::styled(format!("{}", function_count), Style::default().fg(func_color).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("Functions", Style::default().fg(Color::White)),
        ]),
    ];
    
    let func_block = Paragraph::new(func_text)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(Color::White));
    f.render_widget(func_block, chunks[3]);
}

/// Render second row of 4 new metrics boxes
fn render_second_metrics_row(f: &mut ratatui::Frame, area: Rect, stats: &AggregatedStats) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .split(area);
    
    // Total Lines
    let total_lines = stats.basic.total_lines;
    let lines_color = Color::Blue;
    
    let lines_text = vec![
        Line::from(vec![
            Span::styled("📏", Style::default().fg(lines_color)),
        ]),
        Line::from(vec![
            Span::styled(format!("{}", format_number(total_lines)), Style::default().fg(lines_color).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("Lines", Style::default().fg(Color::White)),
        ]),
    ];
    
    let lines_block = Paragraph::new(lines_text)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(Color::White));
    f.render_widget(lines_block, chunks[0]);
    
    // File Count
    let file_count = stats.basic.total_files;
    let file_color = Color::Yellow;
    
    let file_text = vec![
        Line::from(vec![
            Span::styled("📁", Style::default().fg(file_color)),
        ]),
        Line::from(vec![
            Span::styled(format!("{}", file_count), Style::default().fg(file_color).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("Files", Style::default().fg(Color::White)),
        ]),
    ];
    
    let file_block = Paragraph::new(file_text)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(Color::White));
    f.render_widget(file_block, chunks[1]);
    
    // Average Lines per File
    let avg_lines_per_file = stats.basic.average_lines_per_file;
    let avg_lines_color = Color::Green;
    
    let avg_lines_text = vec![
        Line::from(vec![
            Span::styled("📊", Style::default().fg(avg_lines_color)),
        ]),
        Line::from(vec![
            Span::styled(format!("{:.0}", avg_lines_per_file), Style::default().fg(avg_lines_color).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("Avg Line Count", Style::default().fg(Color::White)),
        ]),
    ];
    
    let avg_lines_block = Paragraph::new(avg_lines_text)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(Color::White));
    f.render_widget(avg_lines_block, chunks[2]);
    
    // Languages Detected
    let language_count = stats.metadata.languages_detected.len();
    let lang_color = Color::Magenta;
    
    let lang_text = vec![
        Line::from(vec![
            Span::styled("🌐", Style::default().fg(lang_color)),
        ]),
        Line::from(vec![
            Span::styled(format!("{}", language_count), Style::default().fg(lang_color).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("Languages", Style::default().fg(Color::White)),
        ]),
    ];
    
    let lang_block = Paragraph::new(lang_text)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(Color::White));
    f.render_widget(lang_block, chunks[3]);
}

/// Helper function to format numbers with commas
fn format_number(num: usize) -> String {
    let num_str = num.to_string();
    let mut result = String::new();
    let chars: Vec<char> = num_str.chars().collect();
    
    for (i, ch) in chars.iter().enumerate() {
        if i > 0 && (chars.len() - i) % 3 == 0 {
            result.push(',');
        }
        result.push(*ch);
    }
    
    result
}

/// Render code breakdown with progress bars
fn render_code_breakdown_bars(f: &mut ratatui::Frame, area: Rect, stats: &AggregatedStats) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .split(area);
    
    let total_lines = stats.basic.total_lines as f64;
    
    if total_lines == 0.0 {
        let no_data = Paragraph::new("No code analyzed")
            .block(Block::default().borders(Borders::ALL).title(" Code Breakdown "))
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center);
        f.render_widget(no_data, area);
        return;
    }
    
    // Code percentage
    let code_pct = (stats.basic.code_lines as f64 / total_lines * 100.0) as u16;
    let code_gauge = Gauge::default()
        .block(Block::default()
            .borders(Borders::ALL)
            .title(format!(" 💻 Code - {}% ", code_pct))
            .title_alignment(Alignment::Center)
            .border_style(Style::default().fg(Color::Green))
        )
        .gauge_style(Style::default().fg(Color::Green).bg(Color::Black))
        .percent(code_pct)
        .label(format!("{}", stats.basic.code_lines));
    f.render_widget(code_gauge, chunks[0]);
    
    // Comments percentage
    let comment_pct = (stats.basic.comment_lines as f64 / total_lines * 100.0) as u16;
    let comment_gauge = Gauge::default()
        .block(Block::default()
            .borders(Borders::ALL)
            .title(format!(" 💬 Comments - {}% ", comment_pct))
            .title_alignment(Alignment::Center)
            .border_style(Style::default().fg(Color::Magenta))
        )
        .gauge_style(Style::default().fg(Color::Magenta).bg(Color::Black))
        .percent(comment_pct)
        .label(format!("{}", stats.basic.comment_lines));
    f.render_widget(comment_gauge, chunks[1]);
    
    // Documentation percentage
    let doc_pct = (stats.basic.doc_lines as f64 / total_lines * 100.0) as u16;
    let doc_gauge = Gauge::default()
        .block(Block::default()
            .borders(Borders::ALL)
            .title(format!(" 📚 Docs - {}% ", doc_pct))
            .title_alignment(Alignment::Center)
            .border_style(Style::default().fg(Color::Cyan))
        )
        .gauge_style(Style::default().fg(Color::Cyan).bg(Color::Black))
        .percent(doc_pct)
        .label(format!("{}", stats.basic.doc_lines));
    f.render_widget(doc_gauge, chunks[2]);
    
    // Blank lines percentage
    let blank_pct = (stats.basic.blank_lines as f64 / total_lines * 100.0) as u16;
    let blank_gauge = Gauge::default()
        .block(Block::default()
            .borders(Borders::ALL)
            .title(format!(" ⬜ Blank - {}% ", blank_pct))
            .title_alignment(Alignment::Center)
            .border_style(Style::default().fg(Color::Gray))
        )
        .gauge_style(Style::default().fg(Color::Gray).bg(Color::Black))
        .percent(blank_pct)
        .label(format!("{}", stats.basic.blank_lines));
    f.render_widget(blank_gauge, chunks[3]);
}

/// Get color based on confidence score (0.0 to 1.0)
fn get_confidence_color(confidence: f64) -> Color {
    if confidence >= 0.8 {
        Color::Green
    } else if confidence >= 0.6 {
        Color::Yellow
    } else if confidence >= 0.4 {
        Color::Rgb(255, 165, 0) // Orange
    } else {
        Color::Red
    }
}

// Helper structures and functions
#[derive(Debug, Clone)]
enum RecommendationPriority {
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone)]
struct QualityRecommendation {
    title: String,
    description: String,
    priority: RecommendationPriority,
}

#[derive(Debug, Clone)]
struct ComplexityInsight {
    icon: String,
    text: String,
    color: Color,
}

fn get_quality_indicator(score: f64) -> String {
    if score >= 80.0 { "🟢 Good".to_string() }
    else if score >= 60.0 { "🟡 Fair".to_string() }
    else { "🔴 Poor".to_string() }
}

fn get_complexity_indicator(complexity: f64) -> String {
    if complexity <= 5.0 { "🟢 Low".to_string() }
    else if complexity <= 10.0 { "🟡 Medium".to_string() }
    else { "🔴 High".to_string() }
}

fn get_debt_indicator(debt: f64) -> String {
    if debt <= 20.0 { "🟢 Low".to_string() }
    else if debt <= 40.0 { "🟡 Medium".to_string() }
    else { "🔴 High".to_string() }
}

fn generate_quality_recommendations(stats: &AggregatedStats) -> Vec<QualityRecommendation> {
    let mut recommendations = Vec::new();
    
    // Check average complexity
    if stats.complexity.quality_metrics.avg_complexity > 10.0 {
        recommendations.push(QualityRecommendation {
            title: "High Complexity Functions".to_string(),
            description: "Break down functions with complexity > 10 into smaller, focused methods".to_string(),
            priority: RecommendationPriority::High,
        });
    }
    
    // Check maintainability index
    if stats.complexity.quality_metrics.maintainability_index < 60.0 {
        recommendations.push(QualityRecommendation {
            title: "Low Maintainability Index".to_string(),
            description: "Refactor large functions and improve code documentation".to_string(),
            priority: RecommendationPriority::High,
        });
    }
    
    // Check documentation coverage
    if stats.complexity.quality_metrics.documentation_coverage < 25.0 {
        recommendations.push(QualityRecommendation {
            title: "Low Documentation Coverage".to_string(),
            description: "Add docstrings and comments to improve code understanding".to_string(),
            priority: RecommendationPriority::Medium,
        });
    }
    
    // Check function size health
    if stats.complexity.quality_metrics.function_size_health < 70.0 {
        recommendations.push(QualityRecommendation {
            title: "Large Functions Detected".to_string(),
            description: "Split functions longer than 50 lines into smaller, focused functions".to_string(),
            priority: RecommendationPriority::Medium,
        });
    }
    
    // Check nesting depth
    if stats.complexity.max_nesting_depth > 5 {
        recommendations.push(QualityRecommendation {
            title: "Deep Nesting Detected".to_string(),
            description: "Reduce nesting depth by extracting functions or using early returns".to_string(),
            priority: RecommendationPriority::Medium,
        });
    }
    
    // Check technical debt
    if stats.complexity.quality_metrics.technical_debt_ratio > 40.0 {
        recommendations.push(QualityRecommendation {
            title: "High Technical Debt".to_string(),
            description: "Prioritize refactoring to reduce complexity and improve documentation".to_string(),
            priority: RecommendationPriority::High,
        });
    }
    
    // Check function length
    if stats.complexity.average_function_length > 50.0 {
        recommendations.push(QualityRecommendation {
            title: "Long Functions".to_string(),
            description: "Consider breaking down long functions into smaller, focused units".to_string(),
            priority: RecommendationPriority::Low,
        });
    }
    
    // Positive feedback
    if stats.complexity.cyclomatic_complexity <= 5.0 {
        recommendations.push(QualityRecommendation {
            title: "Good Code Complexity".to_string(),
            description: "Your code maintains good complexity levels - keep it up!".to_string(),
            priority: RecommendationPriority::Low,
        });
    }
    
    recommendations
}

fn generate_complexity_insights(stats: &AggregatedStats) -> Vec<ComplexityInsight> {
    let mut insights = Vec::new();
    
    let total_functions = stats.complexity.function_count;
    let avg_complexity = stats.complexity.cyclomatic_complexity;
    let high_complexity_count = stats.complexity.complexity_distribution.high_complexity + 
                               stats.complexity.complexity_distribution.very_high_complexity;
    
    // Complexity distribution insight
    if high_complexity_count > 0 {
        let percentage = (high_complexity_count as f64 / total_functions as f64) * 100.0;
        insights.push(ComplexityInsight {
            icon: "⚠️".to_string(),
            text: format!("{:.1}% of functions have high complexity", percentage),
            color: Color::Red,
        });
    }
    
    // Average complexity insight
    if avg_complexity <= 3.0 {
        insights.push(ComplexityInsight {
            icon: "✅".to_string(),
            text: "Excellent complexity management".to_string(),
            color: Color::Green,
        });
    } else if avg_complexity <= 7.0 {
        insights.push(ComplexityInsight {
            icon: "👍".to_string(),
            text: "Good complexity levels overall".to_string(),
            color: Color::Yellow,
        });
    } else {
        insights.push(ComplexityInsight {
            icon: "🔧".to_string(),
            text: "Consider refactoring for better complexity".to_string(),
            color: Color::Red,
        });
    }
    
    // Function size insight
    if stats.complexity.average_function_length <= 20.0 {
        insights.push(ComplexityInsight {
            icon: "📏".to_string(),
            text: "Functions are well-sized".to_string(),
            color: Color::Green,
        });
    } else if stats.complexity.average_function_length <= 40.0 {
        insights.push(ComplexityInsight {
            icon: "📐".to_string(),
            text: "Function sizes are reasonable".to_string(),
            color: Color::Yellow,
        });
    } else {
        insights.push(ComplexityInsight {
            icon: "📊".to_string(),
            text: "Consider breaking down large functions".to_string(),
            color: Color::Red,
        });
    }
    
    // Nesting depth insight
    if stats.complexity.max_nesting_depth <= 3 {
        insights.push(ComplexityInsight {
            icon: "🏗️".to_string(),
            text: "Good nesting depth control".to_string(),
            color: Color::Green,
        });
    } else {
        insights.push(ComplexityInsight {
            icon: "🔄".to_string(),
            text: "Consider reducing nesting depth".to_string(),
            color: Color::Yellow,
        });
    }
    
    insights
} 

/// Advanced language visualizer with detailed statistics and visual charts
pub fn render_advanced_language_visualizer(f: &mut ratatui::Frame, area: Rect, stats: &AggregatedStats) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(12), // Language distribution with advanced metrics
            Constraint::Min(10),    // Detailed language analysis table
        ])
        .split(area);
    
    // Advanced language distribution
    render_advanced_language_distribution(f, chunks[0], stats);
    
    // Detailed language analysis table
    render_detailed_language_analysis(f, chunks[1], stats);
}

/// Render advanced language distribution with enhanced metrics
fn render_advanced_language_distribution(f: &mut ratatui::Frame, area: Rect, stats: &AggregatedStats) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(60), // Language bars with enhanced info
            Constraint::Percentage(40), // Language statistics summary
        ])
        .split(area);
    
    // Enhanced language bars
    render_enhanced_language_bars(f, chunks[0], stats);
    
    // Language statistics summary
    render_language_statistics_summary(f, chunks[1], stats);
}

/// Render enhanced language bars with more detailed information
fn render_enhanced_language_bars(f: &mut ratatui::Frame, area: Rect, stats: &AggregatedStats) {
    let mut bars = Vec::new();
    let total_lines = stats.basic.total_lines as f64;
    
    if total_lines == 0.0 {
        let no_data = Paragraph::new("No code found")
            .block(Block::default()
                .borders(Borders::ALL)
                .title(" 🌐 Language Distribution ")
                .title_alignment(Alignment::Center)
                .border_style(Style::default().fg(Color::Gray))
            )
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center);
        f.render_widget(no_data, area);
        return;
    }
    
    // Sort extensions by line count and take top 6
    let mut extensions: Vec<_> = stats.basic.stats_by_extension.iter().collect();
    extensions.sort_by(|a, b| b.1.total_lines.cmp(&a.1.total_lines));
    
    for (ext, ext_stats) in extensions.iter().take(6) {
        let percentage = (ext_stats.total_lines as f64 / total_lines) * 100.0;
        let (emoji, name) = get_language_info(ext);
        let color = get_language_color(ext);
        
        // Calculate additional metrics
        let avg_file_size = if ext_stats.file_count > 0 {
            ext_stats.total_lines / ext_stats.file_count
        } else {
            0
        };
        
        let code_ratio = if ext_stats.total_lines > 0 {
            (ext_stats.code_lines as f64 / ext_stats.total_lines as f64) * 100.0
        } else {
            0.0
        };
        
        let bar = Gauge::default()
            .block(Block::default()
                .title(format!(" {} {} - {:.1}% ({} files, {:.0}% code, ~{} lines/file) ", 
                    emoji, name, percentage, ext_stats.file_count, code_ratio, avg_file_size))
                .title_alignment(Alignment::Center)
                .border_style(Style::default().fg(color))
            )
            .gauge_style(Style::default().fg(color).bg(Color::Black))
            .ratio(percentage / 100.0)
            .label(format!("{} lines", ext_stats.total_lines));
        
        bars.push(bar);
    }
    
    // Split area into sections for each bar
    let constraints: Vec<Constraint> = bars.iter().map(|_| Constraint::Length(2)).collect();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(area);
    
    for (i, bar) in bars.into_iter().enumerate() {
        if i < chunks.len() {
            f.render_widget(bar, chunks[i]);
        }
    }
}

/// Render language statistics summary
fn render_language_statistics_summary(f: &mut ratatui::Frame, area: Rect, stats: &AggregatedStats) {
    let mut summary_lines = Vec::new();
    
    // Header
    summary_lines.push(Line::from(vec![
        Span::styled("📊 Language Insights", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
    ]));
    summary_lines.push(Line::from(""));
    
    // Total languages
    summary_lines.push(Line::from(vec![
        Span::styled("🌐 Languages: ", Style::default().fg(Color::Blue)),
        Span::styled(stats.metadata.languages_detected.len().to_string(), Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
    ]));
    
    // Find dominant language
    let dominant_language = stats.basic.stats_by_extension.iter()
        .max_by_key(|(_, ext_stats)| ext_stats.total_lines)
        .map(|(ext, _)| {
            let (emoji, name) = get_language_info(ext);
            format!("{} {}", emoji, name)
        })
        .unwrap_or_else(|| "Unknown".to_string());
    
    summary_lines.push(Line::from(vec![
        Span::styled("👑 Dominant: ", Style::default().fg(Color::Yellow)),
        Span::styled(dominant_language, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
    ]));
    
    // Language diversity score
    let diversity_score = calculate_language_diversity_score(stats);
    let diversity_color = if diversity_score > 0.7 { Color::Green } 
                         else if diversity_score > 0.4 { Color::Yellow } 
                         else { Color::Red };
    
    summary_lines.push(Line::from(vec![
        Span::styled("🎯 Diversity: ", Style::default().fg(Color::Cyan)),
        Span::styled(format!("{:.1}%", diversity_score * 100.0), Style::default().fg(diversity_color).add_modifier(Modifier::BOLD)),
    ]));
    
    // Average file size across languages
    let avg_file_size = if stats.basic.total_files > 0 {
        stats.basic.total_lines / stats.basic.total_files
    } else {
        0
    };
    
    summary_lines.push(Line::from(vec![
        Span::styled("📏 Avg File Size: ", Style::default().fg(Color::Magenta)),
        Span::styled(format!("{} lines", avg_file_size), Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
    ]));
    
    // Code density
    let code_density = if stats.basic.total_lines > 0 {
        (stats.basic.code_lines as f64 / stats.basic.total_lines as f64) * 100.0
    } else {
        0.0
    };
    
    let density_color = if code_density > 70.0 { Color::Green } 
                       else if code_density > 50.0 { Color::Yellow } 
                       else { Color::Red };
    
    summary_lines.push(Line::from(vec![
        Span::styled("💻 Code Density: ", Style::default().fg(Color::Green)),
        Span::styled(format!("{:.1}%", code_density), Style::default().fg(density_color).add_modifier(Modifier::BOLD)),
    ]));
    
    let summary_paragraph = Paragraph::new(summary_lines)
        .block(Block::default()
            .borders(Borders::ALL)
            .title(" 📈 Summary ")
            .title_alignment(Alignment::Center)
            .border_style(Style::default().fg(Color::Blue))
        )
        .wrap(Wrap { trim: true });
    
    f.render_widget(summary_paragraph, area);
}

/// Render detailed language analysis table
fn render_detailed_language_analysis(f: &mut ratatui::Frame, area: Rect, stats: &AggregatedStats) {
    let mut lang_rows = Vec::new();
    
    // Sort extensions by line count
    let mut extensions: Vec<_> = stats.basic.stats_by_extension.iter().collect();
    extensions.sort_by(|a, b| b.1.total_lines.cmp(&a.1.total_lines));
    
    for (ext, ext_stats) in extensions.iter().take(10) {
        let (emoji, name) = get_language_info(ext);
        let percentage = if stats.basic.total_lines > 0 {
            (ext_stats.total_lines as f64 / stats.basic.total_lines as f64) * 100.0
        } else {
            0.0
        };
        
        let avg_file_size = if ext_stats.file_count > 0 {
            ext_stats.total_lines / ext_stats.file_count
        } else {
            0
        };
        
        let code_ratio = if ext_stats.total_lines > 0 {
            (ext_stats.code_lines as f64 / ext_stats.total_lines as f64) * 100.0
        } else {
            0.0
        };
        
        let comment_ratio = if ext_stats.total_lines > 0 {
            ((ext_stats.comment_lines + ext_stats.doc_lines) as f64 / ext_stats.total_lines as f64) * 100.0
        } else {
            0.0
        };
        
        lang_rows.push(Row::new(vec![
            Cell::from(format!("{} {}", emoji, name)),
            Cell::from(format!("{}", ext_stats.file_count)),
            Cell::from(format!("{}", ext_stats.total_lines)),
            Cell::from(format!("{:.1}%", percentage)),
            Cell::from(format!("{}", avg_file_size)),
            Cell::from(format!("{:.1}%", code_ratio)),
            Cell::from(format!("{:.1}%", comment_ratio)),
        ]));
    }
    
    let lang_table = Table::new(lang_rows, &[
        Constraint::Length(12), // Language
        Constraint::Length(6),  // Files
        Constraint::Length(8),  // Lines
        Constraint::Length(6),  // %
        Constraint::Length(8),  // Avg Size
        Constraint::Length(6),  // Code %
        Constraint::Length(8),  // Comment %
    ])
    .header(Row::new(vec![
        Cell::from("Language"),
        Cell::from("Files"),
        Cell::from("Lines"),
        Cell::from("%"),
        Cell::from("Avg Size"),
        Cell::from("Code %"),
        Cell::from("Docs %"),
    ]))
    .block(Block::default()
        .borders(Borders::ALL)
        .title(" 📋 Detailed Language Analysis ")
        .title_alignment(Alignment::Center)
        .border_style(Style::default().fg(Color::Green))
    )
    .style(Style::default().fg(Color::White));
    
    f.render_widget(lang_table, area);
}

/// Calculate language diversity score (0.0 to 1.0)
fn calculate_language_diversity_score(stats: &AggregatedStats) -> f64 {
    let total_lines = stats.basic.total_lines as f64;
    if total_lines == 0.0 {
        return 0.0;
    }
    
    // Calculate Shannon diversity index
    let mut entropy = 0.0;
    for (_, ext_stats) in &stats.basic.stats_by_extension {
        let proportion = ext_stats.total_lines as f64 / total_lines;
        if proportion > 0.0 {
            entropy -= proportion * proportion.ln();
        }
    }
    
    // Normalize to 0-1 scale (assuming max 10 languages for practical purposes)
    let max_entropy = (stats.basic.stats_by_extension.len() as f64).ln();
    if max_entropy > 0.0 {
        entropy / max_entropy
    } else {
        0.0
    }
} 