# Task

- Task ID: `TASK-20261113-005`
- Status: `EXECUTING`
- Owner: `@agent`

## Objective

Đảm bảo final gate xác minh được output thay vì chỉ đọc status.

## Scope

Acceptance mapping, evidence verifier, evaluation completeness và regression cases.

## Acceptance criteria

| ID | Criterion | Required evidence |
| --- | --- | --- |
| AC-001 | Validator đọc acceptance IDs | Command output |
| AC-002 | Verifier kiểm tra coverage | PASS/FAIL output |
| AC-003 | Evaluation đủ dimensions | Score table |
| AC-004 | Final gate chặn thiếu evidence | Non-zero test |
| AC-005 | Có regression cases | Case inventory |

## References

- `/.agent/scripts/`
- `/.agent/evaluations/`
