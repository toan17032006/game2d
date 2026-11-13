# Task

- Task ID: `TASK-20261114-003`
- Status: `EXECUTING`
- Owner: `@agent`

## Objective

Thực hiện M4 (sprite animation + Tiny Swords demo) và M3 slice 2 (mesh loading/OBJ); commit.

## Scope

- `monster-scene::animation`: AnimationClip (grid-slice), Animator, sprite_animation_system + tests.
- `monster-core`: Mesh3D (cube builder) + OBJ loader + `Renderer3D::draw_mesh` + tests; Gl3DRenderer draw_mesh.
- Sandbox `--game`: player Character body + Warrior_Run animation + camera follow + coins + score; headless auto-walk verify.
- Docs FR-053..055 + traceability + roadmap M4 partial; commit.

## Non-goals

- Skeletal animation; aseprite metadata; shaders/modern GL; audio.

## Acceptance criteria

| ID | Criterion | Required evidence |
| --- | --- | --- |
| AC-001 | `AnimationClip::from_grid` slice đúng offsets; `sprite_animation_system`: advance time, loop wrap, non-loop clamp, pause | Tests PASS |
| AC-002 | `Mesh3D` (cube builder) + OBJ loader thuần (v/f/faces) parse đúng fixture | Tests PASS |
| AC-003 | `Renderer3D::draw_mesh`: mock đếm draw; Gl3DRenderer draw_mesh emit vertices (unbound đếm, bound gọi GL) | Tests PASS |
| AC-004 | Sandbox `--game`: headless auto-walk — player di chuyển, animation frame tiến, coin được collect (score > 0), camera follow giảm khoảng cách | Output lệnh run |
| AC-005 | Demo gốc (`--headless 120`, `--physics`, `--window` compile) không regression | Logs |
| AC-006 | Docs FR-053..055 + traceability + roadmap M4 partial (tiếng Việt) | Files |
| AC-007 | `cargo test --workspace` exit 0 (≥ 178 tests) + final gates PASS + commit | `logs/` + `git log` |

## References

- `/AGENTS.md`
- `/docs/01-product/roadmap.md`
