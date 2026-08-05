use crate::core::languages::{self, Category};
use ratatui::layout::{Constraint, Direction, Layout, Rect};

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
    /// What the language is for: source, configuration, data, or prose.
    pub category: Category,
    pub extensions: Vec<String>,
}

/// Map file extension to programming language using SherlockIO data when available
pub fn get_language_from_extension(ext: &str) -> LanguageInfo {
    get_language_from_extension_with_sherlock(ext, None)
}

/// Map file extension to programming language with optional SherlockIO data
pub fn get_language_from_extension_with_sherlock(
    ext: &str,
    sherlock_result: Option<&crate::core::detector::SherlockResult>,
) -> LanguageInfo {
    // First try to get info from SherlockIO if available
    if let Some(sherlock) = sherlock_result {
        for language in &sherlock.languages {
            for file in &language.files {
                if let Some(file_ext) = std::path::Path::new(file).extension() {
                    if file_ext.to_string_lossy().to_lowercase() == ext.to_lowercase() {
                        return LanguageInfo {
                            name: language.name.clone(),
                            icon: get_language_icon(&language.name),
                            color: language.color.clone(),
                            category: languages::describe(ext).1,
                            extensions: vec![ext.to_string()],
                        };
                    }
                }
            }
        }
    }

    // Fallback to hardcoded mapping
    get_language_from_extension_fallback(ext)
}

/// Get language icon based on language name
fn get_language_icon(language_name: &str) -> String {
    match language_name.to_lowercase().as_str() {
        "rust" => "🦀".to_string(),
        "python" => "🐍".to_string(),
        "javascript" => "📜".to_string(),
        "typescript" => "📘".to_string(),
        "java" => "☕".to_string(),
        "c++" => "⚡".to_string(),
        "c" => "🔧".to_string(),
        "go" => "🐹".to_string(),
        "ruby" => "💎".to_string(),
        "php" => "🐘".to_string(),
        "c#" => "🔷".to_string(),
        "swift" => "🍎".to_string(),
        "kotlin" => "🎯".to_string(),
        "html" => "🌐".to_string(),
        "css" => "🎨".to_string(),
        "sass" | "scss" => "🎨".to_string(),
        "json" => "📋".to_string(),
        "yaml" => "⚙️".to_string(),
        "toml" => "🔧".to_string(),
        "markdown" => "📝".to_string(),
        "shell" => "🐚".to_string(),
        _ => "📄".to_string(),
    }
}

/// Language identity for `ext` when Sherlock is not available.
///
/// The name comes from [`crate::core::languages`], the same registry the text
/// report reads. This used to be a second table of its own, and everything it
/// had not been taught -- Zig, Lua, Protocol Buffers, Rego, every schema and
/// template format -- collapsed into one "Unknown" row, which then merged into
/// a single bucket large enough to outrank the languages it was hiding.
fn get_language_from_extension_fallback(ext: &str) -> LanguageInfo {
    let (name, category) = languages::describe(ext);
    let (icon, color) = decoration(&name, category);
    LanguageInfo {
        name,
        icon: icon.to_string(),
        color: color.to_string(),
        category,
        extensions: vec![ext.to_string()],
    }
}
/// An icon and a hex colour for a language, by name.
///
/// Only the decoration is decided here. Anything this table has not been
/// taught still gets a sensible look from its category, so an unfamiliar
/// format stays legible instead of turning grey and nameless.
fn decoration(name: &str, category: Category) -> (&'static str, &'static str) {
    match name {
        "Rust" => ("🦀", "#dea584"),
        "Python" | "Python Stubs" => ("🐍", "#3776ab"),
        "JavaScript" => ("📜", "#f7df1e"),
        "TypeScript" => ("📘", "#3178c6"),
        "TypeScript (React)" | "JavaScript (React)" => ("⚛️", "#61dafb"),
        "HTML" => ("🌐", "#e34f26"),
        "CSS" | "Textual CSS" => ("🎨", "#1572b6"),
        "Sass" | "Less" => ("🎨", "#cf649a"),
        "Java" => ("☕", "#ed8b00"),
        "C" | "C++" => ("⚡", "#00599c"),
        "C/C++ Header" => ("📎", "#00599c"),
        "Go" => ("🐹", "#00add8"),
        "Go Template" => ("🐹", "#3d8fb0"),
        "PHP" => ("🐘", "#777bb4"),
        "Ruby" => ("💎", "#cc342d"),
        "Swift" => ("🍎", "#fa7343"),
        "Kotlin" => ("🎯", "#7f52ff"),
        "Dart" => ("🎯", "#0175c2"),
        "Scala" => ("🎭", "#dc322f"),
        "C#" => ("🔷", "#239120"),
        "Visual Basic" => ("🔷", "#945db7"),
        "Shell" => ("🐚", "#89e051"),
        "JSON" | "JSON Lines" | "JSON5" => ("📋", "#7a7a7a"),
        "XML" => ("📄", "#e37933"),
        "YAML" => ("⚙️", "#cb171e"),
        "TOML" => ("🔧", "#9c4221"),
        "Markdown" | "MDX" => ("📝", "#083fa1"),
        "SQL" => ("🗃️", "#e38c00"),
        "Protocol Buffers" | "Cap'n Proto" => ("🔌", "#4a90d9"),
        "R" | "R Markdown" => ("📊", "#198ce7"),
        "MATLAB" => ("📊", "#e16737"),
        "Haskell" => ("λ", "#5e5086"),
        "Scheme" | "Racket" | "Lisp" => ("λ", "#22228f"),
        "Elixir" => ("💧", "#6e4a7e"),
        "Erlang" => ("📞", "#b83998"),
        "Julia" => ("🔬", "#9558b2"),
        "Lua" => ("🌙", "#000080"),
        "Perl" => ("🐪", "#0298c3"),
        "Zig" => ("⚡", "#ec915c"),
        "PowerShell" => ("⚡", "#012456"),
        "Clojure" => ("🔄", "#db5855"),
        "Batch" => ("⚙️", "#c1f12e"),
        "Dockerfile" => ("🐳", "#2496ed"),
        "Makefile" => ("🔨", "#427819"),
        "Jinja" => ("🧩", "#b41717"),
        "Rego" | "Cedar" => ("🛡️", "#7d5bbe"),
        _ => match category {
            Category::Code => ("📄", "#8fb8de"),
            Category::Config => ("⚙️", "#9c4221"),
            Category::Data => ("📋", "#7a7a7a"),
            Category::Docs => ("📝", "#083fa1"),
        },
    }
}

/// Group extensions by language and aggregate their stats
pub fn group_extensions_by_language(
    stats_by_extension: &std::collections::BTreeMap<String, (usize, crate::core::types::FileStats)>,
) -> std::collections::BTreeMap<String, (LanguageInfo, usize, crate::core::types::FileStats)> {
    let mut language_stats: std::collections::BTreeMap<
        String,
        (LanguageInfo, usize, crate::core::types::FileStats),
    > = std::collections::BTreeMap::new();

    for (ext, (file_count, file_stats)) in stats_by_extension {
        let language_info = get_language_from_extension(ext);
        let language_name = language_info.name.clone();

        if let Some((existing_info, existing_count, existing_stats)) =
            language_stats.get_mut(&language_name)
        {
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
            language_stats.insert(
                language_name,
                (language_info, *file_count, file_stats.clone()),
            );
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
