use regex::Regex;

pub struct WebPatterns {
    external_patterns: Vec<Regex>,
    cache_patterns: Vec<Regex>,
    extensions: Vec<String>,
}

impl WebPatterns {
    pub fn new() -> Self {
        let external_patterns = vec![
            // Legacy dependency managers
            Regex::new(r"bower_components").unwrap(),
            
            // Build output directories
            Regex::new(r"dist/").unwrap(),
            Regex::new(r"build/").unwrap(),
            Regex::new(r"out/").unwrap(),
            Regex::new(r"public/").unwrap(),
            Regex::new(r"static/").unwrap(),
            Regex::new(r"assets/").unwrap(),
            
            // Framework specific build outputs
            Regex::new(r"\.next/").unwrap(),
            Regex::new(r"\.nuxt/").unwrap(),
            Regex::new(r"\.output/").unwrap(),
            Regex::new(r"\.svelte-kit/").unwrap(),
            Regex::new(r"\.astro/").unwrap(),
            Regex::new(r"\.docusaurus/").unwrap(),
            Regex::new(r"_site/").unwrap(),
            
            // Deployment and hosting
            Regex::new(r"\.vercel/").unwrap(),
            Regex::new(r"\.netlify/").unwrap(),
            Regex::new(r"\.firebase/").unwrap(),
            Regex::new(r"\.surge/").unwrap(),
            
            // CDN and vendor files
            Regex::new(r"vendor/").unwrap(),
            Regex::new(r"libs/").unwrap(),
            Regex::new(r"lib/").unwrap(),
            
            // Generated files
            Regex::new(r"\.generated/").unwrap(),
            Regex::new(r"generated/").unwrap(),
        ];

        let cache_patterns = vec![
            // General cache directories
            Regex::new(r"\.cache/").unwrap(),
            Regex::new(r"\.tmp/").unwrap(),
            Regex::new(r"tmp/").unwrap(),
            Regex::new(r"temp/").unwrap(),
            
            // CSS preprocessor caches
            Regex::new(r"\.sass-cache/").unwrap(),
            Regex::new(r"\.postcss-cache/").unwrap(),
            Regex::new(r"\.stylelint-cache/").unwrap(),
            
            // Build tool caches
            Regex::new(r"\.parcel-cache/").unwrap(),
            Regex::new(r"\.webpack/").unwrap(),
            Regex::new(r"\.rollup\.cache/").unwrap(),
            Regex::new(r"\.vite/").unwrap(),
            Regex::new(r"\.turbo/").unwrap(),
            Regex::new(r"\.swc/").unwrap(),
            Regex::new(r"\.esbuild/").unwrap(),
            
            // Testing caches
            Regex::new(r"\.jest/").unwrap(),
            Regex::new(r"\.vitest/").unwrap(),
            Regex::new(r"\.playwright/").unwrap(),
            Regex::new(r"\.cypress/").unwrap(),
            
            // Framework specific caches
            Regex::new(r"\.angular/").unwrap(),
            Regex::new(r"\.ember-cli/").unwrap(),
            Regex::new(r"\.storybook-cache/").unwrap(),
            
            // Linting and formatting caches
            Regex::new(r"\.eslintcache").unwrap(),
            Regex::new(r"\.prettiercache").unwrap(),
            Regex::new(r"\.stylelintcache").unwrap(),
        ];

        let extensions = vec![
            // HTML variants
            "html".to_string(), "htm".to_string(), "xhtml".to_string(),
            "shtml".to_string(), "dhtml".to_string(),
            
            // CSS and preprocessors
            "css".to_string(), "scss".to_string(), "sass".to_string(),
            "less".to_string(), "styl".to_string(), "stylus".to_string(),
            
            // Modern web frameworks
            "vue".to_string(), "svelte".to_string(), "astro".to_string(),
            "lit".to_string(), "riot".to_string(),
            
            // Template engines
            "ejs".to_string(), "hbs".to_string(), "handlebars".to_string(),
            "mustache".to_string(), "pug".to_string(), "jade".to_string(),
            "twig".to_string(), "liquid".to_string(), "nunjucks".to_string(),
            
            // Web components
            "webmanifest".to_string(), "webapp".to_string(),
            
            // Modern web formats
            "mjs".to_string(), "cjs".to_string(),
            "wasm".to_string(), "wat".to_string(),
            
            // Web graphics
            "svg".to_string(),
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
            // HTML entry points
            "index.html", "index.htm", "default.html", "main.html",
            
            // Build tools and bundlers
            "webpack.config.js", "webpack.config.ts", "webpack.config.mjs",
            "rollup.config.js", "rollup.config.ts", "rollup.config.mjs",
            "vite.config.js", "vite.config.ts", "vite.config.mjs",
            "esbuild.config.js", "esbuild.config.mjs",
            "parcel.config.js", "parcel.config.json",
            
            // CSS tools
            "postcss.config.js", "postcss.config.mjs", "postcss.config.json",
            "tailwind.config.js", "tailwind.config.ts", "tailwind.config.mjs",
            "unocss.config.js", "unocss.config.ts",
            "windicss.config.js", "windicss.config.ts",
            
            // Linting and formatting
            ".stylelintrc", ".stylelintrc.js", ".stylelintrc.json",
            ".postcssrc", ".postcssrc.js", ".postcssrc.json",
            "browserslist", ".browserslistrc",
            
            // Framework configs
            "next.config.js", "next.config.mjs", "next.config.ts",
            "nuxt.config.js", "nuxt.config.ts",
            "gatsby.config.js", "gatsby.config.ts",
            "svelte.config.js", "svelte.config.ts",
            "astro.config.js", "astro.config.ts", "astro.config.mjs",
            "remix.config.js", "remix.config.ts",
            "angular.json", ".angular-cli.json",
            "ember-cli-build.js",
            
            // Testing
            "jest.config.js", "jest.config.ts", "jest.config.mjs",
            "vitest.config.js", "vitest.config.ts", "vitest.config.mjs",
            "playwright.config.js", "playwright.config.ts",
            "cypress.config.js", "cypress.config.ts",
            "karma.conf.js", "karma.conf.ts",
            
            // Storybook
            ".storybook/main.js", ".storybook/main.ts",
            "storybook.config.js", "storybook.config.ts",
            
            // Web app manifests
            "manifest.json", "manifest.webmanifest",
            "browserconfig.xml", "site.webmanifest",
            
            // Service workers
            "sw.js", "service-worker.js", "workbox-config.js",
            
            // Development servers
            "serve.json", "static.json", "now.json", "vercel.json",
            "netlify.toml", "_redirects", "_headers",
            
            // Modern web standards
            "web.config", ".htaccess", "robots.txt", "sitemap.xml",
        ]
    }
} 