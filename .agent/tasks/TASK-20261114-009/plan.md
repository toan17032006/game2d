# Plan

- Task ID: `TASK-20261114-009`
- Status: `EXECUTING`

## Objective

Character jump + platform collision + game polish; commit.

## Phases

| ID | Action | Input | Expected output | Validation | Status |
| --- | --- | --- | --- | --- | --- |
| P0 | Task workspace + preflight | Yêu cầu user | 7 artifact | preflight exit 0 | `DONE` |
| P1 | Game improvements: jump + platforms + animation states | D-01/D-02/D-03 | Sandbox game update | game --headless exit 0 | `DONE` |
| P2 | Docs FR-069 + traceability + roadmap | Output | 2 file docs | link check | `DONE` |
| P3 | fmt + full test + final gates + commit | Toàn bộ | Gates PASS + commit | final exit 0 | `DONE` |

## Dependencies

- P1 → P2 → P3.

## Checkpoints

- CP-1 sau P1: game improvements compile + headless PASS.
- CP-2 sau P3: gates PASS + commit.

## Risks

- R-01: Jump height cần tuning → dùng jump_speed 400 (đủ cho 32px tile).

## Rollback

- Revert commit.

## References

- `/AGENTS.md`
