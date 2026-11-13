# Prompt

## Input classification

`DIRECT` — người dùng yêu cầu trực tiếp trong phiên: "Commit và thực hiện đề xuất" — đề xuất cuối TASK-20261114-002: M3 slice 2 (mesh loading) và M4 (sprite animation + Tiny Swords game). Thực hiện cả hai, ưu tiên M4.

## Role

Rust game engine developer + technical artist.

## Objective

1. **Sprite animation (M4)**: `AnimationClip` (grid-slice từ spritesheet) + component `Animator` + `sprite_animation_system` (loop/clamp/pause).
2. **Mesh loading (M3 slice 2)**: `Mesh3D` (vertices/indices, builder cube) + OBJ loader thuần Rust + `Renderer3D::draw_mesh` (mock + GL backend).
3. **Sandbox `--game` demo**: mini-game dùng Tiny Swords thật — player Character body (move_and_slide) + Warrior_Run animation + camera follow + coins (Area trigger) + score; headless auto-walk verify.
4. Commit.

## Context

- Tiny Swords sheets chuẩn 192×192/frame: Warrior_Idle 1536×192 (8 frames), Run/Attack1 768×192 (4 frames) — grid-slice cols = width/192.
- Có sẵn: PNG decoder, physics v2 (Character body + move_and_slide), camera_follow_system, physics bridge.
- 173 tests PASS trên `1e6dcae`.

## Inputs

- Yêu cầu trực tiếp người dùng trong phiên.
- `/docs/01-product/roadmap.md` (M3/M4), `/assest/`.

## Scope

- `monster-scene`: `animation.rs` (AnimationClip, Animator, sprite_animation_system, from_grid) + tests.
- `monster-core`: `Mesh3D` + `Renderer3D::draw_mesh` (mở rộng trait) + OBJ loader + tests.
- `monster-engine`: Gl3DRenderer implement draw_mesh + sandbox `--game`.
- Docs FR-053..055 + traceability + roadmap M4 partial.
- Commit git.

## Non-goals

- Modern GL/shaders (M3 tiếp); skeletal/bone animation; aseprite metadata parsing; audio sync animation.

## Constraints

- Zero external crate; giữ 173 tests cũ PASS.
- Tiny Swords sheet không có metadata → animation dùng grid-slice (frame = 192×192 từ chiều cao, cols = width/192) — ghi rõ trong docs.
- Demo `--game` hoạt động cả headless (auto-walk) lẫn window (input A/D + Space).

## Execution rules

- Preflight trước EXECUTING; final gates trước COMPLETED; commit sau gates.

## Output contract

- animation.rs + animation tests; Mesh3D + OBJ loader + draw_mesh (mock + GL) + tests; sandbox --game; docs; commit.

## Acceptance criteria

Xem `task.md` (AC-001..AC-007).

## Verification method

- `cargo test --workspace` exit 0; `cargo run --example sandbox -- --headless 120 --game` exit 0 với assert trong demo; final gates exit 0.

## References

- `/AGENTS.md`
- `/docs/01-product/roadmap.md`
- `/assest/`
