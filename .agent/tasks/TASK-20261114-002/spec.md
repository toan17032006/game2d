# Specification

## Inputs

- Yêu cầu người dùng: "Commit và thực hiện đề xuất" (camera follow + M3 GPU/3D).
- Code hiện tại: scene Transform, Camera2D, Mat4 (có perspective), platform Win32 window.

## Confirmed facts

- F-01: Repo sạch trên `764b60d`; 163 tests PASS.
- F-02: `monster-math::Mat4` đã có `perspective` + `view_2d`; chưa có `look_at`.
- F-03: `coding-standards.md` quy định FFI gom vào `monster-platform`.
- F-04: Legacy `wglCreateContext` cho GL 1.1 fixed-function không cần extension loader.

## Inferences

- I-1 (inference): Camera follow đặt trong `monster-scene` (system trên ECS) — dùng `Camera2D` của core.
- I-2 (inference): GL renderer đặt ở facade (`monster-engine::gl3d`) vì cần platform (context) + core (trait) + math — đúng vai trò integration layer của facade, không tạo dependency cycle mới.
- I-3 (inference): GL 1.1 fixed-function đủ cho cube demo M3-slice; shaders/modern GL là phần tiếp của M3.

## Unknowns

- U-1: Desktop session có sẵn khi chạy test tự động không — test tạo GL context sẽ skip-graceful nếu không tạo được.

## Constraints

- C-01: Zero external crate; opengl32/opengl32 là system DLL — FFI tự khai báo trong `monster-platform::gl`.
- C-02: Không phá vỡ 163 tests cũ.
- C-03: GL rendering đúng đắn chỉ verify thủ công (window thật) — ghi limitation.

## Decisions

- D-01: Camera follow = lerp `camera.position` về `target.position + offset` với `speed*dt` clamp 0..1, theo trục (không xoay).
- D-02: `Camera3D` = position + look_at target + up (0,1,0) + fov_y + aspect + near/far; `view_projection = perspective * look_at`.
- D-03: Trait `Renderer3D`: `begin(camera3d)` → `draw_cube(pos, half_extents, color)` / `draw_grid(size, divisions, color)` → `end()`.
- D-04: `GlContext` trong `monster-platform::gl`: wglCreateContext legacy trên HDC của window; `Gl3DRenderer` ở facade gọi qua đó.
- D-05: Cube = 6 mặt quads màu khác nhau; grid = lines trên mặt phẳng XZ; depth test bật.
- D-06: Test Command: `cargo test --workspace --manifest-path monster-engine/Cargo.toml`.

## Output contract

- Camera follow + Camera3D + Renderer3D + GlContext + Gl3DRenderer + example cube_3d + ADR-002 + FR-050..052 + traceability + roadmap + commit.

## Test Command

`cargo test --workspace --manifest-path monster-engine/Cargo.toml`

## Approved Documentation Changes

- `/docs/03-architecture/adr/ADR-002-monster-engine-3d-gl-backend.md` — ADR mới (phê duyệt trực tiếp qua yêu cầu thực hiện M3)
- `/docs/02-requirements/functional-requirements.md` — thêm FR-050..FR-052
- `/docs/02-requirements/traceability-matrix.md` — thêm dòng FR-050..052
- `/docs/01-product/roadmap.md` — cập nhật M3 ✅ partial

## References

- `/AGENTS.md`
- `/docs/04-engineering/coding-standards.md`
- OpenGL 1.1 Reference: <https://registry.khronos.org/OpenGL-Refpages/gl2.1/> (truy cập 2026-11-14)
