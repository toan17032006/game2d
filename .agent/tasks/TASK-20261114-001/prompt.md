# Prompt

## Input classification

`DIRECT` — người dùng yêu cầu trực tiếp trong phiên: "Commit và thực hiện đề xuất" — tức 2 đề xuất cuối TASK-20261113-010: (1) sync `PhysicsWorld2` ↔ scene ECS qua facade; (2) demo physics vào sandbox.

## Role

Rust game engine developer, mở rộng facade `monster-engine`.

## Objective

1. **PhysicsBridge**: đồng bộ scene ECS (`monster-scene`) ↔ `PhysicsWorld2` qua facade — entity có component link nhận transform từ physics body (dynamic) / đẩy transform sang body (static).
2. **Engine tích hợp**: `EngineBuilder::with_physics(gravity)` — Engine sở hữu `Option<PhysicsWorld2>`, step physics trong fixed-step loop, `UpdateContext.physics` cho game code.
3. **Sandbox demo**: chế độ `--physics` — ground + quả bóng rơi có restitution + Area trigger log events, chạy `--headless N` exit 0 và assert bóng đọng trên mặt đất.
4. Commit toàn bộ.

## Context

- TASK-20261113-010 đã COMPLETED + commit `725cf07`: physics taxonomy đầy đủ.
- 158 tests PASS toàn workspace.
- `.gitignore` có thay đổi của người dùng (`test-game-engine`) — bảo toàn, commit kèm.

## Inputs

- Yêu cầu trực tiếp của người dùng trong phiên.
- Code `monster-engine` facade + `monster-physics`.

## Scope

- `crates/monster-engine/src/physics_bridge.rs` (mới) + `engine.rs` (with_physics, contexts, run loop) + `lib.rs` (exports).
- `examples/sandbox.rs` (--physics mode).
- Tests facade mới (physics bridge).
- `/docs/02-requirements/` FR-049 + traceability (docs tiếng Việt đã được người dùng yêu cầu duy trì trong phiên).
- Commit git.

## Non-goals

- Không viết physics engine mới; không renderer 3D; không sửa `monster-physics` core.

## Constraints

- Zero external crate; giữ 158 tests cũ PASS.
- Bridge theo hướng: dynamic body → scene Transform (simulation thắng); static → scene → physics (authoring thắng).

## Execution rules

- Preflight trước EXECUTING; final gates trước COMPLETED; commit sau final.

## Output contract

- Bridge + engine integration + sandbox mode + tests + docs FR-049 + commit.

## Acceptance criteria

Xem `task.md` (AC-001..AC-005).

## Verification method

- `cargo test --workspace` exit 0; final gates exit 0; `git log` có commit.

## References

- `/AGENTS.md`
- `/docs/03-architecture/adr/ADR-001-monster-engine-foundation.md`
- `/monster-engine/crates/monster-engine/src/engine.rs`
