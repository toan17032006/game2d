# Extract runtime assets from `assest/Tiny Swords (Free Pack)` into `game/assets/`
# and generate `game/data/assets.csv` (+ `game/data/units.csv`).
#
# Geometry derivation (measured, see spec.md):
# - PNG IHDR gives width/height.
# - Per-folder rule table encodes the aseprite canvas width for strips
#   (frames = png_width / canvas_width) or the square-frame rule
#   (frame = png_height, frames = png_width / png_height).
# - Every computed frames count must divide evenly; totals cross-check
#   against the .aseprite header frame counts (printed at the end).

[CmdletBinding()]
param(
    [string]$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..\..')).Path,
    [switch]$DryRun
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$source = Join-Path $RepoRoot 'assest\Tiny Swords (Free Pack)'
$gameDir = Join-Path $RepoRoot 'game'
$assetsDir = Join-Path $gameDir 'assets'
$dataDir = Join-Path $gameDir 'data'

if (-not (Test-Path -LiteralPath $source -PathType Container)) {
    throw "Source asset pack not found: $source"
}

function Convert-ToSnake([string]$Text) {
    # Split camelCase boundaries before normalizing, so
    # 'BigBlueButton_Regular' -> big_blue_button_regular.
    $t = $Text -creplace '([a-z0-9])([A-Z])', '$1_$2'
    $t = $t.ToLowerInvariant()
    $t = $t -replace '[^a-z0-9]+', '_'
    $t = $t -replace '_+', '_'
    $t = $t.Trim('_')
    return $t
}

function Read-PngSize([string]$Path) {
    $fs = [IO.File]::OpenRead($Path)
    try {
        $b = New-Object byte[] 24
        $null = $fs.Read($b, 0, 24)
        $sigOK = ($b[0] -eq 137 -and $b[1] -eq 80 -and $b[2] -eq 78 -and $b[3] -eq 71)
        if (-not $sigOK) { throw "Not a PNG: $Path" }
        $w = [int]$b[16] * 16777216 + [int]$b[17] * 65536 + [int]$b[18] * 256 + [int]$b[19]
        $h = [int]$b[20] * 16777216 + [int]$b[21] * 65536 + [int]$b[22] * 256 + [int]$b[23]
        return @{ Width = $w; Height = $h }
    } finally {
        $fs.Close()
    }
}

# Per-source-folder strip rules: frame width comes from the measured
# aseprite canvas width; frames = png_width / frameWidth.
# 'square' means the aseprite canvas is square and equals png height.
$stripRules = @{
    'Particle FX'                                                              = 'square'
    'Terrain\Decorations\Bushes'                                               = 128
    'Terrain\Decorations\Rocks in the Water'                                   = 64
    'Terrain\Decorations\Rubber Duck'                                          = 32
    'Terrain\Resources\Gold\Gold Resource'                                     = 128
    'Terrain\Resources\Gold\Gold Stones'                                       = 128
    'Terrain\Resources\Meat\Sheep'                                             = 128
    'Terrain\Resources\Wood\Trees'                                             = 192
}

# Name-specific strip rule inside Terrain\Tileset (Water Foam.aseprite).
$nameStripRules = @{
    'Water Foam' = 192
}

$excludedDirNames = @('__MACOSX')
$excludedPathFragments = @('UI Banners from the store page')

# Manual rename map applied to the snake_case stem (source typo fix).
$stemRewrite = @{
    'bushe' = 'bush'
}

function Get-Faction([string]$FolderName) {
    if ($FolderName -match '^(Black|Blue|Purple|Red|Yellow) ') {
        return $Matches[1].ToLowerInvariant()
    }
    throw "Unknown faction folder: $FolderName"
}

function Get-Destination([string]$RelSource) {
    # Returns @{ Dest = relative dest path under game/; Category = top category }
    $parts = $RelSource -split '\\'
    if ($parts[0] -eq 'Units' -and $parts[1] -match ' Units$') {
        $faction = Get-Faction $parts[1]
        $unit = Convert-ToSnake $parts[2]
        $file = Convert-ToSnake ([IO.Path]::GetFileNameWithoutExtension($parts[3]))
        return @{ Dest = "assets/units/$faction/$unit/$file.png"; Category = 'units' }
    }
    if ($parts[0] -eq 'Buildings' -and $parts[1] -match ' Buildings$') {
        $faction = Get-Faction $parts[1]
        $file = Convert-ToSnake ([IO.Path]::GetFileNameWithoutExtension($parts[2]))
        return @{ Dest = "assets/buildings/$faction/$file.png"; Category = 'buildings' }
    }
    if ($parts[0] -eq 'Terrain') {
        if ($parts[1] -eq 'Tileset') {
            $file = Convert-ToSnake ([IO.Path]::GetFileNameWithoutExtension($parts[2]))
            return @{ Dest = "assets/terrain/tileset/$file.png"; Category = 'terrain' }
        }
        if ($parts[1] -eq 'Decorations') {
            $kind = Convert-ToSnake $parts[2]
            $file = Convert-ToSnake ([IO.Path]::GetFileNameWithoutExtension($parts[3]))
            return @{ Dest = "assets/terrain/decorations/$kind/$file.png"; Category = 'terrain' }
        }
        if ($parts[1] -eq 'Resources') {
            $type = Convert-ToSnake $parts[2]
            $fileLeaf = [IO.Path]::GetFileNameWithoutExtension($parts[-1])
            $file = Convert-ToSnake $fileLeaf
            if ($parts.Count -gt 4) {
                $sub = Convert-ToSnake $parts[3]
                return @{ Dest = "assets/terrain/resources/$type/$sub/$file.png"; Category = 'terrain' }
            }
            return @{ Dest = "assets/terrain/resources/$type/$file.png"; Category = 'terrain' }
        }
        throw "Unknown Terrain layout: $RelSource"
    }
    if ($parts[0] -eq 'Particle FX') {
        $file = Convert-ToSnake ([IO.Path]::GetFileNameWithoutExtension($parts[1]))
        return @{ Dest = "assets/fx/$file.png"; Category = 'fx' }
    }
    if ($parts[0] -eq 'UI Elements' -and $parts[1] -eq 'UI Elements') {
        $group = Convert-ToSnake $parts[2]
        $file = Convert-ToSnake ([IO.Path]::GetFileNameWithoutExtension($parts[3]))
        return @{ Dest = "assets/ui/$group/$file.png"; Category = 'ui' }
    }
    throw "Unhandled source path: $RelSource"
}

function Get-Geometry([string]$RelSource, [int]$Width, [int]$Height) {
    # Returns @{ FrameW; FrameH; Frames }
    $parts = $RelSource -split '\\'
    $dirRel = ($parts[0..($parts.Count - 2)] -join '\')
    $stem = [IO.Path]::GetFileNameWithoutExtension($parts[-1])

    if ($dirRel -eq 'Terrain\Tileset' -and $nameStripRules.ContainsKey($stem)) {
        $fw = [int]$nameStripRules[$stem]
        if ($Width % $fw -ne 0) { throw "Non-divisible strip: $RelSource ($Width % $fw)" }
        return @{ FrameW = $fw; FrameH = $Height; Frames = [int]($Width / $fw) }
    }

    $rule = $null
    if ($stripRules.ContainsKey($dirRel)) { $rule = $stripRules[$dirRel] }
    elseif ($dirRel -match '^Units\\') { $rule = 'square' }

    if ($null -eq $rule) {
        return @{ FrameW = $Width; FrameH = $Height; Frames = 1 }
    }

    $fw = if ($rule -eq 'square') { $Height } else { [int]$rule }
    if ($fw -le 0 -or $Width % $fw -ne 0) {
        throw "Non-divisible strip: $RelSource ($Width % $fw)"
    }
    return @{ FrameW = $fw; FrameH = $Height; Frames = [int]($Width / $fw) }
}

# ---- Collect source PNGs ----
$allPngs = Get-ChildItem -LiteralPath $source -Recurse -Filter '*.png' -File
$selected = @()
foreach ($png in $allPngs) {
    $rel = $png.FullName.Substring($source.Length + 1)
    $skip = $false
    foreach ($frag in $excludedPathFragments) {
        if ($rel -like "*$frag*") { $skip = $true; break }
    }
    if (-not $skip) {
        foreach ($d in $excludedDirNames) {
            if (($rel -split '\\') -contains $d) { $skip = $true; break }
        }
    }
    if (-not $skip) { $selected += @{ Png = $png; Rel = $rel } }
}

Write-Output ("Source PNGs total: " + $allPngs.Count)
Write-Output ("Selected for extraction: " + $selected.Count)

if ($DryRun) { return }

# ---- Reset destination assets + generated data ----
if (Test-Path -LiteralPath $assetsDir) {
    Remove-Item -LiteralPath $assetsDir -Recurse -Force
}
New-Item -ItemType Directory -Path $assetsDir -Force | Out-Null
New-Item -ItemType Directory -Path $dataDir -Force | Out-Null

$rows = [System.Collections.Generic.List[string]]::new()
$rows.Add('key,category,path,frame_w,frame_h,frames')

$categoryCounts = @{}
$framesByUnitFolder = @{}
$seenKeys = @{}

foreach ($item in $selected) {
    $png = $item.Png
    $rel = $item.Rel
    $size = Read-PngSize $png.FullName
    $map = Get-Destination $rel
    $geo = Get-Geometry $rel $size.Width $size.Height

    $destRel = $map.Dest
    $stem = [IO.Path]::GetFileNameWithoutExtension($destRel)
    foreach ($key in $stemRewrite.Keys) {
        if ($stem -match ('^' + $key)) {
            $stem = $stem -replace ('^' + $key), $stemRewrite[$key]
        }
    }
    $leafDir = $destRel.Substring(0, $destRel.Length - ([IO.Path]::GetFileName($destRel).Length + 1))
    $destRel = "$leafDir/$stem.png"

    $key = $destRel.Substring('assets/'.Length) -replace '\.png$', ''
    if ($seenKeys.ContainsKey($key)) { throw "Duplicate asset key: $key" }
    $seenKeys[$key] = $true

    $destAbs = Join-Path $gameDir ($destRel.Replace('/', '\'))
    $destDir = Split-Path -Parent $destAbs
    if (-not (Test-Path -LiteralPath $destDir)) {
        New-Item -ItemType Directory -Path $destDir -Force | Out-Null
    }
    Copy-Item -LiteralPath $png.FullName -Destination $destAbs -Force

    $category = $map.Category
    if (-not $categoryCounts.ContainsKey($category)) { $categoryCounts[$category] = 0 }
    $categoryCounts[$category]++

    $dirKey = ($rel -split '\\')[0..($rel.Split('\').Count - 2)] -join '\'
    if (-not $framesByUnitFolder.ContainsKey($dirKey)) { $framesByUnitFolder[$dirKey] = 0 }
    $framesByUnitFolder[$dirKey] += [int]$geo.Frames

    $rows.Add(("{0},{1},{2},{3},{4},{5}" -f $key, $category, $destRel, $geo.FrameW, $geo.FrameH, $geo.Frames))
}

$manifestPath = Join-Path $dataDir 'assets.csv'
[IO.File]::WriteAllText($manifestPath, ($rows -join "`r`n") + "`r`n", (New-Object System.Text.UTF8Encoding($false)))

Write-Output ("Manifest rows: " + ($rows.Count - 1))
$categoryCounts.GetEnumerator() | Sort-Object Name | ForEach-Object {
    Write-Output ("  category {0}: {1}" -f $_.Key, $_.Value)
}

# ---- Cross-checks: computed frames vs .aseprite header frame counts ----
Write-Output 'Cross-check computed strip frames vs aseprite header totals:'
function Get-AsepriteFrames([string]$Path) {
    $b = [IO.File]::ReadAllBytes($Path)
    return [int]$b[6] + [int]$b[7] * 256
}
$checks = @(
    @{ Label = 'Particle FX (all fx files)'; Folder = 'Particle FX'; Expected = 75 },
    @{ Label = 'Bushes'; Folder = 'Terrain\Decorations\Bushes'; Expected = 32 },
    @{ Label = 'Rocks in the Water (per file x4)'; Folder = 'Terrain\Decorations\Rocks in the Water'; Expected = 64 },
    @{ Label = 'Rubber Duck'; Folder = 'Terrain\Decorations\Rubber Duck'; Expected = 3 },
    @{ Label = 'Gold Resource'; Folder = 'Terrain\Resources\Gold\Gold Resource'; Expected = 7 },
    @{ Label = 'Gold Stones'; Folder = 'Terrain\Resources\Gold\Gold Stones'; Expected = 42 },
    @{ Label = 'Sheep'; Folder = 'Terrain\Resources\Meat\Sheep'; Expected = 22 },
    @{ Label = 'Trees + Stumps'; Folder = 'Terrain\Resources\Wood\Trees'; Expected = 36 }
)
foreach ($c in $checks) {
    $folder = Join-Path $source $c.Folder
    $sum = 0
    foreach ($f in (Get-ChildItem -LiteralPath $folder -Filter '*.png' -File)) {
        $rel = $f.FullName.Substring($source.Length + 1)
        if ($rel -like '*UI Banners from the store page*') { continue }
        $s = Read-PngSize $f.FullName
        $g = Get-Geometry $rel $s.Width $s.Height
        $sum += [int]$g.Frames
    }
    $status = if ($sum -eq $c.Expected) { 'OK' } else { 'MISMATCH' }
    Write-Output ("  [{0}] {1}: computed {2}, expected {3}" -f $status, $c.Label, $sum, $c.Expected)
}

# Units: frames per (faction,unit) must equal the blue aseprite header totals.
$unitExpect = @{ 'archer' = 19; 'lancer' = 63; 'monk' = 32; 'pawn' = 131; 'warrior' = 28 }
foreach ($faction in @('black', 'blue', 'purple', 'red', 'yellow')) {
    $unitsDir = Join-Path $source ("Units\" + (Get-Culture).TextInfo.ToTitleCase($faction) + ' Units')
    foreach ($u in $unitExpect.Keys) {
        $uDir = Join-Path $unitsDir (Get-Culture).TextInfo.ToTitleCase($u)
        $sum = 0
        foreach ($f in (Get-ChildItem -LiteralPath $uDir -Filter '*.png' -File)) {
            $rel = $f.FullName.Substring($source.Length + 1)
            $s = Read-PngSize $f.FullName
            $g = Get-Geometry $rel $s.Width $s.Height
            $sum += [int]$g.Frames
        }
        $status = if ($sum -eq $unitExpect[$u]) { 'OK' } else { 'MISMATCH' }
        Write-Output ("  [{0}] units/{1}/{2}: computed {3}, expected {4}" -f $status, $faction, $u, $sum, $unitExpect[$u])
    }
}

# Water Foam tileset strip
$wf = Read-PngSize (Join-Path $source 'Terrain\Tileset\Water Foam.png')
$g = Get-Geometry 'Terrain\Tileset\Water Foam.png' $wf.Width $wf.Height
$status = if ([int]$g.Frames -eq 16) { 'OK' } else { 'MISMATCH' }
Write-Output ("  [{0}] terrain/tileset/water_foam: computed {1}, expected 16" -f $status, $g.Frames)

Write-Output 'Extraction done.'
