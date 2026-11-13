# ADR-001: Nền móng monster-engine — Rust, trait-based, 2D software renderer

- Owner: Architecture Board
- Status: `Accepted`
- Date: `2026-11-13`
- Last updated: `2026-11-13`
- Review date: `2027-05-13`

## Context

Mục tiêu sản phẩm: engine 2D trong `monster-engine`, sau này mở rộng thành 3D; yêu cầu dễ mở rộng, dễ thêm hàm; dạng thư viện "code thuần" (không editor, game code gọi API trực tiếp). Ràng buộc môi trường: máy phát triển chỉ có Rust toolchain (rustc 1.99.1 + cargo), không có compiler C/C++ và không có .NET SDK; không dựa vào network cho dependency. Repo đã có asset 2D PNG (Tiny Swords) để phát triển game sau này. Nền móng được hiện thực và kiểm chứng trong TASK-20261113-008 (105 tests PASS, final quality gates PASS).

## Decision

1. **Ngôn ngữ:** Rust 2021, cargo workspace tại `/monster-engine/`.
2. **Kiến trúc:** trait-based với contract nằm ở `monster-core` (trait `Renderer`, `AudioBackend`, `PresentTarget`), implementation nằm ở các crate ngoài; facade `monster-engine` + `EngineBuilder` + trait `GameApp` cho game code.
3. **Render nền móng:** software 2D renderer (framebuffer RGBA8, alpha blend, camera, z-order), present qua Win32 GDI; zero external crate (chỉ `std` + FFI `user32`/`gdi32` tự khai báo).
4. **Math chuẩn bị 3D từ đầu:** Vec2/Vec3/Vec4, Mat4 column-major, Quat.
5. **Scene:** ECS-lite (EntityId generational + component store theo type + system là hàm).
6. **Assets:** registry + cache + decoder BMP/PNG/WAV thuần Rust (tự viết inflate DEFLATE).
7. **Kiểm thử:** headless-first (NullWindow, NullAudioBackend, assertion trên framebuffer, smoke test PNG thật).

## Alternatives

- **C++ (MSVC/MinGW):** chuẩn industry cho engine, phù hợp lộ trình 3D lâu dài — nhưng máy không có compiler và việc cài toolchain phụ thuộc quyền/network. `Rejected` vì ràng buộc môi trường.
- **TypeScript/WebGL:** dễ demo, test nhanh — nhưng biên trừu tượng yếu hơn cho lộ trình 3D engine (GC, không kiểm soát bộ đệm). `Rejected`.
- **Rust + crate ngoài (winit/wgpu):** nhanh hơn nhưng phụ thuộc network và mất giá trị nghiên cứu internals của "code thuần". `Rejected`.
- **GameObject–Component thay vì ECS-lite:** quen thuộc hơn nhưng dữ liệu–hành vi dính nhau, khó thêm component mới mà không sửa core. `Rejected` cho nền móng.
- **GPU renderer ngay từ đầu:** phụ thuộc FFI DirectX phức tạp, không test headless được. `Deferred` sang giai đoạn M3.

## Consequences

- **Tích cực:** thêm backend mới (GPU/3D, audio playback thật) là thêm crate/module mới implement trait có sẵn — không sửa core; game code chỉ phụ thuộc trait + facade; toàn bộ hệ thống test được headless trong CI; PNG của Tiny Swords dùng được ngay.
- **Đánh đổi:** software renderer chậm hơn GPU (chấp nhận cho nền móng 2D); phải tự viết/maintain decoder PNG/WAV (được bù bằng test suite 105 tests); Win32 FFI tự khai báo phải tự kiểm soát ABI (được bù bằng NullWindow test).
- **Trung lập:** crate `monster-*` là biên cứng — di chuyển code giữa crate tốn công nhưng ngăn dependency cycle.

## References

- `/docs/03-architecture/README.md`
- `/docs/03-architecture/adr/ADR-000-decision-log.md`
- `/monster-engine/` (hiện thực)
- `/.agent/tasks/TASK-20261113-008/research/engine-foundation-study.md`
- `/.agent/tasks/TASK-20261113-008/verification.md` (bằng chứng kiểm chứng)
- RFC 1951 — DEFLATE: <https://www.rfc-editor.org/rfc/rfc1951> (truy cập 2026-11-13)
- RFC 2083 — PNG: <https://www.rfc-editor.org/rfc/rfc2083> (truy cập 2026-11-13)
- "Fix Your Timestep!" — Glenn Fiedler: <https://gafferongames.com/post/fix_your_timestep/> (truy cập 2026-11-13)
