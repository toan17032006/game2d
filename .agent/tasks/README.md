# Agent Task Workspace

- Owner: Engineering Enablement
- Status: `Approved`
- Review: `2027-05-13`

## Rules

- Mỗi task MUST có thư mục `TASK-YYYYMMDD-NNN`.
- Mỗi task MUST có `prompt.md`, `task.md`, `spec.md`, `plan.md`, `handoff.md`, `verification.md` và `evaluation.md`.
- Agent MUST dùng cùng task workspace đến `COMPLETED` hoặc `BLOCKED`.
- Agent MUST bắt đầu từ `Next action` trong `handoff.md`.
- Agent MUST chạy hooks và evaluations trước `COMPLETED`.

## References

- `/AGENTS.md`
- `/docs/00-governance/agent-execution-standard.md`
- `/docs/09-reference/templates/agent-task-template.md`
