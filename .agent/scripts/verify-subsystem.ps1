[CmdletBinding()]
param(
    [string]$Subsystem = '',
    [switch]$All,
    [switch]$List,
    [string]$TaskId = '',
    [int]$TimeoutSeconds = 600
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

. (Join-Path $PSScriptRoot 'lib\agent-runtime.ps1')

$repoRoot = Get-AgentRepoRoot
$registryPath = Join-Path $repoRoot '.agent\verification\subsystems.json'

if (-not (Test-Path -LiteralPath $registryPath -PathType Leaf)) {
    Write-Output 'FAIL: subsystem verification registry is missing: /.agent/verification/subsystems.json'
    exit 1
}

try {
    $registry = Get-Content -LiteralPath $registryPath -Raw | ConvertFrom-Json
} catch {
    Write-Output "FAIL: subsystem verification registry is not valid JSON: $($_.Exception.Message)"
    exit 1
}

$subsystems = @($registry.subsystems)
$failures = [System.Collections.Generic.List[string]]::new()
$results = [System.Collections.Generic.List[object]]::new()

function Fail([string]$Message) { [void]$failures.Add($Message) }

if ($List) {
    foreach ($subsystemEntry in $subsystems) {
        Write-Output "$($subsystemEntry.id) [$($subsystemEntry.owner)] $($subsystemEntry.invariants.Count) invariant(s)"
    }
    exit 0
}

$selected = @()
if ($All) {
    $selected = $subsystems
} elseif (-not [string]::IsNullOrWhiteSpace($Subsystem)) {
    $selected = @($subsystems | Where-Object { $_.id -eq $Subsystem })
    if ($selected.Count -eq 0) {
        Write-Output "FAIL: unknown subsystem '$Subsystem'"
        exit 1
    }
} else {
    Write-Output 'FAIL: specify -Subsystem <id> or -All'
    exit 1
}

$reviewBy = ''
$reviewProperty = $registry.PSObject.Properties['reviewBy']
if ($null -ne $reviewProperty) { $reviewBy = [string]$reviewProperty.Value }
if (-not [string]::IsNullOrWhiteSpace($reviewBy)) {
    try {
        if ([datetime]::Parse($reviewBy, [System.Globalization.CultureInfo]::InvariantCulture).Date -lt (Get-Date).Date) {
            Fail "subsystem registry review is overdue (reviewBy $reviewBy)"
        }
    } catch { }
}

foreach ($subsystemEntry in $selected) {
    $invariants = @($subsystemEntry.invariants)
    if ($invariants.Count -eq 0) {
        Fail "subsystem '$($subsystemEntry.id)' declares no invariants; a subsystem skill without a check is documentation, not verification"
        continue
    }

    foreach ($invariant in $invariants) {
        $statement = [string]$invariant.statement
        $check = [string]$invariant.check
        if ([string]::IsNullOrWhiteSpace($statement) -or [string]::IsNullOrWhiteSpace($check)) {
            Fail "subsystem '$($subsystemEntry.id)' has an invariant without a statement or a runnable check ($($invariant.id))"
            continue
        }
        if ($check -match '<[^>]+>' -or $check -match '(?i)\b(tbd|todo|pending)\b') {
            Fail "subsystem '$($subsystemEntry.id)' invariant $($invariant.id) has a placeholder check"
            continue
        }

        $tokens = @(Split-AgentCommandLine -CommandLine $check)
        if ($tokens.Count -eq 0) {
            Fail "subsystem '$($subsystemEntry.id)' invariant $($invariant.id) has an empty check"
            continue
        }

        $leaf = [System.IO.Path]::GetFileNameWithoutExtension($tokens[0]).ToLowerInvariant()
        if ($leaf -eq 'powershell' -or $leaf -eq 'pwsh') {
            $arguments = @()
            if ($tokens.Count -gt 1) { $arguments = $tokens[1..($tokens.Count - 1)] }
            $result = Invoke-ExternalProcess -FilePath (Get-AgentPowerShellPath) -Arguments $arguments -WorkingDirectory $repoRoot -TimeoutSeconds $TimeoutSeconds
        } else {
            $result = Invoke-ExternalProcess -FilePath (Get-AgentPowerShellPath) -Arguments @('-NoProfile', '-ExecutionPolicy', 'Bypass', '-Command', $check) -WorkingDirectory $repoRoot -TimeoutSeconds $TimeoutSeconds
        }

        $passed = ($result.ExitCode -eq 0 -and -not $result.TimedOut)
        if (-not $passed) {
            Fail "subsystem '$($subsystemEntry.id)' invariant $($invariant.id) failed (exit $($result.ExitCode)): $statement"
            @($result.StdOut) + @($result.StdErr) | Select-Object -Last 8 | ForEach-Object { Write-Output "  $_" }
        }

        [void]$results.Add([PSCustomObject]@{
            subsystem   = $subsystemEntry.id
            invariantId = $invariant.id
            statement   = $statement
            check       = $check
            exitCode    = $result.ExitCode
            timedOut    = $result.TimedOut
            result      = if ($passed) { 'PASS' } else { 'FAIL' }
        })
    }
}

if (-not [string]::IsNullOrWhiteSpace($TaskId)) {
    if ($TaskId -notmatch '^TASK-\d{8}-\d{3}$') {
        Write-Output "FAIL: invalid task id '$TaskId'"
        exit 1
    }
    $logPath = Join-Path $repoRoot ".agent\tasks\$TaskId\logs\subsystem-verification.json"
    $summary = [ordered]@{
        taskId      = $TaskId
        generatedAt = (Get-Date).ToString('yyyy-MM-ddTHH:mm:ssK')
        results     = @($results)
        failures    = @($failures)
        status      = if ($failures.Count -eq 0) { 'PASS' } else { 'FAIL' }
    }
    $summary | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath $logPath -Encoding utf8
}

if ($failures.Count -gt 0) {
    Write-Output 'FAIL: subsystem verification'
    $failures | ForEach-Object { Write-Output "- $_" }
    exit 1
}

Write-Output "PASS: subsystem verification ($($results.Count) invariants across $($selected.Count) subsystem(s))"
exit 0
