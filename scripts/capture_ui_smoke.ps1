param(
    [string]$OutputDir = "docs\verification",
    [int]$Frames = 8
)

$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
$outDir = Join-Path $repoRoot $OutputDir

Set-Location $repoRoot
$targetDir = (cargo metadata --format-version 1 --no-deps | ConvertFrom-Json).target_directory
$exe = Join-Path $targetDir "debug\finallanding.exe"
cargo build

if ($LASTEXITCODE -ne 0) {
    throw "Capture build failed with exit code $LASTEXITCODE."
}

if (!(Test-Path -LiteralPath $exe)) {
    throw "Missing executable: $exe"
}

New-Item -ItemType Directory -Force -Path $outDir | Out-Null
Add-Type -AssemblyName System.Drawing

function Get-RegionStats {
    param(
        [System.Drawing.Bitmap]$Bitmap,
        [int]$X,
        [int]$Y,
        [int]$Width,
        [int]$Height,
        [int]$Step = 6
    )

    $xEnd = [Math]::Min($Bitmap.Width - 1, $X + $Width)
    $yEnd = [Math]::Min($Bitmap.Height - 1, $Y + $Height)
    $nonBlack = 0
    $colorful = 0
    $count = 0
    $minBrightness = 765
    $maxBrightness = 0
    $maxGreen = 0

    for ($py = [Math]::Max(0, $Y); $py -lt $yEnd; $py += $Step) {
        for ($px = [Math]::Max(0, $X); $px -lt $xEnd; $px += $Step) {
            $pixel = $Bitmap.GetPixel($px, $py)
            $brightness = [int]$pixel.R + [int]$pixel.G + [int]$pixel.B
            $spread = [Math]::Max([Math]::Max($pixel.R, $pixel.G), $pixel.B) - [Math]::Min([Math]::Min($pixel.R, $pixel.G), $pixel.B)

            if ($brightness -gt 45) {
                $nonBlack++
            }
            if ($spread -gt 18) {
                $colorful++
            }
            if ($brightness -lt $minBrightness) {
                $minBrightness = $brightness
            }
            if ($brightness -gt $maxBrightness) {
                $maxBrightness = $brightness
            }
            if ($pixel.G -gt $maxGreen) {
                $maxGreen = $pixel.G
            }
            $count++
        }
    }

    return @{
        Count = $count
        NonBlackRatio = if ($count -gt 0) { $nonBlack / $count } else { 0 }
        ColorfulRatio = if ($count -gt 0) { $colorful / $count } else { 0 }
        BrightnessRange = $maxBrightness - $minBrightness
        MaxGreen = $maxGreen
    }
}

function Assert-RegionVisible {
    param(
        [System.Drawing.Bitmap]$Bitmap,
        [string]$Name,
        [int]$X,
        [int]$Y,
        [int]$Width,
        [int]$Height,
        [double]$MinNonBlackRatio,
        [int]$MinBrightnessRange
    )

    $stats = Get-RegionStats -Bitmap $Bitmap -X $X -Y $Y -Width $Width -Height $Height
    if ($stats.NonBlackRatio -lt $MinNonBlackRatio -or $stats.BrightnessRange -lt $MinBrightnessRange) {
        throw "Capture failed: $Name region looks blank or flat at ($X,$Y) size ${Width}x${Height} (nonblack=$([Math]::Round($stats.NonBlackRatio, 3)), range=$($stats.BrightnessRange))."
    }
}

