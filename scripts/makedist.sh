#!/bin/sh
orig_cwd=$(pwd)
workspace_dir=$(cd $(dirname $(dirname "$0")) && pwd)
[ -z $rust_target ] && rust_target=$(rustc -vV | grep host | cut -d' ' -f 2)
[ -z $node_target ] && node_target=$(node -p "process.platform + '-' + process.arch")
if ! command -v vsce >/dev/null; then
    echo vsce command must be installed globally.
    exit 1
fi

cleanup() {
    cd $orig_cwd && exit 1
}
trap cleanup EXIT

# -----------------------------------------------------------------------------
set -ex
cd $workspace_dir
mkdir -p dist

# Rust
rustc --version
cargo --version
cargo build --release --target $rust_target
cd target/$rust_target/release
tar -zcvf $workspace_dir/dist/journalint-$rust_target.tar.gz --owner=0 --group=0 journalint
cd $workspace_dir

# Node
node --version
ls -lF target/$rust_target/release
cd tools/journalint-vscode
npm cache verify
npm ci

# Here I intentionally avoid using npm exec because executing vsce in that way
# makes it fail to parse command arguments and I cannot fix the problem...
vsce package --target ${node_target} --out $workspace_dir/dist/
