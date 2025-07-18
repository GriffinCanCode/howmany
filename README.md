# HowMany

A fast, intelligent code analysis tool for counting lines of code and analyzing project statistics.

## Overview

HowMany is a command-line tool that analyzes codebases to provide comprehensive statistics about files, lines of code, complexity, and development time estimates. It automatically detects user-created code files while filtering out dependencies, build artifacts, and generated files.

## Features

- **Line Counting**: Accurate counting of code lines, comments, documentation, and blank lines
- **Language Detection**: Supports 40+ programming languages and file types
- **Complexity Analysis**: Calculates cyclomatic complexity, function counts, and maintainability metrics
- **Time Estimates**: Provides development time estimates based on code volume
- **Quality Metrics**: Evaluates code health, documentation coverage, and maintainability
- **Multiple Output Formats**: Text, JSON, CSV, and HTML reports
- **Interactive Mode**: Terminal-based interface with charts and navigation
- **Parallel Processing**: Fast analysis using multi-threading
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
# Generate HTML report
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

**Programming Languages:**
- Rust, Python, JavaScript, TypeScript, Java, C/C++, C#, Go, Swift, Kotlin
- Ruby, PHP, Scala, Clojure, Haskell, Erlang, Elixir, Julia, Dart, Lua
- Perl, R, MATLAB, Zig

**Web Technologies:**
- HTML, CSS, SCSS, Sass, Less, Vue, Svelte

**Configuration & Markup:**
- JSON, XML, YAML, TOML, Markdown, reStructuredText

**Scripts:**
- Shell scripts (bash, zsh, fish), PowerShell, Batch files

## Output Examples

### Text Output
```
=== Code Statistics ===
Total files: 45
Total lines: 12,847
Code lines: 9,234
Comment lines: 1,456
Documentation lines: 892
Blank lines: 1,265
Total size: 445,678 bytes (435.2 KB)

=== Complexity Analysis ===
Functions: 234
Average complexity: 3.2
Max nesting depth: 4

=== Time Estimates ===
Total development time: 2 days, 4 hours
Code writing time: 1 day, 8 hours
Documentation time: 4 hours
```

### HTML Report Features
- Interactive charts and visualizations
- Language distribution analysis
- Complexity metrics with color coding
- Quality scores and recommendations
- Detailed file-by-file breakdown
- Responsive design for mobile viewing

## Command Line Options

| Option | Short | Description |
|--------|-------|-------------|
| `--output` | `-o` | Output format (text, json, csv, html) |
| `--files` | `-f` | Show individual file statistics |
| `--verbose` | `-v` | Show detailed breakdown by extension |
| `--no-interactive` | | Disable interactive mode |
| `--depth` | `-d` | Maximum directory depth to traverse |
| `--ext` | `-e` | Only count specific extensions (comma-separated) |
| `--hidden` | | Include hidden files and directories |
| `--sort` | `-s` | Sort by: files, lines, code, comments, size |
| `--desc` | | Sort in descending order |
| `--ignore` | | Additional ignore patterns (comma-separated) |
| `--list` | `-l` | List files that would be counted |

## Interactive Mode

The default interactive mode provides:
- **Overview Tab**: Summary statistics with charts
- **Languages Tab**: Breakdown by programming language
- **Export Tab**: Export results to various formats
- **Search**: Find files by name, extension, or content
- **Help**: Built-in help system

### Navigation
- `Tab` / `Shift+Tab`: Switch between tabs
- `1`, `2`, `3`: Jump to specific tabs
- `↑`/`↓` or `j`/`k`: Navigate lists
- `/` or `s`: Toggle search mode
- `h` or `F1`: Show help
- `q` or `Esc`: Quit

## Smart File Detection

HowMany automatically excludes:
- **Dependencies**: node_modules, target, __pycache__, site-packages
- **Build Artifacts**: *.o, *.class, *.pyc, build/, dist/
- **Version Control**: .git/, .svn/, .hg/
- **IDE Files**: .vscode/, .idea/, .DS_Store
- **Temporary Files**: *.tmp, *.log, cache/

## Performance

- **Parallel Processing**: Uses all available CPU cores
- **Smart Caching**: Caches results for faster subsequent runs
- **Memory Efficient**: Processes files in chunks to minimize memory usage
- **Fast Analysis**: Typically analyzes 10,000+ files in under 5 seconds

## Configuration

HowMany supports configuration files for custom settings:

**Location**: `~/.config/howmany/config.toml`

```toml
[output_preferences]
default_format = "interactive"
show_progress = true
use_colors = true

[performance]
parallel_processing = true
chunk_size = 100

[custom_ignore_patterns]
patterns = ["*.tmp", "*.log", "node_modules/"]
```

## Examples

### Analyze a Rust Project
```bash
howmany ~/my-rust-project --ext rs,toml --verbose
```

### Generate HTML Report for Web Project
```bash
howmany ~/my-web-app --ext js,ts,html,css --output html
```

### Quick JSON Export
```bash
howmany --output json --no-interactive > stats.json
```

### Analyze Only Documentation
```bash
howmany --ext md,rst,txt --sort lines --desc
```

## License

MIT License. See [LICENSE](LICENSE) for details.

## Contributing

Contributions are welcome! Please feel free to submit pull requests or open issues.

## Version

Current version: 0.3.1 