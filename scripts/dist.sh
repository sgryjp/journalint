#!/bin/sh
orig_cwd=$(pwd)
. $(cd $(dirname "$0") && pwd)/variables.sh
trap 'cd $orig_cwd' EXIT

if ! command -v vsce >/dev/null; then
    echo vsce command must be installed globally.
    exit 1
fi

# -----------------------------------------------------------------------------
set -ex
cd $workspace_dir
mkdir -p dist

# Rust
rustc --version
cargo --version
./scripts/compile-crates.sh
cd target/$rust_target/release
tar -zcvf $workspace_dir/dist/journalint-$rust_target.tar.gz --owner=0 --group=0 journalint
cd $workspace_dir

# Node
node --version
ls -lF target/$rust_target/release
./scripts/compile-node.sh

# Here I intentionally avoid using npm exec because executing vsce in that way
# makes it fail to parse command arguments and I cannot fix the problem...
vsce package --target ${node_target} --out $workspace_dir/dist/
