use regex::Regex;
use std::collections::HashMap;
use lazy_static::lazy_static;

lazy_static! {
    /// Lazily compiled OS patterns - compiled once and reused
    static ref OS_PATTERNS: Vec<Regex> = vec![
        Regex::new(r"\.DS_Store").unwrap(),
        Regex::new(r"Thumbs\.db").unwrap(),
        Regex::new(r"desktop\.ini").unwrap(),
        Regex::new(r"\.directory").unwrap(),
        Regex::new(r"\.Spotlight-V100").unwrap(),
        Regex::new(r"\.Trashes").unwrap(),
        Regex::new(r"\.VolumeIcon\.icns").unwrap(),
        Regex::new(r"\.AppleDouble").unwrap(),
        Regex::new(r"\.LSOverride").unwrap(),
        Regex::new(r"\.DocumentRevisions-V100").unwrap(),
        Regex::new(r"\.fseventsd").unwrap(),
        Regex::new(r"\.TemporaryItems").unwrap(),
        Regex::new(r"\.com\.apple\.timemachine\.donotpresent").unwrap(),
        Regex::new(r"\.AppleDB").unwrap(),
        Regex::new(r"\.AppleDesktop").unwrap(),
        Regex::new(r"Network Trash Folder").unwrap(),
        Regex::new(r"Temporary Items").unwrap(),
        Regex::new(r"\.apdisk").unwrap(),
        Regex::new(r"ehthumbs\.db").unwrap(),
        Regex::new(r"\._ ").unwrap(),
    ];

    static ref IDE_PATTERNS: Vec<Regex> = vec![
        Regex::new(r"\.vscode/").unwrap(),
        Regex::new(r"\.idea/").unwrap(),
        Regex::new(r"\.vs/").unwrap(),
        Regex::new(r"\.sublime-").unwrap(),
        Regex::new(r"\.atom/").unwrap(),
        Regex::new(r"\.eclipse/").unwrap(),
        Regex::new(r"\.metadata/").unwrap(),
        Regex::new(r"\.settings/").unwrap(),
        Regex::new(r"\.spyproject/").unwrap(),
        Regex::new(r"\.nova/").unwrap(),
        Regex::new(r"\.zed/").unwrap(),
        Regex::new(r"\.brackets\.json").unwrap(),
        Regex::new(r"\.emacs\.d/").unwrap(),
        Regex::new(r"\.vim/").unwrap(),
        Regex::new(r"nbproject/").unwrap(),
        Regex::new(r"\.buildpath").unwrap(),
        Regex::new(r"\.project").unwrap(),
        Regex::new(r"\.classpath").unwrap(),
        Regex::new(r"\.aptana/").unwrap(),
        Regex::new(r"\.phpstorm\.meta\.php").unwrap(),
    ];

    static ref TEMP_PATTERNS: Vec<Regex> = vec![
        Regex::new(r"\.tmp$").unwrap(),
        Regex::new(r"\.temp$").unwrap(),
        Regex::new(r"\.swp$").unwrap(),
        Regex::new(r"\.swo$").unwrap(),
        Regex::new(r"~$").unwrap(),
        Regex::new(r"\.bak$").unwrap(),
        Regex::new(r"\.backup$").unwrap(),
        Regex::new(r"\.orig$").unwrap(),
        Regex::new(r"\.rej$").unwrap(),
        Regex::new(r"\.cache$").unwrap(),
        Regex::new(r"\.log$").unwrap(),
        Regex::new(r"\.out$").unwrap(),
        Regex::new(r"\.err$").unwrap(),
        Regex::new(r"\.pid$").unwrap(),
        Regex::new(r"\.lock$").unwrap(),
        Regex::new(r"\.lockb$").unwrap(),
        Regex::new(r"\.moved-aside").unwrap(),
        Regex::new(r"\._ ").unwrap(),
    ];

    static ref VCS_PATTERNS: Vec<Regex> = vec![
        Regex::new(r"\.git/").unwrap(),
        Regex::new(r"\.svn/").unwrap(),
        Regex::new(r"\.hg/").unwrap(),
        Regex::new(r"\.bzr/").unwrap(),
        Regex::new(r"\.gitignore").unwrap(),
        Regex::new(r"\.gitattributes").unwrap(),
        Regex::new(r"\.gitmodules").unwrap(),
        Regex::new(r"\.gitkeep").unwrap(),
    ];

    /// Language-specific build patterns compiled once
    static ref LANGUAGE_BUILD_PATTERNS: HashMap<String, Vec<Regex>> = {
        let mut patterns = HashMap::new();

        // Node.js
        patterns.insert("nodejs".to_string(), vec![
            Regex::new(r"node_modules/").unwrap(),
            Regex::new(r"\.npm/").unwrap(),
            Regex::new(r"\.yarn/").unwrap(),
            Regex::new(r"\.pnpm-store/").unwrap(),
            Regex::new(r"\.bun/").unwrap(),
            Regex::new(r"\.next/").unwrap(),
            Regex::new(r"\.nuxt/").unwrap(),
            Regex::new(r"\.output/").unwrap(),
            Regex::new(r"\.svelte-kit/").unwrap(),
            Regex::new(r"\.astro/").unwrap(),
            Regex::new(r"\.remix/").unwrap(),
            Regex::new(r"\.vercel/").unwrap(),
            Regex::new(r"\.netlify/").unwrap(),
            Regex::new(r"\.firebase/").unwrap(),
            Regex::new(r"\.parcel-cache/").unwrap(),
            Regex::new(r"\.turbo/").unwrap(),
            Regex::new(r"\.webpack/").unwrap(),
            Regex::new(r"\.rollup\.cache/").unwrap(),
            Regex::new(r"\.vite/").unwrap(),
            Regex::new(r"\.swc/").unwrap(),
            Regex::new(r"\.esbuild/").unwrap(),
        ]);

        // Python
        patterns.insert("python".to_string(), vec![
            Regex::new(r"__pycache__/").unwrap(),
            Regex::new(r"\.pytest_cache/").unwrap(),
            Regex::new(r"\.tox/").unwrap(),
            Regex::new(r"\.nox/").unwrap(),
            Regex::new(r"\.coverage").unwrap(),
            Regex::new(r"htmlcov/").unwrap(),
            Regex::new(r"\.mypy_cache/").unwrap(),
            Regex::new(r"\.pytype/").unwrap(),
            Regex::new(r"\.pyre/").unwrap(),
            Regex::new(r"\.ruff_cache/").unwrap(),
            Regex::new(r"\.ipynb_checkpoints/").unwrap(),
            Regex::new(r"\.eggs/").unwrap(),
            Regex::new(r"\.pip/").unwrap(),
            Regex::new(r"\.venv/").unwrap(),
            Regex::new(r"venv/").unwrap(),
            Regex::new(r"env/").unwrap(),
            Regex::new(r"\.env/").unwrap(),
            Regex::new(r"virtualenv/").unwrap(),
        ]);

        // Rust
        patterns.insert("rust".to_string(), vec![
            Regex::new(r"target/").unwrap(),
            Regex::new(r"\.cargo/").unwrap(),
            Regex::new(r"\.rustup/").unwrap(),
        ]);

        // Java
        patterns.insert("java".to_string(), vec![
            Regex::new(r"target/").unwrap(),
            Regex::new(r"build/").unwrap(),
            Regex::new(r"\.gradle/").unwrap(),
            Regex::new(r"\.m2/").unwrap(),
            Regex::new(r"\.mvn/").unwrap(),
            Regex::new(r"\.sbt/").unwrap(),
            Regex::new(r"\.ivy2/").unwrap(),
        ]);

        // Go
        patterns.insert("go".to_string(), vec![
            Regex::new(r"vendor/").unwrap(),
            Regex::new(r"\.go/pkg/").unwrap(),
        ]);

        // C/C++
        patterns.insert("cpp".to_string(), vec![
            Regex::new(r"\.ccache/").unwrap(),
            Regex::new(r"\.sccache/").unwrap(),
            Regex::new(r"build/").unwrap(),
            Regex::new(r"cmake-build-").unwrap(),
            Regex::new(r"CMakeFiles/").unwrap(),
        ]);

        // .NET
        patterns.insert("dotnet".to_string(), vec![
            Regex::new(r"bin/").unwrap(),
            Regex::new(r"obj/").unwrap(),
            Regex::new(r"packages/").unwrap(),
            Regex::new(r"\.nuget/").unwrap(),
            Regex::new(r"TestResults/").unwrap(),
            Regex::new(r"publish/").unwrap(),
            Regex::new(r"\.publish/").unwrap(),
        ]);

        // PHP
        patterns.insert("php".to_string(), vec![
            Regex::new(r"vendor/").unwrap(),
            Regex::new(r"bootstrap/cache/").unwrap(),
            Regex::new(r"storage/framework/").unwrap(),
            Regex::new(r"storage/logs/").unwrap(),
            Regex::new(r"var/cache/").unwrap(),
            Regex::new(r"var/logs/").unwrap(),
            Regex::new(r"tmp/cache/").unwrap(),
            Regex::new(r"application/cache/").unwrap(),
        ]);

        // Ruby
        patterns.insert("ruby".to_string(), vec![
            Regex::new(r"\.bundle/").unwrap(),
            Regex::new(r"vendor/bundle/").unwrap(),
            Regex::new(r"\.gem/").unwrap(),
            Regex::new(r"log/").unwrap(),
            Regex::new(r"tmp/").unwrap(),
            Regex::new(r"coverage/").unwrap(),
            Regex::new(r"\.yardoc/").unwrap(),
            Regex::new(r"\.sass-cache/").unwrap(),
            Regex::new(r"\.spring/").unwrap(),
        ]);

        // Swift
        patterns.insert("swift".to_string(), vec![
            Regex::new(r"\.build/").unwrap(),
            Regex::new(r"\.swiftpm/").unwrap(),
            Regex::new(r"DerivedData/").unwrap(),
            Regex::new(r"Pods/").unwrap(),
            Regex::new(r"Carthage/").unwrap(),
            Regex::new(r"\.xcodeproj/").unwrap(),
            Regex::new(r"\.xcworkspace/").unwrap(),
            Regex::new(r"\.xcarchive/").unwrap(),
        ]);

        patterns
    };

    /// Binary file extensions - compiled once
    static ref BINARY_EXTENSIONS: Vec<String> = vec![
        // Executables
        "exe".to_string(), "dll".to_string(), "so".to_string(), "dylib".to_string(),
        "a".to_string(), "lib".to_string(), "o".to_string(), "obj".to_string(),
        "bin".to_string(), "dat".to_string(), "rlib".to_string(), "pdb".to_string(),
        // Archives
        "zip".to_string(), "tar".to_string(), "gz".to_string(), "bz2".to_string(),
        "rar".to_string(), "7z".to_string(), "dmg".to_string(), "iso".to_string(),
        // Images
        "jpg".to_string(), "jpeg".to_string(), "png".to_string(), "gif".to_string(),
        "bmp".to_string(), "tiff".to_string(), "ico".to_string(), "svg".to_string(),
        "webp".to_string(), "heic".to_string(), "heif".to_string(),
        // Audio/Video
        "mp3".to_string(), "mp4".to_string(), "avi".to_string(), "mov".to_string(),
        "wmv".to_string(), "flv".to_string(), "wav".to_string(), "flac".to_string(),
        "m4a".to_string(), "m4v".to_string(), "aiff".to_string(),
        // Fonts
        "ttf".to_string(), "otf".to_string(), "woff".to_string(), "woff2".to_string(),
        "eot".to_string(),
        // Documents
        "pdf".to_string(), "doc".to_string(), "docx".to_string(), "xls".to_string(),
        "xlsx".to_string(), "ppt".to_string(), "pptx".to_string(),
        // Databases
        "db".to_string(), "sqlite".to_string(), "sqlite3".to_string(),
        // Compiled code
        "class".to_string(), "jar".to_string(), "war".to_string(), "ear".to_string(),
        "pyc".to_string(), "pyo".to_string(), "pyd".to_string(),
        // Mobile
        "apk".to_string(), "ipa".to_string(), "aab".to_string(), "dex".to_string(),
        // Package files
        "whl".to_string(), "egg".to_string(), "gem".to_string(), "nupkg".to_string(),
        "snupkg".to_string(), "phar".to_string(),
    ];

    /// Generated file indicators - compiled once
    static ref GENERATED_INDICATORS: Vec<String> = vec![
        "generated".to_string(), "auto".to_string(), "autogen".to_string(),
        "codegen".to_string(), "_gen".to_string(), ".gen".to_string(),
        "build".to_string(), "dist".to_string(), "out".to_string(),
        "output".to_string(), "bin".to_string(), "obj".to_string(),
        "bundle".to_string(), "minified".to_string(), ".min.".to_string(),
        "compiled".to_string(), "protobuf".to_string(), ".pb.".to_string(),
        "thrift".to_string(), ".thrift.".to_string(), "swagger".to_string(),
        "openapi".to_string(), "schema".to_string(), "_generated".to_string(),
        "bindata".to_string(), ".pb.gw.".to_string(),
    ];
}

