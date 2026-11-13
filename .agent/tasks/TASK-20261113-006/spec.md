# Specification

- Task ID: `TASK-20261113-006`

## Inputs
- Implementation plan đã được người dùng phê duyệt: `lauren-tan-enforcement-plan.md`.
- 8 khoảng trống kỹ thuật được chỉ định cụ thể.
- Quy chuẩn tại `/AGENTS.md` và `/docs/00-governance/agent-execution-standard.md`.

## Confirmed facts
- Hệ thống hiện tại có `run-quality-gates.ps1`, `validate-task.ps1`, `verify-task.ps1`.
- Các file hook markdown hiện chỉ được kiểm tra sự tồn tại, chưa thực thi logic kiểm tra.
- Repository chưa có file CI trong `.github/`.
- Regression cases `CASE-001` đến `CASE-005` chưa có script tự động chạy.
- Verifier hiện chỉ kiểm tra chuỗi tĩnh, chưa đối chiếu git status/diff thật.
- Verifier hiện hardcode 4 dimension đánh giá.
- Chưa có script cơ học ngăn sửa Markdown tài liệu dự án ngoài task workspace.
- Các placeholder như `<TASK-ID>` cần cơ chế bỏ qua hoặc xác thực cú pháp riêng trong link checker.

## Inferences
- Khi bổ sung `check-doc-boundaries.ps1`, hệ thống cần một cơ chế whitelist minh bạch trong `spec.md` (`## Approved Documentation Changes`) để cho phép cập nhật tài liệu kiểm thử hoặc governance khi có phê duyệt.
- Việc kiểm tra cú pháp script trong `verify-product.ps1` có thể thực hiện an toàn thông qua AST parser của PowerShell mà không cần thực thi mã không an toàn.

## Unknowns
- Không có (tất cả yêu cầu và giao diện kỹ thuật đã được làm rõ và phê duyệt).

## Constraints
- Phải bảo toàn toàn bộ hành vi hợp lệ của các task đã hoàn thành (`TASK-20261113-002` đến `005`).
- Tương thích cả Windows PowerShell 5.1 và PowerShell Core (`pwsh`).
- Không tạo secret, credential hay file rác.
- Mọi failure mode phải trả exit code khác 0.

## Decisions
- Phân loại input: `DIRECT`.
- Triển khai 8 module kỹ thuật theo đúng cấu trúc trong implementation plan.
- Bổ sung `CASE-006` (unauthorized markdown boundary violation) và `CASE-007` (broken markdown link) vào danh mục regression cases.
- Sử dụng thư mục `.agent/tasks/<TASK-ID>/logs/` để chứa file log chứng minh lệnh kiểm thử runtime thực tế.

## Test Command
powershell -NoProfile -ExecutionPolicy Bypass -File .agent/scripts/run-regression-tests.ps1

## Approved Documentation Changes
- `/.agent/evaluations/cases/README.md` (cập nhật danh mục kiểm thử hồi quy CASE-001 đến CASE-007 theo phê duyệt trong plan).

## Output contract
- `.agent/scripts/invoke-hook.ps1` [NEW]
- `.agent/scripts/verify-product.ps1` [NEW]
- `.agent/scripts/check-doc-boundaries.ps1` [NEW]
- `.agent/scripts/test-markdown-links.ps1` [NEW]
- `.agent/scripts/run-regression-tests.ps1` [NEW]
- `.github/workflows/quality-gates.yml` [NEW]
- `.agent/scripts/run-quality-gates.ps1` [MODIFY]
- `.agent/scripts/validate-task.ps1` [MODIFY]
- `.agent/scripts/verify-task.ps1` [MODIFY]
- `.agent/evaluations/cases/README.md` [MODIFY]

## References
- `/AGENTS.md`
- `/docs/00-governance/agent-execution-standard.md`
- `/.agent/hooks/README.md`
- `/.agent/evaluations/README.md`
