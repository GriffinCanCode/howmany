use regex::Regex;

pub struct RustPatterns {
    external_patterns: Vec<Regex>,
    cache_patterns: Vec<Regex>,
    extensions: Vec<String>,
}

impl RustPatterns {
    pub fn new() -> Self {
        let external_patterns = vec![
            // Build artifacts
            Regex::new(r"target/").unwrap(),
            Regex::new(r"Cargo\.lock").unwrap(),
            
            // Cargo directories
            Regex::new(r"\.cargo/").unwrap(),
            Regex::new(r"registry/").unwrap(),
            Regex::new(r"git/").unwrap(),
            
            // Dependencies and caches
            Regex::new(r"deps/").unwrap(),
            Regex::new(r"incremental/").unwrap(),
            Regex::new(r"fingerprint/").unwrap(),
            
            // Cross-compilation targets
            Regex::new(r"target/debug/").unwrap(),
            Regex::new(r"target/release/").unwrap(),
            Regex::new(r"target/x86_64-").unwrap(),
            Regex::new(r"target/aarch64-").unwrap(),
            Regex::new(r"target/armv7-").unwrap(),
            Regex::new(r"target/i686-").unwrap(),
            Regex::new(r"target/wasm32-").unwrap(),
            Regex::new(r"target/thumbv").unwrap(),
            
            // Platform-specific builds
            Regex::new(r"target/.*-linux-").unwrap(),
            Regex::new(r"target/.*-windows-").unwrap(),
            Regex::new(r"target/.*-darwin").unwrap(),
            Regex::new(r"target/.*-android").unwrap(),
            Regex::new(r"target/.*-ios").unwrap(),
            
            // Generated binaries
            Regex::new(r"\.exe$").unwrap(),
            Regex::new(r"\.dll$").unwrap(),
            Regex::new(r"\.so$").unwrap(),
            Regex::new(r"\.dylib$").unwrap(),
            Regex::new(r"\.a$").unwrap(),
            Regex::new(r"\.lib$").unwrap(),
            Regex::new(r"\.rlib$").unwrap(),
            Regex::new(r"\.pdb$").unwrap(),
        ];

        let cache_patterns = vec![
            // Build caches
            Regex::new(r"target/debug").unwrap(),
            Regex::new(r"target/release").unwrap(),
            Regex::new(r"target/doc").unwrap(),
            Regex::new(r"target/tmp").unwrap(),
            Regex::new(r"target/\.rustc_info\.json").unwrap(),
            
            // Cargo caches
            Regex::new(r"\.cargo/bin/").unwrap(),
            Regex::new(r"\.cargo/registry/").unwrap(),
            Regex::new(r"\.cargo/git/").unwrap(),
            Regex::new(r"\.cargo/\.crates\.toml").unwrap(),
            Regex::new(r"\.cargo/\.crates2\.json").unwrap(),
            
            // Toolchain caches
            Regex::new(r"\.rustup/").unwrap(),
            Regex::new(r"\.rustc_info\.json").unwrap(),
            
            // IDE and editor caches
            Regex::new(r"\.vscode/").unwrap(),
            Regex::new(r"\.idea/").unwrap(),
            Regex::new(r"rust-project\.json").unwrap(),
            
            // Testing and benchmarking
            Regex::new(r"target/criterion/").unwrap(),
            Regex::new(r"target/coverage/").unwrap(),
            Regex::new(r"target/tarpaulin/").unwrap(),
            
            // Documentation
            Regex::new(r"target/doc/").unwrap(),
            Regex::new(r"target/package/").unwrap(),
            
            // Profiling and debugging
            Regex::new(r"target/profdata/").unwrap(),
            Regex::new(r"target/llvm-cov/").unwrap(),
            Regex::new(r"flamegraph\.svg").unwrap(),
            Regex::new(r"perf\.data").unwrap(),
            
            // WebAssembly specific
            Regex::new(r"pkg/").unwrap(),
            Regex::new(r"wasm-pack\.log").unwrap(),
            
            // Embedded development
            Regex::new(r"\.gdb_history").unwrap(),
            Regex::new(r"openocd\.log").unwrap(),
            
            // Temporary files
            Regex::new(r"\.tmp").unwrap(),
            Regex::new(r"\.swp").unwrap(),
            Regex::new(r"\.swo").unwrap(),
            Regex::new(r"~$").unwrap(),
        ];

        let extensions = vec![
            // Rust source files
            "rs".to_string(),
            
            // Configuration files
            "toml".to_string(),
            
            // WebAssembly
            "wat".to_string(), "wast".to_string(),
            
            // Embedded and low-level
            "s".to_string(), "asm".to_string(),
            
            // Linker scripts
            "ld".to_string(), "x".to_string(),
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
            // Core Cargo files
            "Cargo.toml", "Cargo.lock",
            
            // Entry points
            "main.rs", "lib.rs", "mod.rs",
            
            // Build scripts
            "build.rs", "build.sh", "build.bat",
            
            // Code formatting and linting
            "rustfmt.toml", ".rustfmt.toml",
            "clippy.toml", ".clippy.toml",
            
            // Documentation
            "README.md", "CHANGELOG.md", "CONTRIBUTING.md",
            "LICENSE", "LICENSE-MIT", "LICENSE-APACHE",
            
            // CI/CD
            ".github/workflows/rust.yml", ".github/workflows/ci.yml",
            ".gitlab-ci.yml", "azure-pipelines.yml",
            "Jenkinsfile", "buildkite.yml",
            
            // Testing
            "tests/", "benches/", "examples/",
            "criterion.toml", "proptest-regressions/",
            
            // Cross-compilation
            "Cross.toml", ".cross.toml",
            "rust-toolchain", "rust-toolchain.toml",
            
            // WebAssembly
            "wasm-pack.toml", "Cargo-wasm.toml",
            
            // Embedded development
            "memory.x", "link.x", "build.x",
            "openocd.cfg", "openocd.gdb",
            ".gdbinit", "gdb.txt",
            
            // IDE configuration
            "rust-project.json", ".vscode/settings.json",
            ".idea/", "*.iml",
            
            // Package management
            "deny.toml", "cargo-deny.toml",
            "audit.toml", "cargo-audit.toml",
            "machete.toml", "cargo-machete.toml",
            
            // Release management
            "release.toml", "cargo-release.toml",
            "cliff.toml", "git-cliff.toml",
            
            // Performance and profiling
            "flamegraph.toml", "criterion.toml",
            "tarpaulin.toml", "llvm-cov.toml",
            
            // Docker and containerization
            "Dockerfile", "docker-compose.yml",
            ".dockerignore", "Containerfile",
            
            // Deployment
            "Procfile", "app.yaml", "serverless.yml",
            "fly.toml", "shuttle.toml",
            
            // Configuration
            "config.toml", "settings.toml",
            ".env", ".env.local", ".env.example",
            
            // Scripts
            "install.sh", "setup.sh", "run.sh",
            "test.sh", "build.sh", "deploy.sh",
            
            // Workspace configuration
            "workspace.toml", "members.toml",
            
            // Security
            "security.md", "SECURITY.md",
            "advisory-db/", "rustsec/",
        ]
    }
} 