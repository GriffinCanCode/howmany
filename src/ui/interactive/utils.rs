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

/// Language information for programming languages
#[derive(Debug, Clone)]
pub struct LanguageInfo {
    pub name: String,
    pub icon: String,
    pub color: String,
    pub extensions: Vec<String>,
}

/// Map file extension to programming language
pub fn get_language_from_extension(ext: &str) -> LanguageInfo {
    match ext {
        "rs" => LanguageInfo {
            name: "Rust".to_string(),
            icon: "🦀".to_string(),
            color: "#dea584".to_string(),
            extensions: vec!["rs".to_string()],
        },
        "py" => LanguageInfo {
            name: "Python".to_string(),
            icon: "🐍".to_string(),
            color: "#3776ab".to_string(),
            extensions: vec!["py".to_string()],
        },
        "js" => LanguageInfo {
            name: "JavaScript".to_string(),
            icon: "📜".to_string(),
            color: "#f7df1e".to_string(),
            extensions: vec!["js".to_string()],
        },
        "jsx" => LanguageInfo {
            name: "React JSX".to_string(),
            icon: "⚛️".to_string(),
            color: "#61dafb".to_string(),
            extensions: vec!["jsx".to_string()],
        },
        "ts" => LanguageInfo {
            name: "TypeScript".to_string(),
            icon: "📘".to_string(),
            color: "#3178c6".to_string(),
            extensions: vec!["ts".to_string()],
        },
        "tsx" => LanguageInfo {
            name: "React TSX".to_string(),
            icon: "⚛️".to_string(),
            color: "#61dafb".to_string(),
            extensions: vec!["tsx".to_string()],
        },
        "html" => LanguageInfo {
            name: "HTML".to_string(),
            icon: "🌐".to_string(),
            color: "#e34f26".to_string(),
            extensions: vec!["html".to_string()],
        },
        "css" => LanguageInfo {
            name: "CSS".to_string(),
            icon: "🎨".to_string(),
            color: "#1572b6".to_string(),
            extensions: vec!["css".to_string()],
        },
        "scss" | "sass" => LanguageInfo {
            name: "Sass".to_string(),
            icon: "🎨".to_string(),
            color: "#cf649a".to_string(),
            extensions: vec!["scss".to_string(), "sass".to_string()],
        },
        "java" => LanguageInfo {
            name: "Java".to_string(),
            icon: "☕".to_string(),
            color: "#ed8b00".to_string(),
            extensions: vec!["java".to_string()],
        },
        "c" => LanguageInfo {
            name: "C".to_string(),
            icon: "⚡".to_string(),
            color: "#00599c".to_string(),
            extensions: vec!["c".to_string()],
        },
        "cpp" | "cc" | "cxx" => LanguageInfo {
            name: "C++".to_string(),
            icon: "⚡".to_string(),
            color: "#00599c".to_string(),
            extensions: vec!["cpp".to_string(), "cc".to_string(), "cxx".to_string()],
        },
        "h" | "hpp" => LanguageInfo {
            name: "C/C++ Header".to_string(),
            icon: "📎".to_string(),
            color: "#00599c".to_string(),
            extensions: vec!["h".to_string(), "hpp".to_string()],
        },
        "go" => LanguageInfo {
            name: "Go".to_string(),
            icon: "🐹".to_string(),
            color: "#00add8".to_string(),
            extensions: vec!["go".to_string()],
        },
        "php" => LanguageInfo {
            name: "PHP".to_string(),
            icon: "🐘".to_string(),
            color: "#777bb4".to_string(),
            extensions: vec!["php".to_string()],
        },
        "rb" => LanguageInfo {
            name: "Ruby".to_string(),
            icon: "💎".to_string(),
            color: "#cc342d".to_string(),
            extensions: vec!["rb".to_string()],
        },
        "swift" => LanguageInfo {
            name: "Swift".to_string(),
            icon: "🍎".to_string(),
            color: "#fa7343".to_string(),
            extensions: vec!["swift".to_string()],
        },
        "kt" => LanguageInfo {
            name: "Kotlin".to_string(),
            icon: "🎯".to_string(),
            color: "#7f52ff".to_string(),
            extensions: vec!["kt".to_string()],
        },
        "scala" => LanguageInfo {
            name: "Scala".to_string(),
            icon: "🎭".to_string(),
            color: "#dc322f".to_string(),
            extensions: vec!["scala".to_string()],
        },
        "cs" => LanguageInfo {
            name: "C#".to_string(),
            icon: "🔷".to_string(),
            color: "#239120".to_string(),
            extensions: vec!["cs".to_string()],
        },
        "sh" | "bash" | "zsh" => LanguageInfo {
            name: "Shell".to_string(),
            icon: "🐚".to_string(),
            color: "#89e051".to_string(),
            extensions: vec!["sh".to_string(), "bash".to_string(), "zsh".to_string()],
        },
        "json" => LanguageInfo {
            name: "JSON".to_string(),
            icon: "📋".to_string(),
            color: "#000000".to_string(),
            extensions: vec!["json".to_string()],
        },
        "xml" => LanguageInfo {
            name: "XML".to_string(),
            icon: "📄".to_string(),
            color: "#e37933".to_string(),
            extensions: vec!["xml".to_string()],
        },
        "yaml" | "yml" => LanguageInfo {
            name: "YAML".to_string(),
            icon: "⚙️".to_string(),
            color: "#cb171e".to_string(),
            extensions: vec!["yaml".to_string(), "yml".to_string()],
        },
        "toml" => LanguageInfo {
            name: "TOML".to_string(),
            icon: "🔧".to_string(),
            color: "#9c4221".to_string(),
            extensions: vec!["toml".to_string()],
        },
        "md" => LanguageInfo {
            name: "Markdown".to_string(),
            icon: "📝".to_string(),
            color: "#083fa1".to_string(),
            extensions: vec!["md".to_string()],
        },
        "txt" => LanguageInfo {
            name: "Text".to_string(),
            icon: "📄".to_string(),
            color: "#6c757d".to_string(),
            extensions: vec!["txt".to_string()],
        },
        "sql" => LanguageInfo {
            name: "SQL".to_string(),
            icon: "🗃️".to_string(),
            color: "#e38c00".to_string(),
            extensions: vec!["sql".to_string()],
        },
        "r" => LanguageInfo {
            name: "R".to_string(),
            icon: "📊".to_string(),
            color: "#198ce7".to_string(),
            extensions: vec!["r".to_string()],
        },
        "dart" => LanguageInfo {
            name: "Dart".to_string(),
            icon: "🎯".to_string(),
            color: "#0175c2".to_string(),
            extensions: vec!["dart".to_string()],
        },
        _ => LanguageInfo {
            name: "Unknown".to_string(),
            icon: "📄".to_string(),
            color: "#6c757d".to_string(),
            extensions: vec![ext.to_string()],
        },
    }
}

