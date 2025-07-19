# Integration Plan: HowMany with Linters, VS Code, and GitHub Actions

Based on the current `howmany` codebase analysis, here's a comprehensive integration plan to expand its ecosystem reach.

## Phase 1: Foundation & Core Integration (Weeks 1-4)

### 1.1 Standardize Output Formats
**Goal**: Create consistent, machine-readable outputs for integrations

**Tasks**:
- [ ] **SARIF Format Support** (Industry standard)
  ```rust
  // Add to src/ui/cli/mod.rs
  #[arg(long = "sarif")]
  pub sarif_output: bool,
  ```
  - Implement SARIF v2.1.0 schema
  - Map howmany metrics to SARIF format
  - Include file locations, rule IDs, severity levels

- [ ] **Enhanced JSON Schema**
  ```json
  {
    "version": "0.3.2",
    "timestamp": "2024-01-15T10:30:00Z",
    "results": {
      "files": [...],
      "metrics": {...},
      "issues": [
        {
          "ruleId": "complexity-high",
          "level": "warning",
          "message": "Function complexity exceeds threshold",
          "location": {
            "file": "src/main.rs",
            "line": 42,
            "column": 8
          }
        }
      ]
    }
  }
  ```

- [ ] **Language Server Protocol (LSP) Foundation**
  - Create `src/lsp/` module
  - Implement basic LSP server structure
  - Add diagnostic publishing capability

### 1.2 GitHub Actions Integration
**Goal**: Official GitHub Action for seamless CI/CD integration

**Tasks**:
- [ ] **Create GitHub Action Repository**
  ```
  howmany-action/
  ├── action.yml
  ├── src/
  │   └── main.ts
  ├── dist/
  └── README.md
  ```

- [ ] **Action Implementation** (`action.yml`)
  ```yaml
  name: 'HowMany Code Analysis'
  description: 'Analyze code complexity, quality, and generate reports'
  inputs:
    path:
      description: 'Path to analyze'
      required: false
      default: '.'
    output-format:
      description: 'Output format (json, html, sarif)'
      required: false
      default: 'json'
    fail-on-quality-gate:
      description: 'Fail if quality gate not met'
      required: false
      default: 'false'
    quality-threshold:
      description: 'Minimum quality score (0-100)'
      required: false
      default: '70'
  outputs:
    report-path:
      description: 'Path to generated report'
    quality-score:
      description: 'Overall quality score'
    passed:
      description: 'Whether quality gate passed'
  runs:
    using: 'node20'
    main: 'dist/index.js'
  ```

- [ ] **TypeScript Implementation** (`src/main.ts`)
  ```typescript
  import * as core from '@actions/core';
  import * as exec from '@actions/exec';
  import * as github from '@actions/github';
  
  async function run(): Promise<void> {
    try {
      const path = core.getInput('path');
      const format = core.getInput('output-format');
      
      // Install howmany
      await exec.exec('cargo', ['install', 'howmany']);
      
      // Run analysis
      await exec.exec('howmany', [
        path,
        '--output', format,
        '--no-interactive'
      ]);
      
      // Parse results and create PR comments
      await createPRComments();
      
    } catch (error) {
      core.setFailed(error.message);
    }
  }
  ```

- [ ] **PR Comment Integration**
  - Parse analysis results
  - Create inline code comments for issues
  - Generate summary comment with metrics
  - Update existing comments on subsequent runs

## Phase 2: VS Code Extension (Weeks 5-8)

### 2.1 Extension Structure
```
howmany-vscode/
├── package.json
├── src/
│   ├── extension.ts
│   ├── diagnostics.ts
│   ├── lsp-client.ts
│   └── commands.ts
├── syntaxes/
├── resources/
└── README.md
```

### 2.2 Core Extension Features

- [ ] **Package Configuration** (`package.json`)
  ```json
  {
    "name": "howmany-analyzer",
    "displayName": "HowMany Code Analyzer",
    "description": "Real-time code quality and complexity analysis",
    "version": "0.1.0",
    "engines": { "vscode": "^1.74.0" },
    "categories": ["Linters", "Other"],
    "activationEvents": [
      "onLanguage:rust",
      "onLanguage:python",
      "onLanguage:javascript",
      "onLanguage:typescript"
    ],
    "contributes": {
      "commands": [
        {
          "command": "howmany.analyzeWorkspace",
          "title": "Analyze Workspace",
          "category": "HowMany"
        }
      ],
      "configuration": {
        "title": "HowMany",
        "properties": {
          "howmany.enableRealTime": {
            "type": "boolean",
            "default": true,
            "description": "Enable real-time analysis"
          }
        }
      }
    }
  }
  ```

- [ ] **Real-time Diagnostics**
  ```typescript
  // src/diagnostics.ts
  export class HowManyDiagnostics {
    private diagnosticCollection: vscode.DiagnosticCollection;
    
    constructor() {
      this.diagnosticCollection = vscode.languages.createDiagnosticCollection('howmany');
    }
    
    async updateDiagnostics(document: vscode.TextDocument) {
      const results = await this.runAnalysis(document.fileName);
      const diagnostics = this.convertToDiagnostics(results);
      this.diagnosticCollection.set(document.uri, diagnostics);
    }
  }
  ```

- [ ] **Status Bar Integration**
  ```typescript
  // Show quality metrics in status bar
  const statusBar = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Left);
  statusBar.text = `$(pulse) Quality: ${qualityScore}/100`;
  statusBar.show();
  ```

- [ ] **Code Actions & Quick Fixes**
  ```typescript
  export class HowManyCodeActionProvider implements vscode.CodeActionProvider {
    provideCodeActions(document: vscode.TextDocument, range: vscode.Range) {
      const actions: vscode.CodeAction[] = [];
      
      // Suggest complexity reduction
      const refactorAction = new vscode.CodeAction(
        'Refactor to reduce complexity',
        vscode.CodeActionKind.Refactor
      );
      actions.push(refactorAction);
      
      return actions;
    }
  }
  ```

