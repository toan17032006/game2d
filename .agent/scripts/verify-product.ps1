[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [ValidatePattern('^TASK-\d{8}-\d{3}$')]
    [string]$TaskId,

    [int]$CommandTimeoutSeconds = 900
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

. (Join-Path $PSScriptRoot 'lib\agent-runtime.ps1')

$repoRoot = Get-AgentRepoRoot
$taskRoot = Join-Path $repoRoot ".agent\tasks\$TaskId"
$logsDir = Join-Path $taskRoot 'logs'
if (-not (Test-Path -LiteralPath $logsDir -PathType Container)) {
    $null = New-Item -ItemType Directory -Path $logsDir -Force
}

$executionLog = Join-Path $logsDir 'test-execution.log'
$artifactPath = Join-Path $logsDir 'product-verification.json'
$logEntries = [System.Collections.Generic.List[string]]::new()
$failures = [System.Collections.Generic.List[string]]::new()
$status = 'FAILED'
$policyKind = 'missing'
$commandLine = ''
$commandExitCode = $null
$commandDuration = 0

function Fail([string]$Message) {
    [void]$failures.Add($Message)
    [void]$logEntries.Add("[$(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')] FAIL: $Message")
}

function Log([string]$Message) {
    [void]$logEntries.Add("[$(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')] $Message")
}

Log "Starting product verification for $TaskId"

# 1. AST syntax check on every agent script (including the shared runtime library)
$scripts = Get-ChildItem -LiteralPath (Join-Path $repoRoot '.agent\scripts') -Filter '*.ps1' -File -Recurse
foreach ($script in $scripts) {
    $tokens = $null
    $errors = $null
    $null = [System.Management.Automation.Language.Parser]::ParseFile($script.FullName, [ref]$tokens, [ref]$errors)
    if ($errors -and $errors.Count -gt 0) {
        Fail "Script syntax error in $($script.Name): $($errors[0].Message)"
    } else {
        Log "PASS: Syntax OK for $($script.Name)"
    }
}

# 2. Markdown link and anchor validation
$linkChecker = Join-Path $PSScriptRoot 'test-markdown-links.ps1'
if (Test-Path -LiteralPath $linkChecker -PathType Leaf) {
    $linkResult = Invoke-ExternalProcess -FilePath (Get-AgentPowerShellPath) `
        -Arguments @('-NoProfile', '-ExecutionPolicy', 'Bypass', '-File', $linkChecker) `
        -WorkingDirectory $repoRoot -TimeoutSeconds 300
    if ($linkResult.ExitCode -ne 0) {
        Fail "Markdown link verification failed with exit code $($linkResult.ExitCode)"
        @($linkResult.StdOut) + @($linkResult.StdErr) | ForEach-Object { Log "  $_" }
    } else {
        Log 'PASS: Markdown links verified'
        @($linkResult.StdOut) | ForEach-Object { Log "  $_" }
    }
}

# 3. Product verification contract from spec.md
$specPath = Join-Path $taskRoot 'spec.md'
if (-not (Test-Path -LiteralPath $specPath -PathType Leaf)) {
    Fail 'Missing spec.md; product verification contract cannot be resolved'
} else {
    $spec = Get-Content -LiteralPath $specPath -Raw
    $policy = Get-ProductVerificationPolicy -SpecContent $spec -TaskId $TaskId
    $policyKind = $policy.Kind

    switch ($policy.Kind) {
        'command' {
            $commandLine = $policy.Command
            Log "Declared product verification command: $commandLine"

            $tokens = @(Split-AgentCommandLine -CommandLine $commandLine)
            if ($tokens.Count -eq 0) {
                Fail 'Declared product verification command is empty after parsing'
                break
            }

            $leaf = [System.IO.Path]::GetFileNameWithoutExtension($tokens[0]).ToLowerInvariant()
            if ($leaf -eq 'powershell' -or $leaf -eq 'pwsh') {
                $hostPath = Get-AgentPowerShellPath
                $arguments = @()
                if ($tokens.Count -gt 1) { $arguments = $tokens[1..($tokens.Count - 1)] }
                $commandResult = Invoke-ExternalProcess -FilePath $hostPath -Arguments $arguments -WorkingDirectory $repoRoot -TimeoutSeconds $CommandTimeoutSeconds
            } else {
                $commandResult = Invoke-ExternalProcess -FilePath (Get-AgentPowerShellPath) `
                    -Arguments @('-NoProfile', '-ExecutionPolicy', 'Bypass', '-Command', $commandLine) `
                    -WorkingDirectory $repoRoot -TimeoutSeconds $CommandTimeoutSeconds
            }

            $commandDuration = $commandResult.Duration
            if ($commandResult.TimedOut) {
                Fail "Product verification command timed out after $CommandTimeoutSeconds seconds"
                break
            }

            $commandExitCode = $commandResult.ExitCode
            @($commandResult.StdOut) | ForEach-Object { Log "  $_" }
            @($commandResult.StdErr) | ForEach-Object { Log "  stderr: $_" }

            if ($commandExitCode -ne 0) {
                Fail "Product verification command failed with exit code $commandExitCode"
                break
            }

            Log "PASS: Product verification command succeeded (exit code 0, $commandDuration s)"
            $status = 'VERIFIED'
        }
        'not-applicable' {
            $status = 'SKIPPED'
            Log "SKIPPED: task declares product verification as not-applicable: $($policy.Reason)"
        }
        'legacy-exempt' {
            $status = 'LEGACY_EXEMPT'
            Log "SKIPPED: legacy product verification exemption applies ($($policy.Reason)). This is not evidence that the product works."
        }
        'expired' {
            Fail "$($policy.Detail). Remove the stale baseline entry or declare a real product verification command."
        }
        'invalid' {
            Fail $policy.Detail
        }
        default {
            Fail "spec.md does not declare product verification. Add a '## Test Command' section with a runnable product check, or '## Product Verification: not-applicable' plus a substantive reason. No product command means no product evidence."
        }
    }
}

# 4. Evidence artifact (machine readable, reviewable, reproducible)
if ($failures.Count -gt 0) { $status = 'FAILED' }

$relativeLog = $executionLog.Substring($repoRoot.Length).TrimStart('\', '/').Replace('\', '/')
$artifact = [ordered]@{
    taskId          = $TaskId
    generatedAt     = (Get-Date).ToString('yyyy-MM-ddTHH:mm:ssK')
    status          = $status
    policy          = $policyKind
    command         = $commandLine
    exitCode        = $commandExitCode
    durationSeconds = $commandDuration
    failures        = @($failures)
    logPath         = $relativeLog
}

Log "Product verification status: $status"
Write-AgentLog -Path $executionLog -Lines $logEntries.ToArray()
$artifact | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath $artifactPath -Encoding utf8

if ($failures.Count -gt 0) {
    Write-Output "FAIL: product verification harness for $TaskId"
    $failures | ForEach-Object { Write-Output "- $_" }
    exit 1
}

Write-Output "PASS: product verification harness for $TaskId ($status)"
exit 0
