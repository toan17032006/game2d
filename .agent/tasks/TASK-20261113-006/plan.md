# Plan

- Task ID: `TASK-20261113-006`
- Status: `COMPLETED`

## Objective
Triển khai hệ thống bảo đảm thực thi tất định (Deterministic Enforcement & Paved Path) theo Lauren Tan, khắc phục triệt để 8 khoảng trống.

## Phases

| ID | Action | Input | Expected output | Validation | Status |
| --- | --- | --- | --- | --- | --- |
| P1 | Thiết lập Task Workspace và chạy Preflight Quality Gate | `prompt.md`, `task.md`, `spec.md`, `plan.md`, `handoff.md` | Preflight gate PASS | `run-quality-gates.ps1 -Mode preflight` | `DONE` |
| P2 | Xây dựng Hook Engine (`invoke-hook.ps1`) và Boundary Checker (`check-doc-boundaries.ps1`) | Spec thiết kế component 1 & 7 | Hai script hoàn thiện, có exit code | Cú pháp PowerShell AST & unit run | `DONE` |
| P3 | Xây dựng Link Checker (`test-markdown-links.ps1`) và Product Harness (`verify-product.ps1`) | Spec thiết kế component 2 & 8 | Hai script hoàn thiện, hỗ trợ template | Quét toàn bộ repo không có link gãy | `DONE` |
| P4 | Nâng cấp Verifier Ground-Truth và Dynamic Rubric (`verify-task.ps1`) | Spec thiết kế component 5 & 6 | Verifier đối chiếu git status và rubric động | Test trên các task hiện có | `DONE` |
| P5 | Xây dựng Automated Regression Runner (`run-regression-tests.ps1`) và cập nhật cases doc | Spec thiết kế component 4 | Test runner tự động chạy PASS cả 7 ca | `run-regression-tests.ps1` exit code 0 | `DONE` |
| P6 | Xây dựng GitHub Actions CI (`quality-gates.yml`) và tích hợp `run-quality-gates.ps1` | Spec thiết kế component 3 | CI workflow hoàn thiện, runner tích hợp đủ chốt | AST syntax & dry-run runner | `DONE` |
| P7 | Chạy toàn diện Final Verification, Dynamic Evaluation và Final Quality Gate | Bộ scripts và task artifact | Mọi gate PASS, log ghi nhận đầy đủ | `run-quality-gates.ps1 -Mode final` exit 0 | `DONE` |

## Final status

`COMPLETED`

## Dependencies
- Phải hoàn thành P2, P3, P4 trước khi chạy đầy đủ P5 (regression test suite kiểm tra cả verifier, hooks và link checker).
- Phải hoàn thành P5 trước khi chốt P6 và P7.

## Checkpoints
- Checkpoint 1: Preflight quality gate PASS trước khi vào P2.
- Checkpoint 2: Từng script đơn lẻ vượt qua kiểm tra cú pháp AST.
- Checkpoint 3: Regression test suite chạy 7/7 ca PASS.
- Checkpoint 4: Final quality gate PASS toàn diện trước khi báo hoàn tất.

## Risks
- Rủi ro git diff giữa Windows CRLF và Linux LF: xử lý bằng `-c core.autocrlf=false` và kiểm tra `git diff --check`.
- Rủi ro phá vỡ tính tương thích của các task cũ: regression test suite và verifier phải bảo đảm tương thích ngược với `TASK-20261113-002` đến `005`.

## Rollback
- Hoàn tác mã nguồn thông qua Git checkout nếu phát sinh lỗi không tương thích.

## References
- `/AGENTS.md`
- `/docs/00-governance/agent-execution-standard.md`
- `/.agent/hooks/README.md`
- `/.agent/evaluations/README.md`
