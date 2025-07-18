use regex::Regex;

pub struct DotnetPatterns {
    external_patterns: Vec<Regex>,
    cache_patterns: Vec<Regex>,
    extensions: Vec<String>,
}

impl DotnetPatterns {
    pub fn new() -> Self {
        let external_patterns = vec![
            // Build outputs
            Regex::new(r"bin/").unwrap(),
            Regex::new(r"obj/").unwrap(),
            
            // Compiled assemblies
            Regex::new(r"\.dll$").unwrap(),
            Regex::new(r"\.exe$").unwrap(),
            Regex::new(r"\.pdb$").unwrap(),
            Regex::new(r"\.lib$").unwrap(),
            Regex::new(r"\.exp$").unwrap(),
            Regex::new(r"\.ilk$").unwrap(),
            
            // Package management
            Regex::new(r"packages/").unwrap(),
            Regex::new(r"\.nuget/").unwrap(),
            Regex::new(r"\.nupkg$").unwrap(),
            Regex::new(r"\.snupkg$").unwrap(),
            
            // Testing outputs
            Regex::new(r"TestResults/").unwrap(),
            Regex::new(r"\.trx$").unwrap(),
            Regex::new(r"\.coverage$").unwrap(),
            Regex::new(r"\.coveragexml$").unwrap(),
            
            // IDE generated files
            Regex::new(r"\.vs/").unwrap(),
            Regex::new(r"\.vscode/").unwrap(),
            Regex::new(r"\.idea/").unwrap(),
            Regex::new(r"\.suo$").unwrap(),
            Regex::new(r"\.user$").unwrap(),
            Regex::new(r"\.userosscache$").unwrap(),
            Regex::new(r"\.sln\.docstates$").unwrap(),
            
            // Publish outputs
            Regex::new(r"publish/").unwrap(),
            Regex::new(r"\.publish/").unwrap(),
            Regex::new(r"PublishProfiles/").unwrap(),
            
            // ASP.NET
            Regex::new(r"wwwroot/").unwrap(),
            Regex::new(r"\.publish\.xml$").unwrap(),
            
            // Entity Framework
            Regex::new(r"Migrations/").unwrap(),
            
            // Xamarin
            Regex::new(r"\.dSYM/").unwrap(),
            Regex::new(r"\.app/").unwrap(),
            Regex::new(r"\.ipa$").unwrap(),
            Regex::new(r"\.apk$").unwrap(),
            
            // Unity
            Regex::new(r"Library/").unwrap(),
            Regex::new(r"Temp/").unwrap(),
            Regex::new(r"\.tmp$").unwrap(),
            
            // Resharper
            Regex::new(r"_ReSharper").unwrap(),
            Regex::new(r"\.resharper").unwrap(),
            
            // dotCover
            Regex::new(r"\.dotCover$").unwrap(),
            
            // NCrunch
            Regex::new(r"\.ncrunch").unwrap(),
            Regex::new(r"_NCrunch_").unwrap(),
            
            // MightyMoose
            Regex::new(r"\.mm\.").unwrap(),
            Regex::new(r"AutoTest\.Net/").unwrap(),
            
            // Web deploy package
            Regex::new(r"\.zip$").unwrap(),
            
            // Backup files
            Regex::new(r"\.bak$").unwrap(),
            Regex::new(r"\.cache$").unwrap(),
            
            // StyleCop
            Regex::new(r"StyleCopReport\.xml$").unwrap(),
            
            // Files built by Visual Studio
            Regex::new(r"\.pidb$").unwrap(),
            Regex::new(r"\.svclog$").unwrap(),
            Regex::new(r"\.scc$").unwrap(),
            
            // Chutzpah Test files
            Regex::new(r"_Chutzpah").unwrap(),
            
            // Visual C++ cache files
            Regex::new(r"ipch/").unwrap(),
            Regex::new(r"\.aps$").unwrap(),
            Regex::new(r"\.ncb$").unwrap(),
            Regex::new(r"\.opendb$").unwrap(),
            Regex::new(r"\.opensdf$").unwrap(),
            Regex::new(r"\.sdf$").unwrap(),
            Regex::new(r"\.cachefile$").unwrap(),
            Regex::new(r"\.VC\.db$").unwrap(),
            Regex::new(r"\.VC\.VC\.opendb$").unwrap(),
            
            // Visual Studio profiler
            Regex::new(r"\.psess$").unwrap(),
            Regex::new(r"\.vsp$").unwrap(),
            Regex::new(r"\.vspx$").unwrap(),
            Regex::new(r"\.sap$").unwrap(),
            
            // TFS 2012 Local Workspace
            Regex::new(r"\$tf/").unwrap(),
            
            // Guidance Automation Toolkit
            Regex::new(r"\.gpState$").unwrap(),
            
            // ReSharper is a .NET coding add-in
            Regex::new(r"_ReSharper\.Caches/").unwrap(),
            
            // TeamCity is a build add-in
            Regex::new(r"\.teamcity/").unwrap(),
            
            // DotCover is a Code Coverage Tool
            Regex::new(r"\.dotCover$").unwrap(),
            
            // Visual Studio code coverage results
            Regex::new(r"\.coverage$").unwrap(),
            Regex::new(r"\.coveragexml$").unwrap(),
            
            // NCrunch
            Regex::new(r"\.ncrunch").unwrap(),
            Regex::new(r"_NCrunch_").unwrap(),
            
            // MightyMoose
            Regex::new(r"\.mm\.").unwrap(),
            Regex::new(r"AutoTest\.Net/").unwrap(),
            
            // Installshield output folder
            Regex::new(r"Express/").unwrap(),
            
            // DocProject is a documentation generator add-in
            Regex::new(r"DocProject/buildhelp/").unwrap(),
            Regex::new(r"DocProject/Help/").unwrap(),
            Regex::new(r"DocProject/Help/Html2/").unwrap(),
            Regex::new(r"DocProject/Help/html/").unwrap(),
            
            // Click-Once directory
            Regex::new(r"publish/").unwrap(),
            
            // Publish Web Output
            Regex::new(r"\.Publish\.xml$").unwrap(),
            
            // Windows Azure Build Output
            Regex::new(r"csx/").unwrap(),
            Regex::new(r"\.build\.csdef$").unwrap(),
            
            // Windows Store app package directory
            Regex::new(r"AppPackages/").unwrap(),
            
            // Others
            Regex::new(r"\.Cache$").unwrap(),
            Regex::new(r"ClientBin/").unwrap(),
            Regex::new(r"\[Ss\]tyle[Cc]op\.\*$").unwrap(),
            Regex::new(r"~\$").unwrap(),
            Regex::new(r"\.dbmdl$").unwrap(),
            Regex::new(r"\.dbproj\.schemaview$").unwrap(),
            Regex::new(r"\.pfx$").unwrap(),
            Regex::new(r"\.publishsettings$").unwrap(),
            Regex::new(r"orleans\.codegen\.cs$").unwrap(),
            
            // RIA/Silverlight projects
            Regex::new(r"Generated_Code/").unwrap(),
            
            // Backup & report files from converting an old project file
            Regex::new(r"_UpgradeReport_Files/").unwrap(),
            Regex::new(r"Backup").unwrap(),
            Regex::new(r"UpgradeLog").unwrap(),
            
            // SQL Server files
            Regex::new(r"\.mdf$").unwrap(),
            Regex::new(r"\.ldf$").unwrap(),
            
            // Business Intelligence projects
            Regex::new(r"\.rdl\.data$").unwrap(),
            Regex::new(r"\.bim\.layout$").unwrap(),
            Regex::new(r"\.bim_").unwrap(),
            
            // Microsoft Fakes
            Regex::new(r"FakesAssemblies/").unwrap(),
            
            // GhostDoc plugin setting file
            Regex::new(r"\.GhostDoc\.xml$").unwrap(),
            
            // Node.js Tools for Visual Studio
            Regex::new(r"\.ntvs_analysis\.dat$").unwrap(),
            Regex::new(r"\.ntvs_analysis\.dat\.tmp$").unwrap(),
            
            // Visual Studio 6 build log
            Regex::new(r"\.plg$").unwrap(),
            
            // Visual Studio 6 workspace options file
            Regex::new(r"\.opt$").unwrap(),
            
            // Visual Studio 6 auto-generated workspace file
            Regex::new(r"\.vbw$").unwrap(),
            
            // Visual Studio LightSwitch build output
            Regex::new(r"\.HTMLClient/GeneratedArtifacts/").unwrap(),
            Regex::new(r"\.DesktopClient/GeneratedArtifacts/").unwrap(),
            Regex::new(r"\.DesktopClient/ModelManifest\.xml$").unwrap(),
            Regex::new(r"\.Server/GeneratedArtifacts/").unwrap(),
            Regex::new(r"\.Server/ModelManifest\.xml$").unwrap(),
            Regex::new(r"_Pvt_Extensions/").unwrap(),
            
            // Paket dependency manager
            Regex::new(r"\.paket/").unwrap(),
            Regex::new(r"paket-files/").unwrap(),
            
            // FAKE - F# Make
            Regex::new(r"\.fake/").unwrap(),
            
            // JetBrains Rider
            Regex::new(r"\.idea/").unwrap(),
            
            // CodeRush
            Regex::new(r"\.cr/").unwrap(),
            
            // Python Tools for Visual Studio (PTVS)
            Regex::new(r"__pycache__/").unwrap(),
            Regex::new(r"\.pyc$").unwrap(),
            
            // Cake - Uncomment if you are using it
            Regex::new(r"tools/").unwrap(),
            
            // Tabs Studio
            Regex::new(r"\.tss$").unwrap(),
            
            // Telerik's JustMock configuration file
            Regex::new(r"\.jmconfig$").unwrap(),
            
            // BizTalk build output
            Regex::new(r"\.btp\.cs$").unwrap(),
            Regex::new(r"\.btm\.cs$").unwrap(),
            Regex::new(r"\.odx\.cs$").unwrap(),
            Regex::new(r"\.xsd\.cs$").unwrap(),
            
            // OpenCover UI analysis results
            Regex::new(r"OpenCover/").unwrap(),
            
            // Azure Stream Analytics local run output
            Regex::new(r"ASALocalRun/").unwrap(),
            
            // MSBuild Binary and Structured Log
            Regex::new(r"\.binlog$").unwrap(),
            
            // NVidia Nsight GPU debugger configuration file
            Regex::new(r"\.nvuser$").unwrap(),
            
            // MFractors (Xamarin productivity tool) working folder
            Regex::new(r"\.mfractor/").unwrap(),
        ];

        let cache_patterns = vec![
            // NuGet caches
            Regex::new(r"\.nuget/packages/").unwrap(),
            Regex::new(r"\.nuget/v3-cache/").unwrap(),
            Regex::new(r"\.nuget/tmp-cache/").unwrap(),
            
            // MSBuild caches
            Regex::new(r"obj/").unwrap(),
            Regex::new(r"\.vs/").unwrap(),
            
            // Visual Studio caches
            Regex::new(r"\.vscode/").unwrap(),
            Regex::new(r"\.idea/").unwrap(),
            
            // ReSharper caches
            Regex::new(r"_ReSharper\.Caches/").unwrap(),
            
            // dotCover caches
            Regex::new(r"\.dotCover$").unwrap(),
            
            // NCrunch caches
            Regex::new(r"\.ncrunch").unwrap(),
            Regex::new(r"_NCrunch_").unwrap(),
            
            // Test caches
            Regex::new(r"TestResults/").unwrap(),
            
            // Temporary files
            Regex::new(r"\.tmp$").unwrap(),
            Regex::new(r"\.cache$").unwrap(),
            
            // Publish caches
            Regex::new(r"\.publish/").unwrap(),
            
            // Entity Framework caches
            Regex::new(r"\.ef/").unwrap(),
            
            // Xamarin caches
            Regex::new(r"\.dSYM/").unwrap(),
            
            // Unity caches
            Regex::new(r"Library/").unwrap(),
            Regex::new(r"Temp/").unwrap(),
            
            // Paket caches
            Regex::new(r"\.paket/").unwrap(),
            
            // FAKE caches
            Regex::new(r"\.fake/").unwrap(),
            
            // JetBrains Rider caches
            Regex::new(r"\.idea/").unwrap(),
            
            // CodeRush caches
            Regex::new(r"\.cr/").unwrap(),
            
            // Python Tools for Visual Studio caches
            Regex::new(r"__pycache__/").unwrap(),
            
            // Cake caches
            Regex::new(r"tools/").unwrap(),
            
            // OpenCover caches
            Regex::new(r"OpenCover/").unwrap(),
            
            // Azure Stream Analytics caches
            Regex::new(r"ASALocalRun/").unwrap(),
            
            // MFractors caches
            Regex::new(r"\.mfractor/").unwrap(),
        ];

        let extensions = vec![
            // C# files
            "cs".to_string(), "csx".to_string(),
            
            // VB.NET files
            "vb".to_string(),
            
            // F# files
            "fs".to_string(), "fsx".to_string(), "fsi".to_string(),
            
            // Project files
            "csproj".to_string(), "fsproj".to_string(), "vbproj".to_string(),
            "vcxproj".to_string(), "vcproj".to_string(),
            
            // Solution files
            "sln".to_string(),
            
            // Package files
            "nuspec".to_string(),
            
            // Resource files
            "resx".to_string(), "resw".to_string(), "resources".to_string(),
            
            // Web files
            "aspx".to_string(), "ascx".to_string(), "asax".to_string(),
            "ashx".to_string(), "asmx".to_string(),
            "razor".to_string(), "cshtml".to_string(), "vbhtml".to_string(),
            
            // XAML files
            "xaml".to_string(),
            
            // Configuration files
            "config".to_string(), "settings".to_string(),
            
            // Database files
            "edmx".to_string(), "dbml".to_string(),
            
            // Deployment files
            "targets".to_string(), "props".to_string(),
            
            // Assembly info
            "asm".to_string(),
            
            // T4 templates
            "tt".to_string(), "ttinclude".to_string(),
            
            // StyleCop
            "stylecop".to_string(),
            
            // WiX
            "wxs".to_string(), "wxi".to_string(),
            
            // MSBuild
            "msbuild".to_string(),
            
            // PowerShell
            "ps1".to_string(), "psm1".to_string(), "psd1".to_string(),
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
            // Solution and project files
            "*.sln", "*.csproj", "*.fsproj", "*.vbproj",
            "*.vcxproj", "*.vcproj", "*.proj",
            
            // Package management
            "packages.config", "*.nuspec", "nuget.config",
            "Directory.Build.props", "Directory.Build.targets",
            "Directory.Packages.props", "global.json",
            
            // MSBuild
            "*.targets", "*.props", "*.tasks",
            
            // Configuration files
            "app.config", "web.config", "machine.config",
            "appsettings.json", "appsettings.Development.json",
            "appsettings.Production.json", "appsettings.Staging.json",
            "launchSettings.json", "bundleconfig.json",
            
            // ASP.NET Core
            "Program.cs", "Startup.cs", "*.razor",
            "_ViewStart.cshtml", "_ViewImports.cshtml",
            "_Layout.cshtml", "*.cshtml", "*.vbhtml",
            
            // Entity Framework
            "*.edmx", "*.dbml", "DbContext.cs",
            
            // Testing
            "*.runsettings", "*.testsettings",
            
            // Deployment
            "*.pubxml", "*.wpp.targets", "*.deploy.cmd",
            "Dockerfile", "docker-compose.yml", "docker-compose.yaml",
            
            // CI/CD
            ".github/workflows/dotnet.yml", ".github/workflows/build.yml",
            "azure-pipelines.yml", "appveyor.yml",
            "GitVersion.yml", "version.json",
            
            // Code analysis
            "*.ruleset", "StyleCop.json", "editorconfig",
            ".editorconfig", "omnisharp.json",
            
            // Documentation
            "README.md", "CHANGELOG.md", "LICENSE",
            "*.shfbproj", "*.ndoc",
            
            // Xamarin
            "Info.plist", "AndroidManifest.xml",
            "*.storyboard", "*.xib",
            
            // Unity
            "*.unity", "*.prefab", "*.mat", "*.asset",
            "ProjectSettings/", "UserSettings/",
            
            // WPF/WinForms
            "*.xaml", "*.resx", "*.settings",
            
            // WiX
            "*.wxs", "*.wxi", "*.wxl",
            
            // PowerShell
            "*.ps1", "*.psm1", "*.psd1",
            
            // Azure Functions
            "host.json", "local.settings.json",
            "function.json", "proxies.json",
            
            // Blazor
            "_Imports.razor", "App.razor",
            "MainLayout.razor", "NavMenu.razor",
            
            // Service Fabric
            "ServiceManifest.xml", "ApplicationManifest.xml",
            
            // Orleans
            "*.grain", "*.silo",
            
            // Paket
            "paket.dependencies", "paket.lock",
            "paket.references", "paket.template",
            
            // FAKE
            "build.fsx", "fake.cmd", "fake.sh",
            
            // Cake
            "build.cake", "cake.config",
            
            // BenchmarkDotNet
            "*.benchmarks",
            
            // Roslyn Analyzers
            "*.analyzer",
            
            // Source generators
            "*.generator",
            
            // gRPC
            "*.proto",
            
            // OpenAPI/Swagger
            "swagger.json", "openapi.json",
            
            // SignalR
            "*.hub",
            
            // Minimal APIs
            "*.endpoints",
            
            // Health checks
            "*.healthchecks",
            
            // Background services
            "*.backgroundservice",
            
            // Middleware
            "*.middleware",
            
            // Filters
            "*.filter",
            
            // Custom attributes
            "*.attribute",
            
            // Extension methods
            "*.extensions",
            
            // Converters
            "*.converter",
            
            // Validators
            "*.validator",
            
            // Mappers
            "*.mapper",
            
            // Repositories
            "*.repository",
            
            // Services
            "*.service",
            
            // Controllers
            "*.controller",
            
            // Models
            "*.model", "*.dto", "*.viewmodel",
            
            // Data contexts
            "*.context",
            
            // Migrations
            "*.migration",
            
            // Seeders
            "*.seeder",
            
            // Policies
            "*.policy",
            
            // Handlers
            "*.handler",
            
            // Options
            "*.options",
            
            // Constants
            "*.constants",
            
            // Enums
            "*.enum",
            
            // Interfaces
            "*.interface",
            
            // Abstractions
            "*.abstraction",
            
            // Implementations
            "*.implementation",
            
            // Factories
            "*.factory",
            
            // Builders
            "*.builder",
            
            // Strategies
            "*.strategy",
            
            // Decorators
            "*.decorator",
            
            // Adapters
            "*.adapter",
            
            // Facades
            "*.facade",
            
            // Proxies
            "*.proxy",
            
            // Singletons
            "*.singleton",
            
            // Utilities
            "*.utility", "*.helper",
            
            // Configurations
            "*.configuration",
            
            // Profiles
            "*.profile",
            
            // Specifications
            "*.specification",
            
            // Queries
            "*.query",
            
            // Commands
            "*.command",
            
            // Events
            "*.event",
            
            // Notifications
            "*.notification",
            
            // Requests
            "*.request",
            
            // Responses
            "*.response",
            
            // Results
            "*.result",
            
            // Exceptions
            "*.exception",
            
            // Resources
            "*.resource",
            
            // Localizations
            "*.localization",
            
            // Cultures
            "*.culture",
            
            // Themes
            "*.theme",
            
            // Styles
            "*.style",
            
            // Templates
            "*.template",
            
            // Layouts
            "*.layout",
            
            // Components
            "*.component",
            
            // Directives
            "*.directive",
            
            // Pipes
            "*.pipe",
            
            // Guards
            "*.guard",
            
            // Resolvers
            "*.resolver",
            
            // Interceptors
            "*.interceptor",
            
            // Modules
            "*.module",
            
            // Providers
            "*.provider",
            
            // Injectors
            "*.injector",
            
            // Containers
            "*.container",
            
            // Registrations
            "*.registration",
            
            // Bindings
            "*.binding",
            
            // Scopes
            "*.scope",
            
            // Lifetimes
            "*.lifetime",
            
            // Activators
            "*.activator",
            
            // Disposables
            "*.disposable",
            
            // Finalizers
            "*.finalizer",
            
            // Destructors
            "*.destructor",
            
            // Constructors
            "*.constructor",
            
            // Initializers
            "*.initializer",
            
            // Terminators
            "*.terminator",
            
            // Cleaners
            "*.cleaner",
            
            // Managers
            "*.manager",
            
            // Coordinators
            "*.coordinator",
            
            // Supervisors
            "*.supervisor",
            
            // Monitors
            "*.monitor",
            
            // Watchers
            "*.watcher",
            
            // Observers
            "*.observer",
            
            // Listeners
            "*.listener",
            
            // Subscribers
            "*.subscriber",
            
            // Publishers
            "*.publisher",
            
            // Producers
            "*.producer",
            
            // Consumers
            "*.consumer",
            
            // Processors
            "*.processor",
            
            // Transformers
            "*.transformer",
            
            // Serializers
            "*.serializer",
            
            // Deserializers
            "*.deserializer",
            
            // Encoders
            "*.encoder",
            
            // Decoders
            "*.decoder",
            
            // Compressors
            "*.compressor",
            
            // Decompressors
            "*.decompressor",
            
            // Encryptors
            "*.encryptor",
            
            // Decryptors
            "*.decryptor",
            
            // Hashers
            "*.hasher",
            
            // Signers
            "*.signer",
            
            // Verifiers
            "*.verifier",
            
            // Authenticators
            "*.authenticator",
            
            // Authorizers
            "*.authorizer",
            
            // Identifiers
            "*.identifier",
            
            // Generators
            "*.generator",
            
            // Parsers
            "*.parser",
            
            // Lexers
            "*.lexer",
            
            // Tokenizers
            "*.tokenizer",
            
            // Analyzers
            "*.analyzer",
            
            // Optimizers
            "*.optimizer",
            
            // Compilers
            "*.compiler",
            
            // Interpreters
            "*.interpreter",
            
            // Executors
            "*.executor",
            
            // Runners
            "*.runner",
            
            // Launchers
            "*.launcher",
            
            // Starters
            "*.starter",
            
            // Stoppers
            "*.stopper",
            
            // Pausers
            "*.pauser",
            
            // Resumers
            "*.resumer",
            
            // Resetters
            "*.resetter",
            
            // Refreshers
            "*.refresher",
            
            // Updaters
            "*.updater",
            
            // Deleters
            "*.deleter",
            
            // Creators
            "*.creator",
            
            // Destroyers
            "*.destroyer",
            
            // Builders
            "*.builder",
            
            // Constructors
            "*.constructor",
            
            // Assemblers
            "*.assembler",
            
            // Disassemblers
            "*.disassembler",
            
            // Linkers
            "*.linker",
            
            // Loaders
            "*.loader",
            
            // Unloaders
            "*.unloader",
            
            // Savers
            "*.saver",
            
            // Readers
            "*.reader",
            
            // Writers
            "*.writer",
            
            // Streamers
            "*.streamer",
            
            // Buffers
            "*.buffer",
            
            // Caches
            "*.cache",
            
            // Stores
            "*.store",
            
            // Repositories
            "*.repository",
            
            // Databases
            "*.database",
            
            // Connections
            "*.connection",
            
            // Clients
            "*.client",
            
            // Servers
            "*.server",
            
            // Hosts
            "*.host",
            
            // Guests
            "*.guest",
            
            // Peers
            "*.peer",
            
            // Nodes
            "*.node",
            
            // Clusters
            "*.cluster",
            
            // Networks
            "*.network",
            
            // Protocols
            "*.protocol",
            
            // Channels
            "*.channel",
            
            // Pipes
            "*.pipe",
            
            // Queues
            "*.queue",
            
            // Stacks
            "*.stack",
            
            // Heaps
            "*.heap",
            
            // Trees
            "*.tree",
            
            // Graphs
            "*.graph",
            
            // Lists
            "*.list",
            
            // Arrays
            "*.array",
            
            // Dictionaries
            "*.dictionary",
            
            // Maps
            "*.map",
            
            // Sets
            "*.set",
            
            // Collections
            "*.collection",
            
            // Iterators
            "*.iterator",
            
            // Enumerators
            "*.enumerator",
            
            // Comparers
            "*.comparer",
            
            // Equalizers
            "*.equalizer",
            
            // Sorters
            "*.sorter",
            
            // Searchers
            "*.searcher",
            
            // Finders
            "*.finder",
            
            // Matchers
            "*.matcher",
            
            // Filters
            "*.filter",
            
            // Selectors
            "*.selector",
            
            // Projectors
            "*.projector",
            
            // Aggregators
            "*.aggregator",
            
            // Accumulators
            "*.accumulator",
            
            // Reducers
            "*.reducer",
            
            // Folders
            "*.folder",
            
            // Mappers
            "*.mapper",
            
            // Flatteners
            "*.flattener",
            
            // Groupers
            "*.grouper",
            
            // Partitioners
            "*.partitioner",
            
            // Splitters
            "*.splitter",
            
            // Joiners
            "*.joiner",
            
            // Mergers
            "*.merger",
            
            // Combiners
            "*.combiner",
            
            // Concatenators
            "*.concatenator",
            
            // Appenders
            "*.appender",
            
            // Prependers
            "*.prepender",
            
            // Inserters
            "*.inserter",
            
            // Removers
            "*.remover",
            
            // Replacers
            "*.replacer",
            
            // Substituters
            "*.substituter",
            
            // Swappers
            "*.swapper",
            
            // Exchangers
            "*.exchanger",
            
            // Traders
            "*.trader",
            
            // Negotiators
            "*.negotiator",
            
            // Mediators
            "*.mediator",
            
            // Arbitrators
            "*.arbitrator",
            
            // Judges
            "*.judge",
            
            // Evaluators
            "*.evaluator",
            
            // Assessors
            "*.assessor",
            
            // Testers
            "*.tester",
            
            // Validators
            "*.validator",
            
            // Checkers
            "*.checker",
            
            // Verifiers
            "*.verifier",
            
            // Confirmers
            "*.confirmer",
            
            // Approvers
            "*.approver",
            
            // Rejecters
            "*.rejecter",
            
            // Acceptors
            "*.acceptor",
            
            // Deniers
            "*.denier",
            
            // Granters
            "*.granter",
            
            // Revokers
            "*.revoker",
            
            // Enablers
            "*.enabler",
            
            // Disablers
            "*.disabler",
            
            // Activators
            "*.activator",
            
            // Deactivators
            "*.deactivator",
            
            // Triggers
            "*.trigger",
            
            // Schedulers
            "*.scheduler",
            
            // Timers
            "*.timer",
            
            // Clocks
            "*.clock",
            
            // Calendars
            "*.calendar",
            
            // Dates
            "*.date",
            
            // Times
            "*.time",
            
            // Durations
            "*.duration",
            
            // Intervals
            "*.interval",
            
            // Periods
            "*.period",
            
            // Spans
            "*.span",
            
            // Ranges
            "*.range",
            
            // Bounds
            "*.bound",
            
            // Limits
            "*.limit",
            
            // Constraints
            "*.constraint",
            
            // Rules
            "*.rule",
            
            // Policies
            "*.policy",
            
            // Strategies
            "*.strategy",
            
            // Algorithms
            "*.algorithm",
            
            // Heuristics
            "*.heuristic",
            
            // Patterns
            "*.pattern",
            
            // Templates
            "*.template",
            
            // Blueprints
            "*.blueprint",
            
            // Schemas
            "*.schema",
            
            // Specifications
            "*.specification",
            
            // Definitions
            "*.definition",
            
            // Declarations
            "*.declaration",
            
            // Annotations
            "*.annotation",
            
            // Attributes
            "*.attribute",
            
            // Properties
            "*.property",
            
            // Fields
            "*.field",
            
            // Variables
            "*.variable",
            
            // Constants
            "*.constant",
            
            // Parameters
            "*.parameter",
            
            // Arguments
            "*.argument",
            
            // Values
            "*.value",
            
            // Objects
            "*.object",
            
            // Instances
            "*.instance",
            
            // References
            "*.reference",
            
            // Pointers
            "*.pointer",
            
            // Handles
            "*.handle",
            
            // Identifiers
            "*.identifier",
            
            // Keys
            "*.key",
            
            // Tokens
            "*.token",
            
            // Symbols
            "*.symbol",
            
            // Names
            "*.name",
            
            // Labels
            "*.label",
            
            // Tags
            "*.tag",
            
            // Marks
            "*.mark",
            
            // Flags
            "*.flag",
            
            // Indicators
            "*.indicator",
            
            // Signals
            "*.signal",
            
            // Messages
            "*.message",
            
            // Notifications
            "*.notification",
            
            // Alerts
            "*.alert",
            
            // Warnings
            "*.warning",
            
            // Errors
            "*.error",
            
            // Exceptions
            "*.exception",
            
            // Faults
            "*.fault",
            
            // Failures
            "*.failure",
            
            // Problems
            "*.problem",
            
            // Issues
            "*.issue",
            
            // Bugs
            "*.bug",
            
            // Defects
            "*.defect",
            
            // Flaws
            "*.flaw",
            
            // Vulnerabilities
            "*.vulnerability",
            
            // Threats
            "*.threat",
            
            // Risks
            "*.risk",
            
            // Hazards
            "*.hazard",
            
            // Dangers
            "*.danger",
            
            // Perils
            "*.peril",
            
            // Jeopardies
            "*.jeopardy",
            
            // Exposures
            "*.exposure",
            
            // Liabilities
            "*.liability",
            
            // Responsibilities
            "*.responsibility",
            
            // Obligations
            "*.obligation",
            
            // Duties
            "*.duty",
            
            // Tasks
            "*.task",
            
            // Jobs
            "*.job",
            
            // Works
            "*.work",
            
            // Operations
            "*.operation",
            
            // Actions
            "*.action",
            
            // Activities
            "*.activity",
            
            // Processes
            "*.process",
            
            // Procedures
            "*.procedure",
            
            // Methods
            "*.method",
            
            // Functions
            "*.function",
            
            // Routines
            "*.routine",
            
            // Subroutines
            "*.subroutine",
            
            // Modules
            "*.module",
            
            // Components
            "*.component",
            
            // Parts
            "*.part",
            
            // Pieces
            "*.piece",
            
            // Elements
            "*.element",
            
            // Items
            "*.item",
            
            // Entities
            "*.entity",
            
            // Units
            "*.unit",
            
            // Blocks
            "*.block",
            
            // Chunks
            "*.chunk",
            
            // Segments
            "*.segment",
            
            // Sections
            "*.section",
            
            // Divisions
            "*.division",
            
            // Partitions
            "*.partition",
            
            // Regions
            "*.region",
            
            // Areas
            "*.area",
            
            // Zones
            "*.zone",
            
            // Districts
            "*.district",
            
            // Neighborhoods
            "*.neighborhood",
            
            // Communities
            "*.community",
            
            // Societies
            "*.society",
            
            // Groups
            "*.group",
            
            // Teams
            "*.team",
            
            // Crews
            "*.crew",
            
            // Squads
            "*.squad",
            
            // Units
            "*.unit",
            
            // Divisions
            "*.division",
            
            // Departments
            "*.department",
            
            // Sections
            "*.section",
            
            // Branches
            "*.branch",
            
            // Offices
            "*.office",
            
            // Bureaus
            "*.bureau",
            
            // Agencies
            "*.agency",
            
            // Organizations
            "*.organization",
            
            // Institutions
            "*.institution",
            
            // Establishments
            "*.establishment",
            
            // Enterprises
            "*.enterprise",
            
            // Businesses
            "*.business",
            
            // Companies
            "*.company",
            
            // Corporations
            "*.corporation",
            
            // Firms
            "*.firm",
            
            // Partnerships
            "*.partnership",
            
            // Associations
            "*.association",
            
            // Alliances
            "*.alliance",
            
            // Coalitions
            "*.coalition",
            
            // Federations
            "*.federation",
            
            // Unions
            "*.union",
            
            // Leagues
            "*.league",
            
            // Clubs
            "*.club",
            
            // Societies
            "*.society",
            
            // Fraternities
            "*.fraternity",
            
            // Sororities
            "*.sorority",
            
            // Brotherhoods
            "*.brotherhood",
            
            // Sisterhoods
            "*.sisterhood",
            
            // Fellowships
            "*.fellowship",
            
            // Memberships
            "*.membership",
            
            // Citizenships
            "*.citizenship",
            
            // Residencies
            "*.residency",
            
            // Inhabitants
            "*.inhabitant",
            
            // Residents
            "*.resident",
            
            // Occupants
            "*.occupant",
            
            // Tenants
            "*.tenant",
            
            // Renter
            "*.renter",
            
            // Lessee
            "*.lessee",
            
            // Lessor
            "*.lessor",
            
            // Landlord
            "*.landlord",
            
            // Owner
            "*.owner",
            
            // Proprietors
            "*.proprietor",
            
            // Possessors
            "*.possessor",
            
            // Holders
            "*.holder",
            
            // Bearers
            "*.bearer",
            
            // Carriers
            "*.carrier",
            
            // Transporters
            "*.transporter",
            
            // Movers
            "*.mover",
            
            // Shippers
            "*.shipper",
            
            // Senders
            "*.sender",
            
            // Receivers
            "*.receiver",
            
            // Recipients
            "*.recipient",
            
            // Addressees
            "*.addressee",
            
            // Consignees
            "*.consignee",
            
            // Consignors
            "*.consignor",
            
            // Vendors
            "*.vendor",
            
            // Suppliers
            "*.supplier",
            
            // Providers
            "*.provider",
            
            // Distributors
            "*.distributor",
            
            // Dealers
            "*.dealer",
            
            // Retailers
            "*.retailer",
            
            // Wholesalers
            "*.wholesaler",
            
            // Merchants
            "*.merchant",
            
            // Traders
            "*.trader",
            
            // Sellers
            "*.seller",
            
            // Buyers
            "*.buyer",
            
            // Purchasers
            "*.purchaser",
            
            // Customers
            "*.customer",
            
            // Clients
            "*.client",
            
            // Patrons
            "*.patron",
            
            // Consumers
            "*.consumer",
            
            // Users
            "*.user",
            
            // Operators
            "*.operator",
            
            // Administrators
            "*.administrator",
            
            // Managers
            "*.manager",
            
            // Directors
            "*.director",
            
            // Executives
            "*.executive",
            
            // Officers
            "*.officer",
            
            // Officials
            "*.official",
            
            // Authorities
            "*.authority",
            
            // Leaders
            "*.leader",
            
            // Chiefs
            "*.chief",
            
            // Heads
            "*.head",
            
            // Principals
            "*.principal",
            
            // Supervisors
            "*.supervisor",
            
            // Overseers
            "*.overseer",
            
            // Foremen
            "*.foreman",
            
            // Bosses
            "*.boss",
            
            // Employers
            "*.employer",
            
            // Employees
            "*.employee",
            
            // Workers
            "*.worker",
            
            // Laborers
            "*.laborer",
            
            // Staff
            "*.staff",
            
            // Personnel
            "*.personnel",
            
            // Crew
            "*.crew",
            
            // Team
            "*.team",
            
            // Group
            "*.group",
            
            // Squad
            "*.squad",
            
            // Unit
            "*.unit",
            
            // Division
            "*.division",
            
            // Department
            "*.department",
            
            // Section
            "*.section",
            
            // Branch
            "*.branch",
            
            // Office
            "*.office",
            
            // Bureau
            "*.bureau",
            
            // Agency
            "*.agency",
            
            // Organization
            "*.organization",
            
            // Institution
            "*.institution",
            
            // Establishment
            "*.establishment",
            
            // Enterprise
            "*.enterprise",
            
            // Business
            "*.business",
            
            // Company
            "*.company",
            
            // Corporation
            "*.corporation",
            
            // Firm
            "*.firm",
            
            // Partnership
            "*.partnership",
            
            // Association
            "*.association",
            
            // Alliance
            "*.alliance",
            
            // Coalition
            "*.coalition",
            
            // Federation
            "*.federation",
            
            // Union
            "*.union",
            
            // League
            "*.league",
            
            // Club
            "*.club",
            
            // Society
            "*.society",
            
            // Fraternity
            "*.fraternity",
            
            // Sorority
            "*.sorority",
            
            // Brotherhood
            "*.brotherhood",
            
            // Sisterhood
            "*.sisterhood",
            
            // Fellowship
            "*.fellowship",
            
            // Membership
            "*.membership",
            
            // Citizenship
            "*.citizenship",
            
            // Residency
            "*.residency",
            
            // Inhabitant
            "*.inhabitant",
            
            // Resident
            "*.resident",
            
            // Occupant
            "*.occupant",
            
            // Tenant
            "*.tenant",
            
            // Renter
            "*.renter",
            
            // Lessee
            "*.lessee",
            
            // Lessor
            "*.lessor",
            
            // Landlord
            "*.landlord",
            
            // Owner
            "*.owner",
            
            // Proprietor
            "*.proprietor",
            
            // Possessor
            "*.possessor",
            
            // Holder
            "*.holder",
            
            // Bearer
            "*.bearer",
            
            // Carrier
            "*.carrier",
            
            // Transporter
            "*.transporter",
            
            // Mover
            "*.mover",
            
            // Shipper
            "*.shipper",
            
            // Sender
            "*.sender",
            
            // Receiver
            "*.receiver",
            
            // Recipient
            "*.recipient",
            
            // Addressee
            "*.addressee",
            
            // Consignee
            "*.consignee",
            
            // Consignor
            "*.consignor",
            
            // Vendor
            "*.vendor",
            
            // Supplier
            "*.supplier",
            
            // Provider
            "*.provider",
            
            // Distributor
            "*.distributor",
            
            // Dealer
            "*.dealer",
            
            // Retailer
            "*.retailer",
            
            // Wholesaler
            "*.wholesaler",
            
            // Merchant
            "*.merchant",
            
            // Trader
            "*.trader",
            
            // Seller
            "*.seller",
            
            // Buyer
            "*.buyer",
            
            // Purchaser
            "*.purchaser",
            
            // Customer
            "*.customer",
            
            // Client
            "*.client",
            
            // Patron
            "*.patron",
            
            // Consumer
            "*.consumer",
            
            // User
            "*.user",
            
            // Operator
            "*.operator",
            
            // Administrator
            "*.administrator",
            
            // Manager
            "*.manager",
            
            // Director
            "*.director",
            
            // Executive
            "*.executive",
            
            // Officer
            "*.officer",
            
            // Official
            "*.official",
            
            // Authority
            "*.authority",
            
            // Leader
            "*.leader",
            
            // Chief
            "*.chief",
            
            // Head
            "*.head",
            
            // Principal
            "*.principal",
            
            // Supervisor
            "*.supervisor",
            
            // Overseer
            "*.overseer",
            
            // Foreman
            "*.foreman",
            
            // Boss
            "*.boss",
            
            // Employer
            "*.employer",
            
            // Employee
            "*.employee",
            
            // Worker
            "*.worker",
            
            // Laborer
            "*.laborer",
        ]
    }
} 