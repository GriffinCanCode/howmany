use regex::Regex;

pub struct KotlinPatterns {
    external_patterns: Vec<Regex>,
    cache_patterns: Vec<Regex>,
    extensions: Vec<String>,
}

impl KotlinPatterns {
    pub fn new() -> Self {
        let external_patterns = vec![
            // Compiled bytecode (shared with Java)
            Regex::new(r"\.class$").unwrap(),
            Regex::new(r"\.jar$").unwrap(),
            Regex::new(r"\.war$").unwrap(),
            Regex::new(r"\.aar$").unwrap(),
            
            // Kotlin-specific compiled files
            Regex::new(r"\.kotlin_module$").unwrap(),
            Regex::new(r"\.kotlin_metadata$").unwrap(),
            
            // Build directories
            Regex::new(r"build/").unwrap(),
            Regex::new(r"target/").unwrap(),
            Regex::new(r"out/").unwrap(),
            Regex::new(r"bin/").unwrap(),
            Regex::new(r"classes/").unwrap(),
            
            // Gradle (primary build tool for Kotlin)
            Regex::new(r"\.gradle/").unwrap(),
            Regex::new(r"gradle-app\.setting").unwrap(),
            Regex::new(r"gradlew").unwrap(),
            Regex::new(r"gradlew\.bat").unwrap(),
            Regex::new(r"gradle\.properties").unwrap(),
            
            // Maven (also used with Kotlin)
            Regex::new(r"\.m2/").unwrap(),
            Regex::new(r"\.mvn/").unwrap(),
            
            // Kotlin/Native
            Regex::new(r"\.klib$").unwrap(),
            Regex::new(r"\.knm$").unwrap(),
            
            // Kotlin Multiplatform
            Regex::new(r"\.kmp/").unwrap(),
            
            // Android specific (Kotlin is primary for Android)
            Regex::new(r"\.apk$").unwrap(),
            Regex::new(r"\.aab$").unwrap(),
            Regex::new(r"\.dex$").unwrap(),
            Regex::new(r"\.ap_$").unwrap(),
            
            // IDE generated files
            Regex::new(r"\.idea/").unwrap(),
            Regex::new(r"\.eclipse/").unwrap(),
            Regex::new(r"\.metadata/").unwrap(),
            Regex::new(r"\.settings/").unwrap(),
            Regex::new(r"\.project$").unwrap(),
            Regex::new(r"\.classpath$").unwrap(),
            
            // IntelliJ IDEA specific
            Regex::new(r"\.iml$").unwrap(),
            Regex::new(r"\.ipr$").unwrap(),
            Regex::new(r"\.iws$").unwrap(),
            
            // Android Studio
            Regex::new(r"local\.properties").unwrap(),
            Regex::new(r"\.externalNativeBuild/").unwrap(),
            Regex::new(r"\.cxx/").unwrap(),
            
            // Kotlin compiler cache
            Regex::new(r"\.kotlin/").unwrap(),
            
            // Documentation
            Regex::new(r"dokka/").unwrap(),
            
            // Testing
            Regex::new(r"\.kover/").unwrap(),
            
            // Temporary files
            Regex::new(r"\.tmp/").unwrap(),
            Regex::new(r"tmp/").unwrap(),
            
            // Logs
            Regex::new(r"\.log$").unwrap(),
            Regex::new(r"logs/").unwrap(),
            
            // OS generated files
            Regex::new(r"\.DS_Store$").unwrap(),
            Regex::new(r"Thumbs\.db$").unwrap(),
            Regex::new(r"desktop\.ini$").unwrap(),
            
            // Version control
            Regex::new(r"\.git/").unwrap(),
            Regex::new(r"\.svn/").unwrap(),
            Regex::new(r"\.hg/").unwrap(),
            
            // Dependency management
            Regex::new(r"node_modules/").unwrap(),  // For Kotlin/JS
            Regex::new(r"package-lock\.json").unwrap(),  // For Kotlin/JS
            
            // Kotlin/JS specific
            Regex::new(r"\.js\.map$").unwrap(),
            Regex::new(r"kotlin-js-store/").unwrap(),
        ];

        let cache_patterns = vec![
            // Gradle cache
            Regex::new(r"\.gradle/caches/").unwrap(),
            Regex::new(r"\.gradle/wrapper/").unwrap(),
            
            // Maven cache
            Regex::new(r"\.m2/repository/").unwrap(),
            
            // Kotlin compiler cache
            Regex::new(r"\.kotlin/sessions/").unwrap(),
            Regex::new(r"\.kotlin/incremental/").unwrap(),
            
            // IDE cache
            Regex::new(r"\.idea/caches/").unwrap(),
            Regex::new(r"\.idea/shelf/").unwrap(),
            
            // Android build cache
            Regex::new(r"\.android/build-cache/").unwrap(),
            
            // Kotlin/Native cache
            Regex::new(r"\.konan/").unwrap(),
            
            // Test cache
            Regex::new(r"\.kover/cache/").unwrap(),
        ];

        let extensions = vec![
            "kt".to_string(),
            "kts".to_string(),  // Kotlin script
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

    pub fn is_external(&self, path: &str) -> bool {
        self.external_patterns.iter().any(|pattern| pattern.is_match(path))
    }

    pub fn is_cache(&self, path: &str) -> bool {
        self.cache_patterns.iter().any(|pattern| pattern.is_match(path))
    }

    pub fn is_supported_extension(&self, ext: &str) -> bool {
        self.extensions.contains(&ext.to_lowercase())
    }

    pub fn get_script_names() -> Vec<&'static str> {
        vec![
            // Gradle (primary build tool for Kotlin)
            "build.gradle", "build.gradle.kts", "settings.gradle", "settings.gradle.kts",
            "gradle.properties", "gradlew", "gradlew.bat",
            
            // Maven (also used with Kotlin)
            "pom.xml", "pom.properties", "settings.xml",
            "mvnw", "mvnw.cmd",
            
            // Android specific
            "local.properties", "proguard-rules.pro",
            
            // Kotlin specific
            "kotlin-js-store", "kotlin-native-store",
            
            // Package managers
            "package.json", "package-lock.json", // For Kotlin/JS
        ]
    }
}

impl Default for KotlinPatterns {
    fn default() -> Self {
        Self::new()
    }
} 