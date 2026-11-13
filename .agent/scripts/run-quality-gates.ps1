[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [ValidatePattern('^TASK-\d{8}-\d{3}$')]
    [string]$TaskId,

    [ValidateSet('preflight', 'final')]
    [string]$Mode = 'preflight'
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

. (Join-Path $PSScriptRoot 'lib\agent-runtime.ps1')

$repoRoot = Get-AgentRepoRoot
$boundaryChecker = Join-Path $PSScriptRoot 'check-doc-boundaries.ps1'
$hookRunner = Join-Path $PSScriptRoot 'invoke-hook.ps1'
$validator = Join-Path $PSScriptRoot 'validate-task.ps1'
$productVerifier = Join-Path $PSScriptRoot 'verify-product.ps1'
$verifier = Join-Path $PSScriptRoot 'verify-task.ps1'
$lintChecker = Join-Path $PSScriptRoot 'test-agent-lint.ps1'
$lifecycleModel = Join-Path $PSScriptRoot 'model-check-task-lifecycle.ps1'

# 1. Boundary check: Enforce read-only documentation outside task workspace
if (Test-Path -LiteralPath $boundaryChecker -PathType Leaf) {
    & $boundaryChecker -TaskId $TaskId -Mode $Mode
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
}

# 2. Preflight Mode
if ($Mode -eq 'preflight') {
    if (Test-Path -LiteralPath $hookRunner -PathType Leaf) {
        & $hookRunner -TaskId $TaskId -Hook 'preflight'
        if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
    }

    & $validator -TaskId $TaskId -Mode 'preflight'
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

    if (Test-Path -LiteralPath $hookRunner -PathType Leaf) {
        & $hookRunner -TaskId $TaskId -Hook 'plan-gate'
        if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
    }
}

# 3. Final Mode
if ($Mode -eq 'final') {
    & $validator -TaskId $TaskId -Mode 'final'
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

    if (Test-Path -LiteralPath $hookRunner -PathType Leaf) {
        & $hookRunner -TaskId $TaskId -Hook 'output-gate'
        if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
    }

    if (Test-Path -LiteralPath $productVerifier -PathType Leaf) {
        & $productVerifier -TaskId $TaskId
        if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
    }

    & $verifier -TaskId $TaskId
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

    # Repository lint rules codify recurring failure modes (no unsupported
    # product-health claims, no unguarded native git calls, parseable scripts).
    if (Test-Path -LiteralPath $lintChecker -PathType Leaf) {
        & $lintChecker
        if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
    }

    # Executable lifecycle model: COMPLETED is unreachable without every gate.
    if (Test-Path -LiteralPath $lifecycleModel -PathType Leaf) {
        & $lifecycleModel
        if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
    }

    if (Test-Path -LiteralPath $hookRunner -PathType Leaf) {
        & $hookRunner -TaskId $TaskId -Hook 'handoff-gate'
        if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
    }

    # Generated evidence logs are excluded: their tabular output contains
    # padding that is not a source whitespace defect.
    $diffResult = Invoke-Git -RepoRoot $repoRoot -Arguments @('diff', '--check', '--', '.', ':(exclude,glob).agent/tasks/*/logs/*')
    foreach ($warning in (Write-GitDiagnostics -Result $diffResult -Context 'git diff --check')) { Write-Output $warning }

    if ($diffResult.TimedOut) {
        Write-Output 'FAIL: git diff --check timed out'
        exit 1
    }
    $whitespaceIssues = @($diffResult.StdOut | Where-Object { $_.Trim() -ne '' })
    if ($diffResult.ExitCode -ne 0 -or $whitespaceIssues.Count -gt 0) {
        Write-Output 'FAIL: git diff --check found whitespace or conflict issues'
        $whitespaceIssues | ForEach-Object { Write-Output "- $_" }
        exit 1
    }
}

Write-Output "PASS: $Mode quality gates for $TaskId"
exit 0
