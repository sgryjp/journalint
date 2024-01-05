#!/bin/sh
workspace_dir=$(cd $(dirname $(dirname "$0")) && pwd)
[ -z $rust_target ] && rust_target=$(rustc -vV | grep host | cut -d' ' -f 2)
[ -z $node_target ] && node_target=$(node -p "process.platform + '-' + process.arch")
executable_suffix=
native_executable_path=$workspace_dir/target/$rust_target/release/journalint$executable_suffix

echo ---
echo workspace_dir: $workspace_dir
echo rust_target: $rust_target
echo node_target: $node_target
echo executable_suffix: $executable_suffix
echo native_executable_path: $native_executable_path
echo ---
