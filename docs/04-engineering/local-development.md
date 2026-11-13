# Local Development — monster-engine

- Owner: Engineering
- Status: `Approved`
- Last updated: `2026-11-13`
- Review date: `2027-05-13`

## Purpose

Hướng dẫn dựng môi trường và làm việc hằng ngày với `monster-engine`. Mọi lệnh dưới đây đã chạy thật trên máy tham chiếu (Windows 10+, Git Bash + PowerShell).

## Toolchain cần có

| Công cụ | Phiên bản tham chiếu | Ghi chú |
| --- | --- | --- |
| rustc + cargo | 1.99.1 (stable-x86_64-pc-windows-msvc) | link user32/gdi32 qua MSVC libs có sẵn |
| rustfmt | 1.10.0-stable | `cargo fmt` |
| Git Bash | bất kỳ | shell dùng trong lệnh dưới đây |
| PowerShell | 5.1+ | chạy quality gates |

KHÔNG cần: compiler C/C++, .NET, network (zero external crate).

## Cấu trúc workspace

```
monster-engine/
├── Cargo.toml                  # workspace: 9 members
├── crates/
│   ├── monster-math/           # Vec2/3/4, Mat4, Quat, Rect, Color
│   ├── monster-core/           # contracts (traits) + services
│   ├── monster-render/         # software 2D renderer
│   ├── monster-platform/       # Win32 FFI + Window + NullWindow
│   ├── monster-assets/         # registry + PNG/BMP decoder
│   ├── monster-scene/          # ECS-lite + systems
│   ├── monster-physics/        # AABB + integration
│   ├── monster-audio/          # AudioBackend + WAV
│   └── monster-engine/         # facade + examples/sandbox
└── target/                     # build artifacts (gitignored)
```

## Lệnh hằng ngày (chạy từ thư mục `monster-engine/`)

| Mục đích | Lệnh |
| --- | --- |
| Build toàn bộ | `cargo build --workspace` |
| Chạy toàn bộ test | `cargo test --workspace` |
| Test một crate | `cargo test -p monster-math` |
| Chạy sandbox không GUI (smoke test) | `cargo run --example sandbox -- --headless 120` |
| Chạy sandbox có cửa sổ | `cargo run --example sandbox -- --window` |
| Format code | `cargo fmt` |
| Kiểm tra docs API | `cargo doc --workspace --open` |

Lệnh khi chạy từ repository root (dùng trong CI/gates):

```
cargo test --workspace --manifest-path monster-engine/Cargo.toml
cargo run --manifest-path monster-engine/crates/monster-engine/Cargo.toml --example sandbox -- --headless 120
```

## Chạy quality gates (từ repository root)

```
powershell -NoProfile -ExecutionPolicy Bypass -File .agent/scripts/run-quality-gates.ps1 -TaskId TASK-YYYYMMDD-NNN -Mode preflight
powershell -NoProfile -ExecutionPolicy Bypass -File .agent/scripts/run-quality-gates.ps1 -TaskId TASK-YYYYMMDD-NNN -Mode final
```

Gates FAIL khi exit code ≠ 0 — bắt buộc PASS trước khi báo COMPLETED (xem `/AGENTS.md`).

## Ví dụ game nhỏ trên engine (điểm bắt đầu)

```rust
use monster_engine::prelude::*;

struct MyGame;

impl GameApp for MyGame {
    fn init(&mut self, ctx: &mut InitContext<'_>) {
        let tex = ctx.assets.insert_rgba("dot", 8, 8, vec![255, 120, 40, 255].repeat(64));
        ctx.world.spawn()
            .with(Transform::at(Vec2::new(0.0, 0.0)))
            .with(Sprite { texture: tex, size: Vec2::splat(32.0), src: None })
            .build();
    }
    fn update(&mut self, ctx: &mut UpdateContext<'_>) {
        if ctx.input.was_pressed(KeyCode::Escape) { ctx.exit(); }
        movement_system(ctx.world, ctx.dt);
    }
    fn render(&mut self, _ctx: &mut RenderContext<'_>) {}
}

fn main() {
    let mut engine = EngineBuilder::new()
        .with_window(WindowOptions { width: 960, height: 540, title: "My Game".into() })
        .build().unwrap();
    engine.run(&mut MyGame);
}
```

## Sự cố thường gặp

| Triệu chứng | Nguyên nhân | Xử lý |
| --- | --- | --- |
| `link.exe`/user32.lib lỗi khi build platform | Toolchain MSVC thiếu | Dùng rustup toolchain `stable-x86_64-pc-windows-msvc` (đã cấu hình sẵn) |
| Sandbox chạy vô hạn trong test | Headless không tự thoát | App phải gọi `ctx.exit()` (xem `tests/engine_loop.rs`) |
| PNG decode lỗi CRC | File hỏng hoặc chunk bị sửa | Kiểm tra file gốc; decoder verify CRC mọi chunk |
| Test `real_asset_smoke_test` bị skip | Không tìm thấy `/assest/` | Chạy từ đúng vị trí repo (path tính từ CARGO_MANIFEST_DIR) |

## References

- `/AGENTS.md`
- `/docs/04-engineering/development-workflow.md`
- `/docs/04-engineering/coding-standards.md`
- `/docs/05-quality/quality-strategy.md`
- `/monster-engine/crates/monster-engine/examples/sandbox.rs`
