use crate::core::stats::aggregation::AggregatedStats;
use crate::core::stats::complexity::ComplexityLevel;
use crate::core::stats::visualization::{PieChartData, ChartConfig};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, Borders, Cell, Gauge, List, ListItem, Paragraph, Row, Table
    },
};

// Using PieChartData and ChartConfig from visualization module

pub struct AsciiPieChart {
    data: PieChartData,
    config: ChartConfig,
}

impl AsciiPieChart {
    pub fn new(data: PieChartData, config: ChartConfig) -> Self {
        Self { data, config }
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
        let mut current_angle = 0.0;
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
    
    // Quality metrics table
    let quality_rows = vec![
        Row::new(vec![
            Cell::from("Overall Quality"),
            Cell::from(format!("{:.1}%", stats.complexity.quality_metrics.overall_quality_score)),
            Cell::from(get_quality_indicator(stats.complexity.quality_metrics.overall_quality_score)),
        ]),
        Row::new(vec![
            Cell::from("Maintainability"),
            Cell::from(format!("{:.1}%", stats.complexity.quality_metrics.maintainability_score)),
            Cell::from(get_quality_indicator(stats.complexity.quality_metrics.maintainability_score)),
        ]),
        Row::new(vec![
            Cell::from("Readability"),
            Cell::from(format!("{:.1}%", stats.complexity.quality_metrics.readability_score)),
            Cell::from(get_quality_indicator(stats.complexity.quality_metrics.readability_score)),
        ]),
        Row::new(vec![
            Cell::from("Testability"),
            Cell::from(format!("{:.1}%", stats.complexity.quality_metrics.testability_score)),
            Cell::from(get_quality_indicator(stats.complexity.quality_metrics.testability_score)),
        ]),
        Row::new(vec![
            Cell::from("Documentation"),
            Cell::from(format!("{:.1}%", stats.ratios.quality_metrics.documentation_score)),
            Cell::from(get_quality_indicator(stats.ratios.quality_metrics.documentation_score)),
        ]),
        Row::new(vec![
            Cell::from("Code Coverage"),
            Cell::from(format!("{:.1}%", (stats.basic.code_lines as f64 / stats.basic.total_lines as f64) * 100.0)),
            Cell::from(get_quality_indicator((stats.basic.code_lines as f64 / stats.basic.total_lines as f64) * 100.0)),
        ]),
    ];
    
    let quality_table = Table::new(quality_rows, &[
        Constraint::Length(15),
        Constraint::Length(10),
        Constraint::Length(10),
    ])
    .header(Row::new(vec!["Metric", "Score", "Status"]))
    .block(Block::default().borders(Borders::ALL).title(" 🎯 Quality Metrics "))
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

/// Enhanced complexity summary with visual indicators
pub fn render_complexity_summary(f: &mut ratatui::Frame, area: Rect, stats: &AggregatedStats) {
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
            Constraint::Length(8),  // Basic stats
            Constraint::Length(6),  // Complexity summary
            Constraint::Length(6),  // Quality metrics
            Constraint::Min(12),    // Language distribution
            Constraint::Min(8),     // Structure distribution
            Constraint::Min(8),     // Complexity distribution
        ])
        .split(area);
    
    // Basic stats (existing)
    render_basic_stats_summary(f, chunks[0], stats);
    
    // Complexity summary
    render_complexity_summary(f, chunks[1], stats);
    
    // Quality metrics
    render_quality_metrics(f, chunks[2], stats);
    
    // Language distribution
    render_language_bars(f, chunks[3], stats);
    
    // Structure distribution
    render_structure_bars(f, chunks[4], stats);
    
    // Complexity distribution
    render_complexity_bars(f, chunks[5], stats);
}

/// Render basic stats summary
fn render_basic_stats_summary(f: &mut ratatui::Frame, area: Rect, stats: &AggregatedStats) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .split(area);
    
    // Enhanced stats blocks with better styling
    let stats_data = vec![
        ("📊 Files", stats.basic.total_files.to_string(), "tracked", Color::Blue),
        ("📐 Lines", stats.basic.total_lines.to_string(), "total", Color::Green),
        ("🔧 Functions", stats.complexity.function_count.to_string(), "detected", Color::Yellow),
        ("🏗️ Structures", stats.complexity.total_structures.to_string(), "found", Color::Magenta),
    ];
    
    for (i, (title, value, subtitle, color)) in stats_data.into_iter().enumerate() {
        let content = vec![
            Line::from(vec![
                Span::styled(title, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled(value, Style::default().fg(color).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled(subtitle, Style::default().fg(Color::DarkGray)),
            ]),
        ];
        
        let block = Paragraph::new(content)
            .block(Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(color))
                .title(format!(" {} ", title))
                .title_alignment(Alignment::Center)
            )
            .style(Style::default().fg(color))
            .alignment(Alignment::Center);
        
        f.render_widget(block, chunks[i]);
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

fn generate_quality_recommendations(stats: &AggregatedStats) -> Vec<QualityRecommendation> {
    let mut recommendations = Vec::new();
    
    // Check complexity
    if stats.complexity.cyclomatic_complexity > 10.0 {
        recommendations.push(QualityRecommendation {
            title: "High Complexity Detected".to_string(),
            description: "Consider breaking down complex functions into smaller, more manageable pieces".to_string(),
            priority: RecommendationPriority::High,
        });
    }
    
    // Check maintainability
    if stats.complexity.maintainability_index < 60.0 {
        recommendations.push(QualityRecommendation {
            title: "Low Maintainability".to_string(),
            description: "Improve code structure and reduce complexity to enhance maintainability".to_string(),
            priority: RecommendationPriority::High,
        });
    }
    
    // Check documentation
    if stats.ratios.quality_metrics.documentation_score < 50.0 {
        recommendations.push(QualityRecommendation {
            title: "Insufficient Documentation".to_string(),
            description: "Add more comments and documentation to improve code readability".to_string(),
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