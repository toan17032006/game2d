# Prompt

## Role

Gameplay engineer làm việc trên repository `game2d`, nhánh `game`, crate `game/` (Tiny Swords demo trên `monster-engine`).

## Objective

Pull những thành phần mới nhất từ nhánh `main` sang nhánh `game` để tối ưu hóa game, và cho một nhân vật trong game điều khiển được (di chuyển bằng bàn phím, v.v).

## Input classification

- `DIRECT` — yêu cầu rõ ràng, đủ input; không cần hỏi thêm.

## Context

- `origin/main` tiến 2 commit mới: `725cf07` (physics taxonomy đầy đủ: 14 body kinds, 15 colliders, joints, particles, raycast, vehicle) và `764b60d` (physics bridge ECS ↔ PhysicsWorld2: `PhysicsLink`, `EngineBuilder::with_physics`, `ctx.physics`, engine tự sync scene↔physics mỗi fixed step).
- Commit `764b60d` của main cũng chứa task workspace `.agent/tasks/TASK-20261114-001/` trùng tên với task workspace local đã COMPLETED của phiên này → cần đổi tên task local thành `TASK-20261114-002` trước khi merge.
- Game hiện tại: scene demo data-driven (163 entities), unit NPC đi lại tự động, chưa có nhân vật điều khiển được.

## Inputs

- `origin/main` (764b60d) — engine physics + bridge.
- `monster-core` input API: `InputState::is_down(KeyCode)` (WASD + Arrow keys có sẵn).
- `game/` — crate game hiện tại từ TASK-20261114-002 (đã đổi tên).

## Scope

- Merge `origin/main` vào nhánh `game` (fast-forward) và cập nhật local `main`.
- Nhân vật điều khiển được: WASD/Arrow keys di chuyển, animation idle↔run tự chuyển theo trạng thái, giới hạn biên và va chạm với building qua physics bridge (`BodyKind2::Character` + static colliders).
- Chế độ `--autodrive` headless: inject chuỗi input kịch bản để chứng minh end-to-end pipeline input → velocity → physics → transform → sprite.

## Non-goals

- Không sửa engine `monster-engine/` (dùng nguyên trạng thái từ main).
- Không làm camera follow, jump, attack, AI NPC mới.
- Không commit/push lên remote (chỉ fast-forward branch pointer khi merge).

## Constraints

- Crate `game` giữ dependency chỉ `monster-engine` (path) + std.
- Giữ nguyên toàn bộ acceptance criteria của TASK-20261114-002 (403 assets, manifest, headless validation vẫn phải PASS).
- Chạy quality gates preflight/final theo AGENTS.md.

## Output

- Nhánh `game` = 764b60d (chứa physics bridge), task local đổi tên thành TASK-20261114-002.
- `game/src` có player system; `game/data/scene.csv` có player spawn; tests cập nhật; headless `--autodrive` exit 0 với bằng chứng nhân vật di chuyển.

## Acceptance

- Xem `task.md` — các tiêu chí `AC-NNN`.

## Verification

- `cargo test` pass; `run-headless.ps1` exit 0 (có `--autodrive`, log xác nhận player di chuyển thực sự); final gate exit 0.
- Mở `--window` để user chơi trực tiếp.

## References

- `/AGENTS.md`
- `/docs/00-governance/agent-execution-standard.md`
- `/monster-engine/crates/monster-engine/src/physics_bridge.rs`
- `/monster-engine/crates/monster-physics/src/body.rs`
