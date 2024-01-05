$workspace_dir = (Get-Item $MyInvocation.MyCommand.Path).Directory.Parent.FullName
if (-not $env:rust_target) {
    $rust_target = (rustc -vV | Select-String "host").ToString().Split(' ')[-1]
}
if (-not $env:node_target) {
    $node_target = "$((node -p "process.platform") + '-' + (node -p "process.arch"))"
}
$executable_suffix = ".exe"
$native_executable_path = Join-Path $workspace_dir "target\$rust_target\release\journalint$executable_suffix"
$journalint_configured = $true

Write-Host "---
workspace_dir: $workspace_dir
rust_target: $rust_target
node_target: $node_target
executable_suffix: $executable_suffix
native_executable_path: $native_executable_path
---"
