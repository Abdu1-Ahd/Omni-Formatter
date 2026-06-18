$ErrorActionPreference = "Stop"

Write-Host "==========================================" -ForegroundColor Cyan
Write-Host "   OmniFormatter Release Workflow (CI/CD)" -ForegroundColor Cyan
Write-Host "==========================================" -ForegroundColor Cyan
Write-Host ""

cd extension

# Dynamically sync README and LICENSE from workspace root
Write-Host "[1/3] Syncing documentation and LICENSE..." -ForegroundColor Yellow
if (Test-Path "../README.md") {
    $readme = Get-Content ../README.md -Raw -Encoding utf8
    # Filter out the logo image and its trailing <br/>
    $readme = $readme -replace '(?s)<img src=".*media/Omni-Formatter-Logo\.svg"[^>]*>\s*\r?\n?\s*<br/>\s*\r?\n?', ''
    $readme = $readme -replace '(?m)^\s*# OmniFormatter\s*$\r?\n?', ''
    $readme = $readme -replace '(?m)^\s*\[\!\[VS[_ ]Code[^\]]*\].*$\r?\n?', ''
    $readme = $readme -replace '(?m)^\s*\[\!\[Open VSX[^\]]*\].*$\r?\n?', ''
    $readme = $readme -replace '\*\*One extension\. Every language\. Zero configuration\.\*\*', '**One extension. Every language. Zero configuration.**<br/>'

    # Replace the mermaid block with the unicode box diagram
    $mermaidBlock = '(?s)```mermaid\s*\r?\n.*?\r?\n```'
    $unicodeDiagram = '```text
   ┌───────────────────────────────────┐
   │       🔌 VS Code Extension        │
   │           (TypeScript)            │
   └─────────────────┬─────────────────┘
                     │
             [ Zero-Copy IPC ]
                     │
                     ▼
   ┌───────────────────────────────────┐
   │          ⚡ Worker Pool           │
   │             (Node.js)             │
   └─────────────────┬─────────────────┘
                     │
            [ Fast WASM Call ]
                     │
                     ▼
   ┌───────────────────────────────────┐
   │           ⚙️ WASM Core            │
   │              (Rust)               │
   └────────┬─────────────────┬────────┘
            │                 │
  [ Loads on Demand ] [ Reads Configs ]
            │                 │
            ▼                 ▼
    ┌───────────────┐ ┌───────────────┐
    │📦 Lang Modules│ │🛠️ Config Adpt│
    │ (.wasm binary)│ │(Native Format)│
    └───────┬───────┘ └───────────────┘
            │
  [ Fetched & Cached]
            │
            ▼
    ┌───────────────┐
    │ ☁️ Registry   │
    │(GitHub Pages) │
    └───────────────┘
```'
    $readme = $readme -replace $mermaidBlock, $unicodeDiagram
    # ponytail: disabled auto-sync for this release to preserve manual edits
    # $readme | Set-Content README.md -Force -Encoding utf8
}
if (Test-Path "../LICENSE") {
    Copy-Item ../LICENSE LICENSE -Force
}
Write-Host "[OK] Sync complete." -ForegroundColor Green
Write-Host ""

Write-Host "[2/3] Bumping extension version..." -ForegroundColor Yellow
$versionOutput = npm version patch --no-git-tag-version
if ($LASTEXITCODE -ne 0) { throw "Failed to bump version" }
# $versionOutput is e.g. "v0.1.5"
$newVersion = $versionOutput.Trim()
Write-Host "[OK] Version bumped to $newVersion." -ForegroundColor Green
Write-Host ""

Write-Host "[3/3] Committing, tagging, and pushing to GitHub..." -ForegroundColor Yellow
cd ..
git add extension/package.json extension/package-lock.json extension/README.md extension/LICENSE
git commit -m "chore: release $newVersion"
git tag $newVersion
git push origin main
git push origin $newVersion

Write-Host ""
Write-Host "==========================================" -ForegroundColor Cyan
Write-Host "   Release triggered successfully! " -ForegroundColor Green
Write-Host "   GitHub Actions is now publishing $newVersion in the background." -ForegroundColor Green
Write-Host "==========================================" -ForegroundColor Cyan
