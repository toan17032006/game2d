# Shared deterministic runtime for agent quality gates.
# This file is executable control code, not project documentation.
# Native commands are always executed through Invoke-ExternalProcess so that
# stderr diagnostics can never become a terminating PowerShell error.

Set-StrictMode -Version Latest

$script:AgentRepoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..\..')).Path
$script:AgentBaselinePath = Join-Path $script:AgentRepoRoot '.agent\scripts\product-verification-baseline.json'

function Get-AgentRepoRoot {
    return $script:AgentRepoRoot
}

function Get-AgentPowerShellPath {
    $hostProcess = $null
    try { $hostProcess = Get-Process -Id $PID -ErrorAction Stop } catch { $hostProcess = $null }
    if ($null -ne $hostProcess -and -not [string]::IsNullOrWhiteSpace($hostProcess.Path)) {
        return $hostProcess.Path
    }
    $candidate = Join-Path $PSHOME 'powershell.exe'
    if (Test-Path -LiteralPath $candidate -PathType Leaf) { return $candidate }
    $candidate = Join-Path $PSHOME 'pwsh.exe'
    if (Test-Path -LiteralPath $candidate -PathType Leaf) { return $candidate }
    return 'powershell.exe'
}

function Format-NativeArgument {
    param([string]$Value)

    if ([string]::IsNullOrEmpty($Value)) { return '""' }
    if ($Value -match '^[A-Za-z0-9_\-.:+/\\=@]+$') { return $Value }

    $escaped = $Value
    $escaped = [regex]::Replace($escaped, '(\\*)"', '$1$1\"')
    $escaped = [regex]::Replace($escaped, '(\\+)$', '$1$1')
    return '"' + $escaped + '"'
}

function Split-AgentCommandLine {
    param([Parameter(Mandatory = $true)][string]$CommandLine)

    $tokens = New-Object System.Collections.Generic.List[string]
    $current = New-Object System.Text.StringBuilder
    $inQuotes = $false
    $hasToken = $false

    foreach ($ch in $CommandLine.ToCharArray()) {
        if ($ch -eq '"') {
            $inQuotes = -not $inQuotes
            $hasToken = $true
            continue
        }
        if (-not $inQuotes -and [char]::IsWhiteSpace($ch)) {
            if ($hasToken) {
                [void]$tokens.Add($current.ToString())
                [void]$current.Clear()
                $hasToken = $false
            }
            continue
        }
        [void]$current.Append($ch)
        $hasToken = $true
    }

    if ($hasToken) { [void]$tokens.Add($current.ToString()) }
    return $tokens.ToArray()
}

function Invoke-ExternalProcess {
    param(
        [Parameter(Mandatory = $true)][string]$FilePath,
        [string[]]$Arguments = @(),
        [string]$WorkingDirectory = '',
        [int]$TimeoutSeconds = 300,
        [hashtable]$EnvironmentVariables = @{}
    )

    if ([string]::IsNullOrWhiteSpace($WorkingDirectory)) { $WorkingDirectory = $script:AgentRepoRoot }

    $argumentString = (@($Arguments | ForEach-Object { Format-NativeArgument ([string]$_) }) -join ' ')

    $info = New-Object System.Diagnostics.ProcessStartInfo
    $info.FileName = $FilePath
    $info.Arguments = $argumentString
    $info.WorkingDirectory = $WorkingDirectory
    $info.UseShellExecute = $false
    $info.CreateNoWindow = $true
    $info.RedirectStandardOutput = $true
    $info.RedirectStandardError = $true
    if ($null -ne $info.EnvironmentVariables) {
        foreach ($key in $EnvironmentVariables.Keys) {
            $info.EnvironmentVariables[[string]$key] = [string]$EnvironmentVariables[$key]
        }
    }

    $process = New-Object System.Diagnostics.Process
    $process.StartInfo = $info

    try {
        $started = $process.Start()
    } catch {
        return [PSCustomObject]@{
            FilePath  = $FilePath
            ExitCode  = -1
            StdOut    = @()
            StdErr    = @("Unable to start process '$FilePath': $($_.Exception.Message)")
            TimedOut  = $false
            Started   = $false
            Duration  = 0
        }
    }

    if (-not $started) {
        return [PSCustomObject]@{
            FilePath = $FilePath
            ExitCode = -1
            StdOut   = @()
            StdErr   = @("Unable to start process '$FilePath'")
            TimedOut = $false
            Started  = $false
            Duration = 0
        }
    }

    $stopwatch = [System.Diagnostics.Stopwatch]::StartNew()
    $stdoutTask = $process.StandardOutput.ReadToEndAsync()
    $stderrTask = $process.StandardError.ReadToEndAsync()

    $timeoutMs = -1
    if ($TimeoutSeconds -gt 0) { $timeoutMs = $TimeoutSeconds * 1000 }

    if ($timeoutMs -gt 0) {
        $finished = $process.WaitForExit($timeoutMs)
    } else {
        $process.WaitForExit()
        $finished = $true
    }

    $timedOut = -not $finished
    if ($timedOut) {
        try { $process.Kill() } catch { }
        $null = $process.WaitForExit(5000)
    }

    $stdout = ''
    $stderr = ''
    try { $stdout = $stdoutTask.Result } catch { $stdout = '' }
    try { $stderr = $stderrTask.Result } catch { $stderr = '' }

    $exitCode = -1
    if (-not $timedOut) {
        try { $exitCode = $process.ExitCode } catch { $exitCode = -1 }
    }
    try { $process.Dispose() } catch { }
    $stopwatch.Stop()

    $outLines = @()
    if (-not [string]::IsNullOrEmpty($stdout)) {
        $outLines = @($stdout -split '\r?\n' | Where-Object { $_ -ne '' })
    }
    $errLines = @()
    if (-not [string]::IsNullOrEmpty($stderr)) {
        $errLines = @($stderr -split '\r?\n' | Where-Object { $_ -ne '' })
    }

    return [PSCustomObject]@{
        FilePath = $FilePath
        ExitCode = $exitCode
        StdOut   = $outLines
        StdErr   = $errLines
        TimedOut = $timedOut
        Started  = $true
        Duration = [math]::Round($stopwatch.Elapsed.TotalSeconds, 2)
    }
}

