# Task

- Task ID: `TASK-20261113-004`
- Status: `COMPLETED`
- Owner: `@agent`

## Objective

Thêm enforcement thực thi được cho quality gates.

## Scope

Validator, runner, hooks, evaluation và agent rules.

## Acceptance criteria

- `validate-task.ps1` kiểm tra task integrity.
- `run-quality-gates.ps1` chạy preflight và final gate.
- Lỗi validation trả non-zero exit code.
- Có lệnh chạy chuẩn trong `AGENTS.md`.

## Acceptance IDs

| ID | Criterion | Required evidence |
| --- | --- | --- |
| AC-001 | Validator checks task integrity | `validate-task.ps1` |
| AC-002 | Runner executes preflight and final | `run-quality-gates.ps1` |
| AC-003 | Failure returns non-zero | Gate run |
| AC-004 | Standard command is documented | `AGENTS.md` |

## References

- `/.agent/hooks/README.md`
- `/.agent/evaluations/README.md`
