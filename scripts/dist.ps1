$orig_cwd = (Get-Location)
$workspace_dir = (Resolve-Path $PSScriptRoot\..)
if ($env:rust_target -eq $null) {
    $env:rust_target = (rustc -vV | Select-String host | %{ $_.ToString().Split(' ')[-1] })
}
if ($env:node_target -eq $null) {
    $env:node_target = (node -p "process.platform + '-' + process.arch")
}
Get-Command -CommandType Application -ErrorAction Stop vsce >$null

# -----------------------------------------------------------------------------
Write-Host -ForegroundColor Yellow "+ pushd $workspace_dir"
pushd $workspace_dir

try {
    New-Item -ItemType Directory -ErrorAction Ignore dist

    # Rust
    rustc --version
    cargo --version
    cargo build --release --target $env:rust_target
    if ($LASTEXITCODE -ne 0) { throw }

    Set-Location target/$env:rust_target/release
    tar -zcvf $workspace_dir/dist/journalint-$env:rust_target.tar.gz journalint.exe
    if ($LASTEXITCODE -ne 0) { throw }
    Set-Location $workspace_dir

    # Node
    Write-Host -ForegroundColor Yellow "+ node --version"
    node --version
    if ($LASTEXITCODE -ne 0) { throw }

    Write-Host -ForegroundColor Yellow "+ ls target/$env:rust_target/release"
    Get-ChildItem target/$env:rust_target/release

    Write-Host -ForegroundColor Yellow "+ pushd tools/journalint-vscode"
    pushd tools/journalint-vscode

    Write-Host -ForegroundColor Yellow "+ Remove-Item -Recurse -Force node_modules"
    Remove-Item -Recurse -Force node_modules
    if ($LASTEXITCODE -ne 0) { throw }

    Write-Host -ForegroundColor Yellow "+ yarn install --frozen-lockfile"
    yarn install --frozen-lockfile
    if ($LASTEXITCODE -ne 0) { throw }

    Write-Host -ForegroundColor Yellow "+ vsce package --target $env:node_target --out $workspace_dir/dist/"
    vsce package --yarn --target $env:node_target --out $workspace_dir/dist/
    if ($LASTEXITCODE -ne 0) { throw }
}
finally {
    Write-Host -ForegroundColor Yellow "+ popd"
    popd
}
