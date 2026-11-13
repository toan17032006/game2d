# Plan

- Task ID: `TASK-20261114-004`
- Status: `EXECUTING`

## Objective

M4 slice 2: text HUD + tilemap + particles + winmm audio; sandbox tích hợp; docs; commit.

## Phases

| ID | Action | Input | Expected output | Validation | Status |
| --- | --- | --- | --- | --- | --- |
| P0 | Task workspace + preflight | Yêu cầu user | 7 artifact | preflight exit 0 | `DONE` |
| P1 | `monster-render::text` — font 3×5 + draw_text_screen (trait default + Software2DRenderer impl) + tests | D-01 | Text module | tests PASS | `DONE` |
| P2 | `monster-scene::tilemap` spawn helper + tests | D-02 | Tilemap helper | tests PASS | `DONE` |
| P3 | `monster-scene::particles2d` emitter + engine render pass + tests | D-03 | Particles | tests PASS | `DONE` |
| P4 | `monster-platform::winmm_audio` FFI + WinmmAudioBackend + tests (skip-graceful) | D-04 | Audio backend | build link winmm + tests | `DONE` |
| P5 | Sandbox: HUD score + coin burst + tilemap walls + coin blip (winmm/Null) | Tất cả | Demo update | run --game exit 0 | `DONE` |
| P6 | Docs FR-056..059 + traceability + roadmap + fmt + full test + final gates + commit | Toàn bộ | Gates PASS + commit | final exit 0 | `DONE` |

## Dependencies

- P1 → P5; P2 → P5; P3 → P5; P4 → P5; P6 cuối.

## Checkpoints

- CP-1 sau P4: đủ 4 subsystem có tests.
- CP-2 sau P5: sandbox --game exit 0 với HUD/audio/walls.
- CP-3 sau P6: gates PASS + commit.

## Risks

- R-01: Font table sai glyph → unit test assert bitmap dims + một vài glyph mẫu.
- R-02: waveOut device thiếu trong CI → constructor Result, sandbox fallback Null.
- R-03: Particle render pass đụng borrow engine → field-split borrows trong render loop.

## Rollback

- Revert commit; không đụng physics/3D hiện có.

## References

- `/AGENTS.md`
