# Plan

- Task ID: `TASK-20261114-003`
- Status: `EXECUTING`

## Objective

M4 sprite animation + Tiny Swords demo + M3 mesh loading + commit.

## Phases

| ID | Action | Input | Expected output | Validation | Status |
| --- | --- | --- | --- | --- | --- |
| P0 | Task workspace + preflight | Yêu cầu user | 7 artifact | preflight exit 0 | `DONE` |
| P1 | `animation.rs`: AnimationClip/Animator/from_grid/sprite_animation_system + tests | D-01/D-02 | Module + tests | `cargo test -p monster-scene` exit 0 | `DONE` |
| P2 | Mesh3D + OBJ loader + draw_mesh (core trait + Gl3DRenderer + mock) + tests | D-03/D-04 | Mesh API + tests | tests PASS | `DONE` |
| P3 | Sandbox `--game` mode (player + camera follow + animation + coins + score, headless verify) | D-05 | Demo chạy exit 0 | run log assert | `DONE` |
| P4 | Docs FR-053..055 + traceability + roadmap M4 partial | Output | 3 file docs | link check | `DONE` |
| P5 | fmt + full test + final gates + commit | Toàn bộ | Gates PASS + commit | final exit 0 | `DONE` |

## Dependencies

- P1, P2 độc lập; P3 cần P1 (animation) + bridge có sẵn; P5 cuối.

## Checkpoints

- CP-1 sau P2: animation + mesh có tests.
- CP-2 sau P3: demo game chạy.
- CP-3 sau P5: gates PASS + commit.

## Risks

- R-01: OBJ loader edge cases (quad faces, âm index) → fixture test phủ; lỗi parse → Err rõ ràng.
- R-02: Sandbox assert phụ thuộc solver → ngưỡng lỏng.

## Rollback

- Revert commit; code nằm trong scene/core/engine crates.

## References

- `/AGENTS.md`
