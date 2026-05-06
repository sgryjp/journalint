# AGENTS.md - Journalint Project Guide

## Project Overview

**Journalint** is a specialized linter for personal journal files written in structured Markdown format. It provides both a command-line interface and VS Code extension with Language Server Protocol (LSP) support for real-time linting and auto-fixing capabilities.

### Key Features

- Parses journal files with YAML front matter and time-tracked entries
- Validates time consistency, duration calculations, and format compliance
- Provides automated corrections for common issues
- Works as both CLI tool and LSP server for editor integration

## Architecture

### Project Structure

```
journalint/
├── crates/                    # Rust workspace
│   ├── journalint/           # Main application (CLI + LSP server)
│   └── journalint-parse/     # Parsing and linting library
├── tools/
│   └── journalint-vscode/    # VS Code extension (TypeScript)
├── scripts/                  # Build and distribution scripts
├── .github/workflows/        # CI/CD pipeline
└── changelog.d/              # News fragments for changelog management
```

### Core Components

#### 1. Parser (journalint-parse crate)

- **Location**: `crates/journalint-parse/`
- **Purpose**: Core parsing and linting logic
- **Key files**:
  - `src/parse.rs` - Parser implementation using chumsky combinators
  - `src/lint.rs` - Linting rules and validation logic
  - `src/ast.rs` - Abstract Syntax Tree definitions with visitor pattern

#### 2. Application (journalint crate)

- **Location**: `crates/journalint/`
- **Purpose**: CLI tool and LSP server
- **Key files**:
  - `src/main.rs` - Entry point (CLI mode or LSP mode with `--stdio`)
  - `src/service.rs` - LSP server implementation
  - `src/commands/` - Auto-fix command implementations

#### 3. VS Code Extension

- **Location**: `tools/journalint-vscode/`
- **Purpose**: Editor integration via LSP client
- **Key files**:
  - `src/extension.ts` - Main extension file
  - `package.json` - Extension manifest

## Journal File Format

Journalint processes files with this structure:

```markdown
---
date: 2023-05-04
start: 09:00
end: 17:30
---

# What I did today

- 09:00-10:00 PROJECT123 TASK456 1.00 Meeting: Daily standup
- 10:15-11:30 PROJECT123 TASK789 1.25 Development: Feature implementation
```

### Validation Rules

- Time consistency between YAML front matter and entry times
- Accurate duration calculations
- Proper format compliance for all time entries
- Date consistency across the file

## Key Technologies

### Rust Dependencies

- **chumsky**: Parser combinator library
- **chrono**: Date/time handling with timezone support
- **lsp-server/lsp-types**: Language Server Protocol implementation
- **clap**: Command-line argument parsing
- **serde**: Serialization framework
- **ariadne**: Error reporting with source highlighting
- **insta**: Snapshot testing for regression tests

### TypeScript Dependencies

- **vscode-languageclient**: LSP client for VS Code
- **@types/vscode**: VS Code API types

## Design Patterns and Architecture Decisions

### Visitor Pattern

The AST module uses the visitor pattern for traversing and processing parsed content:

```rust
pub trait Visitor {
    fn visit_document(&mut self, document: &Document) -> Result<(), Self::Error>;
    fn visit_entry(&mut self, entry: &Entry) -> Result<(), Self::Error>;
    // ... other visit methods
}
```

### Offset-Based Positioning

- Internal use of file offsets rather than line/column for efficiency
- Conversion to LSP position types only when needed for protocol communication

### Error Recovery

- Parser designed to continue processing even with syntax errors
- Provides meaningful diagnostics for partial or malformed input

### Dual Interface Design

- Same core logic serves both CLI and LSP use cases
- Clean separation between parsing/linting logic and application interfaces

## Further Reading

- [`docs/development.md`](docs/development.md) — Build, test, and common development tasks
- [`docs/troubleshooting.md`](docs/troubleshooting.md) — Debugging, CI/CD, and common issues
- [`CONTRIBUTING.md`](CONTRIBUTING.md) — Pull request process and release procedures
