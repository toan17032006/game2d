# Prompt

## Input classification

`DIRECT` — người dùng yêu cầu trực tiếp trong phiên: "Commit và thực hiện đề xuất" — 2 đề xuất cuối TASK-20261114-005: (a) M4 slice 4 đóng gói mini-game hoàn chỉnh (menu + state machine + level flow); (b) M3 tiếp — mesh mở rộng (normals/OBJ) + physics 3D Aabb3.

## Role

Rust game developer + engine developer.

## Objective

1. **M4 slice 4 — mini-game hoàn chỉnh**:
   - `GamePhase` state machine: Menu → Playing → Won (Enter khởi động/replay).
   - Animation switch: Idle clip khi đứng yên, Run clip khi di chuyển.
   - Level tilemap: tường + ground segments.
   - Win khi thu đủ coins; HUD hiện phase + score.
2. **M3 tiếp**:
   - OBJ loader mở rộng: parse `vn` (normals) + `f v//vn`; `Mesh3D` có `normals`.
   - Physics 3D: `Aabb3` (intersects/contains/center/separation MTV 3 trục).
   - GL renderer emit `glNormal3f` khi mesh có normals.
3. Commit.

## Context

- 202 tests PASS trên `c16c9a4`; game slice 1 có player/coins/camera/animation.

## Inputs

- Yêu cầu trực tiếp người dùng trong phiên.

## Scope

- `examples/sandbox.rs`: GamePhase state machine + level flow + animation switch.
- `monster-core::mesh`: OBJ vn parse + Mesh3D.normals + draw_mesh emit normals (gl3d + glNormal3f FFI).
- `monster-physics`: Aabb3 + tests.
- Docs FR-063..065 + traceability + roadmap M4 slice 4/M3 tiếp partial; commit.

## Non-goals

- Menu art/graphics; audio menu music; multiple levels; engine loop 3D tích hợp (defer).

## Constraints

- Zero-dep; giữ 202 tests cũ PASS; headless verify được state transitions.

## Execution rules

- Preflight trước EXECUTING; final gates trước COMPLETED; commit sau gates.

## Output contract

- Sandbox game state machine + animation switch + mesh normals + Aabb3 + tests + docs FR + commit.

## Acceptance criteria

Xem `task.md` (AC-001..AC-007).

## Verification method

- `cargo test --workspace` exit 0; sandbox `--game` headless exit 0 với state transitions; final gates exit 0.

## References

- `/AGENTS.md`
- `/docs/01-product/roadmap.md`
