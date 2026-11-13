# Container & Component — monster-engine

- Owner: Architecture Board
- Status: `Approved`
- Last updated: `2026-11-13`
- Review date: `2027-05-13`

## Purpose

Mô tả cấu trúc bên trong `monster-engine`: 9 crate (container), trait contracts (component biên), và vòng đời engine. Mọi chi tiết dưới đây khớp code thật tại `/monster-engine/` (commit `e1aff19`).

## Sơ đồ crate và dependency (một chiều, cấm cycle)

```
                    ┌────────────────────┐
                    │  monster-engine    │  facade: Engine, EngineBuilder,
                    │  (facade)          │  GameApp, prelude, example sandbox
                    └─────────┬──────────┘
        ┌──────────┬──────────┼──────────┬──────────┐
        ▼          ▼          ▼          ▼          ▼
   assets      scene      physics    audio     render   platform
   (decode)  (ECS,sys)   (AABB)    (WAV,Null) (blit)   (Win32)
        │          │         │          │         │    │
        └──────────┴────┬────┴──────────┴─────────┴────┘
                        ▼
                  ┌───────────┐
                  │ monster-  │  contracts: Renderer, AudioBackend,
                  │ core      │  PresentTarget, EventBus, FixedClock,
                  └─────┬─────┘  InputState, log, Texture, Camera2D, Handle
                        ▼
                  ┌───────────┐
                  │ monster-  │  Vec2/3/4, Mat4, Quat, Rect, Color
                  │ math      │
                  └───────────┘
```

Dependency thực tế trong code (xem các `Cargo.toml`):

| Crate | Phụ thuộc | Trách nhiệm |
| --- | --- | --- |
| `monster-math` | (không) | Vec2/Vec3/Vec4, Mat4 column-major, Quat, Rect, Color, lerp/clamp |
| `monster-core` | math | EventBus, FixedClock (accumulator), InputState, log macros, Texture/Camera2D/DrawOptions, trait `Renderer`/`AudioBackend`/`PresentTarget`, `Handle<T>`, AudioClip |
| `monster-render` | math, core | `Framebuffer` (RGBA8, blend_pixel, blit_sprite, fill_rect) + `Software2DRenderer` implement `Renderer` |
| `monster-platform` | math, core | FFI Win32 (`ffi.rs`), `Window` (Win32 + GDI present), `NullWindow`, trait `PlatformWindow`, keymap VK→KeyCode |
| `monster-assets` | core | `Assets` registry/cache, `decode_png` (chunk+CRC+inflate+defilter+tRNS), `decode_bmp` |
| `monster-scene` | math, core, physics | `World` ECS-lite (EntityId generational, TypedStore), components (Transform/Sprite/Velocity/ZOrder), systems (movement, draw order) |
| `monster-physics` | math | Aabb2, integrate, apply_gravity, bounce, find_collisions (brute-force broadphase), MTV separation |
| `monster-audio` | core | `NullAudioBackend` implement `AudioBackend`, `decode_wav` (RIFF/PCM 8/16/24-bit, mixdown mono) |
| `monster-engine` | tất cả | `Engine` + `EngineBuilder` + `GameApp` + contexts + prelude + example `sandbox` |

## Trait contracts — biên mở rộng

| Trait | Nơi định nghĩa | Hiện thực hiện tại | Backend tương lai |
| --- | --- | --- | --- |
| `Renderer` | monster-core::graphics | `Software2DRenderer` | GPU/3D backend (M3) |
| `PresentTarget` | monster-core::graphics | `Window`, `NullWindow` | Backend surface khác |
| `AudioBackend` | monster-core::audio | `NullAudioBackend` | waveOut/DirectSound FFI, WebAudio |
| `PlatformWindow` | monster-platform::window | `Window`, `NullWindow` | Platform khác (Linux/macOS) |

Quy tắc: subsystem và game code **chỉ gọi trait**, không gọi implementation cụ thể. Thêm backend = thêm struct mới implement trait, đăng ký qua `EngineBuilder` — core không đổi.

## Vòng đời Engine (`Engine::run`)

```
build()                    EngineBuilder: chọn Window (real/null), audio backend,
                           dựng Software2DRenderer, Assets, World, InputState, FixedClock
    │
    ▼
app.init(InitContext)      game nạp asset, spawn entity, đặt camera
    │
    ▼
┌─ loop ────────────────────────────────────────────────┐
│ 1. window.poll_events(&mut EventBus)                  │
│ 2. phân route event: resize/closed → engine;          │
│    key/mouse → InputState.begin_frame                 │
│ 3. steps = clock.tick(now)      (fixed timestep)      │
│    for 0..steps: app.update(UpdateContext{dt cố định})│
│    exit flag → thoát                                  │
│ 4. renderer.begin(camera)                             │
│    sprite_draw_order(world) → draw theo z tăng dần    │
│    app.render(RenderContext{world, renderer, alpha})  │
│    renderer.end(window)         → present             │
└───────────────────────────────────────────────────────┘
    │
    ▼
app.shutdown() → EngineRunResult {frames, updates, elapsed_secs, avg_fps}
```

Mô hình thời gian: **fixed timestep update** (mặc định 60 steps/giây, dt không đổi) + **variable render** (mỗi vòng loop, kèm `alpha` cho interpolation). Clock clamp frame delta tối đa 250ms chống "spiral of death".

## ECS-lite (`monster-scene::world`)

- `EntityId { index, generation }` — generation tăng mỗi lần slot tái sử dụng, id cũ sau despawn bị từ chối (không alias dữ liệu mới).
- Component store: `HashMap<TypeId, Box<dyn AnyStore>>`, store mỗi type là `HashMap<u32, T>`.
- Query: `get/get_mut/iter_mut` (lọc entity còn sống), `for_each2<A, B>` (join 2 component).
- System là hàm thuần: `movement_system(world, dt)`, `sprite_draw_order(world)`.
- Thêm gameplay = thêm struct component + hàm system — `World` không đổi.

## Headless mode

`EngineBuilder::new().headless()` dùng `NullWindow` (không mở cửa sổ, đếm present) — toàn vòng lặp engine chạy không GUI: nền tảng cho test tự động và CI.

## References

- `/docs/03-architecture/system-context.md`
- `/docs/03-architecture/adr/ADR-001-monster-engine-foundation.md`
- `/monster-engine/crates/` (nguồn sự thật)
- `/.agent/tasks/TASK-20261113-008/research/engine-foundation-study.md`
- RFC 1951 — DEFLATE: <https://www.rfc-editor.org/rfc/rfc1951> (truy cập 2026-11-13)
