# Prompt

## Role

Game asset pipeline engineer làm việc trên repository `game2d` với engine `monster-engine` (Rust, zero-dependency).

## Input classification

- `DIRECT` — yêu cầu người dùng rõ ràng, đủ input bắt buộc, không cần hỏi thêm; input còn thiếu đã ghi ở mục Inputs.

## Objective

Bóc tách asset pack thô `assest/Tiny Swords (Free Pack)` thành bộ assets dùng được trong game (PNG runtime, tên chuẩn hóa, có manifest geometry), và đặt toàn bộ code game + dữ liệu game trong thư mục `game/`.

## Context

- Engine `monster-engine/` là Cargo workspace đã hoàn thiện facade `GameApp`/`EngineBuilder`, loader PNG/BMP zero-dependency, ECS `Sprite`/`Transform`/`Velocity`/`ZOrder`, software renderer có hỗ trợ src-rect (spritesheet).
- Asset pack thô nằm ở `assest/` (410 PNG thật trong `Tiny Swords (Free Pack)/`, cộng 410 rác `._*.png` trong `__MACOSX/`) kèm file nguồn `.aseprite` và banner marketing của store page.
- Chưa có thư mục game nào dùng pack này.

## Inputs

- `assest/Tiny Swords (Free Pack)/` — asset pack nguồn.
- `monster-engine/` — engine facade qua path dependency.
- Metadata geometry đã đo: frame unit 192x192 (lancer 320x320), quy tắc `frames = strip_width / aseprite_canvas_width`, FX dùng frame vuông theo chiều cao strip.

## Scope

- Tạo `game/assets/**` — bộ PNG đã bóc tách, tên snake_case, cấu trúc theo category (units, buildings, terrain, fx, ui).
- Tạo `game/data/**` — manifest `assets.csv` (key, category, path, geometry) + `units.csv` + `scene.csv`.
- Tạo crate game trong `game/` (Cargo.toml + `src/`) chạy trên `monster-engine`, có chế độ `--headless N` để xác minh tự động và `--window` để chơi.

## Non-goals

- Không xóa, sửa hay di chuyển `assest/` (nguồn giữ nguyên).
- Không sửa engine `monster-engine/`.
- Không sửa Markdown tài liệu dự án.
- Không làm tilemap editor, không dựng map lớn, không thêm dependency ngoài.

## Constraints

- Crate game chỉ phụ thuộc `monster-engine` (path) + std.
- Mọi geometry trong manifest phải suy ra từ dữ liệu đo được (IHDR PNG + header .aseprite), không đoán.
- Asset path resolve từ `CARGO_MANIFEST_DIR` để chạy đúng từ mọi working directory.
- Không ghi secrets hoặc dữ liệu cá nhân.

## Output

- `game/assets/**` + `game/data/*.csv` + `game/Cargo.toml` + `game/src/*.rs`.
- Script bóc tách tái chạy được trong task workspace này.

## Acceptance

- Xem `task.md` — các tiêu chí `AC-NNN`.

## Verification

- `cargo run --manifest-path game/Cargo.toml -- --headless 120` — game tự validate manifest (403 PNG tồn tại, geometry khớp), load toàn bộ, chạy loop, exit 0.
- Quality gates preflight + final theo `AGENTS.md`.

## References

- `/AGENTS.md`
- `/docs/00-governance/agent-execution-standard.md`
- `/docs/03-architecture/adr/ADR-001-monster-engine-foundation.md`
- `/monster-engine/crates/monster-engine/src/engine.rs`
