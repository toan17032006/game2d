# Agent Evaluations

## Dimensions

- `input-evaluation.md`
- `plan-evaluation.md`
- `output-evaluation.md`
- `adversarial-evaluation.md`
- `handoff-evaluation.md`

## Rules

- MUST evaluate before `COMPLETED`.
- MUST score each dimension from `0` to `5`.
- MUST record evidence for every score.
- MUST mark `FAIL` when evidence is missing.
- MUST not replace evidence with confidence or explanation.

## References

- `/.agent/hooks/README.md`
- `/docs/05-quality/README.md`
