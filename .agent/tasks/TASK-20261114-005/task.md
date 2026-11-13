# Task

- Task ID: `TASK-20261114-005`
- Status: `EXECUTING`
- Owner: `@agent`

## Objective

M4 slice 3 (sprite culling + tilemap multi-layer + music WAV looped) + M3 slice 3 (extension loader + shader pipeline với fallback fixed-function); commit.

## Scope

- `monster-scene`: `sprite_in_view` + tilemap multi-layer test; engine render culling.
- `monster-audio`: `load_wav_file`.
- `monster-platform::gl`: dynamic extension loader + shader FFI + compile/link helpers.
- `monster-engine::gl3d`: shader path (bound + functions available) / fixed-function fallback.
- Docs FR-060..062 + traceability + roadmap; commit.

## Non-goals

- True streaming (callback thread); core profile VAO/VBO; geometry shaders.

## Acceptance criteria

| ID | Criterion | Required evidence |
| --- | --- | --- |
| AC-001 | Sprite culling: sprite ngoài viewport camera bị skip trong render (pure fn + engine draw_count test) | Tests PASS |
| AC-002 | Tilemap multi-layer: 2 layer cùng texture — counts + positions đúng, không đè nhau | Test PASS |
| AC-003 | Music WAV: `load_wav_file` decode file thành AudioClip; winmm looped playback state (skip-graceful) | Tests PASS |
| AC-004 | Extension loader: function pointer table (null khi không khả dụng); shader compile/link helpers với info-log errors; Gl3DRenderer shader path + fallback | Tests PASS (headless state) |
| AC-005 | Docs FR-060..062 + traceability + roadmap M3/M4 partial cập nhật (tiếng Việt) | Files |
| AC-006 | Regression: sandbox game/physics/classic exit 0; `cargo test --workspace` exit 0 (≥ 200 tests) | `logs/` |
| AC-007 | Final gates PASS + commit git | `logs/final-gate.log` + `git log` |

## References

- `/AGENTS.md`
- `/docs/01-product/roadmap.md`
