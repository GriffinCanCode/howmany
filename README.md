# HowMany

A fast, intelligent code analysis tool for counting lines of code and analyzing project statistics.

## Overview

HowMany is a command-line tool that analyzes codebases to provide comprehensive statistics about files, lines of code, complexity, and development time estimates. It automatically detects user-created code files while filtering out dependencies, build artifacts, and generated files.

## Features

- **Line Counting**: Accurate counting of code lines, comments, documentation, and blank lines
- **Language Detection**: Supports 25+ programming languages and file types
- **Complexity Analysis**: Calculates cyclomatic complexity, cognitive complexity, and maintainability metrics
- **Quality Metrics**: Evaluates code health, documentation coverage, and maintainability scores
- **Time Estimates**: Provides development time estimates based on code volume and complexity
- **Multiple Output Formats**: Text, JSON, CSV, and HTML reports with interactive charts
- **Interactive Mode**: Terminal-based interface with charts, navigation, and search functionality
- **Parallel Processing**: Fast analysis using multi-threading and intelligent caching
- **Smart Filtering**: Automatically excludes dependencies, build artifacts, and generated files

## Installation

### From crates.io (Recommended)

```bash
cargo install howmany
```

### From Source

```bash
git clone https://github.com/GriffinCanCode/howmany.git
cd howmany
cargo build --release
```

### Using the Build Script

```bash
./rebuild.sh
```

This will build the project and create a symlink in `/usr/local/bin/howmany` for system-wide access.

## Usage

### Basic Usage

```bash
# Analyze current directory (interactive mode by default)
howmany

# Analyze specific directory
howmany /path/to/project

# Non-interactive text output
howmany --no-interactive

# Show individual file statistics
howmany --files

# Verbose output with detailed breakdown
howmany --verbose
```

### Output Formats

```bash
# Generate HTML report with interactive charts
howmany --output html

# Export to JSON
howmany --output json

# Export to CSV
howmany --output csv

# Plain text output
howmany --output text
```

### Filtering Options

```bash
# Analyze only specific file extensions
howmany --ext rs,py,js

# Set maximum directory depth
howmany --depth 3

# Include hidden files
howmany --hidden

# Add custom ignore patterns
howmany --ignore node_modules,target,dist

# List files that would be analyzed
howmany --list
```

### Sorting and Display

```bash
# Sort by different criteria
howmany --sort lines --desc
howmany --sort code
howmany --sort files
howmany --sort comments
howmany --sort size
```

## Supported Languages

HowMany automatically detects and analyzes these languages:

### Programming Languages
**Systems & Performance:**
- Rust, C/C++, Zig, Go

**Object-Oriented:**
- Java, Kotlin, C#, Swift, Scala

**Scripting & Dynamic:**
- Python, JavaScript, TypeScript, Ruby, PHP, Perl, Lua

**Functional & Academic:**
- Haskell, Clojure, Elixir, Erlang, Julia, R, MATLAB

**Mobile & Cross-Platform:**
- Dart (Flutter), Swift (iOS), Kotlin (Android)

### Web Technologies
- HTML, CSS, SCSS, Sass, Less
- Vue, Svelte, React (JSX/TSX)
- Template engines (Twig, Handlebars, etc.)

### Configuration & Markup
- JSON, XML, YAML, TOML
- Markdown, reStructuredText

### Scripts & Shell
- Shell scripts (bash, zsh, fish)
- PowerShell, Batch files

## Language Distribution

```
Rust         ████████████████████████████████████████ 45.2%
JavaScript   ████████████████████████████ 32.1%
Python       ████████████████ 18.7%
TypeScript   ████ 4.0%
```

*Example distribution from a typical multi-language project*

## Quality Metrics & Analysis

### Maintainability Index
The **Maintainability Index** is an industry-standard metric that predicts code maintainability:

- **85-100**: Excellent maintainability
- **65-84**: Good maintainability  
- **20-64**: Moderate maintainability (refactoring recommended)
- **0-19**: Poor maintainability (consider rewriting)

### Complexity Analysis
- **Cyclomatic Complexity**: Measures decision points and code paths
- **Cognitive Complexity**: Measures mental effort required to understand code
- **Nesting Depth**: Tracks maximum indentation levels
- **Function Length**: Analyzes average and maximum function sizes

### Code Health Metrics
- **Overall Quality Score**: Weighted combination of all metrics (0-100)
- **Documentation Coverage**: Percentage of code with comments/docs
- **Readability Score**: Based on comment density and structure
- **Technical Debt Ratio**: Estimated maintenance burden

## Interactive Mode

The default interactive mode provides:

### Tabs & Navigation
- **Overview Tab**: Summary statistics with animated charts
- **Languages Tab**: Breakdown by programming language with health metrics
- **Export Tab**: Export results to various formats

### Controls
- `Tab` / `Shift+Tab`: Switch between tabs
- `1`, `2`, `3`: Jump to specific tabs
- `↑`/`↓` or `j`/`k`: Navigate lists and tables
- `/` or `s`: Toggle search mode
- `t`: Toggle code health view (in Languages tab)
- `h` or `F1`: Show help
- `q` or `Esc`: Quit

