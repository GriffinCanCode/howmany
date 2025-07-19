use crate::core::types::FileStats;

pub struct FileUtils;

impl FileUtils {
    pub fn new() -> Self {
        Self
    }
    
    pub fn format_size(&self, size: u64) -> String {
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
    
    pub fn get_file_emoji(&self, ext: &str) -> &str {
        match ext {
            "js" | "jsx" => "🟨",
            "ts" | "tsx" => "🔷",
            "py" => "🐍",
            "rs" => "🦀",
            "java" => "☕",
            "css" | "scss" => "🎨",
            "html" => "🌐",
            "md" => "📝",
            "json" => "📋",
            "xml" => "📄",
            "go" => "🐹",
            "cpp" | "c" => "⚙️",
            "php" => "🐘",
            "rb" => "💎",
            "swift" => "🍎",
            "kt" => "🎯",
            _ => "📄",
        }
    }
    
    pub fn get_regret_level(&self, file_stats: &FileStats) -> (String, String) {
        let total_lines = file_stats.total_lines;
        match total_lines {
            0..=100 => ("Mild regret".to_string(), "waste-low".to_string()),
            101..=500 => ("Moderate regret".to_string(), "waste-medium".to_string()),
            501..=2000 => ("High regret".to_string(), "waste-high".to_string()),
            _ => ("Existential crisis".to_string(), "waste-extreme".to_string()),
        }
    }
    
    pub fn get_alternative_activity(&self, ext: &str, file_stats: &FileStats) -> String {
        let total_minutes = (file_stats.code_lines as f64 * 0.2) as usize + 
                           (file_stats.doc_lines as f64 * 0.5) as usize + 
                           (file_stats.comment_lines as f64 * 0.1) as usize;
        
        match ext {
            "js" | "jsx" => {
                if total_minutes > 480 {
                    "Learned React properly"
                } else {
                    "Fixed that CSS bug"
                }
            },
            "ts" | "tsx" => {
                if total_minutes > 600 {
                    "Understood generics"
                } else {
                    "Configured TypeScript"
                }
            },
            "py" => {
                if total_minutes > 300 {
                    "Mastered list comprehensions"
                } else {
                    "Understood decorators"
                }
            },
            "rs" => {
                if total_minutes > 800 {
                    "Befriended the borrow checker"
                } else {
                    "Learned lifetimes"
                }
            },
            "java" => {
                if total_minutes > 400 {
                    "Understood Spring Boot"
                } else {
                    "Configured Maven"
                }
            },
            "css" | "scss" => {
                if total_minutes > 200 {
                    "Learned Flexbox properly"
                } else {
                    "Centered a div"
                }
            },
            "html" => "Learned semantic HTML",
            "md" => "Actually read the docs",
            "json" => "Learned YAML instead",
            "xml" => "Questioned life choices",
            _ => {
                if total_minutes > 300 {
                    "Learned a new language"
                } else {
                    "Taken a coffee break"
                }
            }
        }.to_string()
    }
    
    pub fn calculate_productivity_score(&self, file_stats: &FileStats) -> String {
        let code_ratio = if file_stats.total_lines > 0 {
            (file_stats.code_lines as f64 / file_stats.total_lines as f64) * 100.0
        } else {
            0.0
        };
        
        match code_ratio {
            80.0..=100.0 => "🔥 Code machine".to_string(),
            60.0..=79.9 => "💪 Productive".to_string(),
            40.0..=59.9 => "📝 Documenter".to_string(),
            20.0..=39.9 => "💬 Commenter".to_string(),
            _ => "🌬️ Air enthusiast".to_string(),
        }
    }
} 