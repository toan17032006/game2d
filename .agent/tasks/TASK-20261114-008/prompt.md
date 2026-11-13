# Prompt

## Input classification

`DIRECT` — người dùng yêu cầu trực tiếp trong phiên: "Commit và thực hiện đề xuất" — đề xuất cuối TASK-20261114-007: Engine loop 3D tích hợp (GameApp::render_3d) + audio music WAV loading.

## Role

Rust engine developer.

## Objective

1. **Engine 3D integration**: `GameApp::render_3d` callback + `EngineBuilder::with_3d()` + `RenderContext3D` — engine optionally owns `Gl3DRenderer` + `Camera3D`.
2. **Music WAV loading**: tiện ích load WAV file làm music looped qua winmm backend trong game mode.

## Context

- Repo sạch trên `638c9dd`; 205 engine tests PASS; game tách riêng tại `/game/`.
- Gl3DRenderer dual-mode (bound/unbound) đã có; `Camera3D` + `Renderer3D` trait đã có.

## Inputs

- Yêu cầu trực tiếp người dùng trong phiên.

## Scope

- `monster-engine/src/engine.rs`: GameApp thêm `render_3d` default no-op; EngineBuilder `with_3d(width, height)`; Engine `physics_3d: Option<Gl3DRenderer>`; RenderContext3D.
- `monster-audio`: `load_wav_file` đã có từ TASK-005 — wire vào game.
- Sandbox/game: `--3d` mode demo; music WAV loading.
- Docs FR-067..068 + traceability + commit.

## Non-goals

- Không thêm physics 3D step vào Engine loop (foundation-only); không đổi GameApp breaking (render_3d là default no-op).

## Constraints

- Zero-dep; giữ 205 tests cũ PASS; render_3d chỉ hoạt động khi window mode (cần GlContext).

## Execution rules

- Preflight trước EXECUTING; final gates trước COMPLETED; commit sau gates.

## Output contract

- Engine 3D integration + music loading + tests + docs FR-067..068 + commit.

## Acceptance criteria

Xem `task.md` (AC-001..AC-006).

## Verification method

- `cargo test --workspace` exit 0; sandbox modes exit 0; final gates exit 0.

## References

- `/AGENTS.md`
- `/docs/01-product/roadmap.md`