/// Common patterns shared between detector and filters
pub struct CommonPatterns {
    /// Binary file extensions
    pub binary_extensions: Vec<String>,
    /// Generated file indicators
    pub generated_indicators: Vec<String>,
}

impl CommonPatterns {
    pub fn new() -> Self {
        Self {
            binary_extensions: BINARY_EXTENSIONS.clone(),
            generated_indicators: GENERATED_INDICATORS.clone(),
        }
    }

    /// Check if a path matches any OS-specific patterns
    pub fn matches_os_pattern(&self, path_str: &str) -> bool {
        OS_PATTERNS.iter().any(|pattern| pattern.is_match(path_str))
    }

    /// Check if a path matches any IDE patterns
    pub fn matches_ide_pattern(&self, path_str: &str) -> bool {
        IDE_PATTERNS.iter().any(|pattern| pattern.is_match(path_str))
    }

    /// Check if a path matches any temporary file patterns
    pub fn matches_temp_pattern(&self, path_str: &str) -> bool {
        TEMP_PATTERNS.iter().any(|pattern| pattern.is_match(path_str))
    }

    /// Check if a path matches any VCS patterns
    pub fn matches_vcs_pattern(&self, path_str: &str) -> bool {
        VCS_PATTERNS.iter().any(|pattern| pattern.is_match(path_str))
    }

