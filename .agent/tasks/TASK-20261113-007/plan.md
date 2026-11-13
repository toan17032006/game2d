# Plan

- Task ID: `TASK-20261113-007`

## Phases

| ID | Action | Dependency | Checkpoint | Status |
| --- | --- | --- | --- | --- |
| P1 | Tạo runtime dùng chung cho native process và Git | Không | Script parse thành công | DONE |
| P2 | Sửa boundary checker và gate runner sang runtime mới | P1 | Final gate không còn terminating vì stderr | DONE |
| P3 | Viết lại product verification theo hợp đồng chặt | P1 | CASE-008 và CASE-010 đúng kỳ vọng | DONE |
| P4 | Viết lại regression suite, thêm CASE-008 đến CASE-013 | P2, P3 | 13/13 PASS | DONE |
| P5 | Thêm lint rule, registry subsystem và model check lifecycle | P4 | Lint, subsystem, model đều PASS | DONE |
| P6 | Cập nhật CI và chốt evidence trong task workspace | P5 | Final gate PASS | DONE |

## Rollback

- Mỗi phase là thay đổi script độc lập; rollback bằng cách khôi phục file tương ứng từ commit trước.
- Nếu exemption baseline gây chặn nhầm, xóa entry tương ứng và khai báo `## Test Command` cho task đó.
- Nếu model check chặn gate, sửa transition hoặc invariant đang sai; không nới lỏng gate.

## Checkpoints

- Sau P2: chạy `run-quality-gates.ps1 -Mode final` cho một task đã hoàn tất để xác nhận cảnh báo Git không còn fatal.
- Sau P4: chạy toàn bộ regression suite và xác nhận không còn thư mục `_temp_regression` hay `TASK-2099*`.
- Sau P5: chạy `verify-subsystem.ps1 -All` và `model-check-task-lifecycle.ps1`.

## References

- `/AGENTS.md`
- `/.agent/scripts/run-quality-gates.ps1`
- `/.agent/scripts/run-regression-tests.ps1`
