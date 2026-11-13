# Product verification entry point: build + run the game headless for 120
# frames inside the VS x64 toolchain environment (so MSVC link.exe and the
# Windows SDK libs resolve regardless of the caller's PATH).
#
# Exit code: 0 = game validated and loaded the whole asset set and ran the
# loop; non-zero = any failure.

[CmdletBinding()]
param(
    [int]$Frames = 120
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..\..\..')).Path
$vsEnv = Join-Path $PSScriptRoot 'vs-env.cmd'
$manifest = Join-Path $repoRoot 'game\Cargo.toml'

if (-not (Test-Path -LiteralPath $vsEnv -PathType Leaf)) {
    Write-Error "Missing toolchain wrapper: $vsEnv"
    exit 2
}

Write-Output "run-headless: repo=$repoRoot frames=$Frames"
$cmd = "`"$vsEnv`" cargo run --manifest-path `"$manifest`" -- --headless $Frames"
& cmd /c $cmd
$code = $LASTEXITCODE
Write-Output "run-headless: exit code $code"
exit $code
