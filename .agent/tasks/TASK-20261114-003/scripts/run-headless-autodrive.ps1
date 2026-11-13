# Product verification entry point (TASK-20261114-003): build + run the game
# headless with `--autodrive` inside the VS x64 toolchain environment.
#
# The scripted input sequence proves the control pipeline end-to-end:
# keyboard input -> Character body velocity -> physics step -> bridge sync
# -> scene Transform. Exit 0 requires the player to actually reach both
# east and west walls and end displaced vertically.

[CmdletBinding()]
param(
    [int]$Frames = 120
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..\..\..')).Path
$vsEnv = Join-Path $PSScriptRoot 'vs-env.cmd'
$manifest = Join-Path $repoRoot 'game\Cargo.toml'

Write-Output "run-headless-autodrive: repo=$repoRoot frames=$Frames"
$cmd = "`"$vsEnv`" cargo run --release --manifest-path `"$manifest`" -- --headless $Frames --autodrive"
& cmd /c $cmd
$code = $LASTEXITCODE
Write-Output "run-headless-autodrive: exit code $code"
exit $code
