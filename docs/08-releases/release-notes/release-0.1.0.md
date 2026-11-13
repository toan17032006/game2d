# Release Notes — monster-engine v0.1.0

- Owner: Release Manager
- Status: `Approved`
- Last updated: `2026-11-13`
- Review date: `2027-05-13`

## Thông tin phiên bản

| Hạng mục | Giá trị |
| --- | --- |
| Version | `0.1.0` (nền móng M0–M2) |
| Ngày phát hành | 2026-11-13 |
| Commit nền móng | `e1aff19` — feat: monster-engine foundation (62 files, +8.266 dòng) |
| Toolchain | Rust stable 1.99.1 (x86_64-pc-windows-msvc), Windows 10+ |
| Bằng chứng | `/.agent/tasks/TASK-20261113-008/verification.md` — 105 tests PASS, final gates PASS |

## Có gì mới

### Engine core (2D, chạy được end-to-end)

- Vòng lặp **fixed timestep** (60 Hz mặc định) + render biến đổi, clamp chống spiral of death.
- **Cửa sổ Win32** tự viết FFI (user32/gdi32/kernel32) — không crate ngoài: tạo window, message pump, input bàn phím/chuột, resize.
- **Software renderer 2D**: framebuffer RGBA8, sprite blit (alpha blend, tint, flip, sub-rect spritesheet), fill rect, camera offset + zoom, z-order, present qua GDI DIB.
- **ECS-lite**: EntityId generational (chặn stale id), component store theo type, join query, system thuần.
- **Physics**: AABB overlap, MTV resolution, broadphase brute-force, bounce, tích phân Euler.
- **Facade**: `EngineBuilder` + `GameApp` (init/update/render) + prelude — game mới chỉ cần implement 1 trait.
- **Kiến trúc mở rộng**: trait `Renderer`/`AudioBackend`/`PresentTarget`/`PlatformWindow` ở `monster-core`; thêm backend không sửa core (có test chứng minh).

### Tài nguyên

- **PNG decoder thuần Rust**: DEFLATE inflate đủ 3 block type, CRC32/adler32 verify, palette + tRNS alpha, 5 filter, khớp spec bit depth 8 — decode được spritesheet Tiny Swords thật (smoke test 192×256).
- **BMP decoder** BI_RGB 24/32bpp (top-down + bottom-up).
- **WAV decoder** PCM 8/16/24-bit, mixdown mono.
- Registry + cache theo path, trả `Handle<T>` typed.

### Math (3D-ready)

- Vec2/Vec3/Vec4, Mat4 column-major (ortho + perspective + view 2D), Quat (gimbal-lock free), Rect, Color RGBA8.

### Công cụ

- Example `sandbox`: `--window` (demo 40 sprite bounce 960×540, ESC thoát) và `--headless N` (smoke test tự động, exit 0).
- 105 tests toàn workspace, headless hoàn toàn (NullWindow/NullAudioBackend/framebuffer assertions).
- Bộ tài liệu tiếng Việt đầy đủ trong `/docs/` (architecture, requirements, engineering, quality, releases) + ADR-001.

## Giới hạn đã biết (limitations)

- Window thật + present GDI chỉ verify trực quan (test tự động chạy headless).
- Chưa có: sprite animation, tilemap, text, particle, audio playback phần cứng, GPU/3D — nằm ở lộ trình M3/M4 (`/docs/01-product/roadmap.md`).
- PNG decoder: bit depth 8, không interlace.
- Platform: Windows 10+.

## Hướng dẫn chạy nhanh

```bash
cd monster-engine
cargo test --workspace                              # kiểm chứng: 105 tests
cargo run --example sandbox -- --headless 120       # smoke test headless
cargo run --example sandbox -- --window             # demo trực quan
```

## Tương thích

- Không có phiên bản trước — đây là phát hành đầu tiên.
- Asset format engine dùng: PNG (depth 8), BMP (24/32bpp), WAV (PCM 8/16/24-bit).

## References

- `/docs/03-architecture/adr/ADR-001-monster-engine-foundation.md`
- `/docs/02-requirements/traceability-matrix.md`
- `/docs/08-releases/release-process.md`
- `/monster-engine/` (nguồn)
