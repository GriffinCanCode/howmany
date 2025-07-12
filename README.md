# 🔍 HowMany - Intelligent Code Analysis Tool

[![License: Griffin](https://img.shields.io/badge/License-Griffin-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=flat&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Version](https://img.shields.io/badge/version-0.1.0-green.svg)](Cargo.toml)

**HowMany** is an intelligent code analysis and line counting tool built in Rust that provides comprehensive statistics about project codebases while automatically filtering out dependencies, build artifacts, and generated files.

## ✨ Features

### 🧠 **Intelligent File Detection**
- **Smart Filtering**: Automatically distinguishes between user-created code and external dependencies
- **Dependency Exclusion**: Filters out `node_modules`, `target`, `__pycache__`, and other common dependency directories
- **Build Artifact Detection**: Ignores compiled files, caches, and generated content
- **Language-Aware**: Recognizes 40+ programming languages and file types

### 🎨 **Beautiful Interactive UI**
- **Rich Terminal Interface**: Colorful, modern terminal UI with emojis and visual charts
- **Progress Indicators**: Real-time scanning progress with animated spinners
- **Visual Statistics**: Bar charts and percentage breakdowns
- **File Type Icons**: Language-specific icons for easy identification

### 📊 **Comprehensive Analysis**
- **Line Classification**: Separates code, comments, and blank lines
- **File Type Breakdown**: Detailed statistics by programming language
- **Individual File Stats**: Optional per-file analysis
- **Size Calculations**: File size analysis with human-readable formatting

### 🔧 **Flexible Output Formats**
- **Text**: Human-readable terminal output
- **JSON**: Structured data for programmatic use
- **CSV**: Spreadsheet-compatible format
- **Interactive**: Beautiful terminal UI with charts and colors

### ⚙️ **Advanced Configuration**
- **Custom Ignore Patterns**: Add your own file/directory exclusions
- **Extension Filtering**: Focus on specific file types
- **Depth Control**: Limit directory traversal depth
- **Hidden File Support**: Optionally include hidden files
- **Gitignore Integration**: Respects `.gitignore` files automatically

## 🚀 Installation

### Prerequisites
- Rust 1.70+ (2021 edition)
- Cargo package manager

### Build from Source
```bash
# Clone the repository
git clone <repository-url>
cd howmany

# Build the project
cargo build --release

# Run the tool
./target/release/howmany
```

### Install Locally
```bash
# Install to ~/.cargo/bin
cargo install --path .

# Now you can run from anywhere
howmany
```

## 📖 Usage

### Basic Usage

```bash
# Analyze current directory (interactive mode)
howmany

# Analyze specific directory
howmany /path/to/project

# Count lines with basic output
howmany count

# Analyze with verbose breakdown
howmany count --verbose
```

### Command Modes

#### 🎯 **Interactive Mode** (Default)
Beautiful terminal UI with visual charts and detailed breakdowns:
```bash
howmany interactive [PATH]
```

#### 📊 **Count Mode**
Structured output with customizable formatting:
```bash
howmany count [OPTIONS] [PATH]
```

#### 📋 **List Mode**
Show all files that would be analyzed:
```bash
howmany list [OPTIONS] [PATH]
```

### Options

| Option | Description |
|--------|-------------|
| `--max-depth, -d <DEPTH>` | Maximum directory depth to traverse |
| `--files, -f` | Show individual file statistics |
| `--verbose, -v` | Show detailed breakdown by file type |
| `--include-hidden` | Include hidden files and directories |
| `--ignore-gitignore` | Ignore `.gitignore` files |
| `--ignore <PATTERN>` | Additional patterns to ignore (repeatable) |
| `--ext <EXTENSION>` | Only count specific file extensions (repeatable) |
| `--format <FORMAT>` | Output format: `text`, `json`, `csv` |
| `--sort-by <FIELD>` | Sort by: `files`, `lines`, `code`, `comments`, `size` |
| `--descending` | Sort in descending order |

## 🎯 Examples

### Basic Analysis
```bash
# Quick analysis of current directory
howmany

# Analyze specific project
howmany ~/projects/my-app
```

### Detailed Analysis
```bash
# Verbose output with file breakdown
howmany count --verbose

# Show individual file statistics
howmany count --files

# Focus on specific languages
howmany count --ext rs --ext py --ext js
```

### Custom Filtering
```bash
# Ignore additional patterns
howmany count --ignore "*.test.js" --ignore "docs/"

# Include hidden files
howmany count --include-hidden

# Limit depth
howmany count --max-depth 3
```

### Different Output Formats
```bash
# JSON output for scripts
howmany count --format json > stats.json

# CSV for spreadsheets
howmany count --format csv > stats.csv

# Sorted by code lines
howmany count --sort-by code --descending
```

### File Listing
```bash
# See what files would be counted
howmany list

# Check specific extensions
howmany list --ext rs --ext toml
```

## 🎨 Sample Output

### Interactive Mode
```
╔═══════════════════════════════════════════════════════════════════════════════╗
║                                                                               ║
║                          🔍 HOW MANY CODE ANALYZER 🔍                        ║
║                                                                               ║
║              Intelligent code counting with beautiful visualization           ║
║                                                                               ║
╚═══════════════════════════════════════════════════════════════════════════════╝

📊 OVERVIEW
────────────────────────────────────────────────────────────────────────────────
┌─────────────────┬─────────┬────────────┬─────────────────────┐
│ Metric          │ Count   │ Percentage │ Visual              │
├─────────────────┼─────────┼────────────┼─────────────────────┤
│ 📁 Total Files  │ 42      │ -          │ 📁📁📁📁📁📁📁📁📁📁 │
│ 📏 Total Lines  │ 1,247   │ 100.0%     │ ████████████████████ │
│ 💻 Code Lines   │ 891     │ 71.4%      │ ██████████████▓▓▓▓▓▓ │
│ 💬 Comments     │ 203     │ 16.3%      │ ███▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓ │
│ ⬜ Blank Lines  │ 153     │ 12.3%      │ ██▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓ │
│ 💾 Total Size   │ 47.3 KB │ -          │ 💾💾💾💾💾💾💾💾💾💾 │
└─────────────────┴─────────┴────────────┴─────────────────────┘
```

### Count Mode Output
```
=== Code Statistics ===
Total files: 42
Total lines: 1,247
Code lines: 891
Comment lines: 203
Blank lines: 153
Total size: 48,432 bytes (47.30 KB)

=== Breakdown by File Type ===
Extension    Files    Total      Code       Comments   Blank      Size (KB)
────────────────────────────────────────────────────────────────────────────
rs           15       856        612        144        100        32.45
toml         3        89         67         12         10         2.31
md           2        127        98         8          21         4.82
json         1        45         45         0          0          1.23
```

## 🔍 Supported Languages

**Systems Programming**: Rust, C, C++, Go, Assembly  
**Web Development**: JavaScript, TypeScript, HTML, CSS, PHP  
**Enterprise**: Java, C#, Kotlin, Scala  
**Scripting**: Python, Ruby, Shell, PowerShell  
**Functional**: Haskell, F#, OCaml, Elm  
**Data**: R, SQL, YAML, JSON, XML  
**Mobile**: Swift, Objective-C, Dart  
**Configuration**: TOML, INI, Dockerfile, Makefile  
**Documentation**: Markdown, reStructuredText, AsciiDoc

## 🏗️ Architecture

### Core Components

- **🔍 FileDetector**: Intelligent file classification system
- **📊 CodeCounter**: Line counting with language-specific comment recognition
- **🗂️ FileFilter**: Advanced filtering with `.gitignore` integration
- **🎨 InteractiveDisplay**: Beautiful terminal UI with charts and colors
- **⚙️ CLI**: Flexible command-line interface

### Key Features

- **Parallel Processing**: Fast analysis using Rust's concurrency
- **Memory Efficient**: Streaming file processing
- **Cross-Platform**: Works on Windows, macOS, and Linux
- **Extensible**: Easy to add new languages and patterns

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

### Development Setup
```bash
# Clone the repository
git clone <repository-url>
cd howmany

# Install dependencies
cargo build

# Run tests
cargo test

# Run with debug output
cargo run -- --help
```

## 📄 License

This project is licensed under the Griffin License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- Built with [Rust](https://www.rust-lang.org/) for performance and safety
- Uses [clap](https://clap.rs/) for command-line parsing
- Terminal UI powered by [crossterm](https://github.com/crossterm-rs/crossterm) and [comfy-table](https://github.com/Nukesor/comfy-table)
- File traversal with [ignore](https://github.com/BurntSushi/ripgrep/tree/master/crates/ignore) crate
- Progress indicators by [indicatif](https://github.com/console-rs/indicatif)

---

**Griffin** - Intelligent code analysis made beautiful 🦀✨ 