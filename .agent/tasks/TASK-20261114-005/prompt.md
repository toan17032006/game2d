# Prompt

## Input classification

`DIRECT` — người dùng yêu cầu trực tiếp trong phiên: "Commit và thực hiện đề xuất" — 2 đề xuất cuối TASK-20261114-004: (a) M4 slice 3 (tilemap multi-layer + sprite culling + music WAV looped); (b) M3 slice 3 (modern GL: extension loader + shader pipeline, fallback fixed-function).

## Role

Rust engine developer + graphics programmer.

## Objective

1. **M4 slice 3**:
   - Sprite culling: engine render bỏ qua sprite ngoài viewport camera (pure fn testable).
   - Tilemap multi-layer: spawn nhiều layer (background + solid) trên cùng texture.
   - Music: load WAV file → AudioClip looped playback qua winmm (non-streaming, ghi limitation).
2. **M3 slice 3**:
   - Extension loader: `wglGetProcAddress` + function pointer table cho shader functions (glCreateShader/ShaderSource/CompileShader/GetShaderiv/GetShaderInfoLog/CreateProgram/AttachShader/LinkProgram/GetProgramiv/GetUniformLocation/Uniform3f/UseProgram + glGetShaderiv...).
   - Shader pipeline: compile/link với info-log errors; Gl3DRenderer: khi functions khả dụng → compile color shader + dùng uniform; không khả dụng → fallback fixed-function glColor (như hiện tại).
3. Commit.

## Context

- 196 tests PASS trên `6bc904c`; GL backend legacy 1.1 qua static FFI; modern functions phải load động qua wglGetProcAddress (không nằm trong opengl32.lib).

## Inputs

- Yêu cầu trực tiếp người dùng trong phiên.
- `/docs/01-product/roadmap.md` (M3/M4 tiếp theo).

## Scope

- `monster-scene`: `sprite_in_view` culling helper + engine render integration + tilemap multi-layer test.
- `monster-audio`: `load_wav_file(path) -> AudioClip`.
- `monster-platform::gl`: extension loader + shader FFI (dynamic) + shader compile/link helpers.
- `monster-engine::gl3d`: shader path + fallback.
- Docs FR-060..062 + traceability + roadmap; commit.

## Non-goals

- True streaming music (callback thread); VAO/VBO/core-profile; texture 3D; skeletal.

## Constraints

- Zero-dep; giữ 196 tests cũ PASS; shader unavailable → fallback đúng (không crash).

## Execution rules

- Preflight trước EXECUTING; final gates trước COMPLETED; commit sau gates.

## Output contract

- Culling + multi-layer + music loader + extension loader + shader pipeline + tests + docs + commit.

## Acceptance criteria

Xem `task.md` (AC-001..AC-007).

## Verification method

- `cargo test --workspace` exit 0; sandbox modes exit 0; final gates exit 0.

## References

- `/AGENTS.md`
- `/docs/01-product/roadmap.md`
