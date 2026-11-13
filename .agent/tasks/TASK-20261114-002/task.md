# Task

- Task ID: `TASK-20261114-002`
- Status: `EXECUTING`
- Owner: `@agent`

## Objective

Thực hiện 2 đề xuất: (a) camera follow system; (b) M3 slice đầu tiên — 3D renderer qua OpenGL/WGL legacy (zero-dep) với cube demo + ADR-002; commit.

## Scope

- `monster-scene`: `camera_follow_system` + test.
- `monster-math`: `Mat4::look_at` + test.
- `monster-core`: `Camera3D` + trait `Renderer3D` + test.
- `monster-platform`: `gl.rs` (WGL/opengl32 FFI + `GlContext`) — compile-link + test skip-graceful khi không có desktop.
- `monster-engine`: `gl3d.rs` (`Gl3DRenderer` cube) + `examples/cube_3d.rs`.
- Docs: ADR-002, FR-050..052, traceability, roadmap M3 ✅ partial.

## Non-goals

- Modern GL/shaders, texture 3D, mesh loading, headless GL verify, engine loop 3D tích hợp (cube demo chạy raw loop).

## Acceptance criteria

| ID | Criterion | Required evidence |
| --- | --- | --- |
| AC-001 | `camera_follow_system`: camera tiến về target theo lerp speed*dt, đạt target, offset hỗ trợ | Test PASS |
| AC-002 | `Mat4::look_at` đúng (view matrix chuẩn handedness) + `Camera3D::view_projection` | Test PASS |
| AC-003 | `Renderer3D` trait tồn tại ở `monster-core` (begin/draw_cube/draw_grid/end) | Code + test (headless mock renderer) |
| AC-004 | `GlContext` (WGL legacy) + GL FFI trong `monster-platform::gl`; link opengl32 thành công; test tạo context skip-graceful khi không có desktop session | Code + build log |
| AC-005 | `Gl3DRenderer` implement `Renderer3D` (cube + grid, depth test, perspective) + example `cube_3d` compile | Build log |
| AC-006 | Docs: ADR-002 (Accepted) + FR-050..052 + traceability + roadmap M3 ✅ partial (tiếng Việt) | Files |
| AC-007 | `cargo test --workspace` exit 0 (≥ 165 tests) + final gates PASS + commit git | `logs/` + `git log` |

## References

- `/AGENTS.md`
- `/docs/03-architecture/adr/ADR-001-monster-engine-foundation.md`