function Invoke-Git {
    param(
        [Parameter(Mandatory = $true)][string[]]$Arguments,
        [string]$RepoRoot = '',
        [int]$TimeoutSeconds = 120
    )

    if ([string]::IsNullOrWhiteSpace($RepoRoot)) { $RepoRoot = $script:AgentRepoRoot }

    $gitArguments = @('-C', $RepoRoot) + $Arguments
    $result = Invoke-ExternalProcess -FilePath 'git' -Arguments $gitArguments -WorkingDirectory $RepoRoot -TimeoutSeconds $TimeoutSeconds

    # A missing ignore file, a locked config, or a home-directory permission
    # warning is diagnostic noise on stderr. It must never be treated as a
    # failure on its own; only a non-zero exit code is fatal.
    if (-not $result.Started) {
        $result.StdErr = @('git executable could not be started')
    }
    return $result
}

function Write-GitDiagnostics {
    param(
        [Parameter(Mandatory = $true)]$Result,
        [string]$Context = 'git'
    )

    $lines = @()
    foreach ($line in @($Result.StdErr)) {
        $trimmed = ([string]$line).Trim()
        if ([string]::IsNullOrWhiteSpace($trimmed)) { continue }
        $lines += "WARN: $Context diagnostics: $trimmed"
    }
    return $lines
}

function Get-MarkdownSection {
    param(
        [Parameter(Mandatory = $true)][string]$Content,
        [Parameter(Mandatory = $true)][string[]]$HeadingAliases
    )

    foreach ($alias in $HeadingAliases) {
        $pattern = '(?im)^##\s*' + [regex]::Escape($alias) + '\s*\r?\n([\s\S]*?)(?=\r?\n## |\Z)'
        $match = [regex]::Match($Content, $pattern)
        if ($match.Success) { return $match.Groups[1].Value.Trim() }
    }
    return ''
}

function Get-DeclaredCommand {
    param([Parameter(Mandatory = $true)][string]$Content)

    $section = Get-MarkdownSection -Content $Content -HeadingAliases @('Test Command', 'Verification Command', 'Product Verification Command')
    if ([string]::IsNullOrWhiteSpace($section)) { return '' }

    foreach ($line in ($section -split '\r?\n')) {
        $candidate = $line.Trim()
        if ($candidate -eq '') { continue }
        if ($candidate -match '^```') { continue }
        if ($candidate -match '^[-*]\s*$') { continue }
        $candidate = $candidate.TrimStart('-', '*', ' ').Trim()
        $candidate = $candidate.Trim('`').Trim()
        if ($candidate -eq '') { continue }
        if ($candidate -match '^<.+>$') { return '' }
        if ($candidate -match '^(?i)(tbd|todo|pending|n/?a)$') { return '' }
        return $candidate
    }
    return ''
}

