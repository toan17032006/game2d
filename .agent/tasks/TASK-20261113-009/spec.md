# Specification

## Inputs

- Yêu cầu trực tiếp của người dùng trong phiên: "Commit đi và viết tài liệu đầy đủ vào trong docs - tiếng việt nhá".
- Nghiên cứu + ADR draft từ TASK-20261113-008 (`/.agent/tasks/TASK-20261113-008/research/`).
- Code `monster-engine/` (nguồn sự thật): 9 crate, trait contracts, engine loop, 105 tests.

## Confirmed facts

- F-01: Commit engine đã vào main tại `e1aff19` (62 files).
- F-02: Các README section của `/docs/` liệt kê sẵn tên Documents — tạo đúng tên để index thành thật.
- F-03: `documentation-standard.md` yêu cầu metadata Owner/Status/Last updated/Review date, `## References`, kebab-case, UTF-8.
- F-04: ADR format theo `ADR-000` + `adr-template.md`: Context/Decision/Alternatives/Consequences/References.
- F-05: Người dùng yêu cầu trực tiếp → phê duyệt sửa Markdown ngoài task workspace theo `AGENTS.md`.

## Inferences

- I-1 (inference): "tài liệu đầy đủ" = đủ các file mà README đã liệt kê cho phần áp dụng được của engine (product/requirements/architecture/engineering/quality/releases); 07-operations bỏ qua vì thư viện chưa có service.
- I-2 (inference): Status `Approved`/`Accepted` hợp lệ vì tài liệu mô tả code đã được kiểm chứng (105 tests + final gates PASS) và người dùng yêu cầu trực tiếp.

## Unknowns

- U-1: Không biết git remote — commit cục bộ trên main; push KHÔNG được yêu cầu nên không thực hiện.

## Constraints

- C-01: Chỉ tạo file liệt kê trong `## Approved Documentation Changes` (check-doc-boundaries sẽ kiểm tra).
- C-02: Tiếng Việt, thuật ngữ kỹ thuật tiếng Anh giữ nguyên chuẩn ngành.
- C-03: Lệnh trong docs phải khớp lệnh thật đã chạy trong TASK-20261113-008.
- C-04: Không secrets.

## Decisions

- D-01: Test Command của task này = markdown link checker (phù hợp với deliverable tài liệu).
- D-02: 2 commit: #1 engine (đã xong `e1aff19`), #2 docs + TASK-009 workspace.
- D-03: ADR-001 status `Accepted` (người dùng phê duyệt chuyển bằng yêu cầu trực tiếp).
- D-04: Traceability matrix dùng bảng FR → crate/module → test function (đặt tên test thật trong `logs/test.log` của TASK-008).

## Output contract

- 19 file Markdown mới trong `/docs/` + commit #2 + artifacts task đầy đủ.

## Test Command

`powershell -NoProfile -ExecutionPolicy Bypass -File .agent/scripts/test-markdown-links.ps1`

## Approved Documentation Changes

- `/docs/03-architecture/adr/ADR-001-monster-engine-foundation.md` — chuyển ADR-001 draft thành Accepted (phê duyệt trực tiếp của người dùng)
- `/docs/03-architecture/system-context.md` — ngữ cảnh hệ thống
- `/docs/03-architecture/container-and-component.md` — cấu trúc crate + traits + engine loop
- `/docs/03-architecture/data-architecture.md` — bố trí dữ liệu (RGBA8, asset, component store)
- `/docs/03-architecture/integration-architecture.md` — biên Win32 FFI + cơ chế backend
- `/docs/01-product/vision-and-goals.md`
- `/docs/01-product/scope-and-non-goals.md`
- `/docs/01-product/roadmap.md`
- `/docs/01-product/personas-and-use-cases.md`
- `/docs/02-requirements/functional-requirements.md`
- `/docs/02-requirements/non-functional-requirements.md`
- `/docs/02-requirements/traceability-matrix.md`
- `/docs/04-engineering/local-development.md`
- `/docs/04-engineering/development-workflow.md`
- `/docs/04-engineering/coding-standards.md`
- `/docs/04-engineering/api-guidelines.md`
- `/docs/05-quality/quality-strategy.md`
- `/docs/08-releases/release-process.md`
- `/docs/08-releases/release-notes/release-0.1.0.md`

## References

- `/AGENTS.md`
- `/docs/00-governance/documentation-standard.md`
- `/docs/00-governance/change-management.md`
- `/.agent/scripts/check-doc-boundaries.ps1`
