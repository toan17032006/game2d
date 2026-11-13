# Agent Task Template

## Directory

```text
/.agent/tasks/TASK-YYYYMMDD-NNN/
├── prompt.md
├── task.md
├── spec.md
├── plan.md
├── handoff.md
├── evaluation.md
└── verification.md
```

## File contract

| File | MUST contain |
| --- | --- |
| `prompt.md` | Prompt đã chuẩn hóa theo schema bắt buộc |
| `task.md` | Objective, scope, non-goals, `AC-NNN` acceptance criteria |
| `spec.md` | Inputs, constraints, decisions, output contract |
| `plan.md` | Phases, dependencies, checkpoints, risks, rollback |
| `handoff.md` | Current state, last completed, next action, blockers, evidence |
| `evaluation.md` | Evaluation dimensions, score, result, evidence |
| `verification.md` | `AC-NNN` coverage, commands, results, evidence, limitations |

## References

- `/docs/00-governance/agent-execution-standard.md`
- `/AGENTS.md`
