#!/bin/sh
orig_cwd=$(pwd)
[ -z $journalint_configured ] && . $(cd $(dirname "$0") && pwd)/configure.sh
trap 'cd $orig_cwd' EXIT

# -----------------------------------------------------------------------------
set -ex
cd $workspace_dir
cargo build --release --target $rust_target
ls -l $native_executable_path