### Search Functionality
- **Files**: Search by file name and path
- **Extensions**: Search by file extension
- **Content**: Search by estimated content/keywords
- `Tab`: Cycle between search modes
- `Enter`: Jump to selected result

## Output Examples

### Text Output
```
=== Code Statistics ===
Total files: 127
Total lines: 15,847
Code lines: 11,234
Comment lines: 2,156
Documentation lines: 1,247
Blank lines: 1,210
Total size: 567,890 bytes (554.6 KB)

=== Complexity Analysis ===
Functions: 342
Average complexity: 3.8
Max nesting depth: 5
Maintainability index: 78.2

=== Quality Metrics ===
Overall quality score: 82.4/100
Documentation score: 75.1/100
Maintainability score: 78.2/100

=== Time Estimates ===
Total development time: 3 days, 2 hours
Code writing time: 2 days, 1 hour
Documentation time: 6 hours
```

### HTML Report Features
- **Interactive Charts**: Language distribution, complexity analysis, quality metrics
- **Responsive Design**: Works on desktop and mobile
- **Quality Insights**: Color-coded health indicators and recommendations
- **Detailed Breakdowns**: File-by-file analysis with complexity details
- **Export Options**: Built-in export functionality

## Command Line Options

| Option | Short | Description |
|--------|-------|-------------|
| `--output` | `-o` | Output format: text, json, csv, html |
| `--files` | `-f` | Show individual file statistics |
| `--verbose` | `-v` | Show detailed breakdown by extension |
| `--no-interactive` | | Disable interactive mode (force text output) |
| `--depth` | `-d` | Maximum directory depth to traverse |
| `--ext` | `-e` | Only count specific extensions (comma-separated) |
| `--hidden` | | Include hidden files and directories |
| `--sort` | `-s` | Sort by: files, lines, code, comments, size |
| `--desc` | | Sort in descending order |
| `--ignore` | | Additional ignore patterns (comma-separated) |
| `--list` | `-l` | List files that would be counted (dry run) |

## Smart File Detection

HowMany automatically excludes:

### Dependencies & Package Managers
- `node_modules/`, `target/`, `__pycache__/`, `site-packages/`
- `vendor/`, `deps/`, `build/`, `dist/`, `.next/`

### Build Artifacts & Caches
- `*.o`, `*.class`, `*.pyc`, `*.pyo`, `*.obj`, `*.exe`
- `.cache/`, `.tmp/`, `temp/`, `.serverless/`

### Version Control & IDE
- `.git/`, `.svn/`, `.hg/`, `.bzr/`
- `.vscode/`, `.idea/`, `.vs/`, `.DS_Store`

### Generated & Temporary Files
- `*.generated.*`, `*.min.js`, `*.bundle.*`
- `*.log`, `*.tmp`, `coverage/`, `reports/`

## Performance

- **Parallel Processing**: Utilizes all available CPU cores
- **Smart Caching**: Caches results for faster subsequent runs with 85%+ hit rates
- **Memory Efficient**: Processes files in chunks, handles large codebases
- **Fast Analysis**: Typically processes 10,000+ files in under 5 seconds

## Configuration

Create `~/.config/howmany/config.toml` for custom settings:

```toml
[defaults]
output_format = "interactive"
show_progress = true
use_colors = true
interactive_by_default = true

[performance]
parallel_processing = true
cache_enabled = true
chunk_size = 100

[filtering]
respect_gitignore = true
include_hidden = false
max_depth = 50

[quality_thresholds]
excellent_maintainability = 85.0
good_maintainability = 65.0
min_doc_coverage = 20.0

[custom_patterns]
ignore = ["*.tmp", "*.log", "node_modules/"]
include_extensions = []  # Empty means all supported
```

## Examples

### Analyze a Rust Project
```bash
howmany ~/my-rust-project --ext rs,toml --verbose
```

### Generate Comprehensive HTML Report
```bash
howmany ~/my-project --output html --files
```

### Quick Quality Assessment
```bash
howmany --no-interactive --verbose | grep -A 10 "Quality Metrics"
```

### Analyze Only Documentation
```bash
howmany --ext md,rst,txt --sort lines --desc
```

### Performance Analysis
```bash
# Large codebase analysis with caching
howmany /path/to/large/project --depth 10 --verbose
```

## Advanced Features

### Caching System
- Intelligent file change detection using timestamps and checksums
- Persistent cache across runs for improved performance
- Automatic cache cleanup and optimization

### Language-Specific Analysis
- Detects language-specific patterns (imports, exports, classes, functions)
- Calculates complexity based on language characteristics
- Provides language-specific quality recommendations

### Time Estimation Algorithm
- Based on industry-standard development velocity metrics
- Accounts for code complexity and documentation density
- Separates coding time from documentation time

## License

MIT License. See [LICENSE](LICENSE) for details.

## Contributing

Contributions are welcome! Please feel free to submit pull requests or open issues.

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## Version

Current version: 0.3.2

## Changelog

### v0.3.2 (Latest)
- Enhanced interactive mode with search functionality
- Improved complexity analysis with quality metrics
- Added comprehensive HTML reports with charts
- Performance optimizations with intelligent caching
- Extended language support (25+ languages)
- Better error handling and progress indicators 