function Assert-ActiveToolbarVisible {
    param(
        [System.Drawing.Bitmap]$Bitmap,
        [int]$Width,
        [int]$Height,
        [int]$ActiveIndex
    )

    $toolbarWidth = [Math]::Min([Math]::Max($Width - 14, 308), 760)
    $toolbarX = ($Width - $toolbarWidth) * 0.5
    $buttonWidth = $toolbarWidth / 7
    $buttonX = $toolbarX + $ActiveIndex * $buttonWidth

    $stats = Get-RegionStats `
        -Bitmap $Bitmap `
        -X ([int]($buttonX + $buttonWidth * 0.1)) `
        -Y ([int]($Height - ($(if ($Width -lt 760) { 104 } else { 86 }) - 10))) `
        -Width ([int]($buttonWidth * 0.8)) `
        -Height ([int]($Height * 0.08)) `
        -Step 3

    if ($stats.NonBlackRatio -lt 0.18 -or $stats.ColorfulRatio -lt 0.03 -or $stats.MaxGreen -lt 65) {
        throw "Capture failed: active toolbar region is not visibly highlighted (nonblack=$([Math]::Round($stats.NonBlackRatio, 3)), colorful=$([Math]::Round($stats.ColorfulRatio, 3)), maxGreen=$($stats.MaxGreen))."
    }
}

function Assert-PlacementPreviewVisible {
    param(
        [System.Drawing.Bitmap]$Bitmap,
        [int]$Width,
        [int]$Height
    )

    $stats = Get-RegionStats `
        -Bitmap $Bitmap `
        -X ([int]($Width * 0.35)) `
        -Y ([int]($Height * 0.30)) `
        -Width ([int]($Width * 0.28)) `
        -Height ([int]($Height * 0.28)) `
        -Step 3

    if ($stats.ColorfulRatio -lt 0.06 -or $stats.MaxGreen -lt 140) {
        throw "Capture failed: placement preview is not visible enough (colorful=$([Math]::Round($stats.ColorfulRatio, 3)), maxGreen=$($stats.MaxGreen))."
    }
}

$sizes = @(
    @{ Width = 1280; Height = 720; Name = "ui_smoke_1280x720.png"; Fullscreen = "0"; Mode = "build"; Selected = ""; ActiveIndex = 0; History = "0"; SocialDay = ""; Poses = "0"; Spaces = "0"; SelectedBuilding = ""; PreviewX = ""; PreviewY = "" },
    @{ Width = 1920; Height = 1080; Name = "ui_smoke_1920x1080.png"; Fullscreen = "1"; Mode = "build"; Selected = ""; ActiveIndex = 0; History = "0"; SocialDay = ""; Poses = "0"; Spaces = "0"; SelectedBuilding = ""; PreviewX = ""; PreviewY = "" },
    @{ Width = 1280; Height = 720; Name = "ui_smoke_assign_1280x720.png"; Fullscreen = "0"; Mode = "assign"; Selected = "5"; ActiveIndex = 5; History = "0"; SocialDay = ""; Poses = "0"; Spaces = "1"; SelectedBuilding = ""; PreviewX = ""; PreviewY = "" },
    @{ Width = 1280; Height = 720; Name = "ui_smoke_log_1280x720.png"; Fullscreen = "0"; Mode = "log"; Selected = ""; ActiveIndex = 6; History = "1"; SocialDay = "4"; Poses = "0"; Spaces = "0"; SelectedBuilding = ""; PreviewX = ""; PreviewY = "" },
    @{ Width = 1280; Height = 720; Name = "ui_smoke_placement_1280x720.png"; Fullscreen = "0"; Mode = "rooms"; Selected = ""; ActiveIndex = 1; History = "0"; SocialDay = ""; Poses = "0"; Spaces = "0"; SelectedBuilding = "habitat"; PreviewX = "5"; PreviewY = "9" },
    @{ Width = 1280; Height = 720; Name = "ui_smoke_poses_1280x720.png"; Fullscreen = "0"; Mode = "build"; Selected = ""; ActiveIndex = 0; History = "0"; SocialDay = ""; Poses = "1"; Spaces = "0"; SelectedBuilding = ""; PreviewX = ""; PreviewY = "" }
    @{ Width = 720; Height = 480; Name = "ui_smoke_touch_720x480.png"; Fullscreen = "0"; Mode = "assign"; Selected = "0"; ActiveIndex = 5; History = "0"; SocialDay = ""; Poses = "0"; Spaces = "1"; SelectedBuilding = ""; PreviewX = ""; PreviewY = "" }
)

