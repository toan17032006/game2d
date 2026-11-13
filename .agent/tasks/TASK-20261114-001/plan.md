# Plan

- Task ID: `TASK-20261114-001`
- Status: `EXECUTING`

## Objective

Bridge ECS ↔ physics + sandbox demo + docs + commit.

## Phases

| ID | Action | Input | Expected output | Validation | Status |
| --- | --- | --- | --- | --- | --- |
| P0 | Task workspace + preflight | Yêu cầu user | 7 artifact | preflight exit 0 | `DONE` |
| P1 | `physics_bridge.rs` + Engine with_physics + UpdateContext.physics + facade tests | spec D-01/D-02 | Bridge + integration | `cargo test -p monster-engine` exit 0 | `DONE` |
| P2 | Sandbox `--physics` mode | Bridge | Sandbox headless exit 0, bóng đọng | `cargo run --example sandbox -- --headless 120 --physics` exit 0 | `DONE` |
| P3 | Docs FR-049 + traceability | D-04 | 2 file docs sửa | link check | `DONE` |
| P4 | Final gates + commit + handoff/verification/evaluation | Toàn bộ | Gates PASS + commit | final exit 0 | `DONE` |

## Dependencies

- P1 → P2 → P3 → P4.

## Checkpoints

- CP-1 sau P2: bridge + demo chạy.
- CP-2 sau P4: gates PASS + commit.

## Risks

- R-01: Borrow conflicts khi truyền physics vào UpdateContext → Option<&mut> trong context struct, field-split borrows.
- R-02: Sandbox assert phụ thuộc solver → dùng ngưỡng lỏng (bóng đọng trong dải y rộng).

## Rollback

- Revert commit; code nằm trong facade crate.

## References

- `/AGENTS.md`
