use regex::Regex;

pub struct NodejsPatterns {
    external_patterns: Vec<Regex>,
    cache_patterns: Vec<Regex>,
    extensions: Vec<String>,
}

impl NodejsPatterns {
    pub fn new() -> Self {
        let external_patterns = vec![
            // Package dependencies
            Regex::new(r"node_modules").unwrap(),
            
            // Lock files
            Regex::new(r"package-lock\.json").unwrap(),
            Regex::new(r"yarn\.lock").unwrap(),
            Regex::new(r"pnpm-lock\.yaml").unwrap(),
            Regex::new(r"bun\.lockb").unwrap(),
            
            // Package manager directories
            Regex::new(r"\.npm/").unwrap(),
            Regex::new(r"\.yarn/").unwrap(),
            Regex::new(r"\.pnpm-store/").unwrap(),
            Regex::new(r"\.bun/").unwrap(),
            
            // Build outputs
            Regex::new(r"dist/").unwrap(),
            Regex::new(r"build/").unwrap(),
            Regex::new(r"out/").unwrap(),
            Regex::new(r"lib/").unwrap(),
            Regex::new(r"esm/").unwrap(),
            Regex::new(r"cjs/").unwrap(),
            Regex::new(r"umd/").unwrap(),
            
            // Framework specific outputs
            Regex::new(r"\.next/").unwrap(),
            Regex::new(r"\.nuxt/").unwrap(),
            Regex::new(r"\.output/").unwrap(),
            Regex::new(r"\.svelte-kit/").unwrap(),
            Regex::new(r"\.astro/").unwrap(),
            Regex::new(r"\.remix/").unwrap(),
            
            // Deployment directories
            Regex::new(r"\.vercel/").unwrap(),
            Regex::new(r"\.netlify/").unwrap(),
            Regex::new(r"\.firebase/").unwrap(),
            
            // Generated files
            Regex::new(r"\.d\.ts\.map$").unwrap(),
            Regex::new(r"\.js\.map$").unwrap(),
            Regex::new(r"\.min\.js$").unwrap(),
            Regex::new(r"\.min\.css$").unwrap(),
            
            // Compiled TypeScript
            Regex::new(r"\.tsbuildinfo").unwrap(),
            
            // Electron
            Regex::new(r"app\.asar").unwrap(),
            Regex::new(r"electron-builder\.env").unwrap(),
        ];

        let cache_patterns = vec![
            // Package manager caches
            Regex::new(r"\.npm").unwrap(),
            Regex::new(r"\.yarn").unwrap(),
            Regex::new(r"\.pnpm").unwrap(),
            Regex::new(r"\.bun").unwrap(),
            Regex::new(r"node_modules/\.cache").unwrap(),
            
            // Build tool caches
            Regex::new(r"\.next/").unwrap(),
            Regex::new(r"\.nuxt/").unwrap(),
            Regex::new(r"\.vite/").unwrap(),
            Regex::new(r"\.turbo/").unwrap(),
            Regex::new(r"\.webpack/").unwrap(),
            Regex::new(r"\.rollup\.cache/").unwrap(),
            Regex::new(r"\.parcel-cache/").unwrap(),
            Regex::new(r"\.swc/").unwrap(),
            Regex::new(r"\.esbuild/").unwrap(),
            
            // TypeScript caches
            Regex::new(r"\.tscache/").unwrap(),
            Regex::new(r"tsconfig\.tsbuildinfo").unwrap(),
            
            // Testing caches
            Regex::new(r"\.jest/").unwrap(),
            Regex::new(r"\.vitest/").unwrap(),
            Regex::new(r"\.playwright/").unwrap(),
            Regex::new(r"\.cypress/").unwrap(),
            Regex::new(r"coverage/").unwrap(),
            Regex::new(r"\.nyc_output/").unwrap(),
            
            // Linting and formatting caches
            Regex::new(r"\.eslintcache").unwrap(),
            Regex::new(r"\.prettiercache").unwrap(),
            Regex::new(r"\.stylelintcache").unwrap(),
            
            // Framework specific caches
            Regex::new(r"\.angular/").unwrap(),
            Regex::new(r"\.ember-cli/").unwrap(),
            Regex::new(r"\.storybook-cache/").unwrap(),
            
            // Development server caches
            Regex::new(r"\.cache/").unwrap(),
            Regex::new(r"\.tmp/").unwrap(),
            Regex::new(r"tmp/").unwrap(),
            
            // Serverless caches
            Regex::new(r"\.serverless/").unwrap(),
            Regex::new(r"\.serverless_nextjs/").unwrap(),
            
            // Electron caches
            Regex::new(r"\.electron/").unwrap(),
            Regex::new(r"electron-builder-cache/").unwrap(),
        ];

        let extensions = vec![
            // JavaScript variants
            "js".to_string(), "mjs".to_string(), "cjs".to_string(),
            
            // TypeScript
            "ts".to_string(), "tsx".to_string(), "d.ts".to_string(),
            
            // React
            "jsx".to_string(),
            
            // Configuration and data
            "json".to_string(), "json5".to_string(),
            
            // Modern JavaScript
            "es6".to_string(), "es".to_string(),
            
            // Node.js specific
            "node".to_string(),
            
            // Coffee Script (legacy but still used)
            "coffee".to_string(),
            
            // LiveScript
            "ls".to_string(),
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
            // Package management
            "package.json", "package-lock.json", "yarn.lock", "pnpm-lock.yaml",
            "bun.lockb", "npm-shrinkwrap.json", ".nvmrc", ".node-version",
            
            // TypeScript configuration
            "tsconfig.json", "tsconfig.build.json", "tsconfig.base.json",
            "tsconfig.*.json", "jsconfig.json",
            
            // Build tools and bundlers
            "webpack.config.js", "webpack.config.ts", "webpack.config.mjs",
            "webpack.dev.js", "webpack.prod.js", "webpack.common.js",
            "rollup.config.js", "rollup.config.ts", "rollup.config.mjs",
            "vite.config.js", "vite.config.ts", "vite.config.mjs",
            "esbuild.config.js", "esbuild.config.mjs",
            "parcel.config.js", "parcel.config.json",
            "turbo.json", "nx.json", "lerna.json",
            
            // Framework configurations
            "next.config.js", "next.config.mjs", "next.config.ts",
            "nuxt.config.js", "nuxt.config.ts",
            "gatsby.config.js", "gatsby.config.ts", "gatsby-node.js",
            "svelte.config.js", "svelte.config.ts",
            "astro.config.js", "astro.config.ts", "astro.config.mjs",
            "remix.config.js", "remix.config.ts",
            "angular.json", ".angular-cli.json", "ng-package.json",
            "ember-cli-build.js", ".ember-cli",
            "vue.config.js", "vue.config.ts",
            
            // Testing frameworks
            "jest.config.js", "jest.config.ts", "jest.config.mjs", "jest.config.json",
            "vitest.config.js", "vitest.config.ts", "vitest.config.mjs",
            "playwright.config.js", "playwright.config.ts",
            "cypress.config.js", "cypress.config.ts", "cypress.json",
            "karma.conf.js", "karma.conf.ts",
            "mocha.opts", ".mocharc.json", ".mocharc.js",
            "ava.config.js", "ava.config.mjs",
            
            // Linting and formatting
            ".eslintrc", ".eslintrc.js", ".eslintrc.json", ".eslintrc.yaml",
            ".prettierrc", ".prettierrc.js", ".prettierrc.json",
            ".stylelintrc", ".stylelintrc.js", ".stylelintrc.json",
            ".editorconfig", ".gitignore", ".gitattributes",
            
            // Babel
            ".babelrc", ".babelrc.js", ".babelrc.json", "babel.config.js",
            "babel.config.json", "babel.config.mjs",
            
            // PostCSS and CSS tools
            "postcss.config.js", "postcss.config.mjs", "postcss.config.json",
            "tailwind.config.js", "tailwind.config.ts", "tailwind.config.mjs",
            "unocss.config.js", "unocss.config.ts",
            
            // Storybook
            ".storybook/main.js", ".storybook/main.ts",
            "storybook.config.js", "storybook.config.ts",
            
            // Serverless
            "serverless.yml", "serverless.yaml", "serverless.js",
            "serverless.ts", "now.json", "vercel.json",
            
            // Docker
            "Dockerfile", "docker-compose.yml", "docker-compose.yaml",
            ".dockerignore",
            
            // CI/CD
            ".github/workflows/node.js.yml", ".github/workflows/ci.yml",
            ".gitlab-ci.yml", "azure-pipelines.yml", "Jenkinsfile",
            "bitbucket-pipelines.yml", "circle.yml", ".circleci/config.yml",
            
            // Deployment
            "Procfile", "app.json", "app.yaml", "netlify.toml",
            "_redirects", "_headers", "firebase.json", ".firebaserc",
            
            // Electron
            "electron.js", "main.js", "preload.js", "renderer.js",
            "forge.config.js", "electron-builder.json",
            
            // Monorepo tools
            "rush.json", "pnpm-workspace.yaml", "workspace.json",
            
            // Environment and configuration
            ".env", ".env.local", ".env.development", ".env.production",
            ".env.test", ".env.example", "nodemon.json",
            
            // Documentation
            "README.md", "CHANGELOG.md", "CONTRIBUTING.md",
            "API.md", "docs/", "jsdoc.json",
            
            // License and legal
            "LICENSE", "LICENSE.md", "COPYING", "COPYRIGHT",
            
            // Performance and monitoring
            "lighthouse.config.js", "web-vitals.js",
            
            // GraphQL
            "schema.graphql", "codegen.yml", "graphql.config.js",
            
            // Web standards
            "manifest.json", "sw.js", "service-worker.js",
            "workbox-config.js", "robots.txt", "sitemap.xml",
        ]
    }
} 