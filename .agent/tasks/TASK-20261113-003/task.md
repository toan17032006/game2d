# Task

- Task ID: `TASK-20261113-003`
- Status: `COMPLETED`
- Owner: `@agent`

## Objective

Thêm cơ chế kiểm thử và đánh giá cho agent.

## Scope

Hooks, evaluations, task template và execution standard.

## Acceptance criteria

- Input có gate.
- Plan/spec có gate.
- Execution có gate.
- Output có gate.
- Handoff có gate.
- Có evidence cho mỗi gate.

## Acceptance IDs

| ID | Criterion | Required evidence |
| --- | --- | --- |
| AC-001 | Input gate exists | `preflight.md` |
| AC-002 | Plan gate exists | `plan-gate.md` |
| AC-003 | Execution gate exists | `execution-gate.md` |
| AC-004 | Output gate exists | `output-gate.md` |
| AC-005 | Handoff gate exists | `handoff-gate.md` |
| AC-006 | Evidence is required | Gate evidence |

## References

- `/AGENTS.md`
