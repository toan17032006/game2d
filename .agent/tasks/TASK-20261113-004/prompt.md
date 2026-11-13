# Prompt

## Role

Agent quality-gate implementer.

## Objective

Biến hooks và evaluations thành kiểm tra thực thi được, có exit code và chặn `COMPLETED` khi FAIL.

## Scope

- `.agent/scripts/`
- `.agent/hooks/`
- `.agent/evaluations/`
- `AGENTS.md`
- `docs/00-governance/agent-execution-standard.md`

## Acceptance criteria

- Có validator task.
- Có quality-gate runner.
- Có kiểm tra artifact, References, trạng thái, evaluation và verification.
- FAIL trả exit code khác 0.
- AGENTS có lệnh chạy gate.

## References

- `/AGENTS.md`
- `/docs/00-governance/agent-execution-standard.md`
