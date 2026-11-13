# Prompt

## Role

Agent quality-control designer.

## Objective

Thêm hooks, quality gates và evaluation layers để kiểm soát input, plan, execution, output và handoff.

## Scope

- `.agent/hooks/`
- `.agent/evaluations/`
- `.agent/tasks/`
- `AGENTS.md`
- `/docs/00-governance/agent-execution-standard.md`

## Acceptance criteria

- Có preflight, plan gate, execution gate, output gate và handoff gate.
- Có tiêu chí PASS/FAIL.
- Task workspace có `evaluation.md`.
- Agent MUST chạy đủ gate trước khi báo `COMPLETED`.

## References

- `/AGENTS.md`
- `/docs/00-governance/agent-execution-standard.md`
