# Plan

- Status: `EXECUTING`
- Objective: Bóc tách assets + dựng crate game (code + data) trong `game/`, xác minh bằng headless run.

## Phases

| Phase | Nội dung | Status |
| --- | --- | --- |
| P1 | Task workspace + preflight gate | `COMPLETED` |
| P2 | Bóc tách assets: script `scripts/extract-assets.ps1` (IHDR + header aseprite), copy + rename snake_case + sinh `game/data/assets.csv`; đối chiếu số file 813 | `EXECUTING` |
| P3 | Scene data: đọc trực quan `Tilemap_color1.png` chọn tọa độ tile; viết `game/data/units.csv` + `game/data/scene.csv` | `PLANNED` |
| P4 | Code game: `game/Cargo.toml`, `src/main.rs`, `src/manifest.rs`, `src/defs.rs`, `src/anim.rs`, `src/game.rs`, unit tests | `COMPLETED` |
| P5 | VERIFYING: `cargo build`, `cargo test` (7/7 PASS), headless run dev + release (exit 0, 403/403 textures) | `COMPLETED` |
| P6 | Final gate + hoàn tất verification/evaluation/handoff | `COMPLETED` |

## Dependencies

- P2 phụ thuộc P1 (preflight PASS).
- P3 phụ thuộc P2 (key manifest có thật).
- P4 phụ thuộc P2 (danh sách asset) và P3 (schema data).
- P5 phụ thuộc P4. P6 phụ thuộc P5.

## Checkpoints

- CP1: preflight gate exit 0.
- CP2: 813 file trong `game/assets`, 813 dòng manifest, đúng đếm theo category.
- CP3: data files tham chiếu key hợp lệ.
- CP4: build + test pass.
- CP5: headless run exit 0 với log validation PASS.
- CP6: final gate exit 0, `verification.md` đầy đủ AC → PASS.

## Risks

- PNG decoder lỗi với file PNG nào đó trong pack → ghi nhận, loại file lỗi khỏi runtime set với lý do trong evidence (fallback đã sẵn sàng; hiện decoder phủ toàn bộ pack đã khảo sát).
- Thời gian decode 813 PNG trong run dev profile → chấp nhận, verify-product timeout 900s.
- Nhầm tọa độ tile → xác minh bằng đọc ảnh trực quan trước khi chốt.

## Rollback

- Xóa `game/` và làm lại từ script; không ảnh hưởng nguồn `assest/` hay engine.

## References

- `/AGENTS.md`
- `/docs/00-governance/agent-execution-standard.md`
- `/monster-engine/crates/monster-engine/src/engine.rs`
