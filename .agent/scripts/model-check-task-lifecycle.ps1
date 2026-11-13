[CmdletBinding()]
param(
    [int]$MaxDepth = 6
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

. (Join-Path $PSScriptRoot 'lib\agent-runtime.ps1')

# Executable specification of the task lifecycle.
#
# This is an explicit-state model check over a small automaton: every reachable
# event sequence up to MaxDepth is enumerated and the lifecycle invariants are
# asserted on all of them. It is deliberately independent of the gate scripts,
# so a gate that stops enforcing a rule cannot silently weaken the model.
#
# BLOCKED is modelled per originating state so that "resume" can never restore
# progress that was never earned. Stopping the line is always allowed; skipping
# a gate never is.

$states = @(
    'NEW',
    'PLANNED',
    'EXECUTING',
    'VERIFYING',
    'COMPLETED',
    'BLOCKED_NEW',
    'BLOCKED_PLANNED',
    'BLOCKED_EXECUTING',
    'BLOCKED_VERIFYING'
)

$events = @('preflight-pass', 'plan-gate-pass', 'evidence-complete', 'final-gate-pass', 'block', 'resume')

# Each guarded transition names the artifact that actually enforces it. If an
# enforcing script disappears, the model is no longer a description of reality.
$transitions = @(
    [PSCustomObject]@{ From = 'NEW';               Event = 'preflight-pass';    To = 'PLANNED';           Guard = '.agent/scripts/run-quality-gates.ps1' },
    [PSCustomObject]@{ From = 'PLANNED';           Event = 'plan-gate-pass';    To = 'EXECUTING';         Guard = '.agent/scripts/invoke-hook.ps1' },
    [PSCustomObject]@{ From = 'EXECUTING';         Event = 'evidence-complete'; To = 'VERIFYING';         Guard = '.agent/scripts/verify-task.ps1' },
    [PSCustomObject]@{ From = 'VERIFYING';         Event = 'final-gate-pass';   To = 'COMPLETED';         Guard = '.agent/scripts/run-quality-gates.ps1' },
    [PSCustomObject]@{ From = 'NEW';               Event = 'block';             To = 'BLOCKED_NEW';       Guard = '.agent/scripts/invoke-hook.ps1' },
    [PSCustomObject]@{ From = 'PLANNED';           Event = 'block';             To = 'BLOCKED_PLANNED';   Guard = '.agent/scripts/invoke-hook.ps1' },
    [PSCustomObject]@{ From = 'EXECUTING';         Event = 'block';             To = 'BLOCKED_EXECUTING'; Guard = '.agent/scripts/invoke-hook.ps1' },
    [PSCustomObject]@{ From = 'VERIFYING';         Event = 'block';             To = 'BLOCKED_VERIFYING'; Guard = '.agent/scripts/invoke-hook.ps1' },
    [PSCustomObject]@{ From = 'BLOCKED_NEW';       Event = 'resume';            To = 'NEW';               Guard = '.agent/tasks' },
    [PSCustomObject]@{ From = 'BLOCKED_PLANNED';   Event = 'resume';            To = 'PLANNED';           Guard = '.agent/tasks' },
    [PSCustomObject]@{ From = 'BLOCKED_EXECUTING'; Event = 'resume';            To = 'EXECUTING';         Guard = '.agent/tasks' },
    [PSCustomObject]@{ From = 'BLOCKED_VERIFYING'; Event = 'resume';            To = 'VERIFYING';         Guard = '.agent/tasks' }
)

$gateEvents = @('preflight-pass', 'plan-gate-pass', 'evidence-complete', 'final-gate-pass')
$failures = [System.Collections.Generic.List[string]]::new()
function Fail([string]$Message) { [void]$failures.Add($Message) }

function Get-NextState {
    param([string]$State, [string]$Event)

    foreach ($transition in $transitions) {
        if ($transition.From -eq $State -and $transition.Event -eq $Event) {
            return $transition.To
        }
    }
    return $null
}

function Invoke-TaskLifecycleReplay {
    param([string[]]$EventLog)

    $state = 'NEW'
    foreach ($event in $EventLog) {
        $next = Get-NextState -State $state -Event $event
        if ($null -eq $next) {
            return [PSCustomObject]@{
                Ok    = $false
                State = $state
                Error = "illegal event '$event' from state '$state'"
            }
        }
        $state = $next
    }

    return [PSCustomObject]@{ Ok = $true; State = $state; Error = '' }
}

# --- Model wiring: guards must exist on disk -------------------------------
foreach ($transition in $transitions) {
    if (-not (Test-Path -LiteralPath (Join-Path (Get-AgentRepoRoot) ($transition.Guard.Replace('/', '\'))))) {
        Fail "model guard is missing on disk: $($transition.Guard) ($($transition.From) --$($transition.Event)--> $($transition.To))"
    }
}

foreach ($state in $states) {
    if ($state -eq 'NEW') { continue }
    $incoming = @($transitions | Where-Object { $_.To -eq $state })
    if ($incoming.Count -eq 0) { Fail "state '$state' is unreachable in the model" }
}

# --- Exhaustive reachable-path exploration --------------------------------
$script:explored = 0
$completedPaths = [System.Collections.Generic.List[string]]::new()
$reachableStates = [System.Collections.Generic.HashSet[string]]::new()

function Invoke-Explore {
    param(
        [string]$State,
        [string[]]$Path,
        [int]$Depth
    )

    $script:explored = $script:explored + 1
    [void]$reachableStates.Add($State)

    if ($State -eq 'COMPLETED') {
        [void]$completedPaths.Add(($Path -join ' > '))
        return
    }

    if ($Depth -ge $MaxDepth) { return }

    foreach ($event in $events) {
        $next = Get-NextState -State $State -Event $event
        if ($null -eq $next) { continue }
        $nextPath = @($Path) + @($event)
        Invoke-Explore -State $next -Path $nextPath -Depth ($Depth + 1)
    }
}

Invoke-Explore -State 'NEW' -Path @() -Depth 0

if ($explored -eq 0) { Fail 'model exploration visited no states' }
if ($completedPaths.Count -eq 0) { Fail 'COMPLETED is unreachable within the model depth bound' }

# --- Invariants ------------------------------------------------------------
# INV-LIFE-001: COMPLETED requires every gate, in order.
foreach ($path in $completedPaths) {
    $eventsInPath = @($path -split ' > ')
    $cursor = 0
    foreach ($required in $gateEvents) {
        $found = -1
        for ($i = $cursor; $i -lt $eventsInPath.Count; $i++) {
            if ($eventsInPath[$i] -eq $required) { $found = $i; break }
        }
        if ($found -lt 0) {
            Fail "INV-LIFE-001 violated: COMPLETED reached without '$required' in order ($path)"
            break
        }
        $cursor = $found + 1
    }
}

# INV-LIFE-002: COMPLETED is terminal.
foreach ($event in $events) {
    if ($null -ne (Get-NextState -State 'COMPLETED' -Event $event)) {
        Fail "INV-LIFE-002 violated: COMPLETED accepts event '$event'"
    }
}

# INV-LIFE-003: no dead ends - every reachable state can progress or resume.
foreach ($state in $reachableStates) {
    if ($state -eq 'COMPLETED') { continue }
    $outgoing = @($transitions | Where-Object { $_.From -eq $state })
    if ($outgoing.Count -eq 0) { Fail "INV-LIFE-003 violated: state '$state' has no outgoing transition" }
}

# INV-LIFE-004: the fail-safe path is available from every non-terminal state.
foreach ($state in @('NEW', 'PLANNED', 'EXECUTING', 'VERIFYING')) {
    if ($null -eq (Get-NextState -State $state -Event 'block')) {
        Fail "INV-LIFE-004 violated: '$state' cannot stop the line through BLOCKED"
    }
}

# INV-LIFE-005: no gate can be skipped by starting in the middle.
$illegalEntry = Invoke-TaskLifecycleReplay -EventLog @('plan-gate-pass')
if ($illegalEntry.Ok) {
    Fail "INV-LIFE-005 violated: plan gate accepted without preflight (state '$($illegalEntry.State)')"
}

# --- Deterministic replay fixtures ---------------------------------------
$happyPath = @('preflight-pass', 'plan-gate-pass', 'evidence-complete', 'final-gate-pass')
$replayA = Invoke-TaskLifecycleReplay -EventLog $happyPath
$replayB = Invoke-TaskLifecycleReplay -EventLog $happyPath
if (-not $replayA.Ok -or $replayA.State -ne 'COMPLETED') {
    Fail "Replay of the happy path did not reach COMPLETED: $($replayA.Error)"
}
if ($replayA.State -ne $replayB.State) {
    Fail 'Replay is not deterministic for identical event logs'
}

$shortcut = Invoke-TaskLifecycleReplay -EventLog @('preflight-pass', 'plan-gate-pass', 'final-gate-pass')
if ($shortcut.Ok) {
    Fail "Replay accepted a shortcut to COMPLETED without evidence (state '$($shortcut.State)')"
}

$recovery = Invoke-TaskLifecycleReplay -EventLog @('block', 'resume', 'evidence-complete', 'final-gate-pass')
if ($recovery.Ok) {
    Fail "Replay accepted gates on a resumed task without preflight and plan gate (state '$($recovery.State)')"
}

$resumeKeepsProgress = Invoke-TaskLifecycleReplay -EventLog @('preflight-pass', 'block', 'resume', 'plan-gate-pass', 'evidence-complete', 'final-gate-pass')
if (-not $resumeKeepsProgress.Ok -or $resumeKeepsProgress.State -ne 'COMPLETED') {
    Fail "Replay rejected a legitimate blocked-then-resumed task: $($resumeKeepsProgress.Error)"
}

$evidenceAfterBlock = Invoke-TaskLifecycleReplay -EventLog @('block', 'resume', 'plan-gate-pass')
if ($evidenceAfterBlock.Ok) {
    Fail "Replay accepted plan gate after resume on an unplanned task (state '$($evidenceAfterBlock.State)')"
}

if ($failures.Count -gt 0) {
    Write-Output 'FAIL: task lifecycle model check'
    $failures | ForEach-Object { Write-Output "- $_" }
    exit 1
}

Write-Output "PASS: task lifecycle model check ($explored paths explored, $($completedPaths.Count) COMPLETED paths, $($reachableStates.Count) states reachable)"
exit 0
