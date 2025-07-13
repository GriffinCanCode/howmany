use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
};

pub fn format_size(size: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
    let mut size = size as f64;
    let mut unit_index = 0;
    
    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }
    
    if unit_index == 0 {
        format!("{} {}", size as u64, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}

pub fn get_extension_icon(ext: &str) -> &'static str {
    match ext {
        "rs" => "🦀",
        "py" => "🐍",
        "js" | "jsx" => "📜",
        "ts" | "tsx" => "📘",
        "html" => "🌐",
        "css" | "scss" | "sass" => "🎨",
        "json" => "📋",
        "xml" => "📄",
        "yaml" | "yml" => "⚙️",
        "toml" => "🔧",
        "md" => "📝",
        "txt" => "📄",
        "java" => "☕",
        "c" | "cpp" | "cc" | "cxx" => "⚡",
        "h" | "hpp" => "📎",
        "go" => "🐹",
        "php" => "🐘",
        "rb" => "💎",
        "swift" => "🍎",
        "kt" => "🎯",
        "scala" => "🎭",
        "sh" | "bash" | "zsh" => "🐚",
        _ => "📄",
    }
}

pub fn get_file_icon(file_path: &str) -> &'static str {
    if file_path.ends_with(".rs") {
        "🦀"
    } else if file_path.ends_with(".py") {
        "🐍"
    } else if file_path.ends_with(".js") || file_path.ends_with(".jsx") {
        "📜"
    } else if file_path.ends_with(".ts") || file_path.ends_with(".tsx") {
        "📘"
    } else if file_path.ends_with(".toml") {
        "🔧"
    } else if file_path.ends_with(".json") {
        "📋"
    } else if file_path.ends_with(".md") {
        "📝"
    } else {
        "📄"
    }
}

pub fn shorten_path(path: &str, max_length: usize) -> String {
    if path.len() <= max_length {
        path.to_string()
    } else {
        let parts: Vec<&str> = path.split('/').collect();
        if parts.len() > 2 {
            format!(".../{}", parts[parts.len()-1])
        } else {
            format!("...{}", &path[path.len()-max_length+3..])
        }
    }
}

// Helper function to create a centered rectangle
pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
} 