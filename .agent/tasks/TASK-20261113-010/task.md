# Task

- Task ID: `TASK-20261113-010`
- Status: `EXECUTING`
- Owner: `@agent`

## Objective

Mở rộng `monster-engine` hỗ trợ đầy đủ taxonomy vật lý: cửa sổ (đã có), 14 loại body, joints/constraints, 15 loại collider — hiện thực thật bằng Rust 2D-first, zero-dependency, có test cho từng loại.

## Scope

- Crate `monster-physics`: module mới `shape`, `collision`, `body`, `world`, `joint`, `raycast`, `particles`, `vehicle`.
- Docs: thêm FR mới + traceability vào `/docs/02-requirements/` (tiếng Việt).

## Non-goals

- 3D thật (M3), CCD, sleeping/islands, multithread, sửa renderer/window.

## Acceptance criteria

| ID | Criterion | Required evidence |
| --- | --- | --- |
| AC-001 | Cửa sổ (window) đã hỗ trợ và vẫn hoạt động — không regression | Code `monster-platform` + toàn bộ test cũ PASS |
| AC-002 | 15 loại collider tồn tại trong `ColliderShape2` (map 2D ghi rõ cho Sphere/Cylinder/Cone/Mesh/Terrain...) + narrowphase thật + ≥ 1 test mỗi loại | Code + test output |
| AC-003 | 14 loại body tồn tại trong `BodyKind2` + hành vi thật + ≥ 1 test mỗi loại | Code + test output |
| AC-004 | Joints/constraints: Distance, Pin, Weld, Spring hoạt động thật | Tests PASS |
| AC-005 | `PhysicsWorld2` pipeline: integrate → broadphase → narrowphase → resolve (impulse + positional) → joints; event Area/Trigger; raycast API | Code + tests PASS |
| AC-006 | API cũ `monster-physics` không phá vỡ (Aabb2, integrate, bounce, find_collisions) | Test cũ PASS |
| AC-007 | Docs tiếng Việt cập nhật: FR-032.. mới + traceability matrix map tới test mới | File docs sửa |
| AC-008 | `cargo test --workspace` exit 0 (toàn bộ test cũ + mới) | `logs/` |
| AC-009 | Final quality gates PASS (exit 0) | `logs/final-gate.log` |

## References

- `/AGENTS.md`
- `/docs/02-requirements/functional-requirements.md`
- `/docs/03-architecture/adr/ADR-001-monster-engine-foundation.md`
