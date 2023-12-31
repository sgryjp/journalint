#!/bin/sh
set -e

[ -z $rust_target ] && rust_target=$(rustc -vV | grep host | cut -d' ' -f 2)
[ -z $node_target ] && node_target=$(node -p "process.platform + '-' + process.arch")

set -x

node --version
ls -lF target/$rust_target/release

cd tools/journalint-vscode
npm cache verify
npm ci
npm exec --yes -- @vscode/vsce@latest -- package --target ${node_target}
cd ../..
