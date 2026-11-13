# Specification

## Inputs

- Yêu cầu người dùng: "Commit và thực hiện đề xuất" (Engine 3D loop integration + music WAV loading).

## Confirmed facts

- F-01: Repo sạch trên `638c9dd`; 205 tests PASS; game tách riêng tại `/game/`.
- F-02: Gl3DRenderer dual-mode + Camera3D + Renderer3D trait đã có ở facade/monster-core.
- F-03: `monster-audio::load_wav_file` đã có từ TASK-005.

## Inferences

- I-1 (inference): render_3d là default no-op trên GameApp — non-breaking cho game 2D hiện có.
- I-2 (inference): with_3d chỉ hoạt động window mode (GL context cần real window); headless → Gl3DRenderer unbound.

## Unknowns

- U-1: Desktop session availability khi test — GL context test skip-graceful.

## Constraints

- C-01: Zero-dep; 205 tests cũ PASS; GameApp breaking-free (render_3d là default method).

## Decisions

- D-01: `EngineBuilder::with_3d()` — Engine tạo Gl3DRenderer (window mode: bound; headless: unbound); Camera3D mặc định.
- D-02: `GameApp::render_3d(ctx: &mut RenderContext3D)` default no-op; RenderContext3D { renderer: &mut dyn Renderer3D, camera: &Camera3D }.
- D-03: `monster-audio::load_wav_file` dùng cho music; WinmmAudioBackend `play_clip(clip, vol, looped=true)`.

## Output contract

- Engine 3D integration + music loading + tests + docs FR-067..068 + commit.

## Test Command

`cargo test --workspace --manifest-path monster-engine/Cargo.toml`

## Approved Documentation Changes

- `/docs/02-requirements/functional-requirements.md` — FR-067..068
- `/docs/02-requirements/traceability-matrix.md` — dòng FR-067..068

## References

- `/AGENTS.md`
- `/docs/01-product/roadmap.md`
