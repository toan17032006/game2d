# Task

- Task ID: `TASK-20261114-009`
- Status: `EXECUTING`
- Owner: `@agent`

## Objective

Character jump + tilemap collision + game polish; commit.

## Scope

- Sandbox game: jump input (Space/W/Up), platform collision qua move_and_slide depenetration, level design với platforms.
- Docs FR-069 + traceability; commit.

## Non-goals

- Không đổi engine core.

## Acceptance criteria

| ID | Criterion | Required evidence |
| --- | --- | --- |
| AC-001 | Character jump: Space/W/Up → velocity.y set, gravity kéo xuống, platform chặn | Tests PASS |
| AC-002 | Tilemap platforms: Solid tiles chặn Character qua move_and_slide | Tests PASS |
| AC-003 | Game demo: headless exit 0 với jump + platform collision hoạt động | Run log |
| AC-004 | Regression: engine tests 205 PASS | `logs/test.log` |
| AC-005 | Docs FR-069 + traceability + commit | Files + `git log` |

## References

- `/AGENTS.md`
- `/docs/01-product/roadmap.md`
