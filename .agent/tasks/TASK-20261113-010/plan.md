# Plan

- Task ID: `TASK-20261113-010`
- Status: `EXECUTING`

## Objective

Hiện thực hệ vật lý 2D đầy đủ (14 body + 4 joints + 15 collider + pipeline + raycast) trong `monster-physics`, có test từng loại, cập nhật docs.

## Phases

| ID | Action | Input | Expected output | Validation | Status |
| --- | --- | --- | --- | --- | --- |
| P0 | Task workspace + preflight | Yêu cầu user | 7 artifact + preflight PASS | gates preflight exit 0 | `DONE` |
| P1 | `shape.rs`: ColliderShape2 (15 loại) + geometry helpers (support, bound, closest point) | D-01 | Shape enum + helpers | `cargo test -p monster-physics` exit 0 | `DONE` |
| P2 | `collision.rs`: narrowphase mọi cặp shape → Contact {normal, penetration, points} | D-01/D-03 | Narrowphase + tests | tests PASS | `DONE` |
| P3 | `body.rs` + `world.rs`: BodyKind2, PhysicsBody2, PhysicsWorld2 pipeline (Static/Rigid/Kinematic/Area/Animatable), events | D-02/D-03/D-05 | World pipeline + tests | tests PASS | `DONE` |
| P4 | `joint.rs`: Distance/Pin/Weld/Spring + Bone (chain) + Articulated (builder) | D-03 | Joints + tests | tests PASS | `DONE` |
| P5 | Character move_and_slide + `raycast.rs` + Custom collider trait | D-01 | Character/raycast/custom + tests | tests PASS | `DONE` |
| P6 | `particles.rs` (Soft/Cloth/Fluid/Particle/Deformable — PBD) + `vehicle.rs` (suspension) | D-02/D-04 | Particle systems + vehicle + tests | tests PASS | `DONE` |
| P7 | Docs FR-032.. + traceability + final gates | Toàn bộ output | Docs cập nhật + gates PASS | final gates exit 0 | `DONE` |

## Dependencies

- P1 → P2 → P3 → P4/P5/P6 → P7.

## Checkpoints

- CP-1 sau P2: toàn bộ shape + narrowphase có test.
- CP-2 sau P4: world pipeline + joints chạy.
- CP-3 sau P6: đủ 14 body + 15 collider + joints có test.
- CP-4 sau P7: gates PASS.

## Risks

- R-01: Narrowphase OBB/polygon phức tạp → SAT chuẩn + MTV; test từng cặp.
- R-02: PBD unstable với dt lớn → substep cố định + damping.
- R-03: Scope lớn → mỗi phase build+test riêng, không dồn cuối.

## Rollback

- Code mới nằm trong crate `monster-physics` (tracked) — revert commit trả về v0.1.0.

## References

- `/AGENTS.md`
- `/docs/03-architecture/adr/ADR-001-monster-engine-foundation.md`
