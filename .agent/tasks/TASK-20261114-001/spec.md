# Specification

## Inputs

- Yêu cầu người dùng: "Commit và thực hiện đề xuất" (2 đề xuất cuối TASK-010).
- Code facade hiện tại: Engine/EngineBuilder/GameApp/contexts; monster-physics v2.

## Confirmed facts

- F-01: Repo sạch trên `cda2ed8` + `725cf07` (physics); `.gitignore` có thay đổi người dùng (`test-game-engine`) — bảo toàn.
- F-02: Scene `Transform` có position/rotation/scale; physics `Transform2` có position/rotation.
- F-03: `PhysicsWorld2` standalone; bodies có `tag: u32` — có thể dùng làm entity index.

## Inferences

- I-1 (inference): Bridge nhẹ nhất: component `PhysicsLink { body: usize }` trong facade; sync mỗi fixed step.
- I-2 (inference): Hướng sync: body responds_to_collision() (dynamic) → scene; static → scene→physics (authoring win). Kinematic: scene→physics (code điều khiển velocity trong update, vị trí do physics integrate).

## Unknowns

- U-1: Không xác định trước hiệu năng sync với hàng trăm body — bridge là O(n) mỗi step, chấp nhận cho foundation.

## Constraints

- C-01: Zero-dep; giữ 158 tests cũ PASS.
- C-02: Chỉ sửa docs trong danh sách duyệt.

## Decisions

- D-01: `PhysicsLink` component + 3 hàm bridge tự do (attach/sync_dynamic/sync_static).
- D-02: `EngineBuilder::with_physics(gravity: Vec2)` — engine step physics mỗi fixed step TRƯỚC app.update; UpdateContext thêm `physics: Option<&mut PhysicsWorld2>`.
- D-03: Sandbox `--physics` dùng bridge; assert trong app (bóng đọng) + log trigger events.
- D-04: Test Command: `cargo test --workspace --manifest-path monster-engine/Cargo.toml`.

## Output contract

- Bridge + engine integration + sandbox mode + tests + docs FR-049 + commit `feat(engine): physics bridge`.

## Test Command

`cargo test --workspace --manifest-path monster-engine/Cargo.toml`

## Approved Documentation Changes

- `/docs/02-requirements/functional-requirements.md` — thêm FR-049
- `/docs/02-requirements/traceability-matrix.md` — thêm dòng FR-049

## References

- `/AGENTS.md`
- `/docs/03-architecture/adr/ADR-001-monster-engine-foundation.md`
