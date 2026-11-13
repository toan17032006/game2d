# Prompt

## Input classification

`DIRECT` — người dùng yêu cầu trực tiếp trong phiên hiện tại: "Commit đi và viết tài liệu đầy đủ vào trong docs - tiếng việt nhá". Đây là phê duyệt trực tiếp cho việc tạo/sửa Markdown tài liệu dự án trong `/docs/` (theo `AGENTS.md` điều khoản "nếu người dùng chưa yêu cầu trực tiếp trong phiên hiện tại").

## Role

Technical writer + architect, viết tài liệu kỹ thuật tiếng Việt chuẩn hóa theo `documentation-standard.md`.

## Objective

1. Viết bộ tài liệu đầy đủ tiếng Việt cho `monster-engine` vào `/docs/` đúng các file đã được liệt kê sẵn trong các README section (product, requirements, architecture, engineering, quality, releases).
2. Chuyển ADR-001 từ draft (TASK-20261113-008) vào `/docs/03-architecture/adr/` với status `Accepted` (người dùng đã phê duyệt bằng yêu cầu trực tiếp).
3. Commit phần tài liệu vào git.

## Context

- TASK-20261113-008 đã COMPLETED: monster-engine Rust foundation (9 crate, 105 tests PASS, final gates PASS) đã commit tại `e1aff19`.
- ADR draft chờ duyệt nằm tại `/.agent/tasks/TASK-20261113-008/research/adr-draft-monster-engine-foundation.md`; nghiên cứu đầy đủ tại `/.agent/tasks/TASK-20261113-008/research/engine-foundation-study.md`.
- Các README section trong `/docs/` đã liệt kê sẵn tên Documents cần có — tạo đúng tên đó để index thành thật.
- `documentation-standard.md`: mỗi doc mở đầu Owner/Status/Last updated/Review date; `## References` với path từ repository root; kebab-case; UTF-8.

## Inputs

- Yêu cầu trực tiếp của người dùng (phiên hiện tại).
- `/AGENTS.md`, `/docs/00-governance/documentation-standard.md`, `/docs/README.md`.
- Code thật của `monster-engine/` (nguồn sự thật cho architecture/engineering docs).

## Scope

- Tạo file mới trong `/docs/01-product/`, `/docs/02-requirements/`, `/docs/03-architecture/` (kèm `adr/`), `/docs/04-engineering/`, `/docs/05-quality/`, `/docs/08-releases/` — đúng danh sách trong `## Approved Documentation Changes` của `spec.md`.
- Commit #2: docs + task workspace TASK-20261113-009.
- Không sửa các README section (chỉ tạo file con mà README đã liệt kê).

## Non-goals

- Không viết `/docs/07-operations/` (engine là thư viện, chưa có service vận hành — sẽ viết khi có game release chạy server/ops).
- Không sửa `/AGENTS.md`, `/CONTRIBUTING.md`, các file governance hiện có.
- Không đổi code engine.

## Constraints

- Tiếng Việt là ngôn ngữ chính (thuật ngữ kỹ thuật giữ tiếng Anh khi là chuẩn ngành).
- Tuân thủ `documentation-standard.md` (metadata order, kebab-case, UTF-8, References).
- Nội dung phải khớp sự thật code (lệnh build/test/run phải chạy được như ghi).
- Không secrets trong docs.

## Execution rules

- Thực thi theo `plan.md`; cập nhật `handoff.md` sau mỗi checkpoint.
- Chạy preflight trước `EXECUTING`, final gate trước commit #2 và `COMPLETED`.

## Output contract

1. 19 file Markdown mới trong `/docs/` (liệt kê trong spec).
2. Commit git cho docs.
3. `verification.md`, `evaluation.md`, `handoff.md` hoàn chỉnh.

## Acceptance criteria

Xem `task.md` (AC-001..AC-009).

## Verification method

- `powershell -NoProfile -ExecutionPolicy Bypass -File .agent/scripts/test-markdown-links.ps1` (exit 0).
- Final quality gates (exit 0).
- `git log` hiển thị commit docs.

## References

- `/AGENTS.md`
- `/docs/00-governance/documentation-standard.md`
- `/docs/README.md`
- `/monster-engine/`
