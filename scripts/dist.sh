#!/bin/sh
orig_cwd=$(pwd)
[ -z $journalint_configured ] && . $(cd $(dirname "$0") && pwd)/configure.sh
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
. ./scripts/compile-crates.sh
cd target/$rust_target/release
tar -zcvf $workspace_dir/dist/journalint-$rust_target.tar.gz --owner=0 --group=0 journalint
cd $workspace_dir

# Node
node --version
ls -lF target/$rust_target/release
. ./scripts/compile-node.sh

# As of vsce 2.22.0, without specifying --yarn for a project using workspace
# oddly fails with message as below:
#     Error: invalid relative path: extension/../../.git/config
vsce package --target ${node_target} --out $workspace_dir/dist/
