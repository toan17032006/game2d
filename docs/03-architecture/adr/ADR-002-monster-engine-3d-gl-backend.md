# ADR-002: M3 slice — 3D renderer qua OpenGL legacy (WGL) zero-dependency

- Owner: Architecture Board
- Status: `Accepted`
- Date: `2026-11-14`
- Last updated: `2026-11-14`
- Review date: `2027-05-14`

## Context

Roadmap M3 yêu cầu tầng 3D cho `monster-engine`. Ràng buộc còn hiệu lực: zero external crate, FFI gom về `monster-platform` (theo `coding-standards.md`), giữ nguyên 2D pipeline và 163+ tests. Cần một slice M3 kiểm chứng được: trait 3D + camera 3D + một backend thật chạy demo cube.

## Decision

1. **Trait `Renderer3D`** nằm ở `monster-core` (cùng chỗ với `Renderer` 2D): `begin(Camera3D)` → `draw_cube`/`draw_grid` → `end`.
2. **`Camera3D`** ở `monster-core`: position + look_target + up + fov + aspect + near/far; view = `Mat4::look_at` (mới, right-handed), projection = `Mat4::perspective` (đã có).
3. **Backend GL legacy**: `monster-platform::gl` — WGL (`wglCreateContext` legacy context = GL 1.1 fixed-function) + opengl32 FFI tự khai báo + `GlContext` an toàn (pixel format 32-bit màu/24-bit depth, double-buffer).
4. **`Gl3DRenderer`** ở facade (`monster-engine::gl3d`): implement `Renderer3D`, hoạt động 2 chế độ — bound (GL thật, window demo) và unbound (state-only recorder cho test headless/CI).
5. FFI GL đặt trong `monster-platform::gl` đúng luật FFI tập trung; Gl3DRenderer ở facade là integration layer (platform context + core trait + math).

## Alternatives

- **Modern GL 3.x/4.x qua extension loader (wglChoosePixelFormatARB + wglGetProcAddress):** cần core profile + shaders — chất lượng dài hạn tốt hơn nhưng slice đầu quá lớn; deferred sang phần tiếp của M3. `Deferred`.
- **Crate ngoài (wgpu/glfw/glow):** vi phạm zero-dependency (ADR-001). `Rejected` cho giai đoạn này.
- **DirectX 11 FFI:** FFI COM phức tạp hơn nhiều so với GL legacy. `Deferred`.
- **Software 3D renderer (rasterizer CPU):** giáo dục tốt nhưng không phải "GPU backend" mà roadmap M3 yêu cầu. `Rejected` cho slice này.
- **Renderer3D tích hợp vào Engine loop 2D ngay:** cần thiết kế lại GameApp trait — hoãn; cube demo chạy raw loop (handmade style) trước. `Deferred`.

## Consequences

- **Tích cực:** đường 3D mở ra với zero-dependency; `Renderer3D` là biên mở rộng đúng chuẩn (backend mới không sửa core); Gl3DRenderer test được headless ở tầng state (unbound recorder); legacy GL đủ cho cube/grid demo + validation kiến trúc.
- **Đánh đổi:** GL 1.1 fixed-function deprecated (không shaders); hiệu năng giới hạn; Windows-only (WGL); GL rendering đúng đắn chỉ verify thủ công qua window (desktop session).
- **Trung lập:** trait `Renderer3D` sẽ mở rộng (`draw_mesh`, material) khi M3 tiếp tục; modern-GL backend có thể thay thế Gl3DRenderer phía sau cùng trait.

## References

- `/docs/03-architecture/adr/ADR-001-monster-engine-foundation.md`
- `/docs/01-product/roadmap.md` (M3)
- `/docs/04-engineering/coding-standards.md` (FFI rule)
- `/monster-engine/crates/monster-platform/src/gl.rs` (FFI + GlContext)
- `/monster-engine/crates/monster-engine/src/gl3d.rs` (Gl3DRenderer)
- `/monster-engine/crates/monster-engine/examples/cube_3d.rs` (demo)
- OpenGL 1.1 Registry: <https://registry.khronos.org/OpenGL-Refpages/gl2.1/> (truy cập 2026-11-14)
- WGL — Microsoft Learn: <https://learn.microsoft.com/en-us/windows/win32/api/wgl/> (truy cập 2026-11-14)
