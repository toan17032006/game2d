# Verification

- Task ID: `TASK-20261114-004`
- Status: `COMPLETED`

## Acceptance coverage

| ID | Result | Evidence |
| --- | --- | --- |
| AC-001 | PASS | `text::tests::{glyphs_are_three_wide_five_tall, text_metrics, draw_text_writes_pixels, draw_text_respects_scale_and_advances}` PASS — pixel trong glyph box, scale + advance đúng, ký tự lạ → space |
| AC-002 | PASS | `tilemap::tests::{spawns_solids_skips_empty, all_empty_spawns_nothing}` PASS — 2/4 solid → 2 entity + 2 body, all-empty → 0; sandbox --game spawn 11 walls |
| AC-003 | PASS | `particles2d::tests::{burst_spawns_and_particles_move_and_die, continuous_rate_spawns_over_time, color_lerps_over_lifetime}` PASS — burst 8 → chết hết sau lifetime, continuous > 5 hạt, color lerp midpoint xám; engine render pass vẽ draw_rect |
| AC-004 | PASS | `winmm_audio::tests::{construction_reports_device_presence, play_stop_state_machine_with_or_without_device, waveformatex_layout}` PASS; build link winmm thành công (`logs/build.log`); skip-graceful khi không device |
| AC-005 | PASS | `logs/sandbox-game.log`: exit 0 — HUD score vẽ (draw_text_screen), 11 tilemap walls, coin spark burst, coin blip qua winmm/Null; player đi 700px |
| AC-006 | PASS | `logs/test.log`: 196 passed / 0 failed exit 0; regression `logs/sandbox-physics.log` + `logs/sandbox-game.log` exit 0 |
| AC-007 | PASS | FR-056..059 + traceability + roadmap M4 partial (tiếng Việt); markdown links PASS |
| AC-008 | PASS | `logs/final-gate.log`: final gates PASS (exit 0); commit `logs/commit-log.log` |

## Commands

| Command / method | Result | Evidence | Acceptance coverage | Limitations |
| --- | --- | --- | --- | --- |
| `cargo test --workspace --manifest-path monster-engine/Cargo.toml` | PASS | `logs/test.log` (196 passed / 0 failed) | AC-001..AC-004, AC-006 | Không có |
| `cargo run --manifest-path monster-engine/Cargo.toml --example sandbox -- --headless 120 --game` | PASS (exit 0) | `logs/sandbox-game.log` | AC-005 | Audio nghe thật cần loa — verify thủ công |
| `cargo run --manifest-path monster-engine/Cargo.toml --example sandbox -- --headless 120 --physics` | PASS (exit 0) | `logs/sandbox-physics.log` | AC-006 | Không có |
| `powershell -NoProfile -ExecutionPolicy Bypass -File .agent/scripts/test-markdown-links.ps1` | PASS | terminal output | AC-007 | Không có |
| `powershell -NoProfile -ExecutionPolicy Bypass -File .agent/scripts/run-quality-gates.ps1 -TaskId TASK-20261114-004 -Mode final` | PASS | `logs/final-gate.log` | AC-008 | Không có |

## Coverage

- Font: 4 test (dims, metrics, pixel placement, scale/advance).
- Tilemap: 2 test (counts, positions) + demo 11 walls trong sandbox.
- Particles: 3 test (burst/motion/death, continuous, color lerp) + engine render pass.
- winmm: 3 test (device presence, state machine skip-graceful, WAVEFORMATEX layout) + build-link.
- Regression: game/physics/classic 3 modes exit 0.

## Limitations

- winmm: single-buffer per playback, cleanup lazy (reset + unprepare) — không streaming music.
- Text HUD: bitmap 3×5 uppercase — không TTF/Kerning.
- Particles không collide — decorative.
- Audio nghe thật cần loa — verify thủ công bởi người dùng.

## Final status

`COMPLETED`

## References

- `/AGENTS.md`
- `/docs/02-requirements/functional-requirements.md` (FR-056..059)
- `/monster-engine/crates/monster-platform/src/winmm_audio.rs` (product)
