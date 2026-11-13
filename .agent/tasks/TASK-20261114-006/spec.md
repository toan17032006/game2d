# Specification

## Inputs

- Yêu cầu người dùng: "Commit và thực hiện đề xuất" (M4 slice 4 + M3 tiếp).
- Code hiện tại: 202 tests PASS trên `c16c9a4`; game slice 1 có player/coins/camera.

## Confirmed facts

- F-01: Tiny Swords Warrior có Idle (8 frames) + Run (4 frames) sheets.
- F-02: OBJ format: `vn x y z` normals + `f v//vn` face format phổ biến.
- F-03: KeyCode::Enter tồn tại trong core input.

## Inferences

- I-1 (inference): GamePhase state machine giữ trong SandboxGame (không đưa vào engine core — game-specific).
- I-2 (inference): OBJ normals parse subset `v`/`vn`/`f v//vn` (đủ phổ biến); normals lưu theo thứ tự khai báo, không index hóa chặt (giới hạn ghi).

## Unknowns

- U-1: Headless state transitions phụ thuộc auto-input — dùng step counters (menu 30 bước → auto-Enter).

## Constraints

- C-01: Zero-dep; giữ 202 tests cũ PASS; không đụng engine core render loop.

## Decisions

- D-01: `GamePhase { Menu, Playing, Won }` trong SandboxGame; Enter/E auto-advance; Won khi score == coin tổng; replay reset level (player/coins/score).
- D-02: Animation switch: player đứng yên (velocity x ≈ 0 sau move) → Animator.clip = idle_clip; di chuyển → run_clip; clips load 1 lần giữ trong app.
- D-03: Level tilemap: ground segments + 2 cột tường ở hai đầu sân qua spawn_tilemap; tiles Solid.
- D-04: Mesh3D thêm `normals: Vec<Vec3>`; OBJ parse `vn` + `f v//vn` (token trước `//` là vertex, sau là normal); `glNormal3f` FFI + emit khi normals có mặt.
- D-05: `Aabb3` ở monster-physics: min/max Vec3, intersects/contains/center/half_extents/separation MTV 3 trục; test tương đương Aabb2.
- D-06: Test Command: `cargo test --workspace --manifest-path monster-engine/Cargo.toml`.

## Output contract

- Sandbox state machine + animation switch + level; mesh normals + glNormal3f; Aabb3 + tests; docs FR-063..065 + commit.

## Test Command

`cargo test --workspace --manifest-path monster-engine/Cargo.toml`

## Approved Documentation Changes

- `/docs/02-requirements/functional-requirements.md` — FR-063..065
- `/docs/02-requirements/traceability-matrix.md` — dòng FR-063..065
- `/docs/01-product/roadmap.md` — M4 slice 4 ✅, M3 tiếp partial ✅

## References

- `/AGENTS.md`
- `/docs/01-product/roadmap.md`
