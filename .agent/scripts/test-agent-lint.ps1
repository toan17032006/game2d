[CmdletBinding()]
param(
    [string]$ScriptsPath = ''
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

. (Join-Path $PSScriptRoot 'lib\agent-runtime.ps1')

$repoRoot = Get-AgentRepoRoot
$root = if ([string]::IsNullOrWhiteSpace($ScriptsPath)) { Join-Path $repoRoot '.agent\scripts' } else { Resolve-Path -LiteralPath $ScriptsPath }
$files = @(Get-ChildItem -LiteralPath $root -Filter '*.ps1' -File -Recurse | Sort-Object -Property FullName)

$violations = [System.Collections.Generic.List[string]]::new()

function Get-Relative([string]$FullName) {
    return $FullName.Substring($repoRoot.Length).TrimStart('\', '/').Replace('\', '/')
}

# LINT-001: no unsupported product-health claims. Built from fragments so the
# rule's own definition never matches itself.
$forbiddenClaim = 'baseline product ' + 'health'
$forbiddenClaim2 = 'product ' + 'health verified'

foreach ($file in $files) {
    $relative = Get-Relative $file.FullName
    $content = Get-Content -LiteralPath $file.FullName -Raw
    $isLibrary = $relative -like '.agent/scripts/lib/*'
    $isSelf = $file.Name -eq 'test-agent-lint.ps1'

    # LINT-002: every script must parse
    $tokens = $null
    $errors = $null
    $null = [System.Management.Automation.Language.Parser]::ParseFile($file.FullName, [ref]$tokens, [ref]$errors)
    if ($errors -and $errors.Count -gt 0) {
        [void]$violations.Add("${relative}: LINT-002 script does not parse: $($errors[0].Message)")
    }

    # LINT-003: every non-library script must fail fast
    if (-not $isLibrary -and $content -notmatch "\`$ErrorActionPreference\s*=\s*'Stop'") {
        [void]$violations.Add("${relative}: LINT-003 missing `$ErrorActionPreference = 'Stop'")
    }

    if (-not $isSelf) {
        # LINT-001: forbid claims of verified health without a product command
        if ($content -cmatch [regex]::Escape($forbiddenClaim) -or $content -cmatch [regex]::Escape($forbiddenClaim2)) {
            [void]$violations.Add("${relative}: LINT-001 unsupported product-health claim; record a real command result instead")
        }
    }

    # LINT-004: native git must go through Invoke-Git so stderr cannot abort a gate
    if (-not $isLibrary) {
        $lines = $content -split '\r?\n'
        for ($i = 0; $i -lt $lines.Length; $i++) {
            $line = $lines[$i]
            if ($line -match '^\s*#' ) { continue }
            # Only a statement that invokes git directly is a violation; git
            # named inside a message string or passed as -FilePath is fine.
            if ($line -cmatch '^(\s*&\s*git\b|\s*git(\.exe)?\s+\S|\s*\$\w+\s*=\s*&\s*git\b)') {
                [void]$violations.Add("${relative}:$($i + 1): LINT-004 direct git invocation; use Invoke-Git from lib/agent-runtime.ps1")
            }
        }
    }
}

if ($files.Count -eq 0) {
    Write-Output 'FAIL: agent lint found no PowerShell scripts to lint'
    exit 1
}

if ($violations.Count -gt 0) {
    Write-Output 'FAIL: agent lint'
    $violations | ForEach-Object { Write-Output "- $_" }
    exit 1
}

Write-Output "PASS: agent lint ($($files.Count) scripts)"
exit 0
