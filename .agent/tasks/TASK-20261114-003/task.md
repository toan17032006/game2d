# Task

- Task ID: `TASK-20261114-003`
- Status: `EXECUTING`

## Objective

Pull thành phần mới nhất từ `origin/main` sang nhánh `game` (physics taxonomy + physics bridge) và thêm nhân vật điều khiển được bằng bàn phím (WASD/Arrows) vào game.

## Scope

- Đổi tên task workspace local `TASK-20261114-001` → `TASK-20261114-002` (trùng ID với task đã có trên main), cập nhật nội dung tham chiếu.
- `git merge origin/main` (fast-forward) trên nhánh `game`; cập nhật local `main`.
- Player system trong `game/src`: input → velocity → `PhysicsWorld2` (Character body) → bridge sync → `Transform`; chuyển animation idle↔run theo trạng thái di chuyển.
- Static colliders cho buildings + 4 biên world; `EngineBuilder::with_physics`.
- `--autodrive`: input kịch bản headless để xác minh di chuyển end-to-end.

## Non-goals

- Không sửa engine; không camera follow; không jump/attack/AI mới; không commit/push.

## Acceptance criteria

| AC | Tiêu chí |
| --- | --- |
| AC-001 | Nhánh `game` fast-forward đến 764b60d; `git merge-base --is-ancestor 764b60d game` true; local `main` = 764b60d; task local đổi tên thành TASK-20261114-002 với nội dung tham chiếu cập nhật. |
| AC-002 | Scene có đúng 1 entity `player`; điều khiển bằng WASD + Arrow keys; thả phím thì dừng; animation idle↔run chuyển theo trạng thái. |
| AC-003 | Player là `BodyKind2::Character` trong `PhysicsWorld2` (gravity 0) gắn qua `PhysicsLink`; buildings + 4 biên là static bodies; va chạm giữ player trong world (không xuyên biên, không xuyên house/tower). |
| AC-004 | Chế độ `--autodrive` headless exit 0: player đổi vị trí theo kịch bản input (bằng chứng log so sánh vị trí đầu/cuối từng bước). |
| AC-005 | `cargo test` pass (tests cũ + tests mới cho player/scene kind); acceptance criteria của TASK-20261114-002 vẫn giữ nguyên kết quả: headless run 403/403 textures, 403 assets validation PASS. |
| AC-006 | Preflight + final quality gates PASS (exit 0); artifacts đầy đủ. |

## References

- `/AGENTS.md`
- `/docs/00-governance/agent-execution-standard.md`
- `/monster-engine/crates/monster-engine/src/physics_bridge.rs`
