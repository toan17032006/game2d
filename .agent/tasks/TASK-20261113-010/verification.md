# Verification

- Task ID: `TASK-20261113-010`
- Status: `COMPLETED`

## Acceptance coverage

| ID | Result | Evidence |
| --- | --- | --- |
| AC-001 | PASS | Cửa sổ đã hỗ trợ từ v0.1.0 (FR-009/FR-012, commit `e1aff19`); không regression — toàn bộ 105 test cũ + 53 mới PASS (`logs/test.log`), window code không bị sửa |
| AC-002 | PASS | 15 loại collider trong `ColliderShape2` (mapping 2D ghi rõ trong module doc + FR-033): Circle/Box/Stadium/Cone2/Plane/ConvexPolygon/ConcavePolygon/Heightfield/Terrain/Compound/Wheel/Custom; narrowphase từng cặp có test (`collision::tests` 12 tests + `shape::tests` 9 tests) |
| AC-003 | PASS | 14 loại body trong `BodyKind2` (`body::tests::body_kinds_cover_taxonomy` đếm đúng 14); hành vi thật từng loại: Static/Rigid (`rigid_falls_and_lands`), Kinematic/Animatable (`kinematic_pushes_dynamic`, `animatable_pushes...`), Character (`character_move_and_slide_walks_on_floor`, `character_stops_at_wall`), Area (`area_trigger_events_fire_and_exit`), Bone (`bone_body_uses_stadium`), Soft (`soft_blob_falls_stays_connected`), Cloth (`cloth_pinned_top_hangs`), Vehicle (`vehicle_settles_on_suspension`, `vehicle_drives_forward`), Articulated (`articulated_chain_hangs_together`), Particle (`particle_cloud_lands_on_ground`), Fluid (`fluid_stays_on_ground_and_spreads`), Deformable (`deformable_squashes_then_recovers_shape`) |
| AC-004 | PASS | 4 joints có test PASS: `distance_joint_holds_length_under_gravity` (±6px), `pin_joint_keeps_anchors_together` (<8px), `weld_locks_relative_angle` (<0.3 rad), `spring_pulls_towards_rest_length` (60..160) |
| AC-005 | PASS | `PhysicsWorld2::step` pipeline (world.rs) + `PhysicsEvent2` (trigger test) + raycast (`raycast::tests` 5 tests) |
| AC-006 | PASS | Public API cũ nguyên vẹn: 7 legacy tests PASS (intersects/separation/bounce/broadphase) |
| AC-007 | PASS | `/docs/02-requirements/functional-requirements.md` thêm FR-032..FR-048; `/docs/02-requirements/traceability-matrix.md` thêm ma trận physics v2 map test thật; `/docs/01-product/roadmap.md` thêm milestone M2.5 ✅ |
| AC-008 | PASS | `logs/test.log`: `cargo test --workspace` exit 0 — 158 passed / 0 failed |
| AC-009 | PASS | `logs/final-gate.log`: final gates exit 0 (PASS) |

## Commands

| Command / method | Result | Evidence | Acceptance coverage | Limitations |
| --- | --- | --- | --- | --- |
| `cargo test --workspace --manifest-path monster-engine/Cargo.toml` | PASS | `logs/test.log` (158 passed, 0 failed, exit 0) | AC-001..AC-006, AC-008 | Không có |
| `powershell -NoProfile -ExecutionPolicy Bypass -File .agent/scripts/run-quality-gates.ps1 -TaskId TASK-20261113-010 -Mode final` | PASS | `logs/final-gate.log` | AC-009 | Không có |
| `cargo fmt` (trước khi test) | PASS | format sạch toàn workspace | NFR-09 | Không có |

## Coverage

- 14/14 body kind có ít nhất 1 test hành vi thật.
- 15/15 collider type có ít nhất 1 test narrowphase thật (qua dispatcher `collide`).
- 4/4 joints có test hành vi; raycast 5 test; character 2 test; particle systems 5 test; vehicle 3 test.
- Bug thật phát hiện + sửa + có test bảo vệ: SAT phải hủy contact khi BẤT KỲ axis tách đôi (không chỉ bỏ qua), normal inside-case của `circle_obb` bị đảo chiều, dấu positional correction joint bị ngược (bơm năng lượng → NaN), particle tunneling (velocity clamp + displacement bound + collision last).

## Limitations

- Linear-only dynamics: không có angular impulse (friction không tạo mô-men quay) — đã ghi trong spec D-03.
- Không CCD, không sleeping/islands, không multithread — non-goals của task.
- Broadphase brute-force O(n²) — đủ cho foundation; uniform grid là điểm nâng cấp đã ghi.
- Physics world standalone (không tự sync ECS scene) — facade glue là việc của game code / task sau.

## Final status

`COMPLETED`

## References

- `/AGENTS.md`
- `/docs/02-requirements/functional-requirements.md` (FR-032..FR-048)
- `/monster-engine/crates/monster-physics/` (product)
