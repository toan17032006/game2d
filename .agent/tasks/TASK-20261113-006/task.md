# Task

- Task ID: `TASK-20261113-006`
- Status: `COMPLETED`
- Owner: `@agent`

## Objective
Triển khai toàn bộ 8 thành phần bảo đảm thực thi tất định (Deterministic Enforcement & Paved Path) theo Lauren Tan để giải quyết triệt để 8 khoảng trống vận hành của hệ thống agent.

## Scope
- Engine thực thi hook (`invoke-hook.ps1`).
- Runtime product verification harness (`verify-product.ps1`).
- CI pipeline tự động (`.github/workflows/quality-gates.yml`).
- Test suite hồi quy tự động (`run-regression-tests.ps1`).
- Nâng cấp verifier ground-truth và dynamic rubric (`verify-task.ps1`).
- Cơ chế khóa cứng tài liệu read-only (`check-doc-boundaries.ps1`).
- Công cụ kiểm tra link markdown thông minh với template path (`test-markdown-links.ps1`).
- Nâng cấp runner chất lượng (`run-quality-gates.ps1`, `validate-task.ps1`).
- Cập nhật danh mục kiểm thử hồi quy (`.agent/evaluations/cases/README.md`).

## Non-goals
- Sửa đổi các file Markdown nghiệp vụ ngoài task workspace và danh mục phê duyệt trong `spec.md`.
- Thay đổi cấu trúc cốt lõi của repository ngoài phạm vi quản trị agent và CI.

## Acceptance criteria

| ID | Criterion | Required evidence |
| --- | --- | --- |
| AC-001 | Hook execution engine (`invoke-hook.ps1`) thực thi được nội dung kiểm tra của `preflight`, `plan-gate`, `output-gate`, `handoff-gate` và trả non-zero khi vi phạm | Kiểm thử trực tiếp và regression case |
| AC-002 | Product verification harness (`verify-product.ps1`) kiểm tra cú pháp mã nguồn, tính toàn vẹn liên kết và chạy lệnh test sản phẩm thật ghi log | Log thực thi tại `logs/test-execution.log` |
| AC-003 | CI pipeline (`.github/workflows/quality-gates.yml`) tự động phát hiện task, chạy boundary check, quality gate, regression test và link check | File workflow hợp lệ và diff |
| AC-004 | Regression runner (`run-regression-tests.ps1`) thực thi tự động toàn bộ 7 ca kiểm thử (`CASE-001` đến `CASE-007`) | Báo cáo test runner PASS toàn bộ 7 ca |
| AC-005 | Verifier (`verify-task.ps1`) xác thực ground-truth danh sách file thay đổi so với Git diff và kiểm tra log lệnh thực thi | Verifier output kiểm tra git diff |
| AC-006 | Verifier hỗ trợ dynamic rubric, quét các dimension chuẩn từ `.agent/evaluations/` và bắt buộc điểm hợp lệ kèm bằng chứng cho mọi dimension | Verifier output với dynamic rubric |
| AC-007 | Boundary checker (`check-doc-boundaries.ps1`) phát hiện và chặn đứng mọi sửa đổi Markdown tài liệu dự án chưa được duyệt trong `spec.md` | Boundary check test PASS |
| AC-008 | Link checker (`test-markdown-links.ps1`) phân biệt chính xác đường dẫn thật và template placeholder, kiểm tra link và anchor hợp lệ | Link checker report PASS |

## References
- `/AGENTS.md`
- `/docs/00-governance/agent-execution-standard.md`
- `/.agent/hooks/README.md`
- `/.agent/evaluations/README.md`
- `/.agent/evaluations/cases/README.md`
