# Personas & Use Cases — monster-engine

- Owner: Product
- Status: `Approved`
- Last updated: `2026-11-13`
- Review date: `2027-05-13`

## Purpose

Xác định ai dùng engine và các kịch bản sử dụng chính, làm căn cứ cho functional requirements.

## Personas

### P-01 — Game Developer học engine (persona chính)

- **Bối cảnh:** đang học cách một engine hoạt động từ trong ra ngoài; muốn viết game 2D bằng Rust.
- **Nhu cầu:** API nhỏ dễ hiểu; ví dụ chạy được; tài liệu tiếng Việt; kiểm chứng tự động để tự tin sửa code.
- **Thành công:** viết được game nhỏ (sprite di chuyển, va chạm, âm thanh) trên nền móng mà không phải sửa engine core.

### P-02 — Engine Developer / Agent AI (người duy trì engine)

- **Bối cảnh:** phát triển engine theo workflow `/AGENTS.md` (task workspace, gates).
- **Nhu cầu:** biên crate rõ ràng; test headless chạy nhanh; contract trait ổn định để mở rộng.
- **Thành công:** thêm backend/tính năng mới chỉ bằng crate/module mới, gates PASS.

### P-03 — Người chơi cuối

- **Bối cảnh:** chạy demo/game đóng gói từ engine.
- **Nhu cầu:** cửa sổ game mở nhanh, điều khiển bàn phím/chuột mượt.
- **Thành công:** `cargo run --example sandbox -- --window` mở cửa sổ 960×540 chơi được.

## Use cases chính

### UC-01 — Viết game mới trên engine (P-01)

1. Tạo crate/binary mới, phụ thuộc `monster-engine` (facade).
2. Implement `GameApp` (init/update/render).
3. `EngineBuilder::new().with_window(...).build()` rồi `engine.run(&mut game)`.
4. Spawn entity trong `init`, điều khiển trong `update` qua `InputState`, vẽ thêm trong `render`.
- **Kết quả:** game chạy với vòng lặp fixed timestep do engine quản lý.

### UC-02 — Nạp sprite từ Tiny Swords (P-01)

1. `Assets::load_file(path/*.png)` → `Handle<Texture>`.
2. Spawn entity với component `Sprite { texture, size, src }` (`src` chọn frame trong spritesheet).
3. Engine tự vẽ theo `ZOrder`.
- **Kết quả:** sprite hiển thị đúng, không cần viết decoder.

### UC-03 — Kiểm chứng engine không GUI (P-02)

1. `cargo test --workspace` — toàn bộ test headless (NullWindow, framebuffer assertions).
2. Test đọc pixel framebuffer để khẳng định blend/camera đúng.
- **Kết quả:** CI/agent chạy được không cần màn hình.

### UC-04 — Thêm backend mới (P-02)

1. Implement trait (`Renderer`/`AudioBackend`/`PlatformWindow`) trong crate mới.
2. Đăng ký qua `EngineBuilder`.
- **Kết quả:** core không đổi; test hoán đổi backend PASS.

### UC-05 — Chạy demo (P-03)

1. `cargo run --example sandbox -- --window` (hoặc `--headless 120` cho smoke test).
2. Đóng cửa sổ hoặc ESC để thoát sạch.
- **Kết quả:** exit code 0, log tổng kết ra stderr.

## Anti use-cases (không hỗ trợ)

- Dựng game bằng editor GUI.
- Game multiplayer/network.
- Chạy trên nền tảng khác Windows (hiện tại).

## References

- `/docs/01-product/vision-and-goals.md`
- `/docs/02-requirements/functional-requirements.md`
- `/docs/04-engineering/local-development.md`
- `/AGENTS.md`
