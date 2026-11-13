# Task

- Task ID: `TASK-20261114-008`
- Status: `EXECUTING`
- Owner: `@agent`

## Objective

Engine 3D loop integration + music WAV loading; commit.

## Scope

- `GameApp::render_3d` default no-op + `EngineBuilder::with_3d` + `RenderContext3D`.
- `monster-audio::load_wav_file` đã có — wire vào game mode music playback.
- Docs FR-067..068 + traceability + commit.

## Non-goals

- Physics 3D step trong Engine loop; skeletal animation; modern GL.

## Acceptance criteria

| ID | Criterion | Required evidence |
| --- | --- | --- |
| AC-001 | `GameApp::render_3d` default no-op — game không dùng 3D không bị break | Engine tests PASS |
| AC-002 | `EngineBuilder::with_3d` — Engine tạo Gl3DRenderer khi window mode; headless → Gl3DRenderer unbound | Code + tests |
| AC-003 | `RenderContext3D` cho game: renderer3d mut + camera3d | Code + mock test |
| AC-004 | Music WAV loading: `load_wav_file` + `play_clip` looped qua winmm backend | Code + test |
| AC-005 | Regression: game/physics/classic modes exit 0; engine tests 205 PASS | `logs/` |
| AC-006 | Docs FR-067..068 + traceability (tiếng Việt) + commit | Files + `git log` |

## References

- `/AGENTS.md`
- `/docs/01-product/roadmap.md`
