# Prompt

## Input classification

`DIRECT` — người dùng yêu cầu trực tiếp trong phiên: "Commit và thực hiện đề xuất" — đề xuất cuối TASK-20261114-003: M4 slice 2 (tilemap + particle + text HUD + audio playback thật waveOut FFI). M3 slice 3 (modern GL + shaders) defer có ghi rõ.

## Role

Rust engine developer + audio programmer.

## Objective

1. **Text HUD**: blocky bitmap font 3×5 (digits + A-Z + punctuation gọn) + `draw_text_screen` trên Software2DRenderer.
2. **Tilemap**: `spawn_tilemap` — grid Tile {Empty, Solid} → sprite entities + static bodies (physics bridge) cho tile solid.
3. **Particles 2D**: `ParticleEmitter2D` component + system (burst, lifetime, color lerp) + engine render pass vẽ particle (draw_rect, không entity churn).
4. **Audio playback thật**: winmm FFI (`waveOut*`) + `WinmmAudioBackend` implement `AudioBackend` (PCM mono playback, stop/reset) — dùng trong window mode; headless giữ Null.
5. Sandbox: HUD "SCORE n" + particle burst khi ăn coin + tilemap tường + coin blip sound.
6. Commit.

## Context

- 184 tests PASS trên `40cb1a8`; physics bridge + game slice đã có.
- FFI rule: gom vào monster-platform — winmm FFI đặt `platform::winmm_audio`; backend implement `monster_core::audio::AudioBackend` (platform phụ thuộc core ✓).

## Inputs

- Yêu cầu trực tiếp người dùng trong phiên.
- `/docs/01-product/roadmap.md` (M4).

## Scope

- `monster-render::text` (font + draw_text trên framebuffer) + `Renderer::draw_text_screen` default no-op + Software2DRenderer impl.
- `monster-scene::tilemap` (spawn helper) + `monster-scene::particles2d` (emitter + system).
- `monster-platform::winmm_audio` (FFI + WinmmAudioBackend).
- Sandbox HUD/sparks/walls/audio; docs FR-056..059 + traceability + roadmap; commit.

## Non-goals

- Modern GL/shaders (M3 slice 3 defer); TTF font rendering; music streaming; positional audio.

## Constraints

- Zero external crate; giữ 184 tests cũ PASS; FFI gom về platform; audio device thiếu → backend fallback Null (không crash).

## Execution rules

- Preflight trước EXECUTING; final gates trước COMPLETED; commit sau gates.

## Output contract

- text.rs + tilemap.rs + particles2d.rs (scene/render) + winmm_audio.rs (platform) + sandbox updates + tests + docs FR-056..059 + commit.

## Acceptance criteria

Xem `task.md` (AC-001..AC-008).

## Verification method

- `cargo test --workspace` exit 0; sandbox các mode exit 0; final gates exit 0.

## References

- `/AGENTS.md`
- `/docs/01-product/roadmap.md`
- waveOut API — Microsoft Learn: <https://learn.microsoft.com/en-us/windows/win32/api/mmeapi/> (truy cập 2026-11-14)
