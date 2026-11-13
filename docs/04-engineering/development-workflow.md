# Development Workflow — monster-engine

- Owner: Engineering
- Status: `Approved`
- Last updated: `2026-11-13`
- Review date: `2027-05-13`

## Purpose

Quy trình làm việc chuẩn cho mọi thay đổi code/tài liệu trong repo — đã được thực hành qua TASK-20261113-008/009.

## Vòng đời một task (theo `/AGENTS.md`)

```
1. Đọc /AGENTS.md
2. Tạo task workspace /.agent/tasks/TASK-YYYYMMDD-NNN/
   (7 artifact: prompt, task, spec, plan, handoff, verification, evaluation)
3. Chạy preflight gate  → PASS mới được EXECUTING
4. Thực thi theo plan.md, cập nhật handoff sau mỗi checkpoint
5. VERIFYING: chạy lệnh kiểm tra, ghi evidence vào logs/
6. Final gate           → PASS mới được COMPLETED
7. Commit
```

## Trạng thái task

`DIRECT` (trả lời, không đụng file) · `CLARIFYING` (thiếu input) · `AWAITING_APPROVAL` · `EXECUTING` · `VERIFYING` · `COMPLETED` · `BLOCKED`

## Quy tắc Git

- Commit khi có yêu cầu của người dùng; không push nếu không được yêu cầu.
- Commit message theo kiểu conventional (`feat:`, `docs:`, `fix:`) như lịch sử repo.
- Một commit = một khối công việc hợp lý (ví dụ: engine foundation, tài liệu).
- Task workspace tương ứng được commit cùng phần code của nó (truy vết được).
- `.gitignore` loại `target/`, `*.log` — không commit build artifacts; evidence `.json` trong logs được commit.

## Quy tắc tài liệu

- Markdown ngoài task workspace là read-only trừ khi người dùng yêu cầu trực tiếp trong phiên, HOẶC được liệt kê trong `## Approved Documentation Changes` của `spec.md`.
- Doc mới tuân `/docs/00-governance/documentation-standard.md` (metadata, kebab-case, References).
- Kiến trúc thay đổi → ADR mới trong `/docs/03-architecture/adr/`.

## Quy tắc code

- Thay đổi contract (trait/struct public) → cập nhật docs cùng commit.
- Mỗi crate giữ tests trong `#[cfg(test)]` cùng file với code được test; test tích hợp đặt `tests/`.
- Trước khi báo xong: `cargo fmt` + `cargo test --workspace` + gates.

## Checklist trước commit

```
[ ] cargo fmt đã chạy
[ ] cargo test --workspace PASS (exit 0)
[ ] preflight + final gates PASS cho task hiện tại
[ ] handoff.md/verification.md cập nhật, evidence trong logs/
[ ] Không commit: target/, *.log, secrets
[ ] Docs liên quan đã cập nhật (nếu đổi contract/kiến trúc)
```

## Ví dụ thực tế (đã thực hành)

- TASK-20261113-008: nền móng engine — commit `e1aff19` (62 files, 105 tests PASS, final gates PASS).
- TASK-20261113-009: tài liệu tiếng Việt + ADR-001 — commit `docs:` (file này thuộc commit đó).

## References

- `/AGENTS.md`
- `/docs/00-governance/agent-execution-standard.md`
- `/docs/00-governance/change-management.md`
- `/docs/04-engineering/local-development.md`
- `/docs/04-engineering/coding-standards.md`