    /// Check if a file extension indicates a binary file
    pub fn is_binary_extension(&self, extension: &str) -> bool {
        BINARY_EXTENSIONS.contains(&extension.to_lowercase())
    }

    /// Check if a filename indicates a generated file
    pub fn is_generated_file(&self, filename: &str) -> bool {
        let filename_lower = filename.to_lowercase();
        GENERATED_INDICATORS.iter().any(|indicator| filename_lower.contains(indicator))
    }

    /// Get all patterns that should be ignored (combines OS, IDE, temp, VCS)
    pub fn get_ignore_patterns(&self) -> Vec<&Regex> {
        let mut patterns = Vec::new();
        patterns.extend(OS_PATTERNS.iter());
        patterns.extend(IDE_PATTERNS.iter());
        patterns.extend(TEMP_PATTERNS.iter());
        patterns.extend(VCS_PATTERNS.iter());
        patterns
    }

    /// Check if a path should be ignored based on common patterns
    pub fn should_ignore(&self, path_str: &str) -> bool {
        self.matches_os_pattern(path_str) ||
        self.matches_ide_pattern(path_str) ||
        self.matches_temp_pattern(path_str) ||
        self.matches_vcs_pattern(path_str)
    }
}

/// Language-specific build and cache patterns
pub struct LanguageBuildPatterns;

