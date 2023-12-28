rust_target := `rustup show | grep Default | cut -d' ' -f 3`
node_target := `node -p 'process.platform + "-" + process.arch'`

default:
    just --list

coverage:
    cargo tarpaulin

_clean-crates:
    cargo clean

_clean-vsix:
    rm -rf tools/journalint-vscode/bundles
    rm -rf tools/journalint-vscode/out
    rm -rf tools/journalint-vscode/node_modules
    rm -rf tools/journalint-vscode/journalint*.vsix

clean: _clean-crates _clean-vsix

_build-crates:
    rustc --version
    cargo build --release --target {{rust_target}}

_build-vsix:
    node --version
    ls -lF target/{{rust_target}}/release
    cd tools/journalint-vscode && npm cache verify
    cd tools/journalint-vscode && npm ci
    cd tools/journalint-vscode && npm exec @vscode/vsce -- package --target {{node_target}}

build: _build-crates _build-vsix
