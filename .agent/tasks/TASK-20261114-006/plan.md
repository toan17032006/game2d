# Plan

- Task ID: `TASK-20261114-006`
- Status: `EXECUTING`

## Objective

M4 slice 4 (mini-game hoàn chỉnh) + M3 tiếp (mesh normals + Aabb3); commit.

## Phases

| ID | Action | Input | Expected output | Validation | Status |
| --- | --- | --- | --- | --- | --- |
| P0 | Task workspace + preflight | Yêu cầu user | 7 artifact | preflight exit 0 | `DONE` |
| P1 | Sandbox GamePhase state machine + animation switch + level tilemap | D-01/D-02/D-03 | Demo state flow | run --game exit 0 (won ≥ 1) | `DONE` |
| P2 | Mesh normals (OBJ vn + f v//vn) + glNormal3f + Aabb3 + tests | D-04/D-05 | Mesh/Aabb3 + tests | tests PASS | `DONE` |
| P3 | Docs FR-063..065 + traceability + roadmap | Output | 3 file docs | link check | `DONE` |
| P4 | fmt + full test + regression + final gates + commit | Toàn bộ | Gates PASS + commit | final exit 0 | `DONE` |

## Dependencies

- P1, P2 độc lập; P3/P4 cuối.

## Checkpoints

- CP-1 sau P2: state machine + mesh/Aabb3 tests PASS.
- CP-2 sau P4: gates PASS + commit.

## Risks

- R-01: State machine headless timing → step counters rõ ràng (menu 30 bước).
- R-02: OBJ vn parse format biến thể → fixture test phủ v//vn và v/vt/vn bỏ qua.

## Rollback

- Revert commit.

## References

- `/AGENTS.md`
