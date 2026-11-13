# Prompt

## Role
AI Engineering & Agent Governance Architect

## Objective
Nâng cấp toàn diện hệ thống quản trị và thực thi agent trong monster-engine để đạt chuẩn bảo đảm thực thi tất định (Deterministic Enforcement & Paved Path) theo Lauren Tan, khắc phục triệt để 8 khoảng trống đã xác định.

## Context
Repository hiện tại đã có cấu trúc task và quality gate cơ bản (đạt ~70-75% mô hình), nhưng các hook vẫn là markdown mềm, chưa có runtime verification harness cho sản phẩm thật, chưa có CI pipeline, regression cases chưa tự động hóa, verifier chưa kiểm chứng ground-truth diff/command logs, rubric đánh giá còn cố định, và tài liệu dự án chưa được khóa bằng code check.

## Inputs
- Phê duyệt của người dùng đối với implementation plan `lauren-tan-enforcement-plan.md`.
- 8 khoảng trống kỹ thuật được nêu chi tiết trong yêu cầu.
- Quy chuẩn tại `/AGENTS.md` và `/docs/00-governance/agent-execution-standard.md`.

## Scope
- Tạo `invoke-hook.ps1` để thực thi hóa các hook markdown thành script có exit code.
- Tạo `verify-product.ps1` làm runtime verification harness cho sản phẩm.
- Tạo workflow GitHub Actions `.github/workflows/quality-gates.yml`.
- Tạo `run-regression-tests.ps1` tự động hóa `CASE-001` đến `CASE-007`.
- Nâng cấp `verify-task.ps1` xác thực ground-truth (changed files khớp git diff, artifact/log command tồn tại) và dynamic rubric.
- Tạo `check-doc-boundaries.ps1` cơ chế khóa cứng tài liệu read-only.
- Tạo `test-markdown-links.ps1` kiểm tra link markdown thông minh với template placeholder.
- Cập nhật `run-quality-gates.ps1` và `validate-task.ps1` tích hợp toàn bộ các chốt chặn mới.
- Cập nhật `.agent/evaluations/cases/README.md`.

## Non-goals
- Sửa đổi nội dung tài liệu nghiệp vụ ngoài phạm vi đã được phê duyệt trong `spec.md`.
- Thay đổi cấu trúc thư mục cốt lõi ngoài `.agent/` và `.github/`.

## Constraints
- Tuân thủ nghiêm ngặt `/AGENTS.md` và các quy định an toàn file.
- Mọi script phải chạy được trên Windows PowerShell và PowerShell Core (`pwsh`).
- Mọi failure mode phải trả exit code khác 0.

## Execution rules
- Input classification: `DIRECT` (đã được người dùng phê duyệt trực tiếp kế hoạch triển khai).
- Chạy preflight quality gate trước khi chuyển sang `EXECUTING`.
- Cập nhật `handoff.md` sau mỗi phase.
- Chạy final quality gate trước khi báo `COMPLETED`.

## Output contract
- 5 script mới: `invoke-hook.ps1`, `verify-product.ps1`, `check-doc-boundaries.ps1`, `test-markdown-links.ps1`, `run-regression-tests.ps1`.
- 1 workflow CI: `.github/workflows/quality-gates.yml`.
- 3 script nâng cấp: `run-quality-gates.ps1`, `validate-task.ps1`, `verify-task.ps1`.
- Bộ task artifact đầy đủ chứng minh PASS tất cả các gate và acceptance criteria.

## Acceptance criteria
- AC-001: Hook execution engine (`invoke-hook.ps1`) thực thi được nội dung kiểm tra của `preflight`, `plan-gate`, `output-gate`, `handoff-gate` và trả non-zero khi vi phạm.
- AC-002: Product verification harness (`verify-product.ps1`) kiểm tra cú pháp mã nguồn, tính toàn vẹn liên kết và chạy lệnh test sản phẩm thật ghi log.
- AC-003: CI pipeline (`.github/workflows/quality-gates.yml`) tự động phát hiện task, chạy boundary check, quality gate, regression test và link check.
- AC-004: Regression runner (`run-regression-tests.ps1`) thực thi tự động toàn bộ 7 ca kiểm thử (`CASE-001` đến `CASE-007`).
- AC-005: Verifier (`verify-task.ps1`) xác thực ground-truth danh sách file thay đổi so với Git diff và kiểm tra log lệnh thực thi.
- AC-006: Verifier hỗ trợ dynamic rubric, quét các dimension chuẩn từ `.agent/evaluations/` và bắt buộc điểm hợp lệ kèm bằng chứng cho mọi dimension.
- AC-007: Boundary checker (`check-doc-boundaries.ps1`) phát hiện và chặn đứng mọi sửa đổi Markdown tài liệu dự án chưa được duyệt trong `spec.md`.
- AC-008: Link checker (`test-markdown-links.ps1`) phân biệt chính xác đường dẫn thật và template placeholder, kiểm tra link và anchor hợp lệ.

## Verification method
- Chạy unit/regression tests qua `run-regression-tests.ps1`.
- Chạy preflight và final gate qua `run-quality-gates.ps1`.
- Kiểm tra tính toàn vẹn liên kết qua `test-markdown-links.ps1`.
- Kiểm tra `git diff --check`.

## References
- `/AGENTS.md`
- `/docs/00-governance/agent-execution-standard.md`
- `/.agent/hooks/README.md`
- `/.agent/evaluations/README.md`
- `/.agent/evaluations/cases/README.md`
