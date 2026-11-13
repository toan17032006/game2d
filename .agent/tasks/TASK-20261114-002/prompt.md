# Prompt

## Input classification

`DIRECT` — người dùng yêu cầu trực tiếp trong phiên: "Commit và thực hiện đề xuất" — 2 đề xuất cuối TASK-20261114-001: (a) camera follow physics entities; (b) M3 GPU/3D backend (bắt đầu slice đầu tiên).

## Role

Rust engine developer + graphics programmer.

## Objective

1. **Camera follow**: system theo mượt target entity trong `monster-scene` (lerp + offset), có test.
2. **M3 slice đầu tiên — 3D renderer**:
   - `Camera3D` (position, look_at, fov) + `Mat4::look_at` trong monster-math/core.
   - Trait `Renderer3D` ở `monster-core` (begin/draw_cube/draw_grid/end).
   - WGL/OpenGL FFI + `GlContext` trong `monster-platform` (opengl32 legacy 1.1 fixed-function — zero crate).
   - `Gl3DRenderer` (facade `monster-engine::gl3d`) — render rotating cube demo.
   - Example `cube_3d` (cửa sổ, ESC thoát).
   - ADR-002 cho quyết định backend OpenGL legacy.

## Context

- Repo sạch trên `764b60d`; 163 tests PASS; physics v2 + bridge đã có.
- FFI rule trong `coding-standards.md`: gom FFI vào `monster-platform` — GL/WGL FFI đặt ở đó.
- OpenGL legacy qua `wglCreateContext` cho GL 1.1 fixed-function: đủ cube + depth test + perspective, không cần extension loader.

## Inputs

- Yêu cầu trực tiếp người dùng ("Commit và thực hiện đề xuất").
- `/docs/03-architecture/adr/ADR-001-monster-engine-foundation.md` (2D-first, M3 = 3D).

## Scope

- `monster-scene`: `camera_follow_system`.
- `monster-math`: `Mat4::look_at`; `monster-core`: `Camera3D`, trait `Renderer3D`.
- `monster-platform`: `gl.rs` (WGL/opengl32 FFI + `GlContext`).
- `monster-engine`: `gl3d.rs` (`Gl3DRenderer`) + `examples/cube_3d.rs`.
- Docs: ADR-002 + FR-050..052 + traceability + roadmap M3 partial (docs tiếng Việt — đã duyệt duy trì trong phiên).

## Non-goals

- Modern OpenGL (GL 3+/shaders) — cần extension loader, làm tiếp trong M3.
- Textured 3D models/mesh loading; headless GL verification (GL cần window thật).

## Constraints

- Zero external crate.
- Không phá vỡ 163 tests cũ; FFI gom về monster-platform (GL FFI ở `platform::gl`).
- Legacy GL 1.1: fixed-function, đủ cho cube demo.

## Execution rules

- Preflight trước EXECUTING; final gates trước COMPLETED; commit sau gates.

## Output contract

- Camera follow system + Camera3D + Renderer3D trait + GlContext + Gl3DRenderer + example cube_3d + ADR-002 + docs FR + commit.

## Acceptance criteria

Xem `task.md` (AC-001..AC-007).

## Verification method

- `cargo test --workspace` exit 0 (camera follow + Camera3D matrix tests headless).
- `cargo build --workspace` + example compile (GL link opengl32).
- Final gates exit 0; demo cube_3d chạy thủ công bởi người dùng (giới hạn đã ghi).

## References

- `/AGENTS.md`
- `/docs/03-architecture/adr/ADR-001-monster-engine-foundation.md`
- `/docs/04-engineering/coding-standards.md` (FFI rule)
