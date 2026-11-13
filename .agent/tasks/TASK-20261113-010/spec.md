# Specification

## Inputs

- Mục tiêu người dùng: engine hỗ trợ cửa sổ + 14 body + joints + 15 collider (taxonomy kiểu Godot).
- Code `monster-physics` hiện tại: Aabb2, integrate, apply_gravity, bounce, find_collisions.

## Confirmed facts

- F-01: Window đã hỗ trợ từ v0.1.0 (FR-009/FR-012, commit `e1aff19`), test cũ PASS.
- F-02: Kiến trúc 2D-first theo ADR-001 Accepted; 3D thật thuộc M3.
- F-03: 105 tests đang PASS toàn workspace.
- F-04: Người dùng đã yêu cầu docs tiếng Việt đầy đủ trong phiên trước (commit `cda2ed8`) — cập nhật docs thuộc cùng dòng phê duyệt.

## Inferences

- I-1 (inference): taxonomy 3D của Godot được người dùng liệt kê như danh mục "hỗ trợ"; trên nền 2D-first, ánh xạ tương đương 2D có chủ đích là cách hiện thực đúng ngay bây giờ (Sphere→Circle, Cylinder→Stadium, Cone→triangle, Mesh/Convex/Concave→polygon, Terrain→grid heightfield). Ghi rõ trong docs; 3D thật ở M3.
- I-2 (inference): ưu tiên hành vi gameplay thật (rơi, nảy, trượt, va chạm) hơn độ chính xác mô phỏng tuyệt đối (năng lượng bảo toàn, CCD).

## Unknowns

- U-1: Ngưỡng hiệu năng mong muốn không nêu — không đặt SLO; smoke test khối lượng vừa phải (vài trăm body particle).

## Constraints

- C-01: Zero external crate; thuật toán tự viết (SAT, impulse resolution, PBD).
- C-02: Public API cũ giữ nguyên.
- C-03: Linear dynamics v1; angular velocity tồn tại cho collider hướng (OBB) và animatable, nhưng không có angular impulse (ghi limitation).

## Decisions

- D-01: `ColliderShape2` enum: Circle, Box (OBB), Stadium (Capsule/Cylinder 2D), Cone2 (triangle), Plane (half-space), ConvexPolygon, ConcavePolygon (tập convex con), Heightfield (1D), Terrain (grid heightfield), Compound, Wheel (circle + suspension), Custom (trait object).
- D-02: `BodyKind2` enum đủ 14 loại; Static/Rigid/Kinematic/Area/Animatable/Bone chạy trên rigid pipeline; Character dùng move_and_slide riêng; Soft/Cloth/Fluid/Particle/Deformable chạy particle-based (PBD); Vehicle = chassis rigid + wheel suspension.
- D-03: Resolution: sequential impulse (restitution + friction Coulomb) + positional correction (Baumgarte percent); joints solved sau contacts (N iteration).
- D-04: Particles dùng position-based dynamics (PBD): spring constraint satisfaction + va chạm point-vs-shape; Fluid = pressure repulsion lân cận (SPH-lite).
- D-05: Events: `PhysicsEvent2` queue (TriggerEntered/TriggerExited/CollisionStarted) — Area/Trigger không có response.
- D-06: Test Command: `cargo test --workspace --manifest-path monster-engine/Cargo.toml`.

## Output contract

- Module mới trong `monster-physics`: `shape.rs`, `collision.rs`, `body.rs`, `world.rs`, `joint.rs`, `raycast.rs`, `particles.rs`, `vehicle.rs`; re-export qua lib.rs.
- ≥ 40 test mới; docs FR/traceability cập nhật.

## Test Command

`cargo test --workspace --manifest-path monster-engine/Cargo.toml`

## Approved Documentation Changes

- `/docs/02-requirements/functional-requirements.md` — thêm FR-032..FR-047 (physics)
- `/docs/02-requirements/traceability-matrix.md` — map FR mới → test
- `/docs/01-product/roadmap.md` — cập nhật trạng thái M-partial (physics v2)

## References

- `/AGENTS.md`
- `/docs/03-architecture/adr/ADR-001-monster-engine-foundation.md`
- `/.agent/scripts/check-doc-boundaries.ps1`
