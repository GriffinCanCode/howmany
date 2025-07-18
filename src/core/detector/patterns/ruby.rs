use regex::Regex;

pub struct RubyPatterns {
    external_patterns: Vec<Regex>,
    cache_patterns: Vec<Regex>,
    extensions: Vec<String>,
}

impl RubyPatterns {
    pub fn new() -> Self {
        let external_patterns = vec![
            // Gem system
            Regex::new(r"\.bundle/").unwrap(),
            Regex::new(r"vendor/bundle/").unwrap(),
            Regex::new(r"\.gem$").unwrap(),
            Regex::new(r"Gemfile\.lock").unwrap(),
            
            // RubyGems
            Regex::new(r"\.gem/").unwrap(),
            Regex::new(r"gems/").unwrap(),
            
            // Rails specific
            Regex::new(r"log/").unwrap(),
            Regex::new(r"tmp/").unwrap(),
            Regex::new(r"storage/").unwrap(),
            Regex::new(r"public/assets/").unwrap(),
            Regex::new(r"public/packs/").unwrap(),
            Regex::new(r"public/uploads/").unwrap(),
            
            // Coverage
            Regex::new(r"coverage/").unwrap(),
            
            // RSpec
            Regex::new(r"spec/reports/").unwrap(),
            
            // Yard documentation
            Regex::new(r"\.yardoc/").unwrap(),
            Regex::new(r"doc/").unwrap(),
            
            // Capistrano
            Regex::new(r"\.capistrano/").unwrap(),
            
            // Byebug
            Regex::new(r"\.byebug_history").unwrap(),
            
            // IRB
            Regex::new(r"\.irb_history").unwrap(),
            
            // Pry
            Regex::new(r"\.pry_history").unwrap(),
            
            // RVM
            Regex::new(r"\.rvmrc").unwrap(),
            
            // rbenv
            Regex::new(r"\.ruby-version").unwrap(),
            Regex::new(r"\.ruby-gemset").unwrap(),
            
            // Pow
            Regex::new(r"\.powrc").unwrap(),
            Regex::new(r"\.powder").unwrap(),
            
            // Sass cache
            Regex::new(r"\.sass-cache/").unwrap(),
            
            // Capybara
            Regex::new(r"capybara-").unwrap(),
            
            // Spring
            Regex::new(r"\.spring/").unwrap(),
            
            // Bootsnap
            Regex::new(r"\.bootsnap-").unwrap(),
            
            // Listen
            Regex::new(r"\.listen_test").unwrap(),
            
            // Rerun
            Regex::new(r"\.rerun\.pid").unwrap(),
            
            // Spork
            Regex::new(r"\.spork\.pid").unwrap(),
            
            // Guard
            Regex::new(r"\.guard_history").unwrap(),
            
            // Solr
            Regex::new(r"solr/").unwrap(),
            
            // Elasticsearch
            Regex::new(r"elasticsearch/").unwrap(),
            
            // Redis
            Regex::new(r"dump\.rdb").unwrap(),
            
            // Sidekiq
            Regex::new(r"sidekiq\.pid").unwrap(),
            
            // Puma
            Regex::new(r"puma\.pid").unwrap(),
            
            // Unicorn
            Regex::new(r"unicorn\.pid").unwrap(),
            
            // Thin
            Regex::new(r"thin\.pid").unwrap(),
            
            // Passenger
            Regex::new(r"passenger\.").unwrap(),
            
            // Foreman
            Regex::new(r"\.foreman").unwrap(),
            
            // Heroku
            Regex::new(r"\.slugignore").unwrap(),
            
            // Vagrant
            Regex::new(r"\.vagrant/").unwrap(),
            
            // Docker
            Regex::new(r"\.dockerignore").unwrap(),
            
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
        ];

        let cache_patterns = vec![
            // Bundler cache
            Regex::new(r"\.bundle/").unwrap(),
            Regex::new(r"vendor/bundle/").unwrap(),
            
            // Gem cache
            Regex::new(r"\.gem/").unwrap(),
            
            // Rails cache
            Regex::new(r"tmp/cache/").unwrap(),
            Regex::new(r"tmp/pids/").unwrap(),
            Regex::new(r"tmp/sessions/").unwrap(),
            Regex::new(r"tmp/sockets/").unwrap(),
            
            // Asset cache
            Regex::new(r"public/assets/").unwrap(),
            Regex::new(r"public/packs/").unwrap(),
            
            // Test cache
            Regex::new(r"coverage/").unwrap(),
            Regex::new(r"spec/reports/").unwrap(),
            
            // Documentation cache
            Regex::new(r"\.yardoc/").unwrap(),
            Regex::new(r"doc/").unwrap(),
            
            // Sass cache
            Regex::new(r"\.sass-cache/").unwrap(),
            
            // Spring cache
            Regex::new(r"\.spring/").unwrap(),
            
            // Bootsnap cache
            Regex::new(r"\.bootsnap-").unwrap(),
            
            // Log files
            Regex::new(r"log/").unwrap(),
            
            // Temporary files
            Regex::new(r"tmp/").unwrap(),
            
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
        ];

        let extensions = vec![
            // Ruby files
            "rb".to_string(), "rbw".to_string(),
            
            // Rake files
            "rake".to_string(),
            
            // Gemspec files
            "gemspec".to_string(),
            
            // Rackup files
            "ru".to_string(),
            
            // ERB templates
            "erb".to_string(), "rhtml".to_string(),
            
            // Haml templates
            "haml".to_string(),
            
            // Slim templates
            "slim".to_string(),
            
            // Liquid templates
            "liquid".to_string(),
            
            // RDoc
            "rdoc".to_string(),
            
            // YAML (common in Ruby projects)
            "yml".to_string(), "yaml".to_string(),
            
            // JSON (common in Ruby projects)
            "json".to_string(),
            
            // Markdown
            "md".to_string(), "markdown".to_string(),
            
            // Ruby scripts
            "rbx".to_string(),
            
            // Thor
            "thor".to_string(),
            
            // Capfile
            "cap".to_string(),
            
            // Prawn
            "prawn".to_string(),
            
            // Builder
            "builder".to_string(),
            
            // RABL
            "rabl".to_string(),
            
            // JBuilder
            "jbuilder".to_string(),
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
            // Core Ruby files
            "Gemfile", "Gemfile.lock", "*.gemspec", "Rakefile",
            "config.ru", "Capfile", "Guardfile", "Procfile",
            
            // Rails files
            "application.rb", "environment.rb", "boot.rb",
            "routes.rb", "database.yml", "secrets.yml",
            "application.yml", "cable.yml", "storage.yml",
            
            // Configuration
            "config/application.rb", "config/environment.rb",
            "config/boot.rb", "config/routes.rb",
            "config/database.yml", "config/secrets.yml",
            
            // Environments
            "config/environments/development.rb",
            "config/environments/production.rb",
            "config/environments/test.rb",
            
            // Initializers
            "config/initializers/", "config/locales/",
            
            // Database
            "db/schema.rb", "db/seeds.rb", "db/migrate/",
            
            // Testing
            "spec/spec_helper.rb", "spec/rails_helper.rb",
            "test/test_helper.rb", ".rspec", ".rspec-local",
            
            // Coverage
            ".simplecov", "coverage/",
            
            // Deployment
            "config/deploy.rb", "config/deploy/",
            "Capfile", ".caprc",
            
            // Bundler
            ".bundle/config", "vendor/bundle/",
            
            // Version managers
            ".ruby-version", ".ruby-gemset", ".rvmrc",
            
            // Documentation
            ".yardopts", ".document", "README.md",
            "CHANGELOG.md", "LICENSE", "CONTRIBUTING.md",
            
            // CI/CD
            ".github/workflows/ruby.yml", ".travis.yml",
            ".gitlab-ci.yml", "circle.yml", "appveyor.yml",
            
            // Code quality
            ".rubocop.yml", ".reek.yml", ".inch.yml",
            ".codeclimate.yml", ".hound.yml",
            
            // Docker
            "Dockerfile", "docker-compose.yml", ".dockerignore",
            
            // Heroku
            ".slugignore", "app.json", "Procfile.dev",
            
            // Environment
            ".env", ".env.local", ".env.development",
            ".env.production", ".env.test", ".env.example",
            
            // Editors
            ".vscode/", ".idea/", ".sublime-project",
            
            // Git
            ".gitignore", ".gitattributes", ".gitmodules",
            
            // Package managers
            "yarn.lock", "package.json", "package-lock.json",
            
            // Asset pipeline
            "app/assets/", "vendor/assets/", "lib/assets/",
            
            // Views
            "app/views/", "app/helpers/",
            
            // Controllers
            "app/controllers/", "app/models/",
            
            // Services
            "app/services/", "app/jobs/", "app/mailers/",
            
            // Concerns
            "app/concerns/", "app/decorators/",
            
            // Policies
            "app/policies/", "app/validators/",
            
            // Serializers
            "app/serializers/", "app/presenters/",
            
            // Workers
            "app/workers/", "app/channels/",
            
            // Libraries
            "lib/", "lib/tasks/", "lib/generators/",
            
            // Engines
            "engines/", "gems/", "vendor/gems/",
            
            // Public
            "public/", "public/assets/", "public/uploads/",
            
            // Storage
            "storage/", "tmp/storage/",
            
            // Logs
            "log/", "tmp/",
            
            // Scripts
            "bin/", "script/", "exe/",
            
            // Locale
            "config/locales/", "locale/",
            
            // Fixtures
            "spec/fixtures/", "test/fixtures/",
            
            // Factories
            "spec/factories/", "test/factories/",
            
            // Support
            "spec/support/", "test/support/",
            
            // Features
            "features/", "spec/features/",
            
            // Integration
            "spec/integration/", "test/integration/",
            
            // System
            "spec/system/", "test/system/",
            
            // Requests
            "spec/requests/", "test/requests/",
            
            // Controllers
            "spec/controllers/", "test/controllers/",
            
            // Models
            "spec/models/", "test/models/",
            
            // Views
            "spec/views/", "test/views/",
            
            // Helpers
            "spec/helpers/", "test/helpers/",
            
            // Mailers
            "spec/mailers/", "test/mailers/",
            
            // Jobs
            "spec/jobs/", "test/jobs/",
            
            // Channels
            "spec/channels/", "test/channels/",
            
            // Routing
            "spec/routing/", "test/routing/",
        ]
    }
} 