# Verification

- Task ID: `TASK-20261113-007`
- Status: `COMPLETED`

## Acceptance coverage

| ID | Result | Evidence |
| --- | --- | --- |
| AC-001 | PASS | CASE-011 PASS với `GIT_TRACE=1` gây stderr; final gate in `WARN: git diff --check diagnostics` và vẫn exit 0 |
| AC-002 | PASS | `run-regression-tests.ps1` in bảng 13/13 PASS và dọn sạch `_temp_regression`, `TASK-2099*` |
| AC-003 | PASS | CASE-008 PASS: task không khai báo product verification bị `validate-task.ps1` và `verify-product.ps1` chặn |
| AC-004 | PASS | CASE-010 PASS: lệnh `exit 7` làm harness FAIL exit 1; không còn chuỗi claim baseline trong scripts |
| AC-005 | PASS | `logs/product-verification.json` ghi status, command, exitCode, durationSeconds và logPath |
| AC-006 | PASS | CASE-012 PASS: lint phát hiện `& git status` trực tiếp và trả exit 1 |
| AC-007 | PASS | `verify-subsystem.ps1 -All` chạy 11 invariant thuộc 3 subsystem, exit 0 |
| AC-008 | PASS | CASE-013 PASS: model khám phá 48 đường đi, 5 đường COMPLETED, replay tất định |
| AC-009 | PASS | Final gate TASK-20261113-002, TASK-20261113-006, TASK-20261113-007 đều PASS sau khi loại trừ log khỏi diff check |
| AC-010 | PASS | `.github/workflows/quality-gates.yml` chạy lint, link, model check, regression, subsystem và final gate |

## Commands

| Command | Result | Evidence |
| --- | --- | --- |
| `powershell -NoProfile -ExecutionPolicy Bypass -File .agent/scripts/run-regression-tests.ps1` | PASS | 13/13 case PASS, exit 0 |
| `powershell -NoProfile -ExecutionPolicy Bypass -File .agent/scripts/verify-product.ps1 -TaskId TASK-20261113-007` | PASS | Product command exit 0, artifact status VERIFIED |
| `powershell -NoProfile -ExecutionPolicy Bypass -File .agent/scripts/test-agent-lint.ps1` | PASS | 12 scripts, 0 vi phạm |
| `powershell -NoProfile -ExecutionPolicy Bypass -File .agent/scripts/model-check-task-lifecycle.ps1` | PASS | 48 paths explored, 5 COMPLETED paths, 9 states reachable |
| `powershell -NoProfile -ExecutionPolicy Bypass -File .agent/scripts/verify-subsystem.ps1 -All` | PASS | 11 invariant trên 3 subsystem, exit 0 |
| `powershell -NoProfile -ExecutionPolicy Bypass -File .agent/scripts/run-quality-gates.ps1 -TaskId TASK-20261113-007 -Mode final` | PASS | Toàn bộ gate final PASS, exit 0 |
| `powershell -NoProfile -ExecutionPolicy Bypass -File .agent/scripts/run-quality-gates.ps1 -TaskId TASK-20261113-002 -Mode final` | PASS | Task lịch sử đi qua gate với trạng thái LEGACY_EXEMPT |
| `powershell -NoProfile -ExecutionPolicy Bypass -File .agent/scripts/run-quality-gates.ps1 -TaskId TASK-20261113-006 -Mode final` | PASS | Task cũ có Test Command đạt trạng thái VERIFIED |

## Limitations

- Model check là explicit-state trên automaton lifecycle; chưa dùng TLA+ hoặc Lean cho các subsystem khác.
- `/.agent/evaluations/cases/README.md` chưa liệt kê CASE-008 đến CASE-013 vì là Markdown ngoài task workspace và cần phê duyệt trực tiếp.
- Chưa có browser/runtime verification của ứng dụng thật; repository hiện không có app build hay test command riêng.

## Final status

`COMPLETED`

## References

- `/.agent/scripts/run-quality-gates.ps1`
- `/.agent/scripts/run-regression-tests.ps1`
- `/.agent/scripts/verify-product.ps1`
- `/.agent/scripts/verify-subsystem.ps1`
- `/.agent/scripts/model-check-task-lifecycle.ps1`
- `/.agent/scripts/test-agent-lint.ps1`
- `/AGENTS.md`
