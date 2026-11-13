# Verification

- Task ID: `TASK-20261114-005`
- Status: `COMPLETED`

## Acceptance coverage

| ID | Result | Evidence |
| --- | --- | --- |
| AC-001 | PASS | `systems::tests::sprite_culling_by_camera_viewport` PASS (trong/mép/ngoài/margin) + engine render pass gọi `sprite_in_view` margin 64 (engine.rs) |
| AC-002 | PASS | `tilemap::tests::multi_layer_spawn_coexists` PASS — 2 layer × 2 solid → 4 entity + 4 body, không đè vị trí |
| AC-003 | PASS | `wav::tests::load_wav_file_roundtrip` PASS — temp WAV fixture decode khớp samples/rate; winmm looped đã có từ TASK-004 |
| AC-004 | PASS | `gl.rs::modern_tests::{extension_loader_without_context_falls_back, shader_sources_are_glsl_120_pass_through, compile_program_requires_modern_gl}` PASS; `Gl3DRenderer` shader path (bind → load + compile → use_program trong begin/end) + fallback (unbound → shader_program=0) |
| AC-005 | PASS | FR-060..062 + traceability + roadmap M3 slice 3 ✅ / M4 slice 3 ✅ (tiếng Việt); markdown links PASS |
| AC-006 | PASS | `logs/test.log`: `cargo test --workspace` exit 0 — 202 passed / 0 failed; regression `logs/sandbox-game.log` + `logs/sandbox-physics.log` exit 0 |
| AC-007 | PASS | `logs/final-gate.log`: final gates PASS (exit 0); commit `logs/commit-log.log` |

## Commands

| Command / method | Result | Evidence | Acceptance coverage | Limitations |
| --- | --- | --- | --- | --- |
| `cargo test --workspace --manifest-path monster-engine/Cargo.toml` | PASS | `logs/test.log` (202 passed / 0 failed) | AC-001..AC-004, AC-006 | Không có |
| `cargo run --manifest-path monster-engine/Cargo.toml --example sandbox -- --headless 120 --game` | PASS (exit 0) | `logs/sandbox-game.log` | AC-006 | Không có |
| `cargo run --manifest-path monster-engine/Cargo.toml --example sandbox -- --headless 120 --physics` | PASS (exit 0) | `logs/sandbox-physics.log` | AC-006 | Không có |
| `powershell -NoProfile -ExecutionPolicy Bypass -File .agent/scripts/test-markdown-links.ps1` | PASS | terminal output | AC-005 | Không có |
| `powershell -NoProfile -ExecutionPolicy Bypass -File .agent/scripts/run-quality-gates.ps1 -TaskId TASK-20261114-005 -Mode final` | PASS | `logs/final-gate.log` | AC-007 | Không có |

## Coverage

- Culling: pure fn test biên (trong/mép/ngoài/margin) + engine render tích hợp.
- Tilemap multi-layer: 2 layer coexist, positions tách biệt.
- Music: WAV file loader roundtrip + winmm looped đã có từ TASK-004.
- Shader pipeline: loader null-guard từng function, compile/link với info-log, Gl3DRenderer fallback — headless test được tầng state; GL runtime verify thủ công (giới hạn ghi rõ).

## Limitations

- Shader chỉ verify runtime qua window thật (desktop session) — headless chỉ phủ loader/fallback logic.
- Music looped = repeat content buffer (không true streaming) — đã ghi từ TASK-004.
- Culling là AABB viewport — không xử lý sprite xoay/rotate ngoài bounds.

## Final status

`COMPLETED`

## References

- `/AGENTS.md`
- `/docs/02-requirements/functional-requirements.md` (FR-060..062)
- `/docs/01-product/roadmap.md`
