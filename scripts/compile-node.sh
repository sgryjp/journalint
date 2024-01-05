#!/bin/sh
orig_cwd=$(pwd)
. $(cd $(dirname "$0") && pwd)/variables.sh
trap 'cd $orig_cwd' EXIT

# -----------------------------------------------------------------------------
set -ex
cd $workspace_dir/tools/journalint-vscode
npm cache verify >/dev/null
npm ci >/dev/null
npm exec -c 'tsc -p .'
[ -f $native_executable_path ] || $workspace_dir/scripts/compile-crates.sh
mkdir -p $workspace_dir/bundles/$node_target
cp $native_executable_path bundles/$node_target