function Get-ProductVerificationBaseline {
    param([string]$Path = '')

    if ([string]::IsNullOrWhiteSpace($Path)) { $Path = $script:AgentBaselinePath }
    if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) { return @() }

    try {
        $data = Get-Content -LiteralPath $Path -Raw | ConvertFrom-Json
    } catch {
        return @()
    }

    if ($null -eq $data) { return @() }
    $property = $data.PSObject.Properties['exemptions']
    if ($null -eq $property) { return @() }
    return @($property.Value)
}

function Get-ProductVerificationPolicy {
    param(
        [Parameter(Mandatory = $true)][string]$SpecContent,
        [Parameter(Mandatory = $true)][string]$TaskId
    )

    $policy = [PSCustomObject]@{
        Kind    = 'missing'
        Command = ''
        Reason  = ''
        Detail  = ''
    }

    $command = Get-DeclaredCommand -Content $SpecContent
    if (-not [string]::IsNullOrWhiteSpace($command)) {
        $policy.Kind = 'command'
        $policy.Command = $command
        return $policy
    }

    $naMatch = [regex]::Match($SpecContent, '(?im)^##\s*Product Verification\s*[:\-\u2014]\s*not-applicable\b\s*[:\-\u2014]?\s*(.*)$')
    $naReason = ''
    if ($naMatch.Success) {
        $naReason = $naMatch.Groups[1].Value.Trim()
    } else {
        $section = Get-MarkdownSection -Content $SpecContent -HeadingAliases @('Product Verification')
        if (-not [string]::IsNullOrWhiteSpace($section)) {
            $lineMatch = [regex]::Match($section, '(?im)^\s*[-*]?\s*not-applicable\b\s*[:\-\u2014]?\s*(.*)$')
            if ($lineMatch.Success) { $naReason = $lineMatch.Groups[1].Value.Trim() }
        }
    }

    if ($naMatch.Success -or -not [string]::IsNullOrWhiteSpace($naReason)) {
        $normalized = $naReason.Trim().Trim('`').Trim('"').Trim("'").Trim()
        if ($normalized.Length -lt 10 -or $normalized -match '(?i)^(n/?a|tbd|todo|pending|none)$') {
            $policy.Kind = 'invalid'
            $policy.Reason = $normalized
            $policy.Detail = 'not-applicable declaration needs a substantive reason of at least 10 characters'
            return $policy
        }
        $policy.Kind = 'not-applicable'
        $policy.Reason = $normalized
        return $policy
    }

    foreach ($entry in (Get-ProductVerificationBaseline)) {
        if ($null -eq $entry) { continue }
        $taskProperty = $entry.PSObject.Properties['taskId']
        if ($null -eq $taskProperty) { continue }
        if ([string]$taskProperty.Value -ne $TaskId) { continue }

        $reason = ''
        $reasonProperty = $entry.PSObject.Properties['reason']
        if ($null -ne $reasonProperty) { $reason = [string]$reasonProperty.Value }

        $reviewBy = ''
        $reviewProperty = $entry.PSObject.Properties['reviewBy']
        if ($null -ne $reviewProperty) { $reviewBy = [string]$reviewProperty.Value }

        if (-not [string]::IsNullOrWhiteSpace($reviewBy)) {
            $expired = $false
            try {
                $reviewDate = [datetime]::Parse($reviewBy, [System.Globalization.CultureInfo]::InvariantCulture)
                $expired = ($reviewDate.Date -lt (Get-Date).Date)
            } catch {
                $expired = $false
            }
            if ($expired) {
                $policy.Kind = 'expired'
                $policy.Reason = $reason
                $policy.Detail = "legacy product verification exemption expired on $reviewBy"
                return $policy
            }
        }

        $policy.Kind = 'legacy-exempt'
        $policy.Reason = $reason
        return $policy
    }

    return $policy
}

function Write-AgentLog {
    param(
        [Parameter(Mandatory = $true)][string]$Path,
        [Parameter(Mandatory = $true)][string[]]$Lines
    )

    $directory = Split-Path -Parent $Path
    if (-not (Test-Path -LiteralPath $directory -PathType Container)) {
        $null = New-Item -ItemType Directory -Path $directory -Force
    }

    $clean = @($Lines | ForEach-Object { ([string]$_).TrimEnd() })
    Set-Content -LiteralPath $Path -Value $clean -Encoding utf8
}
