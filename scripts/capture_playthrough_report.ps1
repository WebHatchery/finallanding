# Play the standard campaign matrix headlessly with the scripted colony AI and
# write the pacing report to docs\verification\campaign_report.md.
param(
    [string]$OutputPath = "docs\verification\campaign_report.md",
    [int]$MaxDays = 420
)

$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
$output = Join-Path $repoRoot $OutputPath
Set-Location $repoRoot

& ..\rust_management\cargo.ps1 build --release
if ($LASTEXITCODE -ne 0) { throw "Release build failed with exit code $LASTEXITCODE." }

$env:TFL_CAMPAIGN_REPORT_PATH = $output
$env:TFL_CAMPAIGN_MAX_DAYS = "$MaxDays"
$env:TFL_HEADLESS = "1"
try {
    & ..\rust_management\cargo.ps1 run --release
}
finally {
    Remove-Item Env:\TFL_CAMPAIGN_REPORT_PATH, Env:\TFL_CAMPAIGN_MAX_DAYS, Env:\TFL_HEADLESS -ErrorAction SilentlyContinue
}

if (!(Test-Path -LiteralPath $output)) {
    throw "Campaign report was not created: $output"
}
$content = Get-Content -LiteralPath $output -Raw
foreach ($required in @("verdant_basin", "frost_shelf", "ashen_steppe", "Victory")) {
    if ($content -notmatch [Regex]::Escape($required)) {
        throw "Campaign report missing expected text: $required"
    }
}
Write-Host "Wrote $output"
