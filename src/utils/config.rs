use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use crate::utils::errors::{HowManyError, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HowManyConfig {
    pub default_max_depth: Option<usize>,
    pub default_include_hidden: bool,
    pub default_ignore_gitignore: bool,
    pub custom_ignore_patterns: Vec<String>,
    pub language_extensions: HashMap<String, Vec<String>>,
    pub output_preferences: OutputPreferences,
    pub performance: PerformanceConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputPreferences {
    pub default_format: String,
    pub default_sort_by: String,
    pub show_progress: bool,
    pub use_colors: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    pub parallel_processing: bool,
    pub max_threads: Option<usize>,
    pub chunk_size: usize,
}

impl Default for HowManyConfig {
    fn default() -> Self {
        Self {
            default_max_depth: None,
            default_include_hidden: false,
            default_ignore_gitignore: false,
            custom_ignore_patterns: vec![
                "*.tmp".to_string(),
                "*.log".to_string(),
                ".DS_Store".to_string(),
                "node_modules/".to_string(),
                "__pycache__/".to_string(),
                "target/".to_string(),
            ],
            language_extensions: Self::default_language_extensions(),
            output_preferences: OutputPreferences::default(),
            performance: PerformanceConfig::default(),
        }
    }
}

impl Default for OutputPreferences {
    fn default() -> Self {
        Self {
            default_format: "interactive".to_string(),
            default_sort_by: "files".to_string(),
            show_progress: true,
            use_colors: true,
        }
    }
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            parallel_processing: true,
            max_threads: None, // Use system default
            chunk_size: 100,
        }
    }
}

impl HowManyConfig {
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;
        
        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            let config: HowManyConfig = toml::from_str(&content)
                .map_err(|e| HowManyError::invalid_config(format!("Failed to parse config: {}", e)))?;
            Ok(config)
        } else {
            Ok(Self::default())
        }
    }
    
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;
        
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        let content = toml::to_string_pretty(self)
            .map_err(|e| HowManyError::invalid_config(format!("Failed to serialize config: {}", e)))?;
        
        std::fs::write(&config_path, content)?;
        Ok(())
    }
    
    fn config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| HowManyError::invalid_config("Could not find config directory"))?;
        
        Ok(config_dir.join("howmany").join("config.toml"))
    }
    
    fn default_language_extensions() -> HashMap<String, Vec<String>> {
        let mut map = HashMap::new();
        
        map.insert("rust".to_string(), vec!["rs".to_string()]);
        map.insert("python".to_string(), vec!["py".to_string(), "pyw".to_string()]);
        map.insert("javascript".to_string(), vec!["js".to_string(), "mjs".to_string()]);
        map.insert("typescript".to_string(), vec!["ts".to_string(), "tsx".to_string()]);
        map.insert("java".to_string(), vec!["java".to_string()]);
        map.insert("c".to_string(), vec!["c".to_string(), "h".to_string()]);
        map.insert("cpp".to_string(), vec!["cpp".to_string(), "cc".to_string(), "cxx".to_string(), "hpp".to_string()]);
        map.insert("csharp".to_string(), vec!["cs".to_string()]);
        map.insert("go".to_string(), vec!["go".to_string()]);
        map.insert("ruby".to_string(), vec!["rb".to_string()]);
        map.insert("php".to_string(), vec!["php".to_string()]);
        map.insert("swift".to_string(), vec!["swift".to_string()]);
        map.insert("kotlin".to_string(), vec!["kt".to_string(), "kts".to_string()]);
        map.insert("scala".to_string(), vec!["scala".to_string()]);
        map.insert("html".to_string(), vec!["html".to_string(), "htm".to_string()]);
        map.insert("css".to_string(), vec!["css".to_string(), "scss".to_string(), "sass".to_string()]);
        map.insert("markdown".to_string(), vec!["md".to_string(), "markdown".to_string()]);
        map.insert("yaml".to_string(), vec!["yaml".to_string(), "yml".to_string()]);
        map.insert("json".to_string(), vec!["json".to_string()]);
        map.insert("toml".to_string(), vec!["toml".to_string()]);
        map.insert("xml".to_string(), vec!["xml".to_string()]);
        map.insert("haskell".to_string(), vec!["hs".to_string(), "lhs".to_string(), "hsc".to_string()]);
        map.insert("elixir".to_string(), vec!["ex".to_string(), "exs".to_string(), "eex".to_string()]);
        map.insert("erlang".to_string(), vec!["erl".to_string(), "hrl".to_string()]);
        map.insert("julia".to_string(), vec!["jl".to_string()]);
        map.insert("lua".to_string(), vec!["lua".to_string()]);
        map.insert("perl".to_string(), vec!["pl".to_string(), "pm".to_string(), "pod".to_string()]);
        map.insert("matlab".to_string(), vec!["m".to_string(), "mlx".to_string()]);
        map.insert("dart".to_string(), vec!["dart".to_string()]);
        map.insert("r".to_string(), vec!["r".to_string(), "R".to_string(), "rmd".to_string(), "Rmd".to_string()]);
        map.insert("zig".to_string(), vec!["zig".to_string()]);
        map.insert("clojure".to_string(), vec!["clj".to_string(), "cljs".to_string(), "cljc".to_string()]);
        map.insert("powershell".to_string(), vec!["ps1".to_string(), "psm1".to_string(), "psd1".to_string()]);
        map.insert("batch".to_string(), vec!["bat".to_string(), "cmd".to_string()]);
        map.insert("visualbasic".to_string(), vec!["vb".to_string(), "vbs".to_string()]);
        
        map
    }
} 