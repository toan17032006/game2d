# Task

- Task ID: `TASK-20261114-004`
- Status: `EXECUTING`
- Owner: `@agent`

## Objective

M4 slice 2: text HUD (bitmap font), tilemap spawn, particle emitter 2D, audio playback thật (winmm FFI); tích hợp vào sandbox; commit.

## Scope

- `monster-render::text` (font 3×5 + draw_text_screen) + `Renderer::draw_text_screen` default no-op.
- `monster-scene::{tilemap, particles2d}` + engine render pass cho particles.
- `monster-platform::winmm_audio` (FFI + WinmmAudioBackend implement AudioBackend).
- Sandbox: HUD score, coin spark burst, tilemap tường, coin blip (winmm/Null fallback).
- Docs FR-056..059 + traceability + roadmap M4 partial; commit.

## Non-goals

- Modern GL/shaders (defer); TTF; music streaming; positional audio; particle-collision.

## Acceptance criteria

| ID | Criterion | Required evidence |
| --- | --- | --- |
| AC-001 | Blocky font 3×5: glyphs 0-9/A-Z/punctuation; `draw_text_screen` vẽ pixel đúng vị trí + scale; ký tự lạ → space | Tests PASS |
| AC-002 | `spawn_tilemap`: entity/body counts đúng theo grid, solid tile có static body, Empty bỏ qua | Tests PASS |
| AC-003 | `ParticleEmitter2D`: burst spawn đúng số, particles di chuyển theo velocity, lifetime hết → chết, color lerp | Tests PASS |
| AC-004 | `WinmmAudioBackend` implement `AudioBackend` (play/stop/stop_all/active_count); winmm FFI link thành công; không có device → constructor Err → fallback Null | Tests PASS (skip-graceful khi không device) |
| AC-005 | Sandbox: HUD score hiển thị, coin burst particles, tilemap tường chặn player, coin blip audio (winmm hoặc Null) | Output run --game |
| AC-006 | Regression: game/physics/classic modes exit 0; `cargo test --workspace` exit 0 (≥ 195 tests) | `logs/` |
| AC-007 | Docs FR-056..059 + traceability + roadmap (tiếng Việt) | Files |
| AC-008 | Final gates PASS + commit git | `logs/final-gate.log` + `git log` |

## References

- `/AGENTS.md`
- `/docs/01-product/roadmap.md`
