#!/bin/sh
set -e

[ -z $rust_target ] && rust_target=$(rustc -vV | grep host | cut -d' ' -f 2)
[ -z $node_target ] && node_target=$(node -p "process.platform + '-' + process.arch")
if ! command -v vsce 2>/dev/null; then
    echo vsce command must be installed globally.
    exit 1
fi

set -x

node --version
ls -lF target/$rust_target/release

cd tools/journalint-vscode
npm cache verify
npm ci

# Here I intentionally avoid using npm exec because executing vsce in that way makes it
# fail to parse command arguments and I cannot fix the problem...
npm vsce package --target ${node_target}
cd ../..