/// Group extensions by language and aggregate their stats
pub fn group_extensions_by_language(stats_by_extension: &std::collections::HashMap<String, (usize, crate::core::types::FileStats)>) -> std::collections::HashMap<String, (LanguageInfo, usize, crate::core::types::FileStats)> {
    let mut language_stats: std::collections::HashMap<String, (LanguageInfo, usize, crate::core::types::FileStats)> = std::collections::HashMap::new();
    
    for (ext, (file_count, file_stats)) in stats_by_extension {
        let language_info = get_language_from_extension(ext);
        let language_name = language_info.name.clone();
        
        if let Some((existing_info, existing_count, existing_stats)) = language_stats.get_mut(&language_name) {
            // Merge stats for the same language
            *existing_count += file_count;
            existing_stats.total_lines += file_stats.total_lines;
            existing_stats.code_lines += file_stats.code_lines;
            existing_stats.comment_lines += file_stats.comment_lines;
            existing_stats.doc_lines += file_stats.doc_lines;
            existing_stats.blank_lines += file_stats.blank_lines;
            existing_stats.file_size += file_stats.file_size;
            
            // Update extensions list
            if !existing_info.extensions.contains(&ext.to_string()) {
                existing_info.extensions.push(ext.to_string());
            }
        } else {
            // First time seeing this language
            language_stats.insert(language_name, (language_info, *file_count, file_stats.clone()));
        }
    }
    
    language_stats
}

/// Create a centered rectangle for modal dialogs
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

/// Shorten a path to fit within a certain width
pub fn shorten_path(path: &str, max_width: usize) -> String {
    if path.len() <= max_width {
        path.to_string()
    } else {
        let parts: Vec<&str> = path.split('/').collect();
        if parts.len() <= 2 {
            format!("...{}", &path[path.len() - max_width + 3..])
        } else {
            let filename = parts.last().map_or("", |v| v);
            let first_part = parts.first().map_or("", |v| v);
            let remaining_width = max_width - 3 - filename.len() - first_part.len();
            
            if remaining_width > 0 {
                format!("{}/.../{}", first_part, filename)
            } else {
                format!(".../{}", filename)
            }
        }
    }
} 