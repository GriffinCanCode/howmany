use regex::Regex;

pub struct SwiftPatterns {
    external_patterns: Vec<Regex>,
    cache_patterns: Vec<Regex>,
    extensions: Vec<String>,
}

impl SwiftPatterns {
    pub fn new() -> Self {
        let external_patterns = vec![
            // Swift Package Manager
            Regex::new(r"\.build/").unwrap(),
            Regex::new(r"\.swiftpm/").unwrap(),
            Regex::new(r"Packages/").unwrap(),
            Regex::new(r"Package\.resolved").unwrap(),
            
            // Xcode
            Regex::new(r"\.xcodeproj/").unwrap(),
            Regex::new(r"\.xcworkspace/").unwrap(),
            Regex::new(r"DerivedData/").unwrap(),
            Regex::new(r"\.xcuserstate").unwrap(),
            Regex::new(r"\.xccheckout").unwrap(),
            Regex::new(r"\.moved-aside").unwrap(),
            Regex::new(r"\.xcscmblueprint").unwrap(),
            Regex::new(r"\.xcscheme").unwrap(),
            
            // Build artifacts
            Regex::new(r"build/").unwrap(),
            Regex::new(r"\.dSYM/").unwrap(),
            Regex::new(r"\.app/").unwrap(),
            Regex::new(r"\.ipa$").unwrap(),
            Regex::new(r"\.framework/").unwrap(),
            Regex::new(r"\.xcframework/").unwrap(),
            
            // CocoaPods
            Regex::new(r"Pods/").unwrap(),
            Regex::new(r"Podfile\.lock").unwrap(),
            
            // Carthage
            Regex::new(r"Carthage/").unwrap(),
            Regex::new(r"Cartfile\.resolved").unwrap(),
            
            // Fastlane
            Regex::new(r"fastlane/report\.xml").unwrap(),
            Regex::new(r"fastlane/Preview\.html").unwrap(),
            Regex::new(r"fastlane/screenshots").unwrap(),
            Regex::new(r"fastlane/test_output").unwrap(),
            
            // Provisioning profiles
            Regex::new(r"\.mobileprovision$").unwrap(),
            Regex::new(r"\.provisionprofile$").unwrap(),
            
            // Certificates
            Regex::new(r"\.p12$").unwrap(),
            Regex::new(r"\.cer$").unwrap(),
            Regex::new(r"\.certSigningRequest$").unwrap(),
            
            // Simulator
            Regex::new(r"\.xcappdata/").unwrap(),
            Regex::new(r"\.xclog$").unwrap(),
            
            // Archives
            Regex::new(r"\.xcarchive/").unwrap(),
            
            // Instruments
            Regex::new(r"\.trace/").unwrap(),
            
            // Playgrounds
            Regex::new(r"\.playground/").unwrap(),
            
            // Swift compiled modules
            Regex::new(r"\.swiftmodule/").unwrap(),
            Regex::new(r"\.swiftdoc$").unwrap(),
            Regex::new(r"\.swiftsourceinfo$").unwrap(),
            
            // Obj-C
            Regex::new(r"\.pch$").unwrap(),
            
            // Core Data
            Regex::new(r"\.mom$").unwrap(),
            Regex::new(r"\.momd/").unwrap(),
            
            // Interface Builder
            Regex::new(r"\.nib$").unwrap(),
            
            // Asset catalogs
            Regex::new(r"\.actool$").unwrap(),
            
            // Localization
            Regex::new(r"\.lproj/").unwrap(),
            
            // Documentation
            Regex::new(r"\.docset/").unwrap(),
            
            // Backup files
            Regex::new(r"\.orig$").unwrap(),
            Regex::new(r"\.rej$").unwrap(),
            
            // OS files
            Regex::new(r"\.DS_Store").unwrap(),
            Regex::new(r"\.AppleDouble").unwrap(),
            Regex::new(r"\.LSOverride").unwrap(),
            
            // Thumbnails
            Regex::new(r"\._ ").unwrap(),
            
            // Files that might appear in the root of a volume
            Regex::new(r"\.DocumentRevisions-V100").unwrap(),
            Regex::new(r"\.fseventsd").unwrap(),
            Regex::new(r"\.Spotlight-V100").unwrap(),
            Regex::new(r"\.TemporaryItems").unwrap(),
            Regex::new(r"\.Trashes").unwrap(),
            Regex::new(r"\.VolumeIcon\.icns").unwrap(),
            Regex::new(r"\.com\.apple\.timemachine\.donotpresent").unwrap(),
            
            // Directories potentially created on remote AFP share
            Regex::new(r"\.AppleDB").unwrap(),
            Regex::new(r"\.AppleDesktop").unwrap(),
            Regex::new(r"Network Trash Folder").unwrap(),
            Regex::new(r"Temporary Items").unwrap(),
            Regex::new(r"\.apdisk").unwrap(),
            
            // IDE files
            Regex::new(r"\.vscode/").unwrap(),
            Regex::new(r"\.idea/").unwrap(),
            
            // Temporary files
            Regex::new(r"\.tmp$").unwrap(),
            Regex::new(r"\.swp$").unwrap(),
            Regex::new(r"\.swo$").unwrap(),
            Regex::new(r"~$").unwrap(),
            
            // Log files
            Regex::new(r"\.log$").unwrap(),
            
            // Crash reports
            Regex::new(r"\.crash$").unwrap(),
            
            // Test results
            Regex::new(r"\.xcresult/").unwrap(),
            
            // Swift format
            Regex::new(r"\.swiftformat").unwrap(),
            
            // SwiftLint
            Regex::new(r"\.swiftlint\.yml").unwrap(),
            
            // Tuist
            Regex::new(r"\.tuist/").unwrap(),
            Regex::new(r"Tuist/").unwrap(),
            
            // Mint
            Regex::new(r"\.mint/").unwrap(),
            
            // Sourcery
            Regex::new(r"\.sourcery/").unwrap(),
            
            // R.swift
            Regex::new(r"R\.generated\.swift").unwrap(),
            
            // SwiftGen
            Regex::new(r"\.swiftgen/").unwrap(),
            
            // Jazzy
            Regex::new(r"docs/").unwrap(),
            
            // Danger
            Regex::new(r"Dangerfile").unwrap(),
            
            // Bitrise
            Regex::new(r"bitrise\.yml").unwrap(),
            
            // GitHub Actions
            Regex::new(r"\.github/").unwrap(),
            
            // Firebase
            Regex::new(r"GoogleService-Info\.plist").unwrap(),
            
            // TestFlight
            Regex::new(r"ExportOptions\.plist").unwrap(),
            
            // App Store Connect
            Regex::new(r"metadata/").unwrap(),
            
            // Xcode Cloud
            Regex::new(r"ci_scripts/").unwrap(),
            
            // Swift Playgrounds
            Regex::new(r"\.playgroundbook/").unwrap(),
            
            // Reality Composer
            Regex::new(r"\.rcproject/").unwrap(),
            
            // Create ML
            Regex::new(r"\.mlmodel$").unwrap(),
            Regex::new(r"\.mlmodelc/").unwrap(),
            
            // Metal
            Regex::new(r"\.metal$").unwrap(),
            Regex::new(r"\.air$").unwrap(),
            Regex::new(r"\.metallib$").unwrap(),
            
            // SceneKit
            Regex::new(r"\.scn$").unwrap(),
            Regex::new(r"\.scnassets/").unwrap(),
            
            // SpriteKit
            Regex::new(r"\.sks$").unwrap(),
            
            // Audio
            Regex::new(r"\.caf$").unwrap(),
            
            // Video
            Regex::new(r"\.mov$").unwrap(),
            Regex::new(r"\.mp4$").unwrap(),
            
            // Images
            Regex::new(r"\.png$").unwrap(),
            Regex::new(r"\.jpg$").unwrap(),
            Regex::new(r"\.jpeg$").unwrap(),
            Regex::new(r"\.gif$").unwrap(),
            Regex::new(r"\.tiff$").unwrap(),
            Regex::new(r"\.svg$").unwrap(),
            Regex::new(r"\.pdf$").unwrap(),
            Regex::new(r"\.eps$").unwrap(),
            
            // Fonts
            Regex::new(r"\.ttf$").unwrap(),
            Regex::new(r"\.otf$").unwrap(),
            
            // Data
            Regex::new(r"\.json$").unwrap(),
            Regex::new(r"\.plist$").unwrap(),
            Regex::new(r"\.xml$").unwrap(),
            Regex::new(r"\.csv$").unwrap(),
            
            // Backup
            Regex::new(r"\.bak$").unwrap(),
            Regex::new(r"\.backup$").unwrap(),
        ];

        let cache_patterns = vec![
            // Xcode caches
            Regex::new(r"DerivedData/").unwrap(),
            Regex::new(r"\.build/").unwrap(),
            Regex::new(r"build/").unwrap(),
            
            // Swift Package Manager cache
            Regex::new(r"\.swiftpm/").unwrap(),
            
            // CocoaPods cache
            Regex::new(r"Pods/").unwrap(),
            
            // Carthage cache
            Regex::new(r"Carthage/").unwrap(),
            
            // Fastlane cache
            Regex::new(r"fastlane/report\.xml").unwrap(),
            Regex::new(r"fastlane/Preview\.html").unwrap(),
            Regex::new(r"fastlane/screenshots").unwrap(),
            Regex::new(r"fastlane/test_output").unwrap(),
            
            // Test results
            Regex::new(r"\.xcresult/").unwrap(),
            
            // Archives
            Regex::new(r"\.xcarchive/").unwrap(),
            
            // Simulator data
            Regex::new(r"\.xcappdata/").unwrap(),
            
            // Instruments traces
            Regex::new(r"\.trace/").unwrap(),
            
            // Logs
            Regex::new(r"\.log$").unwrap(),
            Regex::new(r"\.xclog$").unwrap(),
            
            // Crash reports
            Regex::new(r"\.crash$").unwrap(),
            
            // Temporary files
            Regex::new(r"\.tmp$").unwrap(),
            Regex::new(r"\.temp$").unwrap(),
            
            // OS caches
            Regex::new(r"\.DS_Store").unwrap(),
            Regex::new(r"\.AppleDouble").unwrap(),
            Regex::new(r"\.LSOverride").unwrap(),
            
            // IDE caches
            Regex::new(r"\.vscode/").unwrap(),
            Regex::new(r"\.idea/").unwrap(),
            
            // Editor files
            Regex::new(r"\.swp$").unwrap(),
            Regex::new(r"\.swo$").unwrap(),
            Regex::new(r"~$").unwrap(),
            
            // Documentation
            Regex::new(r"docs/").unwrap(),
            Regex::new(r"\.docset/").unwrap(),
            
            // Tuist cache
            Regex::new(r"\.tuist/").unwrap(),
            
            // Mint cache
            Regex::new(r"\.mint/").unwrap(),
            
            // Sourcery cache
            Regex::new(r"\.sourcery/").unwrap(),
            
            // SwiftGen cache
            Regex::new(r"\.swiftgen/").unwrap(),
            
            // Backup files
            Regex::new(r"\.orig$").unwrap(),
            Regex::new(r"\.rej$").unwrap(),
            Regex::new(r"\.bak$").unwrap(),
            Regex::new(r"\.backup$").unwrap(),
        ];

        let extensions = vec![
            // Swift files
            "swift".to_string(),
            
            // Objective-C files
            "m".to_string(), "mm".to_string(),
            "h".to_string(), "hpp".to_string(),
            
            // C/C++ files
            "c".to_string(), "cpp".to_string(),
            "cc".to_string(), "cxx".to_string(),
            
            // Swift dependencies
            "swiftdeps".to_string(),
            "swiftmodule".to_string(),
            "swiftsourceinfo".to_string(),
            "swiftdoc".to_string(),
            
            // Interface Builder
            "storyboard".to_string(),
            "xib".to_string(),
            
            // Property lists
            "plist".to_string(),
            
            // Entitlements
            "entitlements".to_string(),
            
            // Metal shaders
            "metal".to_string(),
            
            // SceneKit
            "scn".to_string(),
            
            // SpriteKit
            "sks".to_string(),
            
            // Core Data
            "xcdatamodel".to_string(),
            "xcdatamodeld".to_string(),
            "mom".to_string(),
            "momd".to_string(),
            
            // Mapping models
            "xcmappingmodel".to_string(),
            
            // Asset catalogs
            "xcassets".to_string(),
            
            // Localization
            "strings".to_string(),
            "stringsdict".to_string(),
            
            // Documentation
            "md".to_string(),
            "markdown".to_string(),
            
            // Configuration
            "yaml".to_string(),
            "yml".to_string(),
            "json".to_string(),
            "toml".to_string(),
            
            // Scripts
            "sh".to_string(),
            "py".to_string(),
            "rb".to_string(),
            
            // Certificates
            "p12".to_string(),
            "cer".to_string(),
            "mobileprovision".to_string(),
            "provisionprofile".to_string(),
            
            // Archives
            "ipa".to_string(),
            "xcarchive".to_string(),
            
            // Playgrounds
            "playground".to_string(),
            "playgroundbook".to_string(),
            
            // Reality Composer
            "rcproject".to_string(),
            
            // Create ML
            "mlmodel".to_string(),
            "mlmodelc".to_string(),
            
            // Audio
            "caf".to_string(),
            "aiff".to_string(),
            "wav".to_string(),
            "mp3".to_string(),
            "m4a".to_string(),
            
            // Video
            "mov".to_string(),
            "mp4".to_string(),
            "m4v".to_string(),
            "avi".to_string(),
            
            // Images
            "png".to_string(),
            "jpg".to_string(),
            "jpeg".to_string(),
            "gif".to_string(),
            "tiff".to_string(),
            "svg".to_string(),
            "pdf".to_string(),
            "eps".to_string(),
            "heic".to_string(),
            "heif".to_string(),
            
            // Fonts
            "ttf".to_string(),
            "otf".to_string(),
            
            // Data
            "csv".to_string(),
            "xml".to_string(),
            "sqlite".to_string(),
            "db".to_string(),
            
            // Logs
            "log".to_string(),
            "crash".to_string(),
            
            // Temporary
            "tmp".to_string(),
            "temp".to_string(),
            
            // Backup
            "bak".to_string(),
            "backup".to_string(),
            "orig".to_string(),
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
            // Swift Package Manager
            "Package.swift", "Package.resolved",
            
            // CocoaPods
            "Podfile", "Podfile.lock", "*.podspec",
            
            // Carthage
            "Cartfile", "Cartfile.resolved", "Cartfile.private",
            
            // Xcode project files
            "*.xcodeproj", "*.xcworkspace",
            "project.pbxproj", "*.xcscheme",
            "*.xcconfig", "*.plist",
            
            // Build scripts
            "build.sh", "build.py", "build.rb",
            "archive.sh", "deploy.sh", "test.sh",
            
            // Fastlane
            "Fastfile", "Appfile", "Deliverfile",
            "Gymfile", "Matchfile", "Precheckfile",
            "Scanfile", "Screengrabfile", "Snapshotfile",
            
            // Configuration
            "Info.plist", "*.entitlements",
            "GoogleService-Info.plist", "ExportOptions.plist",
            
            // Documentation
            "README.md", "CHANGELOG.md", "LICENSE",
            "CONTRIBUTING.md", "CODE_OF_CONDUCT.md",
            
            // CI/CD
            ".github/workflows/ios.yml", ".github/workflows/swift.yml",
            ".travis.yml", ".gitlab-ci.yml", "circle.yml",
            "bitrise.yml", "azure-pipelines.yml",
            
            // Xcode Cloud
            "ci_scripts/ci_pre_xcodebuild.sh",
            "ci_scripts/ci_post_xcodebuild.sh",
            "ci_scripts/ci_post_clone.sh",
            
            // Code quality
            ".swiftlint.yml", ".swiftformat",
            ".swiftgen.yml", ".sourcery.yml",
            
            // Testing
            "*.xctest", "TestPlans/",
            
            // Danger
            "Dangerfile", "Dangerfile.swift",
            
            // Tuist
            "Project.swift", "Workspace.swift",
            "Config.swift", "Setup.swift",
            
            // Mint
            "Mintfile",
            
            // Jazzy
            ".jazzy.yaml", ".jazzy.yml",
            
            // R.swift
            "R.generated.swift",
            
            // SwiftGen
            "swiftgen.yml",
            
            // Sourcery
            "sourcery.yml", "*.stencil",
            
            // Git
            ".gitignore", ".gitattributes", ".gitmodules",
            
            // Editors
            ".vscode/", ".idea/", ".sublime-project",
            
            // Environment
            ".env", ".env.local", ".env.example",
            
            // Docker
            "Dockerfile", "docker-compose.yml", ".dockerignore",
            
            // Scripts
            "scripts/", "bin/", "tools/",
            
            // Assets
            "Assets.xcassets/", "*.xcassets",
            
            // Localization
            "*.lproj/", "Localizable.strings",
            "InfoPlist.strings", "*.stringsdict",
            
            // Core Data
            "*.xcdatamodel", "*.xcdatamodeld",
            "*.xcmappingmodel",
            
            // Interface Builder
            "*.storyboard", "*.xib",
            
            // Metal
            "*.metal", "*.air", "*.metallib",
            
            // SceneKit
            "*.scn", "*.scnassets/",
            
            // SpriteKit
            "*.sks", "*.atlas/",
            
            // Reality Composer
            "*.rcproject/",
            
            // Create ML
            "*.mlmodel", "*.mlmodelc/",
            
            // Playgrounds
            "*.playground/", "*.playgroundbook/",
            
            // Audio
            "*.caf", "*.aiff", "*.wav", "*.mp3", "*.m4a",
            
            // Video
            "*.mov", "*.mp4", "*.m4v", "*.avi",
            
            // Images
            "*.png", "*.jpg", "*.jpeg", "*.gif",
            "*.tiff", "*.svg", "*.pdf", "*.eps",
            "*.heic", "*.heif",
            
            // Fonts
            "*.ttf", "*.otf",
            
            // Data
            "*.json", "*.plist", "*.xml", "*.csv",
            "*.sqlite", "*.db",
            
            // Certificates
            "*.p12", "*.cer", "*.mobileprovision",
            "*.provisionprofile", "*.certSigningRequest",
            
            // Archives
            "*.ipa", "*.xcarchive/",
            
            // Frameworks
            "*.framework/", "*.xcframework/",
            
            // Libraries
            "*.a", "*.dylib",
            
            // Modules
            "*.swiftmodule/", "*.swiftdoc",
            "*.swiftsourceinfo", "*.swiftdeps",
            
            // Instruments
            "*.trace/",
            
            // Simulator
            "*.xcappdata/", "*.xclog",
            
            // Test results
            "*.xcresult/",
            
            // Logs
            "*.log", "*.crash",
            
            // Temporary
            "*.tmp", "*.temp", "*.swp", "*.swo",
            
            // Backup
            "*.bak", "*.backup", "*.orig", "*.rej",
            
            // OS
            ".DS_Store", ".AppleDouble", ".LSOverride",
            "._*", ".DocumentRevisions-V100",
            ".fseventsd", ".Spotlight-V100",
            ".TemporaryItems", ".Trashes",
            ".VolumeIcon.icns",
            ".com.apple.timemachine.donotpresent",
            ".AppleDB", ".AppleDesktop",
            "Network Trash Folder", "Temporary Items",
            ".apdisk",
        ]
    }
} 