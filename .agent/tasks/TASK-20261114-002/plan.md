# Plan

- Task ID: `TASK-20261114-002`
- Status: `EXECUTING`

## Objective

Camera follow + M3 slice đầu (3D GL renderer + cube demo) + docs + commit.

## Phases

| ID | Action | Input | Expected output | Validation | Status |
| --- | --- | --- | --- | --- | --- |
| P0 | Task workspace + preflight | Yêu cầu user | 7 artifact | preflight exit 0 | `DONE` |
| P1 | `camera_follow_system` (monster-scene) + test | D-01 | System + test | `cargo test -p monster-scene` exit 0 | `DONE` |
| P2 | `Mat4::look_at` (math) + `Camera3D`/`Renderer3D` (core) + tests | D-02/D-03 | API + tests | tests PASS | `DONE` |
| P3 | `monster-platform::gl` — WGL/opengl32 FFI + GlContext + test skip-graceful | D-04 | GL context module | build link opengl32 OK | `DONE` |
| P4 | `Gl3DRenderer` (facade gl3d.rs) + `examples/cube_3d.rs` | D-03/D-04/D-05 | 3D renderer + demo | build + example compile | `DONE` |
| P5 | Docs: ADR-002 + FR-050..052 + traceability + roadmap | Toàn bộ | 4 file docs | link check | `DONE` |
| P6 | fmt + full test + final gates + commit | Toàn bộ | Gates PASS + commit | final exit 0 | `DONE` |

## Dependencies

- P1, P2 độc lập; P3 → P4; P5/P6 cuối.

## Checkpoints

- CP-1 sau P2: camera follow + Camera3D headless tests PASS.
- CP-2 sau P4: GL stack compile + example.
- CP-3 sau P6: gates PASS + commit.

## Risks

- R-01: WGL pixel format cần set trước window class? — dùng DC của window đã tạo + ChoosePixelFormat (đủ cho legacy).
- R-02: GL test trong CI không có desktop → test skip-graceful, không FAIL.
- R-03: look_at handedness dễ sai → test với camera nhìn -Z chuẩn.

## Rollback

- Revert commit; không đụng 2D pipeline hiện có.

## References

- `/AGENTS.md`
