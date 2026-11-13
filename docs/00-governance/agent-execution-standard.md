# Agent Execution Standard

- Owner: Engineering Enablement
- Status: `Approved`
- Review: `2027-05-13`

## Commands

- Agent MUST assign one state to every task.
- Agent MUST create one Task ID per task.
- Agent MUST use `/.agent/tasks/<TASK-ID>/`.
- Agent MUST keep task artifacts inside the current task workspace.
- Agent MUST create `prompt.md`, `task.md`, `spec.md`, `plan.md`, `handoff.md` and `verification.md`.
- Agent MUST create `evaluation.md`.
- Agent MUST read `handoff.md` before continuing an existing task.
- Agent MUST start from `Next action`.
- Agent MUST update `handoff.md` after every checkpoint.
- Agent MUST update `verification.md` before completion.
- Agent MUST run `/.agent/hooks/` in order.
- Agent MUST run `/.agent/evaluations/` before `COMPLETED`.
- Agent MUST record PASS/FAIL and evidence.
- Agent MUST run `powershell -NoProfile -ExecutionPolicy Bypass -File .agent/scripts/run-quality-gates.ps1 -TaskId <TASK-ID> -Mode preflight` before `EXECUTING`.
- Agent MUST run `powershell -NoProfile -ExecutionPolicy Bypass -File .agent/scripts/run-quality-gates.ps1 -TaskId <TASK-ID> -Mode final` before `COMPLETED`.
- Agent MUST run independent verifier in final gate.
- Gate failure MUST return a non-zero exit code.
- Agent MUST NOT report `COMPLETED` without evidence.

## States

| State | Rule |
| --- | --- |
| `DIRECT` | Trả lời; không tạo hoặc sửa file. |
| `CLARIFYING` | Thiếu input bắt buộc; hỏi và dừng. |
| `AWAITING_APPROVAL` | Đã đủ input; chờ approval theo yêu cầu task. |
| `EXECUTING` | Thực hiện `plan.md`. |
| `VERIFYING` | Chạy validation; ghi evidence. |
| `COMPLETED` | Acceptance criteria đạt. |
| `BLOCKED` | Thiếu quyền, tool hoặc input; ghi blocker và dừng. |

## Task files

| File | MUST contain |
| --- | --- |
| `prompt.md` | Role, Objective, Context, Inputs, Scope, Non-goals, Constraints, Output, Acceptance, Verification, References |
| `task.md` | Task ID, Status, Objective, Scope, Non-goals, `AC-NNN` acceptance criteria, References |
| `spec.md` | Inputs, Confirmed facts, Inferences, Unknowns, Constraints, Decisions, Output contract, References |
| `plan.md` | Status, Objective, Phases, Dependencies, Checkpoints, Risks, Rollback, References |
| `handoff.md` | Status, Current phase, Last completed, Next action, Decisions, Changed files, Validation, Blockers, Handoff note, References |
| `verification.md` | `AC-NNN` coverage, Command, Result, Evidence, Coverage, Limitations, Final status, References |
| `evaluation.md` | Dimension, Score, Result, Evidence, References |

## Prompt rules

- Agent MUST retain confirmed facts.
- Agent MUST label inferences.
- Agent MUST list unknowns.
- Agent MUST remove greetings, repetition, examples outside scope and unconfirmed assumptions.
- Agent MUST NOT convert inference or unknown into confirmed fact.
- Agent MUST NOT change objective, scope or acceptance criteria without approval.

## Execution rules

- Agent MUST execute phases in dependency order.
- Agent MUST update phase status.
- Agent MUST NOT leave `TODO`, placeholder or pseudo-code in completed output.
- Agent MUST preserve approved decisions during delta updates.
- Agent MUST switch to `CLARIFYING` when required input is missing.
- Agent MUST switch to `BLOCKED` when execution or verification is impossible.
- Agent MUST distinguish `EXECUTED`, `VERIFIED` and `DRY-RUN`.
- Agent MUST pass preflight, plan, execution, output and handoff gates.

## Approval rules

- Agent MUST treat `OK`, `Duyệt`, `Làm đi` or equivalent as approval only when it refers to the current plan.
- Agent MUST NOT treat quoted, example or unrelated approval words as approval.

## References

- `/AGENTS.md`
- `/docs/README.md`
- `/docs/00-governance/documentation-standard.md`
- `/docs/09-reference/templates/agent-task-template.md`
