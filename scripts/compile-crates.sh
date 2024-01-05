#!/bin/sh
orig_cwd=$(pwd)
. $(cd $(dirname "$0") && pwd)/variables.sh
trap 'cd $orig_cwd' EXIT

# -----------------------------------------------------------------------------
set -ex
cd $workspace_dir
cargo build --quiet --release --target $rust_target
