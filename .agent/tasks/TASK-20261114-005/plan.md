# Plan

- Task ID: `TASK-20261114-005`
- Status: `EXECUTING`

## Objective

M4 slice 3 + M3 slice 3: culling, multi-layer, music, shader pipeline; commit.

## Phases

| ID | Action | Input | Expected output | Validation | Status |
| --- | --- | --- | --- | --- | --- |
| P0 | Task workspace + preflight | Yêu cầu user | 7 artifact | preflight exit 0 | `DONE` |
| P1 | M4: `sprite_in_view` culling + engine integration + tilemap multi-layer test + `load_wav_file` | D-01/D-02/D-03 | Culling + tests | tests PASS | `DONE` |
| P2 | M3: extension loader + shader compile/link + Gl3DRenderer shader path + fallback + tests | D-04 | Shader pipeline | tests PASS (headless state) | `DONE` |
| P3 | Docs FR-060..062 + traceability + roadmap | Output | 3 file docs | link check | `DONE` |
| P4 | fmt + full test + regression + final gates + commit | Toàn bộ | Gates PASS + commit | final exit 0 | `DONE` |

## Dependencies

- P1, P2 độc lập; P3/P4 cuối.

## Checkpoints

- CP-1 sau P2: culling + shader pipeline tests PASS.
- CP-2 sau P4: gates PASS + commit.

## Risks

- R-01: Culling sai làm mất sprite → margin + test biên (sprite nửa trong/ngoài).
- R-02: Extension pointer NULL gọi crash → mọi call guard null.

## Rollback

- Revert commit.

## References

- `/AGENTS.md`