$manifest = Join-Path $outDir ".capture_manifest_$PID.tsv"
$rows = foreach ($size in $sizes) {
    $path = Join-Path $outDir $size.Name
    if (Test-Path -LiteralPath $path) {
        Remove-Item -LiteralPath $path -Force
    }
    $scene = ([IO.Path]::GetFileNameWithoutExtension($size.Name) -replace '^ui_', '')
    "$scene`t$path"
}
Set-Content -LiteralPath $manifest -Value $rows -Encoding utf8
$env:TFL_CAPTURE_MANIFEST = $manifest
$env:TFL_CAPTURE_FRAMES = "$Frames"
$env:TFL_HEADLESS = "1"
try {
    $proc = Start-Process -FilePath $exe -PassThru -WindowStyle Hidden
    Write-Host ("Capturing {0} smoke scenes in one process (PID {1})..." -f $sizes.Count, $proc.Id)
    $proc.WaitForExit()
    if ($proc.ExitCode -ne 0) { throw "Capture process exited with code $($proc.ExitCode)." }
}
finally {
    Remove-Item Env:\TFL_CAPTURE_MANIFEST, Env:\TFL_CAPTURE_FRAMES, Env:\TFL_HEADLESS -ErrorAction SilentlyContinue
    Remove-Item -LiteralPath $manifest -Force -ErrorAction SilentlyContinue
}

foreach ($size in $sizes) {
    $path = Join-Path $outDir $size.Name
    if (!(Test-Path -LiteralPath $path)) {
        throw "Capture failed: $path was not created."
    }

    $file = Get-Item -LiteralPath $path
    if ($file.Length -lt 100000) {
        throw "Capture failed: $path is unexpectedly small ($($file.Length) bytes)."
    }

    $image = [System.Drawing.Bitmap]::FromFile($path)
    try {
        if ($image.Width -ne $size.Width -or $image.Height -ne $size.Height) {
            throw "Capture failed: $path is $($image.Width)x$($image.Height), expected $($size.Width)x$($size.Height)."
        }

        $phone = $size.Width -lt 760
        $leftX = if ($phone) { 7 } else { 10 }
        $leftWidth = if ($phone) { 140 } else { 278 }
        $rightX = if ($phone) { $size.Width - 149 } else { $size.Width - 292 }
        $rightWidth = if ($phone) { 142 } else { 278 }
        Assert-RegionVisible -Bitmap $image -Name "left rail" -X $leftX -Y 70 -Width $leftWidth -Height 170 -MinNonBlackRatio 0.25 -MinBrightnessRange 35
        Assert-RegionVisible -Bitmap $image -Name "right rail" -X $rightX -Y 70 -Width $rightWidth -Height ([int]($size.Height - 78)) -MinNonBlackRatio 0.12 -MinBrightnessRange 35
        Assert-RegionVisible -Bitmap $image -Name "central map" -X ([int]($size.Width * 0.26)) -Y ([int]($size.Height * 0.18)) -Width ([int]($size.Width * 0.48)) -Height ([int]($size.Height * 0.48)) -MinNonBlackRatio 0.12 -MinBrightnessRange 30
        Assert-ActiveToolbarVisible -Bitmap $image -Width $size.Width -Height $size.Height -ActiveIndex $size.ActiveIndex
        if ($size.SelectedBuilding -ne "") {
            Assert-PlacementPreviewVisible -Bitmap $image -Width $size.Width -Height $size.Height
        }
    }
    finally {
        $image.Dispose()
    }

    Write-Host "Captured $($file.FullName) ($($file.Length) bytes, $($size.Width)x$($size.Height))"
}
