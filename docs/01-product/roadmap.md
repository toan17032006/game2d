# Roadmap — monster-engine

- Owner: Product
- Status: `Approved`
- Last updated: `2026-11-13`
- Review date: `2027-05-13`

## Purpose

Lộ trình phát triển theo milestone. Mỗi milestone có đầu ra kiểm chứng được; thứ tự ưu tiên "nền móng đúng → tính năng game → nền tảng 3D".

## Trạng thái milestone

| Milestone | Nội dung | Trạng thái |
| --- | --- | --- |
| **M0 — Nền tảng tính toán** | `monster-math` (Vec2/3/4, Mat4, Quat, Rect, Color) + `monster-core` (event bus, fixed clock, input, log, trait contracts) | ✅ Hoàn thành (v0.1.0) |
| **M1 — Hiển thị & tài nguyên** | `monster-platform` (Win32 window/input/GDI) + `monster-render` (software 2D) + `monster-assets` (registry, PNG/BMP) | ✅ Hoàn thành (v0.1.0) |
| **M2 — Trạng thái game** | `monster-scene` (ECS-lite) + `monster-physics` (AABB) + `monster-audio` (trait + Null + WAV) + facade `EngineBuilder`/`GameApp` + sandbox | ✅ Hoàn thành (v0.1.0) |
| **M2.5 — Physics v2 (taxonomy đầy đủ)** | 14 loại body + 15 loại collider + joints (Distance/Pin/Weld/Spring) + `PhysicsWorld2` pipeline + raycast + Character move_and_slide + particle systems (Soft/Cloth/Fluid/Particle/Deformable) + Vehicle suspension — FR-032..FR-048 | ✅ Hoàn thành (TASK-20261113-010) |
| **M3 — Tầng 3D** | GPU backend (implement `Renderer`): mesh, depth buffer, shader qua trait; physics 3D (Aabb3); camera 3D | ⬜ Chưa bắt đầu |
| **M4 — Feature game 2D** | Sprite animation (spritesheet Tiny Swords), tilemap, particle, text, camera follow/shake, audio playback thật, UI-less menu flow | ⬜ Chưa bắt đầu |
| **M5 — Game hoàn chỉnh đầu tiên** | Đóng gói một game hoàn chỉnh trên asset Tiny Swords (chạy window mode, có âm thanh) | ⬜ Chưa bắt đầu |

## Chi tiết M3 (mục tiêu kế tiếp)

1. `Renderer` trait bổ sung `draw_mesh(...)` (option trong `begin` — 1 điểm mở rộng duy nhất).
2. Crate `monster-render-gpu` (FFI DirectX hoặc OpenGL qua WGL) hoặc wgpu nếu bỏ ràng buộc zero-dep (cần quyết ADR mới).
3. `monster-physics`: `Aabb3` + broadphase tái dùng cấu trúc hiện có.
4. `monster-math`: bổ sung `Mat4::perspective` (đã có sẵn) vào camera 3D + frustum culling.

## Chi tiết M4

1. Sprite animation: `AnimationClip` component + system điều khiển `Sprite.src` theo frame.
2. Tilemap: loader map grid + render batch theo tile.
3. Text: bitmap font (texture atlas ký tự) — đủ cho HUD/debug.
4. Audio: backend `waveOut` qua FFI (implement `AudioBackend` có sẵn).
5. Particle: pooling + system trên ECS.

## Nguyên tắc lộ trình

- Mỗi milestone đi qua task workspace + quality gates riêng.
- Không bắt M3 trước khi M4 nếu game đích là 2D (ưu tiên giá trị dùng được).
- Mọi quyết định kiến trúc mới → ADR mới trong `/docs/03-architecture/adr/`.

## References

- `/docs/01-product/vision-and-goals.md`
- `/docs/01-product/scope-and-non-goals.md`
- `/docs/03-architecture/adr/ADR-001-monster-engine-foundation.md`
- `/.agent/tasks/TASK-20261113-008/research/engine-foundation-study.md` (mục §4 lộ trình gốc)
