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
$taskRoot = Join-Path $repoRoot ".agent\tasks\$TaskId"
$required = @('prompt.md', 'task.md', 'spec.md', 'plan.md', 'handoff.md', 'verification.md', 'evaluation.md')
$requiredControls = @(
    '.agent\hooks\preflight.md',
    '.agent\hooks\plan-gate.md',
    '.agent\hooks\execution-gate.md',
    '.agent\hooks\output-gate.md',
    '.agent\hooks\handoff-gate.md',
    '.agent\evaluations\input-evaluation.md',
    '.agent\evaluations\plan-evaluation.md',
    '.agent\evaluations\output-evaluation.md',
    '.agent\evaluations\adversarial-evaluation.md',
    '.agent\evaluations\handoff-evaluation.md',
    '.agent\scripts\run-quality-gates.ps1',
    '.agent\scripts\validate-task.ps1',
    '.agent\scripts\verify-task.ps1',
    '.agent\scripts\invoke-hook.ps1',
    '.agent\scripts\check-doc-boundaries.ps1',
    '.agent\scripts\verify-product.ps1',
    '.agent\scripts\test-markdown-links.ps1',
    '.agent\scripts\run-regression-tests.ps1',
    '.agent\scripts\lib\agent-runtime.ps1',
    '.agent\scripts\product-verification-baseline.json',
    '.agent\scripts\test-agent-lint.ps1',
    '.agent\scripts\model-check-task-lifecycle.ps1',
    '.agent\scripts\verify-subsystem.ps1',
    '.agent\verification\subsystems.json'
)
$failures = [System.Collections.Generic.List[string]]::new()

function Fail([string]$Message) { [void]$failures.Add($Message) }

if (-not (Test-Path -LiteralPath $taskRoot -PathType Container)) {
    Fail "Missing task workspace: /.agent/tasks/$TaskId/"
}

foreach ($name in $required) {
    $path = Join-Path $taskRoot $name
    if (-not (Test-Path -LiteralPath $path -PathType Leaf)) {
        Fail "Missing required file: $name"
        continue
    }

    $content = Get-Content -LiteralPath $path -Raw
    if ($content -notmatch '(?im)^## References\s*$') {
        Fail "Missing References section: $name"
    }
}

foreach ($relativePath in $requiredControls) {
    if (-not (Test-Path -LiteralPath (Join-Path $repoRoot $relativePath) -PathType Leaf)) {
        Fail "Missing control: /$($relativePath.Replace('\', '/'))"
    }
}

if (Test-Path -LiteralPath $taskRoot -PathType Container) {
    $taskFiles = Get-ChildItem -LiteralPath $taskRoot -Filter '*.md' -File
    foreach ($file in $taskFiles) {
        $content = Get-Content -LiteralPath $file.FullName -Raw
        if ($content -match '(?im)^\s*TODO\b|\bPending\b') {
            if ($Mode -eq 'final') { Fail "Unresolved TODO/Pending marker: $($file.Name)" }
        }
    }
}

$planPath = Join-Path $taskRoot 'plan.md'
$evaluationPath = Join-Path $taskRoot 'evaluation.md'
$verificationPath = Join-Path $taskRoot 'verification.md'

if (Test-Path -LiteralPath $planPath -PathType Leaf) {
    $plan = Get-Content -LiteralPath $planPath -Raw
    if ($plan -notmatch '(?im)^## Phases\s*$') { Fail 'plan.md missing ## Phases' }
}

$specPath = Join-Path $taskRoot 'spec.md'
if (Test-Path -LiteralPath $specPath -PathType Leaf) {
    $specContent = Get-Content -LiteralPath $specPath -Raw
    $policy = Get-ProductVerificationPolicy -SpecContent $specContent -TaskId $TaskId
    switch ($policy.Kind) {
        'missing' {
            Fail "spec.md must declare product verification: add '## Test Command' with a runnable command, or '## Product Verification: not-applicable' with a substantive reason"
        }
        'invalid' {
            Fail "spec.md product verification declaration is invalid: $($policy.Detail)"
        }
        'expired' {
            Fail "spec.md product verification exemption has expired: $($policy.Detail)"
        }
    }
}

if ($Mode -eq 'final') {
    if (Test-Path -LiteralPath $verificationPath -PathType Leaf) {
        $verification = Get-Content -LiteralPath $verificationPath -Raw
        if ($verification -notmatch '(?im)`COMPLETED`') { Fail 'verification.md is not COMPLETED' }
    }
    if (Test-Path -LiteralPath $evaluationPath -PathType Leaf) {
        $evaluation = Get-Content -LiteralPath $evaluationPath -Raw
        if ($evaluation -notmatch '(?im)^`PASS`\s*$') { Fail 'evaluation.md final result is not PASS' }
        if ($evaluation -notmatch '(?im)\|[^|]+\|\s*[1-5]/5\s*\|\s*PASS\s*\|') { Fail 'evaluation.md has no passing scored evidence' }
    }
}

if ($failures.Count -gt 0) {
    Write-Output 'FAIL'
    $failures | ForEach-Object { Write-Output "- $_" }
    exit 1
}

Write-Output "PASS: $Mode validation for $TaskId"
exit 0
