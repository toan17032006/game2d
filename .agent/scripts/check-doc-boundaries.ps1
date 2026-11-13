[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [ValidatePattern('^TASK-\d{8}-\d{3}$')]
    [string]$TaskId,

    [string]$BaseRef = '',

    [ValidateSet('preflight', 'final', 'check')]
    [string]$Mode = 'check'
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

. (Join-Path $PSScriptRoot 'lib\agent-runtime.ps1')

$repoRoot = Get-AgentRepoRoot
$taskRoot = Join-Path $repoRoot ".agent\tasks\$TaskId"
$specPath = Join-Path $taskRoot 'spec.md'

$approvedPaths = [System.Collections.Generic.HashSet[string]]::new([System.StringComparer]::OrdinalIgnoreCase)

if (Test-Path -LiteralPath $specPath -PathType Leaf) {
    $specContent = Get-Content -LiteralPath $specPath -Raw
    $approvedSectionMatch = [regex]::Match($specContent, '(?im)^## Approved Documentation Changes\s*\r?\n([\s\S]*?)(?=\r?\n## |\Z)')
    if ($approvedSectionMatch.Success) {
        $lines = $approvedSectionMatch.Groups[1].Value -split '\r?\n'
        foreach ($line in $lines) {
            # Matches: - `path` (comment) or - /path (comment) or - path
            $m = [regex]::Match($line, '^\s*-\s*(?:`([^`]+)`|"([^"]+)"|''([^'']+)''|(\S+))')
            if ($m.Success) {
                $val = ''
                for ($g = 1; $g -le 4; $g++) {
                    if ($m.Groups[$g].Success -and -not [string]::IsNullOrWhiteSpace($m.Groups[$g].Value)) {
                        $val = $m.Groups[$g].Value
                        break
                    }
                }
                $raw = $val.Trim().TrimStart('/').Replace('\', '/')
                if (-not [string]::IsNullOrWhiteSpace($raw)) {
                    [void]$approvedPaths.Add($raw)
                }
            }
        }
    }
}

# Collect modified / added / untracked files.
# git stderr is captured separately by Invoke-Git and never escalates:
# global ignore / permission warnings are diagnostics, not failures.
$changedFiles = [System.Collections.Generic.HashSet[string]]::new([System.StringComparer]::OrdinalIgnoreCase)

$statusResult = Invoke-Git -RepoRoot $repoRoot -Arguments @('status', '--porcelain')
foreach ($warning in (Write-GitDiagnostics -Result $statusResult -Context 'git status')) { Write-Output $warning }
if ($statusResult.TimedOut) {
    Write-Output "FAIL: doc boundaries check for $TaskId"
    Write-Output '- git status timed out; boundary cannot be evaluated'
    exit 1
}
if ($statusResult.ExitCode -ne 0) {
    Write-Output "FAIL: doc boundaries check for $TaskId"
    Write-Output "- git status failed with exit code $($statusResult.ExitCode); boundary cannot be evaluated"
    $statusResult.StdErr | ForEach-Object { Write-Output "- git: $_" }
    exit 1
}

foreach ($item in @($statusResult.StdOut)) {
    if ($item.Length -gt 3) {
        $rel = $item.Substring(3).Trim().Trim('"').Replace('\', '/')
        if ($rel -match ' -> ') {
            $rel = ($rel -split ' -> ')[1].Trim().Trim('"')
        }
        [void]$changedFiles.Add($rel)
    }
}

if (-not [string]::IsNullOrWhiteSpace($BaseRef)) {
    $diffResult = Invoke-Git -RepoRoot $repoRoot -Arguments @('diff', '--name-only', $BaseRef)
    foreach ($warning in (Write-GitDiagnostics -Result $diffResult -Context 'git diff')) { Write-Output $warning }
    if ($diffResult.ExitCode -ne 0) {
        Write-Output "FAIL: doc boundaries check for $TaskId"
        Write-Output "- git diff against '$BaseRef' failed with exit code $($diffResult.ExitCode)"
        exit 1
    }
    foreach ($f in @($diffResult.StdOut)) {
        $rel = $f.Trim().Replace('\', '/')
        if (-not [string]::IsNullOrWhiteSpace($rel)) {
            [void]$changedFiles.Add($rel)
        }
    }
}

$taskPrefix = ".agent/tasks/$TaskId/"
$violations = [System.Collections.Generic.List[string]]::new()

foreach ($file in $changedFiles) {
    if (-not $file.EndsWith('.md', [System.StringComparison]::OrdinalIgnoreCase)) {
        continue
    }

    # Allowed inside current task workspace or temp test workspaces
    if ($file.StartsWith($taskPrefix, [System.StringComparison]::OrdinalIgnoreCase) -or
        $file.StartsWith('.agent/tasks/_temp_', [System.StringComparison]::OrdinalIgnoreCase)) {
        continue
    }

    # Check if approved
    if (-not $approvedPaths.Contains($file)) {
        [void]$violations.Add("Unauthorized modification to project documentation: /$file. Markdown files outside $taskPrefix are strictly read-only unless explicitly approved in spec.md under '## Approved Documentation Changes'.")
    }
}

if ($violations.Count -gt 0) {
    Write-Output "FAIL: doc boundaries check for $TaskId"
    $violations | ForEach-Object { Write-Output "- $_" }
    exit 1
}

Write-Output "PASS: doc boundaries check for $TaskId"
exit 0
