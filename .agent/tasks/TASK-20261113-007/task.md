# Task

- Task ID: `TASK-20261113-007`
- Status: `COMPLETED`
- Owner: `@agent`

## Objective

Đóng khoảng trống giữa hệ thống enforcement hiện tại và mô hình verification của Lauren Tan: kiểm thử sản phẩm thật, lỗi lặp thành lint rule, invariant của subsystem có check chạy được, và lifecycle được model-check tất định.

## Scope

- Sửa lỗi xử lý stderr của Git trong toàn bộ gate (`.` agent runtime dùng chung).
- Làm regression suite tất định, tự dọn dẹp và mở rộng coverage.
- Bắt buộc khai báo product verification trong `spec.md`; `verify-product.ps1` FAIL khi thiếu.
- Thêm evidence artifact máy đọc được cho product verification.
- Thêm lint rule codify lỗi lặp.
- Thêm registry verification theo subsystem và model check lifecycle.
- Cập nhật CI để chạy toàn bộ các lớp trên.

## Non-goals

- Không sửa Markdown tài liệu dự án ngoài task workspace.
- Không thêm dependency, không đổi tech stack ứng dụng.
- Không thay đổi nội dung các task đã hoàn tất trước đó.

## Acceptance criteria

| ID | Criterion | Required evidence |
| --- | --- | --- |
| AC-001 | Mọi lời gọi Git đi qua runtime dùng chung; cảnh báo stderr chỉ là diagnostics, không làm gate terminating | CASE-011 PASS và dòng `WARN:` trong final gate |
| AC-002 | Regression suite chạy đủ 13 case, không dừng giữa chừng và không để lại thư mục tạm | Output runner 13/13 PASS |
| AC-003 | `spec.md` bắt buộc khai báo product verification; validator FAIL khi thiếu hoặc sai | CASE-008 PASS và `validate-task.ps1` |
| AC-004 | `verify-product.ps1` chạy lệnh product thật, FAIL khi thiếu khai báo hoặc lệnh lỗi, không còn claim baseline | CASE-008, CASE-010 PASS |
| AC-005 | Product verification ghi artifact máy đọc được với status, command và exit code | `logs/product-verification.json` |
| AC-006 | Lỗi lặp được enforce bằng lint rule có exit code | CASE-012 PASS, `test-agent-lint.ps1` |
| AC-007 | Mỗi subsystem có invariant map tới check chạy được; thiếu check là FAIL | `verify-subsystem.ps1 -All` PASS |
| AC-008 | Lifecycle có model check khám phá toàn bộ và replay tất định | CASE-013 PASS, output model |
| AC-009 | Final gate bỏ qua log evidence khi kiểm whitespace và PASS cho các task đã hoàn tất | Gate TASK-002, TASK-006, TASK-007 PASS |
| AC-010 | CI chạy lint, link, model check, regression, subsystem verification và final gate | `.github/workflows/quality-gates.yml` |

## References

- `/AGENTS.md`
- `/docs/00-governance/agent-execution-standard.md`
- `/.agent/evaluations/README.md`
- `/.agent/hooks/README.md`
