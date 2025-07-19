use regex::Regex;

pub struct GoPatterns {
    external_patterns: Vec<Regex>,
    cache_patterns: Vec<Regex>,
    extensions: Vec<String>,
}

impl GoPatterns {
    pub fn new() -> Self {
        let external_patterns = vec![
            // Go modules
            Regex::new(r"go\.mod$").unwrap(),
            Regex::new(r"go\.sum$").unwrap(),
            Regex::new(r"go\.work$").unwrap(),
            Regex::new(r"go\.work\.sum$").unwrap(),
            
            // Vendor directory
            Regex::new(r"vendor/").unwrap(),
            
            // Build outputs
            Regex::new(r"\.exe$").unwrap(),
            Regex::new(r"\.out$").unwrap(),
            
            // Test outputs
            Regex::new(r"\.test$").unwrap(),
            Regex::new(r"\.prof$").unwrap(),
            Regex::new(r"\.mem$").unwrap(),
            Regex::new(r"\.cpu$").unwrap(),
            
            // Coverage files
            Regex::new(r"\.cover$").unwrap(),
            Regex::new(r"coverage\.out$").unwrap(),
            Regex::new(r"coverage\.html$").unwrap(),
            
            // Binary outputs
            Regex::new(r"bin/").unwrap(),
            Regex::new(r"dist/").unwrap(),
            
            // Go-specific generated files
            Regex::new(r"\.pb\.go$").unwrap(),
            Regex::new(r"\.pb\.gw\.go$").unwrap(),
            Regex::new(r"bindata\.go$").unwrap(),
            Regex::new(r"_gen\.go$").unwrap(),
            Regex::new(r"_generated\.go$").unwrap(),
            
            // IDE files
            Regex::new(r"\.vscode/").unwrap(),
            Regex::new(r"\.idea/").unwrap(),
            
            // OS specific files
            Regex::new(r"\.DS_Store").unwrap(),
            Regex::new(r"Thumbs\.db").unwrap(),
            
            // Temporary files
            Regex::new(r"\.tmp$").unwrap(),
            Regex::new(r"\.swp$").unwrap(),
            Regex::new(r"\.swo$").unwrap(),
            Regex::new(r"~$").unwrap(),
            
            // Docker
            Regex::new(r"\.dockerignore$").unwrap(),
            
            // Air (live reload)
            Regex::new(r"\.air\.toml$").unwrap(),
            Regex::new(r"tmp/").unwrap(),
            
            // Delve debugger
            Regex::new(r"debug$").unwrap(),
            Regex::new(r"debug\.exe$").unwrap(),
            
            // Go workspace
            Regex::new(r"\.vscode/launch\.json$").unwrap(),
            
            // Fyne cross-compilation
            Regex::new(r"fyne-cross/").unwrap(),
            
            // Wire dependency injection
            Regex::new(r"wire_gen\.go$").unwrap(),
            
            // Swagger/OpenAPI
            Regex::new(r"docs/").unwrap(),
            
            // Mage build tool
            Regex::new(r"mage_output_file\.go$").unwrap(),
        ];

        let cache_patterns = vec![
            // Go module cache
            Regex::new(r"\.go/pkg/mod/").unwrap(),
            
            // Go build cache
            Regex::new(r"\.go/pkg/sumdb/").unwrap(),
            
            // Test cache
            Regex::new(r"\.go/pkg/testcache/").unwrap(),
            
            // IDE caches
            Regex::new(r"\.vscode/").unwrap(),
            Regex::new(r"\.idea/").unwrap(),
            
            // Air temporary files
            Regex::new(r"tmp/").unwrap(),
            
            // Build artifacts
            Regex::new(r"bin/").unwrap(),
            Regex::new(r"dist/").unwrap(),
            
            // Coverage files
            Regex::new(r"\.cover$").unwrap(),
            Regex::new(r"coverage\.").unwrap(),
            
            // Profile files
            Regex::new(r"\.prof$").unwrap(),
            Regex::new(r"\.mem$").unwrap(),
            Regex::new(r"\.cpu$").unwrap(),
            
            // Temporary files
            Regex::new(r"\.tmp$").unwrap(),
            Regex::new(r"\.temp$").unwrap(),
            
            // Log files
            Regex::new(r"\.log$").unwrap(),
            
            // Backup files
            Regex::new(r"\.bak$").unwrap(),
            Regex::new(r"\.backup$").unwrap(),
            
            // OS specific
            Regex::new(r"\.DS_Store").unwrap(),
            Regex::new(r"Thumbs\.db").unwrap(),
            
            // Editor files
            Regex::new(r"\.swp$").unwrap(),
            Regex::new(r"\.swo$").unwrap(),
            Regex::new(r"~$").unwrap(),
            
            // GoLand
            Regex::new(r"\.idea/").unwrap(),
            
            // VS Code
            Regex::new(r"\.vscode/").unwrap(),
            
            // Vim
            Regex::new(r"\.vim/").unwrap(),
            
            // Emacs
            Regex::new(r"\.emacs\.d/").unwrap(),
            
            // Sublime Text
            Regex::new(r"\.sublime-").unwrap(),
            
            // Atom
            Regex::new(r"\.atom/").unwrap(),
            
            // Brackets
            Regex::new(r"\.brackets\.json").unwrap(),
            
            // Nova
            Regex::new(r"\.nova/").unwrap(),
            
            // Zed
            Regex::new(r"\.zed/").unwrap(),
        ];

        let extensions = vec![
            // Go source files
            "go".to_string(),
            
            // Go modules
            "mod".to_string(),
            "sum".to_string(),
            "work".to_string(),
            
            // Go assembly
            "s".to_string(),
            
            // Go templates
            "tmpl".to_string(),
            "tpl".to_string(),
            "gotmpl".to_string(),
            
            // Go HTML templates
            "gohtml".to_string(),
            
            // Go text templates
            "gotxt".to_string(),
            
            // Protocol buffers
            "proto".to_string(),
            
            // YAML (common in Go projects)
            "yaml".to_string(),
            "yml".to_string(),
            
            // JSON (common in Go projects)
            "json".to_string(),
            
            // TOML (common in Go projects)
            "toml".to_string(),
            
            // Markdown
            "md".to_string(),
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
            // Go modules
            "go.mod", "go.sum", "go.work", "go.work.sum",
            
            // Main files
            "main.go", "cmd/main.go",
            
            // Build scripts
            "Makefile", "makefile", "build.sh", "build.bat",
            "build.ps1", "build.py", "build.go",
            
            // Mage build files
            "magefile.go", "mage.go",
            
            // Task runners
            "Taskfile.yml", "Taskfile.yaml",
            
            // Docker
            "Dockerfile", "docker-compose.yml", "docker-compose.yaml",
            ".dockerignore", "Dockerfile.dev", "Dockerfile.prod",
            
            // Air (live reload)
            ".air.toml", "air.conf",
            
            // Configuration files
            "config.yaml", "config.yml", "config.json",
            "config.toml", "settings.yaml", "app.yaml",
            
            // Environment files
            ".env", ".env.local", ".env.development", ".env.production",
            ".env.test", ".env.example",
            
            // CI/CD
            ".github/workflows/go.yml", ".github/workflows/test.yml",
            ".github/workflows/build.yml", ".github/workflows/release.yml",
            ".gitlab-ci.yml", "azure-pipelines.yml", "Jenkinsfile",
            "bitbucket-pipelines.yml", "circle.yml", ".circleci/config.yml",
            ".travis.yml", "appveyor.yml", "cloudbuild.yaml",
            
            // Testing
            "go.test", "coverage.out", "coverage.html",
            "benchmark.out", "profile.out",
            
            // Linting and formatting
            ".golangci.yml", ".golangci.yaml", "golangci.yml",
            ".gofmt", ".goimports", ".golint",
            
            // Documentation
            "README.md", "CHANGELOG.md", "CONTRIBUTING.md",
            "LICENSE", "LICENSE.md", "COPYING",
            "doc.go", "docs.go",
            
            // Swagger/OpenAPI
            "swagger.yaml", "swagger.yml", "swagger.json",
            "openapi.yaml", "openapi.yml", "openapi.json",
            "api.yaml", "api.yml", "api.json",
            
            // Protocol Buffers
            "*.proto", "buf.yaml", "buf.yml", "buf.gen.yaml",
            "buf.work.yaml", "buf.lock",
            
            // gRPC
            "grpc.proto", "service.proto",
            
            // Database migrations
            "migrate.go", "schema.sql", "migrations/",
            
            // Kubernetes
            "k8s.yaml", "k8s.yml", "kubernetes.yaml",
            "deployment.yaml", "service.yaml", "ingress.yaml",
            "configmap.yaml", "secret.yaml",
            
            // Helm
            "Chart.yaml", "values.yaml", "values.yml",
            "requirements.yaml", "requirements.lock",
            
            // Terraform
            "main.tf", "variables.tf", "outputs.tf",
            "terraform.tfvars", "terraform.tfstate",
            
            // Monitoring
            "prometheus.yml", "grafana.json",
            "jaeger.yaml", "zipkin.yaml",
            
            // Logging
            "logrus.yaml", "zap.yaml", "zerolog.yaml",
            
            // Web frameworks
            "gin.go", "echo.go", "fiber.go", "chi.go",
            "gorilla.go", "httprouter.go", "mux.go",
            
            // ORM
            "gorm.go", "ent.go", "sqlx.go", "xorm.go",
            
            // Message queues
            "rabbitmq.go", "kafka.go", "nats.go", "redis.go",
            
            // Caching
            "redis.go", "memcached.go", "cache.go",
            
            // Authentication
            "jwt.go", "oauth.go", "auth.go", "session.go",
            
            // Validation
            "validator.go", "validate.go",
            
            // Serialization
            "json.go", "xml.go", "yaml.go", "toml.go",
            "protobuf.go", "msgpack.go", "avro.go",
            
            // HTTP clients
            "client.go", "http.go", "rest.go", "graphql.go",
            
            // Utilities
            "utils.go", "helpers.go", "common.go", "shared.go",
            
            // Constants
            "constants.go", "const.go", "vars.go",
            
            // Errors
            "errors.go", "error.go", "exceptions.go",
            
            // Types
            "types.go", "models.go", "structs.go", "entities.go",
            
            // Interfaces
            "interfaces.go", "interface.go", "contracts.go",
            
            // Handlers
            "handlers.go", "handler.go", "routes.go", "router.go",
            
            // Middleware
            "middleware.go", "middlewares.go",
            
            // Services
            "service.go", "services.go", "business.go",
            
            // Repositories
            "repository.go", "repositories.go", "repo.go",
            "storage.go", "store.go", "dao.go",
            
            // Controllers
            "controller.go", "controllers.go", "ctrl.go",
            
            // Views
            "view.go", "views.go", "templates.go",
            
            // Models
            "model.go", "models.go", "entity.go", "entities.go",
            
            // DTOs
            "dto.go", "dtos.go", "request.go", "response.go",
            
            // Configs
            "config.go", "configuration.go", "settings.go",
            
            // Database
            "db.go", "database.go", "connection.go", "conn.go",
            
            // Migrations
            "migration.go", "migrations.go", "migrate.go",
            
            // Seeds
            "seed.go", "seeds.go", "seeder.go", "seeders.go",
            
            // Tests
            "*_test.go", "test.go", "tests.go", "testing.go",
            "benchmark.go", "benchmarks.go", "example.go",
            
            // Mocks
            "mock.go", "mocks.go", "fake.go", "fakes.go",
            "stub.go", "stubs.go",
            
            // Fixtures
            "fixture.go", "fixtures.go", "testdata.go",
            
            // Factories
            "factory.go", "factories.go", "builder.go", "builders.go",
            
            // Generators
            "generator.go", "generators.go", "generate.go",
            
            // Parsers
            "parser.go", "parsers.go", "parse.go",
            
            // Validators
            "validator.go", "validators.go", "validation.go",
            
            // Serializers
            "serializer.go", "serializers.go", "serialize.go",
            
            // Transformers
            "transformer.go", "transformers.go", "transform.go",
            
            // Converters
            "converter.go", "converters.go", "convert.go",
            
            // Mappers
            "mapper.go", "mappers.go", "mapping.go",
            
            // Filters
            "filter.go", "filters.go", "filtering.go",
            
            // Sorters
            "sorter.go", "sorters.go", "sorting.go",
            
            // Pagination
            "pagination.go", "paginate.go", "paging.go",
            
            // Search
            "search.go", "searcher.go", "searching.go",
            
            // Cache
            "cache.go", "caching.go", "cached.go",
            
            // Queue
            "queue.go", "queues.go", "job.go", "jobs.go",
            "worker.go", "workers.go", "task.go", "tasks.go",
            
            // Scheduler
            "scheduler.go", "schedule.go", "cron.go",
            
            // Events
            "event.go", "events.go", "eventbus.go",
            "dispatcher.go", "listener.go", "listeners.go",
            
            // Notifications
            "notification.go", "notifications.go", "notify.go",
            
            // Email
            "email.go", "mail.go", "mailer.go", "smtp.go",
            
            // SMS
            "sms.go", "text.go", "messaging.go",
            
            // Push notifications
            "push.go", "fcm.go", "apns.go",
            
            // File handling
            "file.go", "files.go", "filesystem.go", "storage.go",
            "upload.go", "uploads.go", "download.go", "downloads.go",
            
            // Image processing
            "image.go", "images.go", "resize.go", "thumbnail.go",
            
            // PDF
            "pdf.go", "pdfs.go", "report.go", "reports.go",
            
            // Excel
            "excel.go", "xlsx.go", "csv.go", "spreadsheet.go",
            
            // Compression
            "compress.go", "compression.go", "zip.go", "gzip.go",
            
            // Encryption
            "encrypt.go", "encryption.go", "crypto.go", "security.go",
            "hash.go", "hashing.go", "jwt.go", "token.go", "tokens.go",
            
            // Logging
            "log.go", "logger.go", "logging.go", "logs.go",
            
            // Metrics
            "metrics.go", "metric.go", "monitoring.go", "stats.go",
            
            // Health checks
            "health.go", "healthcheck.go", "ping.go", "status.go",
            
            // Rate limiting
            "ratelimit.go", "throttle.go", "limiter.go",
            
            // Circuit breaker
            "circuit.go", "breaker.go", "hystrix.go",
            
            // Retry
            "retry.go", "backoff.go", "exponential.go",
            
            // Timeout
            "timeout.go", "deadline.go", "context.go",
            
            // Graceful shutdown
            "shutdown.go", "graceful.go", "signal.go", "signals.go",
            
            // Profiling
            "profile.go", "profiling.go", "pprof.go",
            
            // Benchmarking
            "benchmark.go", "benchmarks.go", "bench.go",
            
            // Load testing
            "load.go", "loadtest.go", "stress.go", "stresstest.go",
            
            // Version
            "version.go", "versions.go", "build.go", "info.go",
            
            // CLI
            "cli.go", "cmd.go", "command.go", "commands.go",
            "flag.go", "flags.go", "args.go", "arguments.go",
            
            // Server
            "server.go", "servers.go", "app.go", "application.go",
            
            // Client
            "client.go", "clients.go", "api.go", "apis.go",
            
            // Proxy
            "proxy.go", "proxies.go", "gateway.go", "gateways.go",
            
            // Load balancer
            "loadbalancer.go", "balancer.go", "lb.go",
            
            // Service discovery
            "discovery.go", "registry.go", "consul.go", "etcd.go",
            
            // Configuration management
            "consul.go", "etcd.go", "vault.go", "k8s.go",
            
            // Tracing
            "trace.go", "tracing.go", "jaeger.go", "zipkin.go",
            "opentracing.go", "opentelemetry.go",
            
            // Deployment
            "deploy.go", "deployment.go", "k8s.go", "kubernetes.go",
            "docker.go", "container.go", "containers.go",
            
            // Infrastructure
            "infrastructure.go", "infra.go", "terraform.go",
            "cloudformation.go", "aws.go", "gcp.go", "azure.go",
            
            // Monitoring
            "prometheus.go", "grafana.go", "datadog.go", "newrelic.go",
            
            // Alerting
            "alert.go", "alerts.go", "pagerduty.go", "slack.go",
            
            // Backup
            "backup.go", "backups.go", "restore.go", "recovery.go",
            
            // Disaster recovery
            "disaster.go", "recovery.go", "failover.go", "replication.go",
            
            // Performance
            "performance.go", "perf.go", "optimization.go", "optimize.go",
            
            // Scalability
            "scale.go", "scaling.go", "autoscale.go", "horizontal.go",
            "vertical.go", "cluster.go", "clusters.go",
            
            // Reliability
            "reliability.go", "reliable.go", "resilience.go", "fault.go",
            "tolerance.go", "availability.go", "uptime.go",
            
            // Maintenance
            "maintenance.go", "maintain.go", "cleanup.go", "clean.go",
            "garbage.go", "gc.go", "housekeeping.go",
            
            // Migration
            "migration.go", "migrations.go", "migrate.go", "upgrade.go",
            "downgrade.go", "rollback.go", "revert.go",
            
            // Internationalization
            "i18n.go", "l10n.go", "locale.go", "locales.go",
            "translation.go", "translations.go", "language.go", "languages.go",
            
            // Accessibility
            "a11y.go", "accessibility.go", "wcag.go", "aria.go",
            
            // SEO
            "seo.go", "sitemap.go", "robots.go", "meta.go",
            
            // Analytics
            "analytics.go", "tracking.go", "ga.go", "gtm.go",
            
            // A/B testing
            "ab.go", "abtest.go", "experiment.go", "experiments.go",
            "feature.go", "features.go", "flag.go", "flags.go",
            
            // Social media
            "social.go", "facebook.go", "twitter.go", "instagram.go",
            "linkedin.go", "youtube.go", "tiktok.go",
            
            // Payment
            "payment.go", "payments.go", "stripe.go", "paypal.go",
            "square.go", "braintree.go", "checkout.go", "billing.go",
            
            // Subscription
            "subscription.go", "subscriptions.go", "plan.go", "plans.go",
            "tier.go", "tiers.go", "pricing.go", "invoice.go", "invoices.go",
            
            // Inventory
            "inventory.go", "stock.go", "warehouse.go", "product.go", "products.go",
            "catalog.go", "category.go", "categories.go",
            
            // Order management
            "order.go", "orders.go", "cart.go", "carts.go",
            "checkout.go", "fulfillment.go", "shipping.go",
            
            // Customer management
            "customer.go", "customers.go", "user.go", "users.go",
            "account.go", "accounts.go", "profile.go", "profiles.go",
            
            // Content management
            "content.go", "cms.go", "page.go", "pages.go",
            "post.go", "posts.go", "article.go", "articles.go",
            "blog.go", "news.go", "media.go",
            
            // Forum
            "forum.go", "forums.go", "thread.go", "threads.go",
            "topic.go", "topics.go", "comment.go", "comments.go",
            "reply.go", "replies.go", "vote.go", "votes.go",
            
            // Chat
            "chat.go", "message.go", "messages.go", "room.go", "rooms.go",
            "channel.go", "channels.go", "websocket.go", "ws.go",
            
            // Real-time
            "realtime.go", "live.go", "streaming.go", "stream.go", "streams.go",
            "sse.go", "websocket.go", "ws.go", "socket.go", "sockets.go",
            
            // Game development
            "game.go", "games.go", "player.go", "players.go",
            "match.go", "matches.go", "tournament.go", "tournaments.go",
            "leaderboard.go", "score.go", "scores.go",
            
            // IoT
            "iot.go", "device.go", "devices.go", "sensor.go", "sensors.go",
            "actuator.go", "actuators.go", "mqtt.go", "coap.go",
            
            // Blockchain
            "blockchain.go", "bitcoin.go", "ethereum.go", "smart.go",
            "contract.go", "contracts.go", "wallet.go", "wallets.go",
            "transaction.go", "transactions.go", "mining.go",
            
            // Machine learning
            "ml.go", "ai.go", "neural.go", "network.go", "networks.go",
            "model.go", "models.go", "training.go", "inference.go",
            "prediction.go", "predictions.go", "classification.go",
            
            // Data science
            "data.go", "dataset.go", "datasets.go", "analysis.go",
            "visualization.go", "chart.go", "charts.go", "graph.go", "graphs.go",
            "statistics.go", "stats.go", "math.go", "algorithm.go", "algorithms.go",
            
            // Big data
            "bigdata.go", "hadoop.go", "spark.go", "kafka.go",
            "elasticsearch.go", "solr.go", "lucene.go", "search.go",
            
            // Time series
            "timeseries.go", "time.go", "series.go", "metric.go", "metrics.go",
            "influxdb.go", "prometheus.go", "grafana.go",
            
            // Geolocation
            "geo.go", "location.go", "locations.go", "map.go", "maps.go",
            "gps.go", "coordinate.go", "coordinates.go", "distance.go",
            
            // Weather
            "weather.go", "forecast.go", "climate.go", "temperature.go",
            "humidity.go", "pressure.go", "wind.go", "rain.go",
            
            // Finance
            "finance.go", "financial.go", "accounting.go", "budget.go", "budgets.go",
            "expense.go", "expenses.go", "income.go", "revenue.go",
            "profit.go", "loss.go", "tax.go", "taxes.go",
            
            // HR
            "hr.go", "human.go", "resources.go", "employee.go", "employees.go",
            "payroll.go", "salary.go", "salaries.go", "benefit.go", "benefits.go",
            "leave.go", "attendance.go", "performance.go", "review.go", "reviews.go",
            
            // Legal
            "legal.go", "law.go", "compliance.go", "regulation.go", "regulations.go",
            "policy.go", "policies.go", "contract.go", "contracts.go",
            "agreement.go", "agreements.go", "license.go", "licenses.go",
            
            // Education
            "education.go", "school.go", "schools.go", "student.go", "students.go",
            "teacher.go", "teachers.go", "course.go", "courses.go",
            "lesson.go", "lessons.go", "exam.go", "exams.go",
            "grade.go", "grades.go", "assignment.go", "assignments.go",
            
            // Healthcare
            "health.go", "healthcare.go", "medical.go", "patient.go", "patients.go",
            "doctor.go", "doctors.go", "appointment.go", "appointments.go",
            "prescription.go", "prescriptions.go", "diagnosis.go", "treatment.go",
            
            // Travel
            "travel.go", "trip.go", "trips.go", "booking.go", "bookings.go",
            "hotel.go", "hotels.go", "flight.go", "flights.go",
            "car.go", "cars.go", "rental.go", "rentals.go",
            
            // Food
            "food.go", "restaurant.go", "restaurants.go", "menu.go", "menus.go",
            "recipe.go", "recipes.go", "ingredient.go", "ingredients.go",
            "nutrition.go", "diet.go", "meal.go", "meals.go",
            
            // Sports
            "sports.go", "sport.go", "team.go", "teams.go",
            "athlete.go", "athletes.go", "competition.go", "competitions.go",
            "season.go", "seasons.go", "league.go", "leagues.go",
            
            // Entertainment
            "entertainment.go", "movie.go", "movies.go", "tv.go", "show.go", "shows.go",
            "music.go", "song.go", "songs.go", "album.go", "albums.go",
            "artist.go", "artists.go", "book.go", "books.go",
            
            // News
            "news.go", "article.go", "articles.go", "story.go", "stories.go",
            "headline.go", "headlines.go", "reporter.go", "reporters.go",
            "editor.go", "editors.go", "publisher.go", "publishers.go",
            
            // Weather
            "weather.go", "forecast.go", "climate.go", "temperature.go",
            "humidity.go", "pressure.go", "wind.go", "rain.go", "snow.go",
            
            // Transportation
            "transport.go", "transportation.go", "vehicle.go", "vehicles.go",
            "bus.go", "buses.go", "train.go", "trains.go",
            "subway.go", "metro.go", "taxi.go", "uber.go", "lyft.go",
            
            // Real estate
            "realestate.go", "property.go", "properties.go", "house.go", "houses.go",
            "apartment.go", "apartments.go", "rent.go", "rental.go", "rentals.go",
            "sale.go", "sales.go", "mortgage.go", "mortgages.go",
            
            // Insurance
            "insurance.go", "policy.go", "policies.go", "claim.go", "claims.go",
            "premium.go", "premiums.go", "coverage.go", "deductible.go",
            
            // Banking
            "bank.go", "banking.go", "account.go", "accounts.go",
            "transaction.go", "transactions.go", "transfer.go", "transfers.go",
            "loan.go", "loans.go", "credit.go", "debit.go",
            
            // Investment
            "investment.go", "investments.go", "portfolio.go", "portfolios.go",
            "stock.go", "stocks.go", "bond.go", "bonds.go",
            "fund.go", "funds.go", "etf.go", "etfs.go",
            
            // Cryptocurrency
            "crypto.go", "cryptocurrency.go", "bitcoin.go", "ethereum.go",
            "blockchain.go", "wallet.go", "wallets.go", "exchange.go", "exchanges.go",
            "trading.go", "trade.go", "trades.go", "market.go", "markets.go",
            
            // Retail
            "retail.go", "store.go", "stores.go", "shop.go", "shops.go",
            "product.go", "products.go", "inventory.go", "stock.go",
            "sale.go", "sales.go", "discount.go", "discounts.go",
            
            // Manufacturing
            "manufacturing.go", "factory.go", "factories.go", "production.go",
            "assembly.go", "quality.go", "control.go", "testing.go",
            "inspection.go", "maintenance.go", "repair.go",
            
            // Logistics
            "logistics.go", "supply.go", "chain.go", "warehouse.go", "warehouses.go",
            "distribution.go", "delivery.go", "deliveries.go", "shipping.go",
            "tracking.go", "route.go", "routes.go",
            
            // Agriculture
            "agriculture.go", "farm.go", "farms.go", "crop.go", "crops.go",
            "livestock.go", "animal.go", "animals.go", "harvest.go",
            "irrigation.go", "fertilizer.go", "pesticide.go",
            
            // Energy
            "energy.go", "power.go", "electricity.go", "solar.go",
            "wind.go", "nuclear.go", "coal.go", "oil.go", "gas.go",
            "renewable.go", "grid.go", "utility.go", "utilities.go",
            
            // Environment
            "environment.go", "environmental.go", "climate.go", "carbon.go",
            "emission.go", "emissions.go", "pollution.go", "waste.go",
            "recycling.go", "sustainability.go", "green.go", "eco.go",
            
            // Government
            "government.go", "gov.go", "public.go", "citizen.go", "citizens.go",
            "voting.go", "election.go", "elections.go", "democracy.go",
            "legislation.go", "law.go", "laws.go", "regulation.go", "regulations.go",
            
            // Non-profit
            "nonprofit.go", "charity.go", "donation.go", "donations.go",
            "volunteer.go", "volunteers.go", "fundraising.go", "grant.go", "grants.go",
            "foundation.go", "foundations.go", "cause.go", "causes.go",
        ]
    }
} 