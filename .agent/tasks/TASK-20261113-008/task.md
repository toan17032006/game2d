# Task

- Task ID: `TASK-20261113-008`
- Status: `EXECUTING`
- Owner: `@agent`

## Objective

Nghiên cứu thành phần cần có của một game engine code thuần và xây dựng nền móng `monster-engine` bằng Rust: engine 2D chạy được, zero-dependency, kiến trúc trait-based dễ mở rộng và dễ thêm hàm, chuẩn bị đường mở rộng sang 3D.

## Scope

- Nghiên cứu + tài liệu nghiên cứu trong task workspace.
- Cargo workspace `/monster-engine/` với các crate `monster-math`, `monster-core`, `monster-render`, `monster-platform`, `monster-assets`, `monster-scene`, `monster-physics`, `monster-audio`, `monster-engine` + example `sandbox`.
- Thêm `.gitignore` entry cho build artifact.
- Unit/integration tests toàn workspace.

## Non-goals

- Render 3D thực tế, GPU/hardware backend, editor, scripting, networking, audio playback phần cứng.
- Sửa Markdown tài liệu dự án ngoài task workspace.
- Tích hợp crate bên ngoài.

## Acceptance criteria

| ID | Criterion | Required evidence |
| --- | --- | --- |
| AC-001 | Tài liệu nghiên cứu thành phần engine code thuần tồn tại, bao ≥ 12 subsystem, mỗi mục có vai trò + interface đề xuất + thứ tự xây dựng | `research/engine-foundation-study.md` |
| AC-002 | `cargo build --workspace` exit 0; workspace không khai báo dependency ngoài `std` | Build log trong `logs/`; các `Cargo.toml` |
| AC-003 | Kiến trúc mở rộng trait-based: trait `Renderer`, `AudioBackend`, `PresentTarget` + `EngineBuilder`; chứng minh hoán đổi backend không sửa core (Null backend test PASS) | Code refs + test output |
| AC-004 | Unit/integration tests PASS cho: math (Vec2/3/4, Mat4, Quat), core (event bus, fixed-step clock, log), scene ECS (spawn/despawn/query), physics AABB, assets (BMP + PNG decode), audio (WAV decode), render (framebuffer blit alpha) | `cargo test` output trong `logs/` |
| AC-005 | Example `sandbox` compile và chạy headless ≥ 120 frames, exit 0 | Output của lệnh run trong `logs/` |
| AC-006 | ADR draft cho quyết định kiến trúc nền móng, hoàn chỉnh context/decision/alternatives/consequences, đặt trong task workspace chờ phê duyệt chuyển vào `/docs/03-architecture/adr/` | `research/adr-draft-monster-engine-foundation.md` |
| AC-007 | `verification.md` map toàn bộ AC-NNN → PASS kèm evidence; `evaluation.md` final `PASS` | Hai file trong task workspace |
| AC-008 | Final quality gates PASS (exit 0) | Output final gate trong `logs/` |

## References

- `/AGENTS.md`
- `/docs/00-governance/agent-execution-standard.md`
- `/.agent/tasks/README.md`
