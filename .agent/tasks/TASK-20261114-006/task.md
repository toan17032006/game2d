# Task

- Task ID: `TASK-20261114-006`
- Status: `EXECUTING`
- Owner: `@agent`

## Objective

M4 slice 4 (mini-game hoàn chỉnh: menu/state machine/level/animation switch) + M3 tiếp (mesh normals/OBJ mở rộng, Aabb3); commit.

## Scope

- Sandbox `--game`: GamePhase Menu/Playing/Won, Enter flow, animation switch Idle/Run, level tilemap.
- `monster-core::mesh`: OBJ `vn` parse, `Mesh3D.normals`, glNormal3f emit.
- `monster-physics`: Aabb3 + tests.
- Docs FR-063..065 + traceability + roadmap; commit.

## Non-goals

- Menu art; multiple levels; audio menu music; engine loop 3D.

## Acceptance criteria

| ID | Criterion | Required evidence |
| --- | --- | --- |
| AC-001 | GamePhase state machine: Menu → Playing (Enter) → Won (3 coins) → Playing (Enter replay); headless auto-flow đạt Won ≥ 1 lần | Output run --game |
| AC-002 | Animation switch: Idle clip khi player đứng yên, Run clip khi di chuyển | Code + demo log |
| AC-003 | Level tilemap: tường + ground segments spawn đúng qua spawn_tilemap | Code + walls spawned log |
| AC-004 | OBJ `vn` parse + `Mesh3D.normals` + glNormal3f emit khi mesh có normals | Tests PASS |
| AC-005 | `Aabb3`: intersects/contains/center/half_extents/separation MTV 3 trục | Tests PASS |
| AC-006 | Docs FR-063..065 + traceability + roadmap (tiếng Việt) | Files |
| AC-007 | `cargo test --workspace` exit 0 (≥ 208 tests) + final gates PASS + commit | `logs/` + `git log` |

## References

- `/AGENTS.md`
- `/docs/01-product/roadmap.md`
