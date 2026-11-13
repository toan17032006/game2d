[CmdletBinding()]
param(
    [string]$Path = ''
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$searchDir = if (-not [string]::IsNullOrWhiteSpace($Path)) {
    if ([System.IO.Path]::IsPathRooted($Path)) { $Path } else { Join-Path $repoRoot $Path }
} else {
    $repoRoot
}

$mdFiles = Get-ChildItem -LiteralPath $searchDir -Filter '*.md' -Recurse -File |
    Where-Object { $_.FullName -notmatch '[\\/]\.git[\\/]' }

$failures = [System.Collections.Generic.List[string]]::new()

function Slugify([string]$header) {
    $clean = $header.Trim().ToLowerInvariant()
    $clean = [regex]::Replace($clean, '[^\w\s-]', '')
    $clean = [regex]::Replace($clean, '\s+', '-')
    return $clean
}

foreach ($file in $mdFiles) {
    $content = Get-Content -LiteralPath $file.FullName -Raw
    $lines = $content -split '\r?\n'

    for ($i = 0; $i -lt $lines.Length; $i++) {
        $line = $lines[$i]
        $lineNum = $i + 1

        # Match markdown links [text](target)
        $matches = [regex]::Matches($line, '\[([^\]]*)\]\(([^)]+)\)')
        foreach ($m in $matches) {
            $linkText = $m.Groups[1].Value
            $target = $m.Groups[2].Value.Trim()

            # Skip external or custom protocols
            if ($target -match '^(https?|mailto|ftp|conversation|sidecar|file):') {
                continue
            }

            # Skip template placeholders or template directories
            if ($target -match '<[^>]+>' -or
                $target -match 'TASK-YYYYMMDD' -or
                $target -match 'AC-NNN' -or
                $target -match 'CASE-NNN' -or
                $file.FullName -match '[\\/]_template[\\/]' -or
                $file.FullName -match '[\\/]docs[\\/]09-reference[\\/]templates[\\/]') {
                continue
            }

            # Split path and anchor
            $parts = $target -split '#', 2
            $filePathPart = $parts[0].Trim()
            $anchorPart = if ($parts.Length -gt 1) { $parts[1].Trim() } else { '' }

            $resolvedFile = $null
            if ([string]::IsNullOrWhiteSpace($filePathPart)) {
                # Same file anchor
                $resolvedFile = $file.FullName
            } elseif ($filePathPart.StartsWith('/')) {
                # Repo-root relative
                $resolvedFile = Join-Path $repoRoot ($filePathPart.TrimStart('/').Replace('/', '\'))
            } else {
                # Relative to current file
                $resolvedFile = Join-Path $file.DirectoryName ($filePathPart.Replace('/', '\'))
            }

            # If resolved file is a directory or path ending with /
            if (Test-Path -LiteralPath $resolvedFile -PathType Container) {
                # Allowed if it's referencing a directory
                continue
            }

            if (-not (Test-Path -LiteralPath $resolvedFile -PathType Leaf)) {
                $relSource = $file.FullName.Substring($repoRoot.Length).TrimStart('\', '/')
                [void]$failures.Add("${relSource}:${lineNum}: Dead link target '$target' (file not found: $resolvedFile)")
                continue
            }

            # Verify anchor if specified
            if (-not [string]::IsNullOrWhiteSpace($anchorPart)) {
                $targetContent = Get-Content -LiteralPath $resolvedFile -Raw
                $headers = [regex]::Matches($targetContent, '(?m)^#{1,6}\s+(.+)$') | ForEach-Object { Slugify $_.Groups[1].Value }
                $normalizedAnchor = $anchorPart.ToLowerInvariant()
                if ($headers -notcontains $normalizedAnchor) {
                    $relSource = $file.FullName.Substring($repoRoot.Length).TrimStart('\', '/')
                    [void]$failures.Add("${relSource}:${lineNum}: Anchor '#$anchorPart' not found in '$resolvedFile'")
                }
            }
        }
    }
}

if ($failures.Count -gt 0) {
    Write-Output "FAIL: Markdown links check"
    $failures | ForEach-Object { Write-Output "- $_" }
    exit 1
}

Write-Output "PASS: Markdown links check"
exit 0
