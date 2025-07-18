use regex::Regex;

pub struct PhpPatterns {
    external_patterns: Vec<Regex>,
    cache_patterns: Vec<Regex>,
    extensions: Vec<String>,
}

impl PhpPatterns {
    pub fn new() -> Self {
        let external_patterns = vec![
            // Composer
            Regex::new(r"vendor/").unwrap(),
            Regex::new(r"composer\.lock").unwrap(),
            Regex::new(r"composer\.phar").unwrap(),
            
            // PEAR
            Regex::new(r"\.pear/").unwrap(),
            
            // Build artifacts
            Regex::new(r"\.phar$").unwrap(),
            
            // Laravel specific
            Regex::new(r"bootstrap/cache/").unwrap(),
            Regex::new(r"storage/app/").unwrap(),
            Regex::new(r"storage/framework/").unwrap(),
            Regex::new(r"storage/logs/").unwrap(),
            Regex::new(r"public/storage/").unwrap(),
            
            // Symfony specific
            Regex::new(r"var/cache/").unwrap(),
            Regex::new(r"var/logs/").unwrap(),
            Regex::new(r"var/sessions/").unwrap(),
            
            // WordPress specific
            Regex::new(r"wp-content/uploads/").unwrap(),
            Regex::new(r"wp-content/cache/").unwrap(),
            Regex::new(r"wp-content/backup/").unwrap(),
            
            // Drupal specific
            Regex::new(r"sites/default/files/").unwrap(),
            Regex::new(r"sites/default/private/").unwrap(),
            
            // Magento specific
            Regex::new(r"var/cache/").unwrap(),
            Regex::new(r"var/session/").unwrap(),
            Regex::new(r"var/log/").unwrap(),
            Regex::new(r"media/").unwrap(),
            
            // CodeIgniter specific
            Regex::new(r"application/cache/").unwrap(),
            Regex::new(r"application/logs/").unwrap(),
            
            // Yii specific
            Regex::new(r"protected/runtime/").unwrap(),
            Regex::new(r"assets/").unwrap(),
            
            // Zend specific
            Regex::new(r"data/cache/").unwrap(),
            Regex::new(r"data/logs/").unwrap(),
            
            // CakePHP specific
            Regex::new(r"tmp/cache/").unwrap(),
            Regex::new(r"tmp/logs/").unwrap(),
            Regex::new(r"tmp/sessions/").unwrap(),
            
            // PHPUnit
            Regex::new(r"\.phpunit\.result\.cache").unwrap(),
            
            // Codeception
            Regex::new(r"tests/_output/").unwrap(),
            
            // Behat
            Regex::new(r"behat\.yml").unwrap(),
            
            // Phing
            Regex::new(r"build/").unwrap(),
            
            // Deployer
            Regex::new(r"\.dep/").unwrap(),
            
            // IDE files
            Regex::new(r"\.vscode/").unwrap(),
            Regex::new(r"\.idea/").unwrap(),
            Regex::new(r"\.sublime-").unwrap(),
            
            // OS files
            Regex::new(r"\.DS_Store").unwrap(),
            Regex::new(r"Thumbs\.db").unwrap(),
            
            // Backup files
            Regex::new(r"\.bak$").unwrap(),
            Regex::new(r"\.swp$").unwrap(),
            Regex::new(r"\.swo$").unwrap(),
            Regex::new(r"~$").unwrap(),
            
            // Log files
            Regex::new(r"\.log$").unwrap(),
            Regex::new(r"error_log").unwrap(),
            
            // Session files
            Regex::new(r"sess_").unwrap(),
            
            // Temporary files
            Regex::new(r"\.tmp$").unwrap(),
            Regex::new(r"\.temp$").unwrap(),
            
            // Upload directories
            Regex::new(r"uploads/").unwrap(),
            
            // Cache directories
            Regex::new(r"cache/").unwrap(),
            
            // Config files with sensitive data
            Regex::new(r"\.env$").unwrap(),
            Regex::new(r"\.env\.local$").unwrap(),
            
            // Database files
            Regex::new(r"\.sqlite$").unwrap(),
            Regex::new(r"\.db$").unwrap(),
            
            // Compiled templates
            Regex::new(r"\.twig\.cache").unwrap(),
            Regex::new(r"\.smarty\.cache").unwrap(),
            
            // Minified files
            Regex::new(r"\.min\.js$").unwrap(),
            Regex::new(r"\.min\.css$").unwrap(),
            
            // Source maps
            Regex::new(r"\.map$").unwrap(),
            
            // Bower
            Regex::new(r"bower_components/").unwrap(),
            
            // Node modules (for frontend)
            Regex::new(r"node_modules/").unwrap(),
            
            // NPM
            Regex::new(r"package-lock\.json").unwrap(),
            
            // Yarn
            Regex::new(r"yarn\.lock").unwrap(),
            
            // Docker
            Regex::new(r"\.dockerignore").unwrap(),
            
            // Vagrant
            Regex::new(r"\.vagrant/").unwrap(),
            
            // Homestead
            Regex::new(r"Homestead\.yaml").unwrap(),
            Regex::new(r"Homestead\.json").unwrap(),
            
            // Valet
            Regex::new(r"\.valet/").unwrap(),
            
            // MAMP
            Regex::new(r"\.mamp/").unwrap(),
            
            // XAMPP
            Regex::new(r"\.xampp/").unwrap(),
            
            // WampServer
            Regex::new(r"\.wamp/").unwrap(),
            
            // PhpStorm
            Regex::new(r"\.phpstorm\.meta\.php").unwrap(),
            
            // NetBeans
            Regex::new(r"nbproject/").unwrap(),
            
            // Eclipse
            Regex::new(r"\.buildpath").unwrap(),
            Regex::new(r"\.project").unwrap(),
            Regex::new(r"\.settings/").unwrap(),
            
            // Aptana
            Regex::new(r"\.aptana/").unwrap(),
        ];

        let cache_patterns = vec![
            // Composer cache
            Regex::new(r"vendor/").unwrap(),
            
            // Framework caches
            Regex::new(r"bootstrap/cache/").unwrap(),
            Regex::new(r"storage/framework/cache/").unwrap(),
            Regex::new(r"storage/framework/sessions/").unwrap(),
            Regex::new(r"storage/framework/views/").unwrap(),
            Regex::new(r"var/cache/").unwrap(),
            Regex::new(r"tmp/cache/").unwrap(),
            Regex::new(r"application/cache/").unwrap(),
            Regex::new(r"protected/runtime/").unwrap(),
            Regex::new(r"data/cache/").unwrap(),
            
            // Test caches
            Regex::new(r"\.phpunit\.result\.cache").unwrap(),
            Regex::new(r"tests/_output/").unwrap(),
            
            // Log files
            Regex::new(r"storage/logs/").unwrap(),
            Regex::new(r"var/logs/").unwrap(),
            Regex::new(r"tmp/logs/").unwrap(),
            Regex::new(r"application/logs/").unwrap(),
            Regex::new(r"data/logs/").unwrap(),
            Regex::new(r"\.log$").unwrap(),
            Regex::new(r"error_log").unwrap(),
            
            // Session files
            Regex::new(r"storage/framework/sessions/").unwrap(),
            Regex::new(r"var/sessions/").unwrap(),
            Regex::new(r"tmp/sessions/").unwrap(),
            Regex::new(r"sess_").unwrap(),
            
            // Temporary files
            Regex::new(r"\.tmp$").unwrap(),
            Regex::new(r"\.temp$").unwrap(),
            
            // IDE caches
            Regex::new(r"\.vscode/").unwrap(),
            Regex::new(r"\.idea/").unwrap(),
            
            // OS caches
            Regex::new(r"\.DS_Store").unwrap(),
            Regex::new(r"Thumbs\.db").unwrap(),
            
            // Editor files
            Regex::new(r"\.swp$").unwrap(),
            Regex::new(r"\.swo$").unwrap(),
            Regex::new(r"~$").unwrap(),
            
            // Build artifacts
            Regex::new(r"build/").unwrap(),
            
            // Upload directories
            Regex::new(r"uploads/").unwrap(),
            
            // Media directories
            Regex::new(r"media/").unwrap(),
            
            // Assets
            Regex::new(r"assets/").unwrap(),
            
            // Public storage
            Regex::new(r"public/storage/").unwrap(),
            
            // Compiled templates
            Regex::new(r"\.twig\.cache").unwrap(),
            Regex::new(r"\.smarty\.cache").unwrap(),
            
            // Frontend build artifacts
            Regex::new(r"node_modules/").unwrap(),
            Regex::new(r"bower_components/").unwrap(),
            
            // Minified files
            Regex::new(r"\.min\.js$").unwrap(),
            Regex::new(r"\.min\.css$").unwrap(),
            
            // Source maps
            Regex::new(r"\.map$").unwrap(),
        ];

        let extensions = vec![
            // PHP files
            "php".to_string(), "php3".to_string(), "php4".to_string(),
            "php5".to_string(), "php7".to_string(), "php8".to_string(),
            "phtml".to_string(), "phps".to_string(),
            
            // PHP archives
            "phar".to_string(),
            
            // Template files
            "twig".to_string(), "blade".to_string(),
            "smarty".to_string(), "tpl".to_string(),
            
            // Configuration files
            "ini".to_string(), "conf".to_string(),
            
            // YAML (common in PHP projects)
            "yml".to_string(), "yaml".to_string(),
            
            // JSON (common in PHP projects)
            "json".to_string(),
            
            // XML (common in PHP projects)
            "xml".to_string(),
            
            // Markdown
            "md".to_string(), "markdown".to_string(),
            
            // SQL
            "sql".to_string(),
            
            // Web files
            "html".to_string(), "htm".to_string(),
            "css".to_string(), "js".to_string(),
            
            // Environment files
            "env".to_string(),
            
            // Lock files
            "lock".to_string(),
            
            // Log files
            "log".to_string(),
            
            // Backup files
            "bak".to_string(), "backup".to_string(),
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
            // Composer
            "composer.json", "composer.lock", "composer.phar",
            
            // PHP configuration
            "php.ini", ".htaccess", "web.config",
            
            // Laravel
            "artisan", ".env", ".env.example", ".env.local",
            "server.php", "webpack.mix.js", "vite.config.js",
            
            // Symfony
            "symfony", "console", "bin/console",
            
            // WordPress
            "wp-config.php", "wp-config-sample.php",
            "wp-load.php", "wp-settings.php",
            "functions.php", "style.css", "index.php",
            
            // Drupal
            "settings.php", "settings.local.php",
            "services.yml", "routing.yml",
            
            // Magento
            "app/etc/config.xml", "app/etc/local.xml",
            "index.php", "cron.php",
            
            // CodeIgniter
            "index.php", "config/config.php",
            "config/database.php", "config/routes.php",
            
            // Yii
            "index.php", "protected/config/main.php",
            "protected/config/console.php",
            
            // Zend
            "index.php", "application/configs/application.ini",
            "public/index.php",
            
            // CakePHP
            "index.php", "app/Config/core.php",
            "app/Config/database.php",
            
            // Build tools
            "build.xml", "phing.xml", "Makefile",
            "gulpfile.js", "Gruntfile.js",
            
            // Testing
            "phpunit.xml", "phpunit.xml.dist",
            "codeception.yml", "behat.yml",
            ".phpunit.result.cache",
            
            // Code quality
            ".php_cs", ".php_cs.dist", "phpcs.xml",
            "phpcs.xml.dist", "phpmd.xml",
            ".phpstan.neon", "psalm.xml",
            
            // Documentation
            "README.md", "CHANGELOG.md", "LICENSE",
            "CONTRIBUTING.md", "phpdoc.xml",
            
            // CI/CD
            ".github/workflows/php.yml", ".travis.yml",
            ".gitlab-ci.yml", "circle.yml", "appveyor.yml",
            
            // Docker
            "Dockerfile", "docker-compose.yml", ".dockerignore",
            
            // Vagrant
            "Vagrantfile", ".vagrant/",
            
            // Deployment
            "deploy.php", "Capfile", "config/deploy.rb",
            
            // Environment
            ".env", ".env.local", ".env.example",
            ".env.development", ".env.production", ".env.testing",
            
            // Package managers
            "package.json", "package-lock.json", "yarn.lock",
            "bower.json", ".bowerrc",
            
            // Frontend build
            "webpack.config.js", "webpack.mix.js",
            "vite.config.js", "rollup.config.js",
            
            // IDE
            ".vscode/", ".idea/", ".sublime-project",
            
            // Git
            ".gitignore", ".gitattributes", ".gitmodules",
            
            // Editors
            ".editorconfig", ".eslintrc", ".prettierrc",
            
            // Security
            ".htaccess", ".htpasswd", "robots.txt",
            
            // Database
            "schema.sql", "migrations/", "seeds/",
            
            // API
            "api.php", "routes/api.php", "swagger.json",
            "openapi.json", "postman.json",
            
            // Caching
            "redis.conf", "memcached.conf",
            
            // Web server
            "nginx.conf", "apache.conf", ".htaccess",
            
            // SSL
            "ssl.conf", "certificate.pem", "private.key",
            
            // Monitoring
            "newrelic.ini", "blackfire.ini",
            
            // Logging
            "monolog.yml", "log4php.xml",
            
            // Localization
            "locale/", "lang/", "resources/lang/",
            
            // Assets
            "public/", "assets/", "resources/",
            
            // Storage
            "storage/", "var/", "tmp/", "temp/",
            
            // Uploads
            "uploads/", "files/", "media/",
            
            // Cache
            "cache/", "tmp/cache/", "var/cache/",
            
            // Sessions
            "sessions/", "tmp/sessions/", "var/sessions/",
            
            // Logs
            "logs/", "log/", "var/logs/",
            
            // Backups
            "backups/", "backup/", "dumps/",
            
            // Vendor
            "vendor/", "lib/", "libraries/",
            
            // Plugins
            "plugins/", "modules/", "extensions/",
            
            // Themes
            "themes/", "templates/", "views/",
            
            // Config
            "config/", "configs/", "configuration/",
            
            // Scripts
            "scripts/", "bin/", "tools/",
            
            // Tests
            "tests/", "test/", "spec/",
            
            // Fixtures
            "fixtures/", "seeds/", "factories/",
            
            // Mocks
            "mocks/", "stubs/", "fakes/",
            
            // Documentation
            "docs/", "doc/", "documentation/",
            
            // Examples
            "examples/", "samples/", "demo/",
            
            // Build
            "build/", "dist/", "release/",
            
            // Deploy
            "deploy/", "deployment/", "releases/",
        ]
    }
} 