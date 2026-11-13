# Task

- Task ID: `TASK-20261114-002`
- Status: `EXECUTING`

## Objective

Bóc tách asset pack thô `assest/Tiny Swords (Free Pack)` thành bộ assets dùng được trong game và dựng crate game hoàn chỉnh (code + dữ liệu) trong thư mục `game/` chạy trên `monster-engine`.

## Scope

- `game/assets/**` — PNG runtime đã lọc (loại `__MACOSX`, `.aseprite`, banner store page), tên snake_case, cấu trúc category.
- `game/data/**` — `assets.csv`, `units.csv`, `scene.csv`.
- `game/Cargo.toml` + `game/src/**` — crate `game` (bin `tiny-swords`) dùng `GameApp` của engine.
- Script bóc tách + parser metadata `.aseprite` trong task workspace.

## Non-goals

- Không thay đổi `assest/`, `monster-engine/`, `docs/`.
- Không thêm dependency ngoài std + `monster-engine`.
- Không làm tilemap editor hoặc map gameplay đầy đủ.

## Acceptance criteria

| AC | Tiêu chí |
| --- | --- |
| AC-001 | `game/assets/` chứa toàn bộ PNG runtime của pack: 410 PNG thật (ngoài `__MACOSX`) − 7 banner store page = 403 file; không còn file `__MACOSX` (`.AppleDouble`) hay `.aseprite`. |
| AC-002 | Tên file và thư mục chuẩn snake_case; cấu trúc `units/<faction>/<unit>/`, `buildings/<faction>/`, `terrain/**`, `fx/`, `ui/<group>/`. |
| AC-003 | `game/data/assets.csv` liệt kê đủ 403 key với `category,path,frame_w,frame_h,frames`; với mọi dòng: `frame_w > 0`, `frames >= 1`, và geometry khớp chiều rộng PNG thật (`frames * frame_w == width` với strip, hoặc `frame_w == width` với ảnh tĩnh). |
| AC-004 | `game/data/units.csv` định nghĩa 25 unit (5 faction × 5 loại) tham chiếu key idle/run tồn tại trong `assets.csv`; `scene.csv` định nghĩa scene demo tham chiếu key hợp lệ. |
| AC-005 | Crate `game` build thành công, `cargo test` pass, phụ thuộc chỉ `monster-engine` (path) + std. |
| AC-006 | Chạy `cargo run --manifest-path game/Cargo.toml -- --headless 120` exit 0, log xác nhận: manifest validation PASS cho 403 asset, scene spawn, loop chạy đủ frame. |
| AC-007 | `assest/` không đổi (kiểm tra `git status` chỉ cho thấy file mới trong `game/` và task workspace). |

## References

- `/AGENTS.md`
- `/docs/00-governance/agent-execution-standard.md`
- `/docs/09-reference/templates/agent-task-template.md`
