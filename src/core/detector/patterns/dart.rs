use regex::Regex;

pub struct DartPatterns {
    external_patterns: Vec<Regex>,
    cache_patterns: Vec<Regex>,
    extensions: Vec<String>,
}

impl DartPatterns {
    pub fn new() -> Self {
        let external_patterns = vec![
            // Dart/Flutter build artifacts
            Regex::new(r"\.dart_tool/").unwrap(),
            Regex::new(r"build/").unwrap(),
            Regex::new(r"\.packages").unwrap(),
            Regex::new(r"pubspec\.lock").unwrap(),
            
            // Flutter specific
            Regex::new(r"\.flutter-plugins").unwrap(),
            Regex::new(r"\.flutter-plugins-dependencies").unwrap(),
            Regex::new(r"\.fvm/").unwrap(),
            
            // Generated files
            Regex::new(r"\.g\.dart$").unwrap(),
            Regex::new(r"\.freezed\.dart$").unwrap(),
            Regex::new(r"\.gr\.dart$").unwrap(),
            Regex::new(r"\.config\.dart$").unwrap(),
            Regex::new(r"\.part\.dart$").unwrap(),
            
            // iOS build artifacts
            Regex::new(r"ios/\.symlinks/").unwrap(),
            Regex::new(r"ios/Flutter/").unwrap(),
            Regex::new(r"ios/Pods/").unwrap(),
            Regex::new(r"ios/Podfile\.lock").unwrap(),
            
            // Android build artifacts
            Regex::new(r"android/\.gradle/").unwrap(),
            Regex::new(r"android/gradle/").unwrap(),
            Regex::new(r"android/build/").unwrap(),
            Regex::new(r"android/app/build/").unwrap(),
            Regex::new(r"android/local\.properties").unwrap(),
            
            // Web build artifacts
            Regex::new(r"web/main\.dart\.js").unwrap(),
            Regex::new(r"web/main\.dart\.js\.map").unwrap(),
            
            // Coverage
            Regex::new(r"coverage/").unwrap(),
            Regex::new(r"\.lcov$").unwrap(),
            
            // Documentation
            Regex::new(r"doc/api/").unwrap(),
            
            // Temporary files
            Regex::new(r"\.tmp/").unwrap(),
            Regex::new(r"tmp/").unwrap(),
            Regex::new(r"\.swp$").unwrap(),
            Regex::new(r"\.swo$").unwrap(),
            Regex::new(r"~$").unwrap(),
        ];

        let cache_patterns = vec![
            // Dart cache
            Regex::new(r"\.dart_tool/").unwrap(),
            
            // Build cache
            Regex::new(r"build/").unwrap(),
            
            // Flutter cache
            Regex::new(r"\.flutter-plugins").unwrap(),
            
            // Coverage cache
            Regex::new(r"coverage/").unwrap(),
        ];

        let extensions = vec![
            // Dart source files
            "dart".to_string(),
            
            // Configuration files
            "yaml".to_string(),
            "yml".to_string(),
            "json".to_string(),
            
            // Documentation
            "md".to_string(),
            "rst".to_string(),
            
            // Web files (for Flutter web)
            "html".to_string(),
            "css".to_string(),
            "js".to_string(),
            
            // Native files
            "swift".to_string(),
            "kt".to_string(),
            "java".to_string(),
            "m".to_string(),
            "h".to_string(),
            
            // Build files
            "gradle".to_string(),
            "xml".to_string(),
            "plist".to_string(),
            
            // Shell scripts
            "sh".to_string(),
            "bat".to_string(),
            "ps1".to_string(),
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
            // Dart/Flutter files
            "pubspec.yaml", "pubspec.lock", "analysis_options.yaml",
            ".packages", "dart_dependencies.yaml",
            
            // Flutter specific
            ".flutter-plugins", ".flutter-plugins-dependencies",
            ".metadata", "flutter_export_environment.sh",
            
            // Build files
            "Makefile", "GNUmakefile", "makefile",
            "build.yaml", "build.gradle",
            
            // iOS files
            "Podfile", "Podfile.lock", "Info.plist",
            "project.pbxproj", "contents.xcworkspacedata",
            
            // Android files
            "build.gradle", "gradle.properties",
            "local.properties", "proguard-rules.pro",
            "AndroidManifest.xml",
            
            // Web files
            "index.html", "manifest.json",
            
            // CI/CD
            ".github/workflows/dart.yml",
            ".github/workflows/flutter.yml",
            ".travis.yml", "appveyor.yml",
            "codemagic.yaml",
            
            // Docker
            "Dockerfile", "docker-compose.yml",
            
            // Editor
            ".editorconfig", ".gitignore",
            
            // Scripts
            "run.sh", "build.sh", "deploy.sh",
        ]
    }

    pub fn get_common_directories() -> Vec<&'static str> {
        vec![
            // Source
            "lib/", "bin/", "web/",
            
            // Tests
            "test/", "integration_test/",
            
            // Documentation
            "doc/", "docs/", "example/",
            
            // Build
            "build/", ".dart_tool/",
            
            // Platform specific
            "android/", "ios/", "linux/", "macos/", "windows/",
            
            // Assets
            "assets/", "fonts/", "images/",
            
            // Tools
            "tool/", "scripts/",
            
            // Configuration
            "config/", ".vscode/", ".idea/",
        ]
    }
} 