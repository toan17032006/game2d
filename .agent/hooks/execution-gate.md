# Execution Gate

- MUST execute phases in dependency order.
- MUST update phase status.
- MUST preserve approved scope and decisions.
- MUST record changed files.
- MUST record commands and results.
- MUST stop on blocker.

## PASS

All planned phases are executed or explicitly marked blocked.

## FAIL

Stop. Record blocker in `handoff.md`.

## References

- `/docs/00-governance/agent-execution-standard.md`
