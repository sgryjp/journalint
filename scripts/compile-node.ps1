$orig_cwd = Get-Location
if (-not $journalint_configured) {
    $script_dir = (Get-Item $MyInvocation.MyCommand.Path).Directory.FullName
    . (Join-Path $script_dir 'configure.ps1')
}

# -----------------------------------------------------------------------------
Write-Host -ForegroundColor Yellow "+ pushd $workspace_dir/tools/journalint-vscode"
Push-Location $workspace_dir/tools/journalint-vscode
try {
    Write-Host -ForegroundColor Yellow "+ Remove-Item -Recurse -Force -ErrorAction Ignore node_modules"
    Remove-Item -Recurse -Force -ErrorAction Ignore node_modules
    if ($LASTEXITCODE -ne 0) { throw }

    Write-Host -ForegroundColor Yellow "+ npm ci"
    npm ci
    if ($LASTEXITCODE -ne 0) { throw }

    Write-Host -ForegroundColor Yellow "+ npm exec -- tsc -p ./"
    npm exec -- tsc -p .
    if ($LASTEXITCODE -ne 0) { throw }

    Write-Host -ForegroundColor Yellow "+ New-Item -ItemType Directory -ErrorAction Ignore bundles/$node_target"
    New-Item -ItemType Directory -ErrorAction Ignore bundles/$node_target
    if ($LASTEXITCODE -ne 0) { throw }

    Write-Host -ForegroundColor Yellow "+ Copy-Item $native_executable_path bundles/$node_target/"
    Copy-Item $native_executable_path bundles/$node_target/
    if ($LASTEXITCODE -ne 0) { throw }

    Get-Item -Path bundles/$node_target/journalint.exe
} finally {
    Pop-Location
}
