$orig_cwd = Get-Location
if (-not $journalint_configured) {
    $script_dir = (Get-Item $MyInvocation.MyCommand.Path).Directory.FullName
    . (Join-Path $script_dir 'configure.ps1')
}

# -----------------------------------------------------------------------------
Write-Host -ForegroundColor Yellow "+ pushd $workspace_dir"
Push-Location $workspace_dir
try {
    $command = "cargo build --release --target $rust_target"
    Write-Host -ForegroundColor Yellow "+ $command"
    $command | Invoke-Expression
    Get-Item -Path $native_executable_path
} finally {
    Pop-Location
}
