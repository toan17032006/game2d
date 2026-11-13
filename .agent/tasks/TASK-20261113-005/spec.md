# Specification

## Inputs

- Task workspace schema hiện tại.
- Quality gate hiện tại.

## Constraints

- PowerShell.
- Exit `0` chỉ khi PASS.
- Không tin status nếu thiếu evidence.
- Không thay đổi code sản phẩm.

## Output contract

- `verify-task.ps1` kiểm tra acceptance, verification và evaluation.
- `validate-task.ps1` kiểm tra cấu trúc.
- `run-quality-gates.ps1` chạy validator rồi verifier.
- Regression cases nằm tại `/.agent/evaluations/cases/`.

## References

- `/docs/00-governance/agent-execution-standard.md`
