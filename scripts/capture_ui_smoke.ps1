# Capture The Final Landing's review set through the shared toolkit wrapper:
# every scene at the normal 1920x1080 canvas, and the busiest scenes again at
# the 1280x720 minimum window (scene names ending in _min).
param(
    [int]$Frames = 12,
    [switch]$SkipBuild
)

$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
$toolkitCapture = Join-Path (Split-Path $repoRoot -Parent) "macroquad-toolkit\scripts\capture_ui.ps1"
if (-not (Test-Path -LiteralPath $toolkitCapture)) {
    throw "Shared capture wrapper not found: $toolkitCapture"
}

$normal = @(
    "title", "setup", "assets", "species", "landing", "colony", "night", "late", "build",
    "mind", "needs", "bonds", "life",
    "research", "colonists", "relations", "expeditions", "chronicle", "colony_overlay",
    "council", "results"
)
$minimum = @("colony_min", "mind_min", "research_min")

& $toolkitCapture -GameDir $repoRoot -Prefix TFL -Scenes $normal -Frames $Frames `
    -WindowWidth 1920 -WindowHeight 1080 -OutputDir "docs\verification" -Release -SkipBuild:$SkipBuild
if (-not $?) { throw "Normal-size capture failed." }

& $toolkitCapture -GameDir $repoRoot -Prefix TFL -Scenes $minimum -Frames $Frames `
    -WindowWidth 1280 -WindowHeight 720 -OutputDir "docs\verification" -Release -SkipBuild
if (-not $?) { throw "Minimum-size capture failed." }