impl LanguageBuildPatterns {
    pub fn new() -> Self {
        Self
    }

    /// Check if a path matches language-specific build patterns
    pub fn matches_build_pattern(&self, path_str: &str) -> bool {
        LANGUAGE_BUILD_PATTERNS.values().any(|patterns| {
            patterns.iter().any(|pattern| pattern.is_match(path_str))
        })
    }

    /// Get patterns for a specific language
    pub fn get_language_patterns(&self, language: &str) -> Option<&Vec<Regex>> {
        LANGUAGE_BUILD_PATTERNS.get(language)
    }
}

/// Centralized pattern matcher that combines common and language-specific patterns
pub struct PatternMatcher {
    common: CommonPatterns,
    language_build: LanguageBuildPatterns,
}

impl PatternMatcher {
    pub fn new() -> Self {
        Self {
            common: CommonPatterns::new(),
            language_build: LanguageBuildPatterns::new(),
        }
    }

    /// Check if a file should be completely ignored (OS, IDE, temp, VCS files)
    pub fn should_ignore_file(&self, path_str: &str) -> bool {
        self.common.should_ignore(path_str)
    }

    /// Check if a file is a binary file based on extension
    pub fn is_binary_file(&self, extension: &str) -> bool {
        self.common.is_binary_extension(extension)
    }

    /// Check if a file is generated based on filename
    pub fn is_generated_file(&self, filename: &str) -> bool {
        self.common.is_generated_file(filename)
    }

    /// Check if a path matches build/cache patterns
    pub fn matches_build_cache_pattern(&self, path_str: &str) -> bool {
        self.language_build.matches_build_pattern(path_str)
    }

    /// Get reference to common patterns
    pub fn common_patterns(&self) -> &CommonPatterns {
        &self.common
    }

    /// Get reference to language build patterns
    pub fn language_patterns(&self) -> &LanguageBuildPatterns {
        &self.language_build
    }
}

impl Default for PatternMatcher {
    fn default() -> Self {
        Self::new()
    }
} 