### 2.3 Extension Commands & UI

- [ ] **Webview Panel for Reports**
  ```typescript
  // src/webview.ts
  export class HowManyWebviewProvider {
    public static createOrShow(context: vscode.ExtensionContext, data: any) {
      const panel = vscode.window.createWebviewPanel(
        'howmanyReport',
        'HowMany Analysis Report',
        vscode.ViewColumn.Beside,
        { enableScripts: true }
      );
      
      panel.webview.html = this.getWebviewContent(data);
    }
  }
  ```

## Phase 3: Linter Ecosystem Integration (Weeks 9-12)

### 3.1 ESLint Plugin Integration
**Goal**: Integrate with existing JavaScript/TypeScript linting workflows

- [ ] **ESLint Plugin Structure**
  ```
  eslint-plugin-howmany/
  ├── package.json
  ├── lib/
  │   ├── index.js
  │   └── rules/
  │       └── complexity-threshold.js
  └── README.md
  ```

- [ ] **Custom ESLint Rules**
  ```javascript
  // lib/rules/complexity-threshold.js
  module.exports = {
    meta: {
      type: "suggestion",
      docs: {
        description: "enforce complexity thresholds using HowMany analysis",
        category: "Best Practices"
      },
      schema: [{
        type: "object",
        properties: {
          max: { type: "integer", minimum: 1 }
        }
      }]
    },
    create(context) {
      return {
        FunctionDeclaration(node) {
          // Integrate with howmany analysis
          const complexity = getComplexityFromHowMany(node);
          if (complexity > threshold) {
            context.report({
              node,
              message: `Function complexity (${complexity}) exceeds threshold`
            });
          }
        }
      };
    }
  };
  ```

### 3.2 Rust Analyzer Integration
- [ ] **Rust Analyzer Extension**
  - Create custom diagnostic provider
  - Integrate with existing rust-analyzer workflow
  - Provide code lens for metrics

### 3.3 Language Server Protocol Implementation
- [ ] **Standalone LSP Server**
  ```rust
  // src/lsp/server.rs
  use tower_lsp::{LspService, Server};
  use tower_lsp::jsonrpc::Result;
  use tower_lsp::lsp_types::*;
  
  #[tower_lsp::async_trait]
  impl LanguageServer for HowManyLsp {
      async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
          Ok(InitializeResult {
              capabilities: ServerCapabilities {
                  text_document_sync: Some(TextDocumentSyncCapability::Kind(
                      TextDocumentSyncKind::FULL,
                  )),
                  diagnostic_provider: Some(DiagnosticServerCapabilities::Options(
                      DiagnosticOptions::default(),
                  )),
                  ..Default::default()
              },
              ..Default::default()
          })
      }
  }
  ```

## Phase 4: Advanced Features & Optimization (Weeks 13-16)

### 4.1 Performance Optimization
- [ ] **Incremental Analysis**
  - Cache analysis results
  - Only re-analyze changed files
  - Background processing for large codebases

- [ ] **Parallel Processing Enhancement**
  ```rust
  // Enhanced parallel processing
  use rayon::prelude::*;
  
  pub fn analyze_files_parallel(files: Vec<PathBuf>) -> Result<Vec<FileAnalysis>> {
      files
          .par_iter()
          .map(|file| analyze_single_file(file))
          .collect()
  }
  ```

### 4.2 Configuration Management
- [ ] **Unified Configuration**
  ```toml
  # .howmany.toml
  [analysis]
  complexity_threshold = 10
  maintainability_threshold = 65
  
  [integrations]
  vscode_enable = true
  github_actions_enable = true
  
  [output]
  formats = ["json", "html", "sarif"]
  ```

### 4.3 Quality Gates & Thresholds
- [ ] **Configurable Quality Gates**
  ```rust
  pub struct QualityGate {
      pub complexity_max: f64,
      pub maintainability_min: f64,
      pub documentation_min: f64,
      pub test_coverage_min: f64,
  }
  
  impl QualityGate {
      pub fn evaluate(&self, stats: &AggregatedStats) -> QualityGateResult {
          // Implementation
      }
  }
  ```

## Phase 5: Documentation & Distribution (Weeks 17-18)

### 5.1 Documentation
- [ ] **Integration Guides**
  - VS Code extension setup
  - GitHub Actions workflow examples
  - CI/CD integration patterns
  - Linter configuration guides

- [ ] **API Documentation**
  - LSP protocol documentation
  - JSON/SARIF schema documentation
  - Extension API reference

### 5.2 Distribution
- [ ] **VS Code Marketplace**
  - Publish extension
  - Create marketplace presence
  - User onboarding flow

- [ ] **GitHub Actions Marketplace**
  - Publish official action
  - Example workflows
  - Integration templates

- [ ] **NPM Packages**
  - ESLint plugin distribution
  - CLI wrapper for Node.js projects

## Implementation Priority Matrix

| Feature | Impact | Effort | Priority |
|---------|--------|--------|----------|
| GitHub Actions | High | Medium | 1 |
| VS Code Extension | High | High | 2 |
| SARIF Output | Medium | Low | 3 |
| LSP Server | High | High | 4 |
| ESLint Plugin | Medium | Medium | 5 |

## Success Metrics

- **Adoption**: Downloads, stars, usage statistics
- **Integration**: Number of projects using integrations
- **Performance**: Analysis speed, memory usage
- **User Satisfaction**: GitHub issues, feedback, ratings

This plan provides a structured approach to expanding `howmany`'s ecosystem while maintaining the high-quality architecture already established in the codebase.