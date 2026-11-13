# API Guidelines — monster-engine

- Owner: Engineering
- Status: `Approved`
- Last updated: `2026-11-13`
- Review date: `2027-05-13`

## Purpose

Quy ước thiết kế API công khai của engine: game developer dùng gì, engine cung cấp thế nào. Áp dụng cho mọi public API mới của `monster-engine`.

## Bề mặt API cho game developer

Game code chỉ chạm qua **một điểm**: `monster_engine::prelude::*` + 2 trait của mình:

```rust
use monster_engine::prelude::*;

struct MyGame;                                  // game = struct bất kỳ
impl GameApp for MyGame {                       // + impl GameApp
    fn init(&mut self, ctx: &mut InitContext<'_>) { /* nạp asset, spawn */ }
    fn update(&mut self, ctx: &mut UpdateContext<'_>) { /* mô phỏng */ }
    fn render(&mut self, ctx: &mut RenderContext<'_>) { /* vẽ thêm */ }
}

let mut engine = EngineBuilder::new()           // cấu hình khai báo
    .with_window(WindowOptions { width: 960, height: 540, title: "Game".into() })
    .build().unwrap();
engine.run(&mut MyGame);
```

## Quy tắc thiết kế

### 1. Builder cho cấu hình, trait cho hành vi

- Cấu hình engine: `EngineBuilder` với `with_*` chaining — thêm option mới không phá vỡ caller cũ.
- Hành vi có nhiều backend: trait (`Renderer`, `AudioBackend`, `PlatformWindow`) + đăng ký qua builder.
- CẤM expose struct implementation cụ thể qua facade trừ khi là giá trị dữ liệu (`Texture`, `AudioClip`).

### 2. Context object chia phase — borrows tường minh

| Context | Cho phép | Không cho phép |
| --- | --- | --- |
| `InitContext` | world, assets, audio, camera | renderer (chưa vào frame) |
| `UpdateContext` | world (mut), assets, input (read), audio, camera, dt, `exit()` | renderer |
| `RenderContext` | world (read), renderer (mut), alpha | assets/audio (giữ render thuần vẽ) |

Lý do: borrow checker hiển thị rõ "cái gì thay đổi khi nào"; render không được nạp asset giữa frame.

### 3. Handle thay vì raw reference

```rust
let h = assets.load_file(path)?;         // Handle<Texture> — Copy, typed
let tex = assets.texture(h)?;            // đọc dữ liệu khi cần vẽ
world.spawn().with(Sprite { texture: h, .. });
```

- Handle ổn định qua cache replace; typed — Handle<Texture> không thể truyền nhầm chỗ cần AudioClip.
- Resource sống trong engine; game code không giữ `&Texture` qua frame (tránh borrow xuyên suốt).

### 4. Dữ liệu thuần cho component, hàm thuần cho system

```rust
pub struct Transform { pub position: Vec2, pub rotation: f32, pub scale: Vec2 }
pub fn movement_system(world: &mut World, dt: f32);
```

- Component KHÔNG có method thay đổi trạng thái game (dữ liệu + hệ thống tách rời).
- System nhận `&mut World` + tham số đơn giản; kết hợp qua `for_each2`.

### 5. Events typed, drain per-frame

- OS/game tạo `EngineEvent` → `EventBus`; engine phân route mỗi frame.
- Thêm loại event = thêm variant (non-exhaustive match xử lý rõ ràng).

### 6. Naming & semantics

| Mẫu | Ý nghĩa | Ví dụ |
| --- | --- | --- |
| `with_*` | builder option | `with_window`, `with_audio_backend` |
| `begin`/`end` | giao thức frame của renderer | `renderer.begin(&cam)` ... `renderer.end(target)` |
| `is_*` / `was_*` | truy vấn trạng thái | `input.was_pressed(KeyCode::Escape)` |
| `world_to_screen` / `screen_to_world` | chuyển đổi hệ tọa độ, có roundtrip | `camera.world_to_screen(p)` |
| `Handle<T>` | tham chiếu tài nguyên | `Handle<Texture>` |

### 7. Tọa độ & đơn vị

- World space: +X phải, +Y **xuống** (khớp screen), đơn vị = pixel ở zoom 1.
- Camera position = điểm world hiển thị ở tâm màn hình.
- Thời gian: giây (f32); góc: radian; màu: RGBA8.

### 8. Errors

- Crate nào error enum của crate đó (`AssetError`, `WavError`, `PngError`, `InflateError`, `PlatformError`) + `impl Display`.
- API trả `Result` cho lỗi môi trường (file, decode); `Option` cho lỗi tra cứu (entity chết, handle không tồn tại).

## Quy tắc khi thêm API mới

1. MUST thêm vào `prelude` nếu game code cần dùng trực tiếp.
2. MUST có doc comment tiếng Anh (rustdoc) + ví dụ nếu API chính.
3. MUST cập nhật `/docs/02-requirements/functional-requirements.md` (FR mới) + `traceability-matrix.md` (test mới) trong cùng task.
4. MUST thêm test headless chứng minh hành vi.
5. Đổi breaking API public MUST có ghi chú trong release notes.

## References

- `/docs/04-engineering/coding-standards.md`
- `/docs/04-engineering/local-development.md`
- `/docs/02-requirements/traceability-matrix.md`
- `/monster-engine/crates/monster-engine/src/lib.rs` (prelude — nguồn sự thật bề mặt API)
