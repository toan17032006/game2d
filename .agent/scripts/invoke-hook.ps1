[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [ValidatePattern('^TASK-\d{8}-\d{3}$')]
    [string]$TaskId,

    [Parameter(Mandatory = $true)]
    [ValidateSet('preflight', 'plan-gate', 'output-gate', 'handoff-gate')]
    [string]$Hook
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$taskRoot = Join-Path $repoRoot ".agent\tasks\$TaskId"
$failures = [System.Collections.Generic.List[string]]::new()

function Fail([string]$Message) { [void]$failures.Add($Message) }

if (-not (Test-Path -LiteralPath $taskRoot -PathType Container)) {
    Fail "Missing task workspace: /.agent/tasks/$TaskId/"
}

switch ($Hook) {
    'preflight' {
        $promptPath = Join-Path $taskRoot 'prompt.md'
        $handoffPath = Join-Path $taskRoot 'handoff.md'

        if (-not (Test-Path -LiteralPath $promptPath -PathType Leaf)) {
            Fail 'preflight: Missing prompt.md'
        } else {
            $prompt = Get-Content -LiteralPath $promptPath -Raw
            if ($prompt -notmatch '(?im)\b(DIRECT|CLARIFYING|AWAITING_APPROVAL)\b') {
                Fail 'preflight: prompt.md missing input classification (DIRECT, CLARIFYING, or AWAITING_APPROVAL)'
            }
            # Verify referenced source paths starting with /
            $matches = [regex]::Matches($prompt, '(?m)^\s*-\s*[`"\'']?(/[^`"''\r\n\s]+)[`"\'']?')
            foreach ($m in $matches) {
                $ref = $m.Groups[1].Value.Trim()
                # Skip template placeholders
                if ($ref -match '<[^>]+>' -or $ref -match 'TASK-YYYYMMDD') { continue }
                $localPath = Join-Path $repoRoot ($ref.TrimStart('/').Replace('/', '\'))
                if (-not (Test-Path -LiteralPath $localPath)) {
                    Fail "preflight: Referenced source path does not exist on disk: $ref"
                }
            }
        }

        if (-not (Test-Path -LiteralPath $handoffPath -PathType Leaf)) {
            Fail 'preflight: Missing handoff.md'
        } else {
            $handoff = Get-Content -LiteralPath $handoffPath -Raw
            if ($handoff -notmatch '(?im)^## Next action\s*\r?\n\s*\S+') {
                Fail 'preflight: handoff.md missing non-empty ## Next action'
            }
        }
    }

    'plan-gate' {
        $planPath = Join-Path $taskRoot 'plan.md'
        if (-not (Test-Path -LiteralPath $planPath -PathType Leaf)) {
            Fail 'plan-gate: Missing plan.md'
        } else {
            $plan = Get-Content -LiteralPath $planPath -Raw
            if ($plan -notmatch '(?im)^## Phases\s*$') {
                Fail 'plan-gate: plan.md missing ## Phases'
            }
            $phaseRows = [regex]::Matches($plan, '(?im)^\|\s*(P\d+)\s*\|')
            if ($phaseRows.Count -eq 0) {
                Fail 'plan-gate: plan.md has no defined phase rows (P1, P2, ...)'
            }
            if ($plan -notmatch '(?im)^## Rollback\s*$') {
                Fail 'plan-gate: plan.md missing ## Rollback'
            }
            if ($plan -notmatch '(?im)^## Checkpoints\s*$') {
                Fail 'plan-gate: plan.md missing ## Checkpoints'
            }
        }
    }

    'output-gate' {
        $taskPath = Join-Path $taskRoot 'task.md'
        $verificationPath = Join-Path $taskRoot 'verification.md'

        if (-not (Test-Path -LiteralPath $taskPath -PathType Leaf)) {
            Fail 'output-gate: Missing task.md'
        }
        if (-not (Test-Path -LiteralPath $verificationPath -PathType Leaf)) {
            Fail 'output-gate: Missing verification.md'
        }

        if ($failures.Count -eq 0) {
            $task = Get-Content -LiteralPath $taskPath -Raw
            $verification = Get-Content -LiteralPath $verificationPath -Raw

            $acceptanceIds = @([regex]::Matches($task, '(?im)^\|\s*(AC-\d{3})\s*\|') | ForEach-Object { $_.Groups[1].Value } | Sort-Object -Unique)
            if ($acceptanceIds.Count -eq 0) {
                Fail 'output-gate: task.md contains no AC-NNN acceptance criteria'
            }

            foreach ($id in $acceptanceIds) {
                $pattern = '(?im)^\|\s*' + [regex]::Escape($id) + '\s*\|\s*PASS\s*\|\s*([^|\r\n]+)\|'
                $match = [regex]::Match($verification, $pattern)
                if (-not $match.Success) {
                    Fail "output-gate: Missing or non-PASS result for $id in verification.md"
                } else {
                    $evidence = $match.Groups[1].Value.Trim()
                    if ([string]::IsNullOrWhiteSpace($evidence) -or $evidence -match '^(Pending|TBD|N/A|TODO)$') {
                        Fail "output-gate: Insufficient or placeholder evidence for $($id): '$evidence'"
                    }
                }
            }

            # Check for unresolved TODO or Pending markers in task files
            $taskFiles = Get-ChildItem -LiteralPath $taskRoot -Filter '*.md' -File
            foreach ($file in $taskFiles) {
                $content = Get-Content -LiteralPath $file.FullName -Raw
                if ($content -match '(?im)^\s*TODO\b|\bPending\b') {
                    Fail "output-gate: Unresolved marker in $($file.Name)"
                }
            }
        }
    }

    'handoff-gate' {
        $handoffPath = Join-Path $taskRoot 'handoff.md'
        if (-not (Test-Path -LiteralPath $handoffPath -PathType Leaf)) {
            Fail 'handoff-gate: Missing handoff.md'
        } else {
            $handoff = Get-Content -LiteralPath $handoffPath -Raw
            $requiredSections = @('Current phase', 'Last completed', 'Next action', 'Changed files', 'Validation', 'Blockers')
            foreach ($sec in $requiredSections) {
                if ($handoff -notmatch "(?im)^##\s*$([regex]::Escape($sec))\s*$") {
                    Fail "handoff-gate: handoff.md missing section ## $sec"
                }
            }
        }
    }
}

if ($failures.Count -gt 0) {
    Write-Output "FAIL: hook $Hook for $TaskId"
    $failures | ForEach-Object { Write-Output "- $_" }
    exit 1
}

Write-Output "PASS: hook $Hook for $TaskId"
exit 0
