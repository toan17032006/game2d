# Data Architecture — monster-engine

- Owner: Architecture Board
- Status: `Approved`
- Last updated: `2026-11-13`
- Review date: `2027-05-13`

## Purpose

Mô tả bố trí dữ liệu chính của engine: định dạng pixel, pipeline asset, lưu trữ component trong ECS, và dữ liệu runtime (framebuffer, clip âm thanh).

## Định dạng pixel trung tâm: RGBA8

Mọi hình ảnh trong engine dùng một định dạng duy nhất: **RGBA8, row-major, top-left origin, +Y hướng xuống**.

| Dữ liệu | Kiểu | Bố trí |
| --- | --- | --- |
| `Texture` (monster-core) | `Vec<u8>` | RGBA8, `width*height*4` bytes |
| `Framebuffer` (monster-render) | `Vec<u8>` | RGBA8, cùng bố trí với Texture |
| PNG decode output | RGBA8 | chuyển từ gray/RGB/palette/grayA/RGBA + tRNS |
| BMP decode output | RGBA8 | chuyển từ BGR/BGRA (24/32bpp, top-down/bottom-up) |

- Lợi ích: decoder → texture → framebuffer không cần chuyển đổi bố trí; GPU backend sau này upload thẳng.
- Present lên màn hình: GDI DIB là BGRA → chuyển RGBA→BGRA tại `Window::present` (biên duy nhất).

## Pipeline asset

```
file (/assest/.../*.png|bmp|wav)
        │ đọc bytes
        ▼
decode (monster-assets / monster-audio)
  PNG: signature → chunks (CRC32 verify) → IHDR/PLTE/tRNS/IDAT/IEND
       → zlib (header check + adler32) → inflate DEFLATE (stored/fixed/dynamic)
       → defilter (None/Sub/Up/Average/Paeth) → RGBA8 (palette + tRNS alpha)
  BMP: 'BM' + BITMAPINFOHEADER → BI_RGB 24/32bpp → RGBA8 (top-down/bottom-up)
  WAV: RIFF chunks → fmt (PCM 8/16/24-bit) → data → mixdown mono i16
        │
        ▼
Assets registry (monster-assets)
  key (path) → slot index → Handle<Texture> (typed, Copy, ổn định khi replace)
  cache: load lần 2 cùng path trả Handle cũ, không decode lại
        │
        ▼
Texture / AudioClip — dùng bởi renderer / audio backend
```

Giới hạn đã ghi nhận (foundation): PNG bit depth 8, không interlace; tRNS hỗ trợ palette (alpha theo index) và gray/RGB (color key).

## ECS storage (`monster-scene::world`)

```
World
 ├─ slots: Vec<Slot { generation: u32, alive: bool }>
 ├─ free:  Vec<u32>                      (slot tái sử dụng)
 └─ stores: HashMap<TypeId, Box<dyn AnyStore>>
        └─ TypedStore<T> { map: HashMap<u32, T> }   (key = slot index)
```

- Dữ liệu component là struct thuần (`Transform`, `Sprite`, `Velocity`, `ZOrder`).
- `EntityId` mang generation → stale id sau despawn trả `None` thay vì đọc nhầm dữ liệu entity khác.
- Hướng phát triển: chuyển `TypedStore` sang SoA (Struct-of-Arrays) khi profile cho thấy cần — không đổi API.

## Dữ liệu runtime khác

| Dữ liệu | Vị trí | Bố trí |
| --- | --- | --- |
| EventBus queue | monster-core | `Vec<EngineEvent>`, drain mỗi frame (FIFO) |
| InputState flags | monster-core | 53 slot dense-index (50 key + 3 mouse), 3 mảng `Vec<bool>` (held/pressed/released) |
| FixedClock | monster-core | accumulator `Duration`, fixed_step, max_frame_delta 250ms |
| AudioClip | monster-core | mono PCM-16 `Vec<i16>` + sample_rate |
| Playback registry | monster-audio (Null) | `Vec<Playback { id, volume, looped, duration }>` |
| Sprite draw list | monster-scene | `Vec<SpriteDraw>` sort ổn định theo `ZOrder` tăng dần |

## Quy tắc dữ liệu

- MUST dùng RGBA8 làm định dạng hình ảnh trung tâm.
- MUST bọc dữ liệu tài nguyên bằng `Handle<T>` (typed index), không trả pointer/raw index cho game code.
- MUST NOT cấp phát mới trong hot loop cho buffer có kích thước biết trước (framebuffer, batch — reuse).
- MUST giữ `EntityId` generational — cấm tái dùng id trực tiếp.

## References

- `/docs/03-architecture/container-and-component.md`
- `/docs/03-architecture/adr/ADR-001-monster-engine-foundation.md`
- `/monster-engine/crates/monster-assets/src/png.rs` (nguồn sự thật decoder)
- `/monster-engine/crates/monster-scene/src/world.rs` (nguồn sự thật ECS)
- RFC 1950 — zlib: <https://www.rfc-editor.org/rfc/rfc1950> (truy cập 2026-11-13)
- RFC 2083 — PNG: <https://www.rfc-editor.org/rfc/rfc2083> (truy cập 2026-11-13)
