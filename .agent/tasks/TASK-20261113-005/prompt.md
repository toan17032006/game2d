# Prompt

## Role

Agent verification-system implementer.

## Objective

Nâng cấp quality gate để kiểm tra acceptance criteria, evidence, evaluation và verification độc lập.

## Scope

- `.agent/scripts/`
- `.agent/evaluations/`
- `.agent/tasks/_template/`
- `AGENTS.md`
- `/docs/00-governance/agent-execution-standard.md`

## Acceptance criteria

- Acceptance criteria có ID.
- Verification map từng criterion với PASS và evidence.
- Evaluation kiểm tra đủ mọi dimension.
- Validator và verifier tách logic.
- Có regression cases.
- Final gate FAIL khi thiếu evidence hoặc có status giả.

## References

- `/AGENTS.md`
- `/docs/00-governance/agent-execution-standard.md`
