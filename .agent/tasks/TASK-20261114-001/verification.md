# Verification

- Task ID: `TASK-20261114-001`
- Status: `COMPLETED`

## Acceptance coverage

| ID | Result | Evidence |
| --- | --- | --- |
| AC-001 | PASS | `physics_bridge::tests::{dynamic_body_drives_scene_transform, static_scene_transform_drives_body, unlinked_entities_are_untouched}` PASS — hướng sync đúng: dynamic body → scene, static scene → body, entity không link không bị đụng |
| AC-002 | PASS | `tests/physics_bridge.rs::{physics_bridge_entity_follows_body, engine_without_physics_leaves_context_none}` PASS — entity rơi theo physics qua `engine.run` (y đọng 170..186 khớp body < 1e-3); engine không bật physics → ctx.physics = None |
| AC-003 | PASS | `logs/sandbox-physics.log`: `cargo run --example sandbox -- --headless 120 --physics` exit 0 — 240 sim steps, 6/6 balls đọng trong dải [210, 240), sprites khớp body < 1px, 12 trigger events; demo gốc `--headless 120` vẫn PASS (`logs/sandbox-classic.log`) |
| AC-004 | PASS | `/docs/02-requirements/functional-requirements.md` FR-049 + `/docs/02-requirements/traceability-matrix.md` dòng FR-049 (tiếng Việt); markdown links PASS |
| AC-005 | PASS | `logs/test.log`: `cargo test --workspace` exit 0 — 163 passed / 0 failed; `logs/final-gate.log`: final gates PASS; commit `git log` ghi `logs/commit-log.log` |

## Commands

| Command / method | Result | Evidence | Acceptance coverage | Limitations |
| --- | --- | --- | --- | --- |
| `cargo test --workspace --manifest-path monster-engine/Cargo.toml` | PASS | `logs/test.log` (163 passed / 0 failed) | AC-001, AC-002, AC-005 | Không có |
| `cargo run --manifest-path crates/monster-engine/Cargo.toml --example sandbox -- --headless 120 --physics` | PASS (exit 0) | `logs/sandbox-physics.log` | AC-003 | Không có |
| `cargo run --manifest-path crates/monster-engine/Cargo.toml --example sandbox -- --headless 120` | PASS (exit 0) | `logs/sandbox-classic.log` | regression | Không có |
| `powershell -NoProfile -ExecutionPolicy Bypass -File .agent/scripts/test-markdown-links.ps1` | PASS | terminal output | AC-004 | Không có |
| `powershell -NoProfile -ExecutionPolicy Bypass -File .agent/scripts/run-quality-gates.ps1 -TaskId TASK-20261114-001 -Mode final` | PASS | `logs/final-gate.log` | AC-005 | Không có |

## Coverage

- Bridge phủ đủ 3 hướng sync (dynamic→scene, static→physics, không-link bỏ qua).
- Engine integration được test qua vòng lặp thật (headless, with_physics).
- Sandbox chạy được cả 2 chế độ (physics + classic) không regression.

## Limitations

- Bridge sync theo index body (usize) — xóa body giữa chừng chưa hỗ trợ (foundation: không xóa body).
- Sync O(n) mỗi fixed step — đủ cho hàng trăm body; uniform grid là nâng cấp sau.

## Final status

`COMPLETED`

## References

- `/AGENTS.md`
- `/docs/02-requirements/functional-requirements.md` (FR-049)
- `/monster-engine/crates/monster-engine/src/physics_bridge.rs` (product)
