# Plan

- Task ID: `TASK-20261114-006`
- Status: `EXECUTING`

## Objective

Tách game khỏi engine; commit.

## Phases

| ID | Action | Input | Expected output | Validation | Status |
| --- | --- | --- | --- | --- | --- |
| P0 | Task workspace + preflight | Yêu cầu user | 7 artifact | preflight exit 0 | `DONE` |
| P1 | Tạo `/game/` crate (Cargo.toml + main.rs từ sandbox.rs) | D-01 | Game crate | `cargo build` từ /game/ exit 0 | `DONE` |
| P2 | Xóa sandbox khỏi engine facade; dọn exports | D-03/D-04 | Engine sạch game code | `cargo test --workspace` exit 0 | `DONE` |
| P3 | Game chạy được 3 mode + bridge tests chuyển | D-01 | Game run log | `cargo run -- --headless 120 --game` exit 0 | `DONE` |
| P4 | Docs FR-066 + traceability + system-context update | Output | 3 file docs | link check | `DONE` |
| P5 | fmt + full test + final gates + commit | Toàn bộ | Gates PASS + commit | final exit 0 | `DONE` |

## Dependencies

- P1 → P2 → P3 → P4 → P5.

## Checkpoints

- CP-1 sau P2: engine sạch game code.
- CP-2 sau P4: docs cập nhật.
- CP-3 sau P5: gates PASS + commit.

## Risks

- R-01: Cargo workspace boundary — game ngoài workspace cần path deps đúng → test build từ /game/.

## Rollback

- Revert commit; game code vẫn có trong git history.

## References

- `/AGENTS.md`
