if ($env:rust_target -eq $null -or $env:node_target -eq $null) {
    Write-Error "Environment variables not properly set:"
    Write-Error "    rust_target: $env:rust_target"
    Write-Error "    node_target: $env:node_target"
    throw
}

Write-Host -ForegroundColor Yellow "+ node --version"
node --version
if ($LASTEXITCODE -ne 0) {
    throw
}

Write-Host -ForegroundColor Yellow "+ ls target/$env:rust_target/release"
Get-ChildItem target/$env:rust_target/release

Write-Host -ForegroundColor Yellow "+ pushd tools/journalint-vscode"
pushd tools/journalint-vscode
try {
    Write-Host -ForegroundColor Yellow "+ Remove-Item -Recurse -Force node_modules"
    Remove-Item -Recurse -Force node_modules
    if ($LASTEXITCODE -ne 0) {
        throw
    }

    Write-Host -ForegroundColor Yellow "+ yarn install --frozen-lockfile"
    yarn install --frozen-lockfile
    if ($LASTEXITCODE -ne 0) {
        throw
    }

    Write-Host -ForegroundColor Yellow "+ vsce package --target $env:node_target"
    vsce package --yarn --target $env:node_target
    if ($LASTEXITCODE -ne 0) {
        throw
    }
}
finally {
    Write-Host -ForegroundColor Yellow "+ popd"
    popd
}
