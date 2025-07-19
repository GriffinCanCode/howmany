use regex::Regex;

pub struct LuaPatterns {
    external_patterns: Vec<Regex>,
    cache_patterns: Vec<Regex>,
    extensions: Vec<String>,
}

impl LuaPatterns {
    pub fn new() -> Self {
        let external_patterns = vec![
            // LuaRocks artifacts
            Regex::new(r"\.luarocks/").unwrap(),
            Regex::new(r"luarocks\.lock").unwrap(),
            
            // Compiled Lua files
            Regex::new(r"\.luac$").unwrap(),
            
            // Love2D builds
            Regex::new(r"\.love$").unwrap(),
            
            // OpenResty/Nginx Lua cache
            Regex::new(r"nginx_tmp/").unwrap(),
            Regex::new(r"logs/").unwrap(),
            
            // Torch/PyTorch Lua artifacts
            Regex::new(r"\.t7$").unwrap(),
            
            // Wireshark Lua artifacts
            Regex::new(r"\.wslua$").unwrap(),
            
            // Redis Lua cache
            Regex::new(r"dump\.rdb").unwrap(),
            
            // Temporary files
            Regex::new(r"\.tmp/").unwrap(),
            Regex::new(r"tmp/").unwrap(),
            Regex::new(r"\.swp$").unwrap(),
            Regex::new(r"\.swo$").unwrap(),
            Regex::new(r"~$").unwrap(),
        ];

        let cache_patterns = vec![
            // LuaRocks cache
            Regex::new(r"\.luarocks/").unwrap(),
            
            // Compiled cache
            Regex::new(r"\.luac$").unwrap(),
            
            // Love2D cache
            Regex::new(r"\.love$").unwrap(),
            
            // Nginx cache
            Regex::new(r"nginx_tmp/").unwrap(),
        ];

        let extensions = vec![
            // Lua source files
            "lua".to_string(),
            
            // Compiled Lua
            "luac".to_string(),
            
            // Love2D files
            "love".to_string(),
            
            // Configuration files
            "json".to_string(),
            "yaml".to_string(),
            "yml".to_string(),
            "toml".to_string(),
            
            // Documentation
            "md".to_string(),
            "rst".to_string(),
            "txt".to_string(),
            
            // Web files (for OpenResty/Nginx)
            "html".to_string(),
            "css".to_string(),
            "js".to_string(),
            
            // Scripts
            "sh".to_string(),
            
            // C files (for Lua extensions)
            "c".to_string(),
            "h".to_string(),
        ];

        Self {
            external_patterns,
            cache_patterns,
            extensions,
        }
    }

    pub fn get_external_patterns(&self) -> &[Regex] {
        &self.external_patterns
    }

    pub fn get_cache_patterns(&self) -> &[Regex] {
        &self.cache_patterns
    }

    pub fn get_extensions(&self) -> &[String] {
        &self.extensions
    }

    pub fn get_script_names() -> Vec<&'static str> {
        vec![
            // LuaRocks files
            "rockspec", "*.rockspec",
            
            // Love2D files
            "main.lua", "conf.lua",
            
            // OpenResty/Nginx files
            "nginx.conf", "mime.types",
            
            // Build files
            "Makefile", "makefile", "build.sh",
            
            // Configuration files
            "config.lua", "settings.lua",
            
            // Documentation
            "README.md", "CHANGELOG.md", "LICENSE",
            
            // CI/CD
            ".github/workflows/lua.yml",
            ".travis.yml", "appveyor.yml",
            "circle.yml", ".circleci/config.yml",
            
            // Git
            ".gitignore", ".gitattributes",
            
            // Docker
            "Dockerfile", "docker-compose.yml",
            
            // Environment
            ".env", ".env.example",
            
            // Test files
            "test.lua", "tests.lua", "spec.lua",
            
            // Init files
            "init.lua", "main.lua",
            
            // Common module files
            "lib.lua", "utils.lua", "helpers.lua",
            
            // Game development (Love2D)
            "game.lua", "player.lua", "enemy.lua",
            
            // Web development (OpenResty)
            "app.lua", "server.lua", "handler.lua",
            
            // Wireshark
            "dissector.lua", "plugin.lua",
            
            // Redis
            "script.lua", "redis.lua",
        ]
    }
} 