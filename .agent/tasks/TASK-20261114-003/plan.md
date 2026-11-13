# Plan

- Status: `EXECUTING`
- Objective: Pull main sang game + nhân vật điều khiển được qua physics bridge.

## Phases

| Phase | Nội dung | Status |
| --- | --- | --- |
| P1 | Task workspace + preflight gate | `COMPLETED` |
| P2 | Đổi tên task local 001→002 (tránh trùng ID với main) + ff-merge origin/main vào game + cập nhật local main | `COMPLETED` |
| P3 | Implement: `player.rs` (Player/PlayerInput/player_system/autodrive), `defs.rs` kind Player, `game.rs` tích hợp physics (with_physics, Character body, static colliders), scene.csv hàng player, tests | `COMPLETED` |
| P4 | VERIFYING: cargo test (13/13) + headless `--autodrive` exit 0 (min_x=-461 max_x=461 max_y=236) + headless chuẩn exit 0 (403/403) | `COMPLETED` |
| P5 | Final gate + hoàn tất artifacts; mở `--window` cho user | `COMPLETED` |

## Dependencies

- P2 trước P3 (code cần API physics mới từ main). P3 trước P4. P4 trước P5.

## Checkpoints

- CP1: preflight exit 0.
- CP2: `git log game` = 764b60d; task workspace 002 tồn tại; không còn 001 local.
- CP3: `cargo check --all-targets` sạch warning.
- CP4: tests PASS; autodrive log thể hiện player đổi vị trí; headless chuẩn exit 0.
- CP5: final gate exit 0; game window mở thành công.

## Risks

- Va chạm engine Character body không hoàn hảo (nhân vật dính góc) → chỉnh collider half-extents nhỏ hơn sprite; gravity 0 nên không rơi.
- API engine mới có breaking change với game crate → cargo check sớm ngay sau merge.

## Rollback

- `git reset --hard cda2ed8` trên nhánh game (chưa có commit nào của ta); game/ là untracked nên an toàn.

## References

- `/AGENTS.md`
- `/docs/00-governance/agent-execution-standard.md`
