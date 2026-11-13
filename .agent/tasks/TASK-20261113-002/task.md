# Task

- Task ID: `TASK-20261113-002`
- Status: `COMPLETED`
- Owner: `@agent`

## Objective

Chuẩn hóa toàn bộ Markdown.

## Scope

Mọi file `*.md` hiện có trong repository.

## Non-goals

- Không đổi cấu trúc code.
- Không thêm nội dung nghiệp vụ chưa được xác nhận.

## Acceptance criteria

- Điều lệnh rõ.
- Nội dung ngắn.
- Có References.
- Có validation.

## Acceptance IDs

| ID | Criterion | Required evidence |
| --- | --- | --- |
| AC-001 | Markdown uses hard rules | Markdown inventory |
| AC-002 | Markdown is concise | Documentation scan |
| AC-003 | References exist | References scan |
| AC-004 | Validation exists | `git diff --check` |

## References

- `/AGENTS.md`
- `/docs/README.md`
