# Troubleshooting

## Common Issues

1. **Build failures**: Ensure all prerequisites are installed
2. **Test failures**: Run `cargo insta review` to update snapshots after parser changes
3. **LSP connection issues**: Check VS Code output panel for server logs
4. **Extension not loading**: Verify VSIX installation and reload VS Code

## Debug Mode

```bash
# Run with debug logging
RUST_LOG=debug cargo run -- --stdio

# Test specific journal file
cargo run -- path/to/journal.md
```

## Development Tools

- **VS Code**: Recommended for development with Rust and TypeScript extensions
- **cargo-insta**: For snapshot test management
- **towncrier**: For changelog management

## CI/CD Pipeline

### Automated Testing

- Runs on all pull requests and main branch pushes
- Cross-platform testing (Windows and Linux)
- Includes both Rust and TypeScript test suites

### Build Artifacts

- Native Rust binaries for Windows and Linux
- VS Code extension VSIX packages
- Automated GitHub releases for version tags

### Configuration

- **Main config**: `.github/workflows/cicd.yaml`
- **Targets**: `x86_64-pc-windows-msvc`, `x86_64-unknown-linux-gnu`
