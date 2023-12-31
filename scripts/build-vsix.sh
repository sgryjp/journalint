#!/bin/sh
set -xe

node --version
ls -lF target/$rust_target/release

cd tools/journalint-vscode
npm cache verify
npm ci
npm exec --yes -- @vscode/vsce@latest -- package --target ${node_target}
cd ../..
