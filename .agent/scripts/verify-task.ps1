[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [ValidatePattern('^TASK-\d{8}-\d{3}$')]
    [string]$TaskId
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

. (Join-Path $PSScriptRoot 'lib\agent-runtime.ps1')

$repoRoot = Get-AgentRepoRoot
$taskRoot = Join-Path $repoRoot ".agent\tasks\$TaskId"
$failures = [System.Collections.Generic.List[string]]::new()
function Fail([string]$Message) { [void]$failures.Add($Message) }

$taskPath = Join-Path $taskRoot 'task.md'
$verificationPath = Join-Path $taskRoot 'verification.md'
$evaluationPath = Join-Path $taskRoot 'evaluation.md'
$handoffPath = Join-Path $taskRoot 'handoff.md'

if (-not (Test-Path -LiteralPath $taskPath -PathType Leaf)) { Fail 'Missing task.md' }
if (-not (Test-Path -LiteralPath $verificationPath -PathType Leaf)) { Fail 'Missing verification.md' }
if (-not (Test-Path -LiteralPath $evaluationPath -PathType Leaf)) { Fail 'Missing evaluation.md' }

if ($failures.Count -eq 0) {
    $task = Get-Content -LiteralPath $taskPath -Raw
    $verification = Get-Content -LiteralPath $verificationPath -Raw
    $evaluation = Get-Content -LiteralPath $evaluationPath -Raw

    # 1. Verify AC-NNN Acceptance Criteria
    $acceptanceIds = @([regex]::Matches($task, '(?im)^\|\s*(AC-\d{3})\s*\|') | ForEach-Object { $_.Groups[1].Value } | Sort-Object -Unique)
    if ($acceptanceIds.Count -eq 0) { Fail 'No AC-NNN acceptance IDs in task.md' }

    foreach ($id in $acceptanceIds) {
        $pattern = '(?im)^\|\s*' + [regex]::Escape($id) + '\s*\|\s*PASS\s*\|\s*([^|\r\n]+)\|'
        $match = [regex]::Match($verification, $pattern)
        if (-not $match.Success -or [string]::IsNullOrWhiteSpace($match.Groups[1].Value)) {
            Fail "Acceptance evidence missing or not PASS: $id"
        } else {
            $evidence = $match.Groups[1].Value.Trim()
            if ($evidence -match '^(Pending|TBD|N/A|TODO)$') {
                Fail "Acceptance evidence for $id cannot be placeholder: '$evidence'"
            }
        }
    }

    # 2. Dynamic Rubric Evaluation
    # Core baseline dimensions required for every task
    $coreDimensions = @('Input fidelity', 'Output correctness', 'Verification quality')
    foreach ($core in $coreDimensions) {
        $pattern = '(?im)^\|\s*' + [regex]::Escape($core) + '\s*\|\s*([1-5]/5)\s*\|\s*PASS\s*\|\s*([^|\r\n]+)\|'
        $match = [regex]::Match($evaluation, $pattern)
        if (-not $match.Success) {
            Fail "Required core evaluation dimension missing or not PASS: $core"
        }
    }

    # Extract ALL rows from evaluation table and enforce rubric rules on every declared dimension
    $evalMatches = [regex]::Matches($evaluation, '(?im)^\|\s*([^|#\-\s][^|]*?)\s*\|\s*([^|]+?)\s*\|\s*([^|]+?)\s*\|\s*([^|\r\n]+)\|')
    $dimensionCount = 0
    foreach ($m in $evalMatches) {
        $dimName = $m.Groups[1].Value.Trim()
        $score = $m.Groups[2].Value.Trim()
        $result = $m.Groups[3].Value.Trim()
        $evidence = $m.Groups[4].Value.Trim()

        # Skip header row
        if ($dimName -eq 'Dimension' -or $dimName -match '^-+$') { continue }
        $dimensionCount++

        if ($score -notmatch '^[1-5]/5$') {
            Fail "Dimension '$dimName' has invalid score format: '$score' (must be [1-5]/5)"
        }
        if ($result -ne 'PASS') {
            Fail "Dimension '$dimName' result is not PASS: '$result'"
        }
        if ([string]::IsNullOrWhiteSpace($evidence) -or $evidence.Length -lt 5 -or $evidence -match '^(Pending|TBD|N/A|TODO)$') {
            Fail "Dimension '$dimName' has insufficient or placeholder evidence: '$evidence'"
        }
    }

    if ($dimensionCount -lt 3) {
        Fail "Evaluation table has fewer than 3 scored dimensions"
    }

    if ($evaluation -notmatch '(?im)^`PASS`\s*$') { Fail 'Evaluation final result is not PASS' }

    # 3. Verification Command Evidence (exclude AC-NNN acceptance rows and table headers)
    $candidateRows = [regex]::Matches($verification, '(?im)^\|\s*(?!ID\b|Dimension\b|Command\b|#|-)[^|\r\n]+\|\s*PASS\s*\|\s*([^|\r\n]+)\|')
    $validCommandRows = [System.Collections.Generic.List[string]]::new()
    foreach ($row in $candidateRows) {
        $cols = $row.Value -split '\|'
        if ($cols.Length -gt 1) {
            $firstCol = $cols[1].Trim()
            if ($firstCol -match '^AC-\d{3}$') { continue }
            [void]$validCommandRows.Add($row.Value)
        }
    }

    if ($validCommandRows.Count -eq 0) {
        Fail 'No PASS command evidence in verification.md'
    }

    # If task has logs/ directory, verify logs exist and show successful execution
    $logsDir = Join-Path $taskRoot 'logs'
    if (Test-Path -LiteralPath $logsDir -PathType Container) {
        $execLog = Join-Path $logsDir 'test-execution.log'
        if (Test-Path -LiteralPath $execLog -PathType Leaf) {
            $logContent = Get-Content -LiteralPath $execLog -Raw
            if ($logContent -match 'FAIL:') {
                Fail "Task execution log contains FAIL entries: logs/test-execution.log"
            }
        }
    }

    # 3b. Product verification artifact must support, not contradict, the claims
    $productArtifact = Join-Path $taskRoot 'logs\product-verification.json'
    $productClaimPattern = '(?i)product\s+(verification|health)\s+(verified|pass(ed)?)'
    if (Test-Path -LiteralPath $productArtifact -PathType Leaf) {
        $artifact = $null
        try {
            $artifact = Get-Content -LiteralPath $productArtifact -Raw | ConvertFrom-Json
        } catch {
            Fail 'logs/product-verification.json is not valid JSON'
        }

        if ($null -ne $artifact) {
            $artifactStatus = [string]$artifact.status
            if ($artifactStatus -eq 'FAILED') {
                Fail 'Product verification artifact records FAILED'
            }
            if ($artifactStatus -eq 'VERIFIED') {
                $exitProperty = $artifact.PSObject.Properties['exitCode']
                $artifactExit = $null
                if ($null -ne $exitProperty) { $artifactExit = $exitProperty.Value }
                if ($null -eq $artifactExit -or [int]$artifactExit -ne 0) {
                    Fail 'Product verification artifact claims VERIFIED without a zero exit code'
                }
            }
            if (($artifactStatus -eq 'SKIPPED' -or $artifactStatus -eq 'LEGACY_EXEMPT') -and $verification -match $productClaimPattern) {
                Fail "verification.md claims product verification passed while the artifact records '$artifactStatus'"
            }
        }
    } elseif ($verification -match $productClaimPattern) {
        Fail 'verification.md claims product verification passed but logs/product-verification.json is missing'
    }

    # 4. Ground-truth Changed Files validation if handoff.md exists
    if (Test-Path -LiteralPath $handoffPath -PathType Leaf) {
        $handoff = Get-Content -LiteralPath $handoffPath -Raw
        $changedSectionMatch = [regex]::Match($handoff, '(?im)^## Changed files\s*\r?\n([\s\S]*?)(?=\r?\n## |\Z)')
        if ($changedSectionMatch.Success) {
            $declaredLines = @($changedSectionMatch.Groups[1].Value -split '\r?\n' |
                ForEach-Object { $_.Trim() } |
                Where-Object { $_ -match '^\s*-\s*\S+' } |
                ForEach-Object { [regex]::Replace($_, '^\s*-\s*[`"\'']?([^`"''\r\n\s]+)[`"\'']?.*$', '$1').TrimStart('/').Replace('\', '/') })

            # Check for non-existent explicit file paths declared in handoff
            foreach ($dec in $declaredLines) {
                # Skip descriptions like "Task workspace này" or generic text
                if ($dec -match '\.(md|ps1|yml|yaml|json|ts|js|txt)$') {
                    $localF = Join-Path $repoRoot ($dec.Replace('/', '\'))
                    if (-not (Test-Path -LiteralPath $localF)) {
                        Fail "handoff.md claims changed file that does not exist on disk: $dec"
                    }
                }
            }
        }
    }
}

if ($failures.Count -gt 0) {
    Write-Output 'FAIL'
    $failures | ForEach-Object { Write-Output "- $_" }
    exit 1
}

Write-Output "PASS: independent verification for $TaskId"
exit 0
