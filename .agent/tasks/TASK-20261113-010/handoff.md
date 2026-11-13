# Handoff Checkpoint

- Task ID: `TASK-20261113-010`
- Status: `COMPLETED`
- Agent: `@agent`
- Updated: `2026-11-14 01:20 +07:00`

## Current phase

`P7` (đã `DONE` — task hoàn tất)

## Last completed

- P1: `shape.rs` — ColliderShape2 15 loại + Transform2 + geometry helpers (9 tests).
- P2: `collision.rs` — flatten-to-primitives + narrowphase mọi cặp + dispatcher (19 tests). Fix bug thật: SAT phải abort khi bất kỳ axis tách đôi.
- P3: `body.rs` (BodyKind2 × 14, PhysicsBody2) + `world.rs` (PhysicsWorld2 pipeline, trigger events, move_and_slide, contact_between).
- P4: `joint.rs` — Distance/Pin/Weld/Spring (impulse + Baumgarte, clamp) + add_bone + build_articulated_chain. Fix bug thật: dấu positional correction ngược gây NaN.
- P5: `raycast.rs` (Ray2/RayHit2, world + per-shape) + `Vec2::normalize_or` lên monster-math. Fix bug thật: circle_obb inside-case normal đảo chiều.
- P6: `particles.rs` (Soft/Cloth/Fluid/Particle/Deformable — PBD, anti-tunneling velocity clamp + displacement bound + collision-last) + `vehicle.rs` (suspension mass-scaled + drive).
- P7: docs FR-032..FR-048 + traceability + roadmap M2.5; final gates PASS; commit.

## Next action

- Không còn action tự động. Đề xuất: (a) demo physics vào sandbox example; (b) sync PhysicsWorld2 ↔ scene ECS qua facade; (c) M3 GPU/3D.

## Decisions

- D-01..D-06 xem `spec.md`.
- Bug thật đã sửa trong quá trình: SAT separation-abort, circle_obb inside normal, joint sign, particle tunneling clamp — tất cả có test bảo vệ.

## Changed files

- `/monster-engine/crates/monster-physics/src/*` (8 module mới + lib.rs).
- `/monster-engine/crates/monster-math/src/vec.rs` (thêm `Vec2::normalize_or`).
- `/docs/02-requirements/functional-requirements.md`, `/docs/02-requirements/traceability-matrix.md`, `/docs/01-product/roadmap.md`.
- `/.agent/tasks/TASK-20261113-010/*`.

## Validation

- `logs/test.log`: cargo test --workspace exit 0 — 158 passed / 0 failed.
- `logs/final-gate.log`: final gates PASS (exit 0).
- `cargo fmt` sạch.

## Blockers

- Không có.

## Handoff note

Hệ vật lý đầy đủ taxonomy đã chạy trong `monster-physics`. Cảnh báo đã ghi trong verification limitations: linear-only (không angular impulse), không CCD, broadphase brute-force. Khi dùng: tạo `PhysicsWorld2`, `add_body` từng `PhysicsBody2`, gọi `step(dt)` mỗi frame fixed-step, đọc `drain_events()`.

## References

- `/AGENTS.md`
- `/docs/02-requirements/functional-requirements.md`
- `/monster-engine/crates/monster-physics/`
