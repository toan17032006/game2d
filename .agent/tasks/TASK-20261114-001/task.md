# Task

- Task ID: `TASK-20261114-001`
- Status: `EXECUTING`
- Owner: `@agent`

## Objective

Thực hiện 2 đề xuất sau TASK-20261113-010: PhysicsBridge (ECS ↔ PhysicsWorld2 qua facade) và demo physics vào sandbox; commit toàn bộ.

## Scope

- `physics_bridge.rs` mới trong facade + tích hợp `EngineBuilder::with_physics` + `UpdateContext.physics`.
- Sandbox `--physics` mode.
- Docs FR-049 + traceability.

## Non-goals

- Sửa `monster-physics` core; renderer 3D; CCD.

## Acceptance criteria

| ID | Criterion | Required evidence |
| --- | --- | --- |
| AC-001 | `PhysicsBridge` đồng bộ 2 chiều đúng hướng: dynamic body → Transform; static Transform → body | Bridge tests PASS |
| AC-002 | `EngineBuilder::with_physics(gravity)` + `UpdateContext.physics` hoạt động: entity có link rơi theo physics qua `engine.run` | Integration test PASS |
| AC-003 | Sandbox `--physics`: headless N frames exit 0, bóng đọng trên mặt đất, trigger events được ghi | Output lệnh run |
| AC-004 | Docs FR-049 + traceability cập nhật tiếng Việt | File docs |
| AC-005 | `cargo test --workspace` exit 0 (≥ 161 tests) + final gates PASS + commit git | `logs/` + `git log` |

## References

- `/AGENTS.md`
- `/.agent/tasks/TASK-20261113-010/handoff.md`
