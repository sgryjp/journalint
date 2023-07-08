#!/bin/sh
set -xe

node --version
ls -lF target/$rust_target/release

cd tools/journalint-vscode
npm cache verify
npm ci
npm exec @vscode/vsce -- package --target ${node_target}
cd ../..
