# Specification

## Inputs

- Nguồn: `assest/Tiny Swords (Free Pack)/` — 410 PNG thật + file `.aseprite` + rác `__MACOSX/` (thư mục `assest/__MACOSX` chứa thêm 410 file `._*.png` AppleDouble, không thuộc pack thật).
- Engine: `monster-engine/crates/monster-engine` (facade `GameApp`, `EngineBuilder`, `Assets::load_file`, `Sprite.src`, `sprite_draw_order`, `movement_system`).
- Metadata đã đo trong khảo sát (evidence trong `logs/asset-survey.txt`).

## Confirmed facts

- PNG decoder của engine hỗ trợ bit depth 8, non-interlaced, grayscale/RGB/palette/gray+alpha/RGBA — phủ toàn bộ pack.
- Frame unit: canvas `.aseprite` Warrior/Archer/Monk/Pawn = 192x192, Lancer = 320x320; mọi strip unit chia hết cho canvas width (tổng frames khớp header: Warrior 28, Pawn 131, Monk 32, Archer 19, Lancer 63).
- Sheep: 6+12+4=22 ✓; Gold Stones: 6 static + 6×6 highlight = 42 ✓; Bushes: 4×8=32 ✓; Water Foam: 3072/192=16 ✓; Water Rocks: mỗi file 1024/64=16 ✓; Rubber Duck: 96/32=3 ✓; Trees+Stumps: 8×4 tree + 4 stump = 36 ✓; Clouds: 8 file = 8 frame ✓.
- Particle FX: tổng 75 frame = 8+10+8+10+8+10+12+9 (frame vuông theo chiều cao strip 64 hoặc 192) ✓.
- Renderer hỗ trợ blit sub-rect (`blit_sprite(texture, screen, src, opts)`); world +y hiển thị xuống đáy màn hình; camera center = screen center.
- `.gitignore` đã phủ `target`, rác macOS.

## Inferences

- Banner trong `UI Elements/UI Banners from the store page/` là ảnh marketing store page, không phải UI runtime (nhân bản của `UI Elements/UI Elements/Banners`) → loại khỏi bộ runtime.
- File `.aseprite` chỉ dùng để suy geometry, không copy vào runtime assets.

## Unknowns

- Tọa độ tile cỏ trong `Tilemap_color1.png` chưa có metadata → xác định bằng đọc ảnh trực tiếp (visual) trước khi viết `scene.csv`.

## Constraints

- Crate game chỉ phụ thuộc `monster-engine` (path `../monster-engine/crates/monster-engine`) + std.
- Không xóa/sửa `assest/` — bóc tách bằng COPY.
- Đặt tên snake_case; map thủ công duy nhất: `bushe` → `bush`.
- Manifest CSV không quote phức tạp (giá trị không chứa dấu phẩy).
- Game resolve asset/data path từ `env!("CARGO_MANIFEST_DIR")`.

## Decisions

- D1: COPY sang `game/assets/**`, giữ nguyên nguồn `assest/`.
- D2: Cấu trúc đích: `units/<faction>/<unit>/`, `buildings/<faction>/`, `terrain/tileset|decorations/<kind>|resources/<type>/[sub]/`, `fx/`, `ui/<group>/`.
- D3: Geometry ghi trong `assets.csv` sinh tự động từ IHDR PNG + header `.aseprite` (script PowerShell trong task workspace), quy tắc: strip có nguồn aseprite → `frame_w = canvas_w`, `frames = png_w / canvas_w`; FX → frame vuông `frame_h`, `frames = png_w / frame_h`; PNG == canvas → 1 frame tĩnh.
- D4: `--headless N` load + validate TOÀN BỘ manifest (đủ bằng chứng bộ assets dùng được), scene demo spawn từ `scene.csv`.
- D5: Tốc độ di động unit là giá trị demo (nêu rõ trong `units.csv` header comment), không phải cân bằng gameplay.

## Output contract

- `game/assets/**` — 403 PNG.
- `game/data/assets.csv` — cột: `key,category,path,frame_w,frame_h,frames`; key = đường dẫn tương đối `assets/` bỏ phần mở rộng.
- `game/data/units.csv` — cột: `faction,unit,idle,run,speed,frame_w,frame_h`.
- `game/data/scene.csv` — cột: `kind,ref,x,y,z,scale,fps,src_x,src_y`.
- `game/Cargo.toml` + `game/src/{main.rs,manifest.rs,anim.rs,game.rs}` + test unit cho parser/validator.

## Test Command

`powershell -NoProfile -ExecutionPolicy Bypass -File .agent/tasks/TASK-20261114-002/scripts/run-headless.ps1 -Frames 120`

## References

- `/AGENTS.md`
- `/docs/00-governance/agent-execution-standard.md`
- `/monster-engine/crates/monster-engine/src/engine.rs`
- `/monster-engine/crates/monster-assets/src/registry.rs`
- `/monster-engine/crates/monster-render/src/software.rs`
