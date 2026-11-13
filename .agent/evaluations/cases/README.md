# Regression Cases

- Case IDs MUST use `CASE-NNN`.
- Each case MUST define input, expected gate and expected result.
- Each case MUST cover one failure mode.

## Cases

- `CASE-001`: Missing required task file → `FAIL`.
- `CASE-002`: Missing acceptance evidence → `FAIL`.
- `CASE-003`: Incomplete evaluation score → `FAIL`.
- `CASE-004`: Verification status without command evidence → `FAIL`.
- `CASE-005`: Complete task with all evidence → `PASS`.
- `CASE-006`: Unauthorized edit to project documentation outside task workspace → `FAIL`.
- `CASE-007`: Broken or unresolved markdown link → `FAIL`.

## Execution

- Run automated suite: `powershell -NoProfile -ExecutionPolicy Bypass -File .agent/scripts/run-regression-tests.ps1`

## References

- `/.agent/scripts/run-regression-tests.ps1`
- `/.agent/scripts/verify-task.ps1`
- `/.agent/scripts/check-doc-boundaries.ps1`
- `/.agent/scripts/test-markdown-links.ps1`
- `/.agent/evaluations/README.md`
