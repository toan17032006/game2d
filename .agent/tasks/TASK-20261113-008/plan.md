# Plan

- Task ID: `TASK-20261113-008`
- Status: `EXECUTING`

## Objective

Thi hành từ nghiên cứu tới engine 2D chạy được theo các phase dưới đây, mỗi phase có validation rõ ràng.

## Phases

| ID | Action | Input | Expected output | Validation | Status |
| --- | --- | --- | --- | --- | --- |
| P0 | Tạo task workspace, chuẩn hóa prompt, chạy preflight gate | Goal user + AGENTS.md | 7 artifact + preflight PASS | `run-quality-gates.ps1 -Mode preflight` exit 0 | `DONE` |
| P1 | Nghiên cứu thành phần engine code thuần; viết study doc + ADR draft | Objective + kiến thức domain | `research/engine-foundation-study.md`, `research/adr-draft-monster-engine-foundation.md` | Doc bao ≥ 12 subsystem, ADR đủ 5 mục | `DONE` |
| P2 | Scaffolding cargo workspace + crate `monster-math` | spec D-01, D-04 | Workspace build được, math unit tests | `cargo build --workspace` + `cargo test -p monster-math` exit 0 | `DONE` |
| P3 | Crate `monster-core`: event bus, fixed-step clock, log, trait contracts (Renderer/AudioBackend/PresentTarget/InputSource) | spec D-02 | Core tests PASS | `cargo test -p monster-core` exit 0 | `DONE` |
| P4 | Crate `monster-render`: framebuffer, software 2D renderer (blit alpha/tint/flip, camera, z-order), texture | spec D-03 | Render headless tests PASS | `cargo test -p monster-render` exit 0 | `DONE` |
| P5 | Crate `monster-platform`: Win32 window, input, GDI present, NullWindow cho headless | spec D-02, I-3 | Window code compile; NullWindow test PASS | `cargo test -p monster-platform` exit 0 | `DONE` |
| P6 | Crates `monster-assets` (registry, BMP, PNG inflate+decode), `monster-scene` (ECS, camera, layer), `monster-physics` (AABB), `monster-audio` (trait, Null, WAV) | spec D-05, D-06 | Tests PASS cho 4 crate | `cargo test --workspace` exit 0 | `DONE` |
| P7 | Facade `monster-engine` (Engine, EngineBuilder, GameApp) + example `sandbox` (window + headless) | Output contract | Sandbox headless 120 frames exit 0 | `cargo run --example sandbox -- --headless 120` exit 0 | `DONE` |
| P8 | Chạy full test suite, final gates, cập nhật verification/evaluation/handoff | Toàn bộ output | Final PASS, artifacts hoàn chỉnh | `run-quality-gates.ps1 -Mode final` exit 0 | `DONE` |

## Dependencies

- P2 → P3 → P4, P5, P6 (P4/P5/P6 phụ thuộc math/core, song song được nhưng thực hiện tuần tự để giữ checkpoint sạch).
- P6 → P7 (facade cần đủ subsystem).
- P1 độc lập, có thể chạy đầu.
- P8 phụ thuộc tất cả.

## Checkpoints

- CP-1 sau P1: nghiên cứu hoàn tất.
- CP-2 sau P4: math+core+render dựng xong (nòng kỹ thuật).
- CP-3 sau P6: toàn bộ subsystem có tests.
- CP-4 sau P8: final gate PASS.

## Risks

- R-01: Win32 FFI chi tiết struct/ABI sai → mitigated: kiểm soát kích thước struct (usize trên win64), test headless không phụ thuộc window thật, sandbox window chạy thủ công bởi người dùng.
- R-02: PNG decoder phức tạp (inflate dynamic Huffman) → mitigated: unit test inflate từng path (stored, fixed, dynamic), smoke test với PNG thật trong `/assest/`, fallback BMP được ghi nhận trong handoff nếu hỏng (không âm thầm bỏ).
- R-03: Thời gian build/test dài → mitigated: crate nhỏ, ít dependency, test thuần logic.

## Rollback

- Toàn bộ code nằm trong `/monster-engine/` (git-tracked, có thể revert về commit rỗng).
- Task artifact nằm riêng trong task workspace; rollback không ảnh hưởng `/docs/`.

## References

- `/AGENTS.md`
- `/docs/00-governance/agent-execution-standard.md`
- `/docs/03-architecture/README.md`
