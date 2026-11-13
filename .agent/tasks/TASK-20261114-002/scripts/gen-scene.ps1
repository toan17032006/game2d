# Generate `game/data/units.csv` and `game/data/scene.csv`.
#
# units.csv  - one row per (faction, unit): idle/run manifest keys, demo
#              walk speed, frame size, animation fps.
# scene.csv  - deterministic demo scene: grass field + water row tiled from
#              the tileset (src coords verified visually from
#              tilemap_color1.png interior grass tiles), props, animated
#              decorations, units and UI. World space: viewport 960x540,
#              center (0,0), +y down, so x in [-480,480], y in [-270,270].

[CmdletBinding()]
param(
    [string]$RepoRoot = 'C:/Users/Admin/Desktop/game2d/game2d'
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$dataDir = Join-Path $RepoRoot 'game\data'
New-Item -ItemType Directory -Path $dataDir -Force | Out-Null

$factions = @('black', 'blue', 'purple', 'red', 'yellow')
$unitDefs = @(
    @{ Unit = 'warrior'; Idle = 'warrior_idle'; Run = 'warrior_run'; Speed = 120; Frame = 192; Fps = 10 },
    @{ Unit = 'archer';  Idle = 'archer_idle';  Run = 'archer_run';  Speed = 110; Frame = 192; Fps = 8 },
    @{ Unit = 'lancer';  Idle = 'lancer_idle';  Run = 'lancer_run';  Speed = 90;  Frame = 320; Fps = 10 },
    @{ Unit = 'monk';    Idle = 'idle';         Run = 'run';         Speed = 100; Frame = 192; Fps = 8 },
    @{ Unit = 'pawn';    Idle = 'pawn_idle';    Run = 'pawn_run';    Speed = 80;  Frame = 192; Fps = 8 }
)

$unitLines = [System.Collections.Generic.List[string]]::new()
$unitLines.Add('faction,unit,idle,run,speed,frame_w,frame_h,fps')
foreach ($f in $factions) {
    foreach ($u in $unitDefs) {
        $unitLines.Add(('{0},{1},units/{2}/{3}/{4},units/{2}/{3}/{5},{6},{7},{7},{8}' -f `
            $f, $u.Unit, $f, $u.Unit, $u.Idle, $u.Run, $u.Speed, $u.Frame, $u.Fps))
    }
}
[IO.File]::WriteAllText((Join-Path $dataDir 'units.csv'), ($unitLines -join "`r`n") + "`r`n", (New-Object System.Text.UTF8Encoding($false)))

# ---- scene ----
$rows = [System.Collections.Generic.List[string]]::new()
$rows.Add('kind,ref,x,y,z,scale_x,scale_y,fps,src_x,src_y,dir')

function Add-Row([string]$Kind, [string]$Ref, [float]$X, [float]$Y, [float]$Z,
                 [float]$Sx, [float]$Sy, [float]$Fps, [int]$SrcX, [int]$SrcY, [int]$Dir) {
    $script:rows.Add(('{0},{1},{2},{3},{4},{5},{6},{7},{8},{9},{10}' -f `
        $Kind, $Ref, $X, $Y, $Z, $Sx, $Sy, $Fps, $SrcX, $SrcY, $Dir))
}

# Ground: 15 x 9 tiles of 64px. Rows 0..7 grass (interior tiles of the
# top-left block in tilemap_color1.png, verified visually), row 8 water.
$grassVariants = @(
    @{ Sx = 0; Sy = 0 }, @{ Sx = 64; Sy = 0 }, @{ Sx = 128; Sy = 0 },
    @{ Sx = 0; Sy = 64 }, @{ Sx = 64; Sy = 64 }, @{ Sx = 128; Sy = 64 }
)
for ($row = 0; $row -lt 9; $row++) {
    for ($col = 0; $col -lt 15; $col++) {
        $cx = -448 + $col * 64
        $cy = -238 + $row * 64
        if ($row -eq 8) {
            Add-Row 'tile' 'terrain/tileset/water_background_color' $cx $cy 0 1 1 0 0 0 0
        } else {
            $v = $grassVariants[(($col + $row) % 6)]
            Add-Row 'tile' 'terrain/tileset/tilemap_color1' $cx $cy 0 1 1 0 $v.Sx $v.Sy 0
        }
    }
}

# Buildings
Add-Row 'prop' 'buildings/blue/house1' -330 60 6 1 1 0 0 0 0
Add-Row 'prop' 'buildings/blue/tower' 395 20 6 1 1 0 0 0 0

# FX on the buildings
Add-Row 'anim' 'fx/fire_01' -330 -55 7 1 1 8 0 0 0
Add-Row 'anim' 'fx/fire_02' 395 -70 7 1 1 8 0 0 0

# Animated decorations and resources
Add-Row 'anim' 'terrain/decorations/bushes/bush1' -80 190 4 1 1 6 0 0 0
Add-Row 'anim' 'terrain/decorations/bushes/bush2' 450 240 4 1 1 6 0 0 0
Add-Row 'anim' 'terrain/resources/wood/trees/tree1' 240 -60 8 1 1 6 0 0 0
Add-Row 'anim' 'terrain/resources/wood/trees/tree2' -440 -80 8 1 1 6 0 0 0
Add-Row 'anim' 'terrain/resources/meat/sheep/sheep_idle' -40 235 5 1 1 6 0 0 0
Add-Row 'anim' 'terrain/decorations/rocks_in_the_water/water_rocks_01' 480 274 9 1 1 6 0 0 0

# Static props
Add-Row 'prop' 'terrain/decorations/rocks/rock1' -440 235 3 1 1 0 0 0 0
Add-Row 'prop' 'terrain/decorations/rocks/rock2' 30 245 3 1 1 0 0 0 0
Add-Row 'prop' 'terrain/resources/gold/gold_stones/gold_stone_1' 140 150 3 1 1 0 0 0 0
Add-Row 'prop' 'terrain/resources/gold/gold_resource/gold_resource' -180 240 3 1 1 0 0 0 0
Add-Row 'prop' 'terrain/resources/meat/meat_resource/meat_resource' 80 190 3 1 1 0 0 0 0
Add-Row 'prop' 'terrain/resources/wood/wood_resource/wood_resource' -240 200 3 1 1 0 0 0 0
Add-Row 'prop' 'terrain/resources/tools/tool_01' -150 130 3 1 1 0 0 0 0

# Player (keyboard-controlled) plus walking and idle NPC units
Add-Row 'player' 'blue:warrior' -200 190 10 1 1 0 0 0 0
Add-Row 'unit' 'red:warrior' 400 190 10 1 1 0 0 0 -1
Add-Row 'unit' 'yellow:lancer' 450 -80 10 1 1 0 0 0 -1
Add-Row 'unit_idle' 'purple:archer' 250 -150 10 1 1 0 0 0 0
Add-Row 'unit_idle' 'black:monk' -250 -180 10 1 1 0 0 0 0

# UI (screen-space feel via high z)
Add-Row 'prop' 'ui/banners/banner' 370 -170 100 0.5 0.5 0 0 0 0
Add-Row 'prop' 'ui/bars/small_bar_base' -310 -240 100 1 1 0 0 0 0
Add-Row 'prop' 'ui/bars/small_bar_fill' -310 -240 101 4 0.5 0 0 0 0
Add-Row 'prop' 'ui/buttons/tiny_round_blue_button' -100 -240 100 1 1 0 0 0 0
Add-Row 'prop' 'ui/icons/icon_01' -40 -240 100 1 1 0 0 0 0
Add-Row 'prop' 'ui/cursors/cursor_01' 20 -240 100 1 1 0 0 0 0

[IO.File]::WriteAllText((Join-Path $dataDir 'scene.csv'), ($rows -join "`r`n") + "`r`n", (New-Object System.Text.UTF8Encoding($false)))

Write-Output ("units.csv rows: " + ($unitLines.Count - 1))
Write-Output ("scene.csv rows: " + ($rows.Count - 1))
