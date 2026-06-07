$ErrorActionPreference = "Stop"

# Set credentials directly as environment variables for the session
$env:VSCE_PAT = $env:VSCE_PAT
$env:OVSX_PAT = $env:OVSX_PAT

Write-Host "==========================================" -ForegroundColor Cyan
Write-Host "   OmniFormatter Release Workflow" -ForegroundColor Cyan
Write-Host "==========================================" -ForegroundColor Cyan
Write-Host ""

cd extension

# Dynamically sync README and LICENSE from workspace root
Write-Host "Syncing documentation and LICENSE..." -ForegroundColor Yellow
if (Test-Path "../README.md") {
    (Get-Content ../README.md) | Where-Object { $_ -notmatch '<img src="extension/media/logo\.png"' -and $_ -notmatch '^\s*<br/>\s*$' } | Set-Content README.md -Force
}
if (Test-Path "../LICENSE") {
    Copy-Item ../LICENSE LICENSE -Force
}
Write-Host "[OK] Sync complete." -ForegroundColor Green
Write-Host ""

Write-Host "Bumping extension version..." -ForegroundColor Yellow
npm version patch --no-git-tag-version
if ($LASTEXITCODE -ne 0) { throw "Failed to bump version" }
Write-Host "[OK] Version bumped." -ForegroundColor Green
Write-Host ""

Write-Host "[1/3] Packaging the .vsix extension..." -ForegroundColor Yellow
npx -y @vscode/vsce package
if ($LASTEXITCODE -ne 0) { throw "Packaging failed with exit code $LASTEXITCODE" }
Write-Host "[OK] Packaging complete!" -ForegroundColor Green
Write-Host ""

Write-Host "[2/3] Publishing to VS Code Marketplace..." -ForegroundColor Yellow
npx -y @vscode/vsce publish -p $env:VSCE_PAT
if ($LASTEXITCODE -ne 0) { throw "VS Code Marketplace publish failed with exit code $LASTEXITCODE" }
Write-Host "[OK] VS Code Marketplace publish complete!" -ForegroundColor Green
Write-Host ""

Write-Host "[3/3] Publishing to Open VSX Marketplace..." -ForegroundColor Yellow
npx -y ovsx publish -p $env:OVSX_PAT
if ($LASTEXITCODE -ne 0) { throw "Open VSX publish failed with exit code $LASTEXITCODE" }
Write-Host "[OK] Open VSX publish complete!" -ForegroundColor Green
Write-Host ""

Write-Host "==========================================" -ForegroundColor Cyan
Write-Host "   Extension Published Successfully!" -ForegroundColor Green
Write-Host "==========================================" -ForegroundColor Cyan
