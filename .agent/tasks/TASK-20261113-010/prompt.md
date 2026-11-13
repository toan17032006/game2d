# Prompt

## Input classification

`DIRECT` — người dùng đặt mục tiêu trực tiếp trong phiên (goal mode): "Engine hỗ trợ: Cửa sổ + các khối bọc (14 loại body + joints + 15 loại collider)". Không còn input bắt buộc thiếu; thiết kế 2D-first theo ADR-001 đã được chấp nhận trước đó.

## Role

Game physics engineer, hiện thực hệ vật lý 2D đầy đủ trên nền `monster-physics` hiện có (Rust, zero-dependency).

## Objective

Mở rộng `monster-engine` hỗ trợ đầy đủ taxonomy vật lý kiểu Godot do người dùng liệt kê:
1. **Cửa sổ** — đã có sẵn từ v0.1.0 (xác minh vẫn hoạt động).
2. **14 loại body**: Static, Rigid (Dynamic), Kinematic, Character, Area/Trigger, Animatable, Physical Bone, Soft, Cloth, Vehicle, Articulated, Particle, Fluid, Deformable.
3. **Joints / Constraints**: Distance, Pin (revolute), Weld (fixed), Spring.
4. **15 loại collider**: Box, Sphere, Capsule, Cylinder, Cone, Plane, Mesh, Convex Mesh, Concave Mesh, Heightmap, Terrain, Compound, Wheel, Raycast, Custom.

## Context

- v0.1.0 đã có: Aabb2, integrate, gravity, bounce, brute-force broadphase (7 tests) — giữ nguyên API cũ, xây lớp mới bên trên.
- Kiến trúc 2D-first (ADR-001 Accepted): các shape 3D ánh xạ tương đương 2D có chủ đích, ghi rõ mapping trong docs (Sphere→Circle, Cylinder→Stadium/Capsule, Cone→triangle polygon, Mesh→polygon, Terrain→heightfield grid). Phiên bản 3D thật thuộc M3.
- Zero external dependency; mọi thuật toán tự viết (SAT, impulse, PBD).

## Inputs

- Mục tiêu người dùng (chuẩn hóa ở trên).
- `/AGENTS.md`; code `monster-physics` hiện tại; `/docs/02-requirements/*`.

## Scope

- Mở rộng crate `monster-physics` (module mới: shape, collision, body, world, joint, raycast, softbody/particles, vehicle).
- Thêm tests cho từng loại body/collider/joint.
- Cập nhật docs `/docs/02-requirements/` (FR mới + traceability) — phê duyệt trực tiếp từ phiên (người dùng đã yêu cầu viết docs đầy đủ tiếng Việt trong phiên trước và đây là cùng dòng công việc).

## Non-goals

- Không hiện thực 3D thật (M3); không continous collision detection (CCD) đầy đủ; không sleeping/islands; không multithread.
- Không đụng renderer/window hiện có.

## Constraints

- Zero external crate.
- Không phá vỡ public API hiện có của `monster-physics` (105 tests cũ phải còn PASS).
- Mọi body/collider/joint phải có ít nhất một test hành vi thật (không stub).

## Execution rules

- Thực thi theo `plan.md`; cập nhật handoff sau checkpoint; preflight trước EXECUTING; final trước COMPLETED.

## Output contract

- `monster-physics` mở rộng với `PhysicsWorld2` chạy pipeline: integrate → broadphase → narrowphase → resolve → joints; API raycast; character move_and_slide; particle-based soft/cloth/fluid/particle/deformable; vehicle suspension.
- Docs FR + traceability cập nhật.
- verification/evaluation/handoff đầy đủ.

## Acceptance criteria

Xem `task.md` (AC-001..AC-009).

## Verification method

- `cargo test --workspace --manifest-path monster-engine/Cargo.toml` exit 0.
- Final gates exit 0.

## References

- `/AGENTS.md`
- `/docs/03-architecture/adr/ADR-001-monster-engine-foundation.md`
- `/docs/02-requirements/functional-requirements.md`
- `/monster-engine/crates/monster-physics/`
