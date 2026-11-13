# Agent Hooks

## Order

1. `preflight.md`
2. `plan-gate.md`
3. `execution-gate.md`
4. `output-gate.md`
5. `handoff-gate.md`

## Rules

- Agent MUST run hooks in order.
- Agent MUST record PASS/FAIL and evidence.
- Agent MUST stop on FAIL.
- Agent MUST fix FAIL before continuing.
- Agent MUST run `/.agent/scripts/run-quality-gates.ps1`.

## References

- `/AGENTS.md`
- `/docs/00-governance/agent-execution-standard.md`
