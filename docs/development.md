# Development Guide

## Prerequisites

- Rust (latest stable)
- Node.js 20+
- VS Code (for extension development)

## Building the Project

```bash
# Compile Rust crates
./scripts/compile-crates.sh

# Compile TypeScript/Node.js components
./scripts/compile-node.sh

# Create distribution packages
./scripts/dist.sh
```

## Testing

```bash
# Run Rust tests
cargo test

# Run VS Code extension tests
cd tools/journalint-vscode && npm test
```

## Common Development Tasks

### Adding New Linting Rules

1. Define the rule logic in `crates/journalint-parse/src/lint.rs`
2. Add tests with example inputs in test files
3. Update snapshot tests if needed: `cargo insta review`

### Extending the Parser

1. Modify grammar in `crates/journalint-parse/src/parse.rs`
2. Update AST definitions in `crates/journalint-parse/src/ast.rs`
3. Add corresponding visitor methods for new AST nodes

### Adding Auto-fix Commands

1. Implement the command in `crates/journalint/src/commands/`
2. Register the command in the LSP service
3. Add corresponding client-side command in VS Code extension

### Testing Changes

1. **Unit Tests**: Add tests for new functionality
2. **Snapshot Tests**: Use `insta` for regression testing
3. **Integration Tests**: Test full CLI and LSP workflows

## Releasing

See `CONTRIBUTING.md` for the full release process.
