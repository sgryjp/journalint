#!/bin/sh
orig_cwd=$(pwd)
workspace_dir=$(cd $(dirname $(dirname "$0")) && pwd)
[ -z $rust_target ] && rust_target=$(rustc -vV | grep host | cut -d' ' -f 2)
[ -z $node_target ] && node_target=$(node -p "process.platform + '-' + process.arch")
if ! command -v vsce >/dev/null; then
    echo vsce command must be installed globally.
    exit 1
fi

trap 'cd $orig_cwd' EXIT

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
rm -rf node_modules
yarn install --frozen-lockfile

# As of vsce 2.22.0, without specifying --yarn for a project using workspace
# oddly fails with message as below:
#     Error: invalid relative path: extension/../../.git/config
# To avoid this error I switch the packaging tool to yarn and rewrote scripts
# using npm.
#
# Here I intentionally avoid using npm exec because executing vsce in that way
# makes it fail to parse command arguments and I cannot fix the problem...
vsce package --yarn --target ${node_target} --out $workspace_dir/dist/
