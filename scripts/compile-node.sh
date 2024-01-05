#!/bin/sh
orig_cwd=$(pwd)
[ -z $journalint_configured ] && . $(cd $(dirname "$0") && pwd)/configure.sh
trap 'cd $orig_cwd' EXIT

# -----------------------------------------------------------------------------
if ! test -f $native_executable_path; then
    echo Native executable must be compiled before compiling journalint-vscode.
    exit 1
fi

set -ex
cd $workspace_dir/tools/journalint-vscode
rm -rf node_modules
yarn install --frozen-lockfile
yarn exec tsc -p .
mkdir -p bundles/$node_target
cp $native_executable_path bundles/$node_target/
ls -l bundles/$node_target/journalint
