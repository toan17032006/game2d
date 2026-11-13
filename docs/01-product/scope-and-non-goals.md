# Scope & Non-goals — monster-engine

- Owner: Product
- Status: `Approved`
- Last updated: `2026-11-13`
- Review date: `2027-05-13`

## Purpose

Ranh giới sản phẩm: gì nằm trong scope nền móng, gì nằm ngoài và vì sao. Ngăn scope creep và làm rõ kỳ vọng.

## Trong scope (nền móng v0.1.0 — đã hoàn thành)

| Lĩnh vực | Nội dung |
| --- | --- |
| Runtime 2D | Vòng lặp fixed timestep, window Win32, input keyboard/mouse, event bus |
| Render 2D | Software renderer RGBA8: sprite blit (alpha/tint/flip/sub-rect), rect, camera offset+zoom, z-order, present qua GDI |
| Assets | Registry + cache; decoder PNG (đủ Tiny Swords), BMP, WAV |
| Scene | ECS-lite, components mặc định, camera 2D, draw order |
| Physics | AABB, tích phân Euler, MTV resolution, broadphase brute-force |
| Audio | WAV decode + trait `AudioBackend` + Null backend |
| Math | Vec2/3/4, Mat4, Quat, Rect, Color |
| Tooling | Example sandbox (window + headless), 105 tests headless |

## Ngoài scope (non-goals của nền móng)

| Non-goal | Lý do | Khi nào xem lại |
| --- | --- | --- |
| Render 3D / GPU backend | Cần biên `Renderer` ổn định trước; FFI DirectX phức tạp | M3 |
| Editor / GUI tool | Engine là thư viện "code thuần" | Khi team tăng lên |
| Hot-reload asset | Cấu trúc cache phải ổn định trước | M4 |
| Scripting language (Lua/...) | Rust trait đủ cho giai đoạn hiện tại | Khi cần nhanh việc gameplay |
| Networking / multiplayer | Ngoài phạm vi engine đơn máy | Không định kỳ |
| Audio playback phần cứng | Trait + Null backend đã đủ; decode WAV đã có | M4 |
| Multi-thread job system | Chỉ đáng làm khi profile cho thấy bottleneck | Sau M4 |
| Sprite animation / tilemap / particle / text | Nằm ở tầng feature game, dựng trên nền móng | M4 |
| Linux/macOS/Web | Win32 là nền tảng phát triển hiện tại | Khi cần phân phối |
| `/docs/07-operations/` | Engine là thư viện, chưa có service vận hành | Khi release game chạy thật |

## Nguyên tắc mở rộng scope

1. Scope mới phải đi qua task workspace + quality gates (`/AGENTS.md`).
2. Feature gameplay mới KHÔNG sửa core — thêm component/system.
3. Backend mới KHÔNG sửa core — implement trait.

## References

- `/docs/01-product/vision-and-goals.md`
- `/docs/01-product/roadmap.md`
- `/docs/03-architecture/adr/ADR-001-monster-engine-foundation.md`
- `/AGENTS.md`
