# ADR-001 (DRAFT): Nền móng monster-engine — Rust, trait-based, 2D software renderer

- Status: `DRAFT` — chờ phê duyệt chuyển vào `/docs/03-architecture/adr/`
- Task: `TASK-20261113-008`
- Date: `2026-11-13`

## Context

Mục tiêu sản phẩm: engine 2D trong `monster-engine`, sau này mở rộng 3D; yêu cầu dễ mở rộng, dễ thêm hàm; dạng thư viện code thuần (không editor). Ràng buộc môi trường: máy chỉ có Rust toolchain (rustc 1.99.1 + cargo), không có compiler C/C++ và .NET SDK; không dựa vào network cho dependency. Repo đã có asset 2D PNG (Tiny Swords) để phát triển sau này.

## Decision

1. Ngôn ngữ: Rust 2021, cargo workspace tại `/monster-engine/`.
2. Kiến trúc: trait-based với contract nằm ở `monster-core` (trait `Renderer`, `AudioBackend`, `PresentTarget`, `InputSource`), implementation nằm ở các crate ngoài; facade `monster-engine` + `EngineBuilder` + trait `GameApp` cho game code.
3. Render nền móng: software 2D renderer (framebuffer RGBA8, alpha blend, camera, z-order), present qua Win32 GDI; zero external crate (chỉ `std` + FFI `user32`/`gdi32`).
4. Math chuẩn bị 3D từ đầu: Vec2/Vec3/Vec4, Mat4 column-major, Quat.
5. Scene: ECS-lite (EntityId generational + component store theo type + system là hàm).
6. Assets: registry + cache + decoder BMP/PNG/WAV thuần Rust.
7. Kiểm thử: headless-first (NullWindow, framebuffer assertions, smoke test PNG thật).

## Alternatives

- **C++ (MSVC/MinGW):** chuẩn industry cho engine, nhưng máy không có compiler và cài toolchain phụ thuộc quyền/network — bị loại vì ràng buộc môi trường.
- **TypeScript/WebGL:** dễ demo nhưng biên trừu tượng yếu hơn cho lộ trình 3D engine (GC, không kiểm soát bộ đệm) — bị loại.
- **Rust + crate ngoài (winit/wgpu):** nhanh hơn nhưng phụ thuộc network + đúng là "code thuần" ít hơn; mất giá trị nghiên cứu internals — bị loại.
- **GameObject–Component thay vì ECS-lite:** quen thuộc nhưng dữ liệu–hành vi dính nhau hơn, khó thêm component mới không sửa core — bị loại cho nền móng.
- **GPU renderer ngay từ đầu:** phụ thuộc FFI DirectX phức tạp, không test headless được — hoãn sang M3.

## Consequences

- Tích cực: thêm backend (GPU/3D, audio thật) là thêm crate mới implement trait có sẵn; game code chỉ phụ thuộc trait + facade; toàn bộ hệ thống test được headless trong CI.
- Đánh đổi: software renderer chậm hơn GPU (chấp nhận cho nền móng 2D); phải tự viết/maintain decoder PNG/WAV (được bù bằng test suite); Win32 FFI tự khai báo phải tự kiểm soát ABI (được bù bằng NullWindow test).
- Trung lập: crate `monster-*` là biên cứng — di chuyển code giữa crate tốn công nhưng ngăn dependency cycle.

## References

- `/docs/03-architecture/README.md` (quy định ADR)
- `/.agent/tasks/TASK-20261113-008/research/engine-foundation-study.md`
- `/docs/00-governance/decision-log.md`
