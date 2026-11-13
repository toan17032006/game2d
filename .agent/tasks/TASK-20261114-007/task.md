# Task

- Task ID: `TASK-20261114-007`
- Status: `EXECUTING`
- Owner: `@agent`

## Objective

Tách hoàn toàn game ra khỏi `monster-engine`: game là binary crate riêng tại `/game/` import engine qua path dependency. Engine chỉ expose API. Sau đó commit.

## Scope

- Tạo `/game/` binary crate.
- Di chuyển game logic từ engine examples sang game crate.
- Dọn engine facade (bỏ game code).
- FR-066 + traceability + commit.

## Non-goals

- Không đổi engine internals; không thêm features.

## Acceptance criteria

| ID | Criterion | Required evidence |
| --- | --- | --- |
| AC-001 | `/game/` binary crate tồn tại với `Cargo.toml` phụ thuộc `monster-engine` path + `src/main.rs` chứa game code | Files |
| AC-002 | `cargo build` từ `/game/` exit 0 — game build độc lập | Build log |
| AC-003 | `cargo run` từ `/game/` với `--headless 120 --game` exit 0 — game chạy được qua bridge | Run log |
| AC-004 | `monster-engine` facade KHÔNG chứa game logic (không có sandbox.rs, không có GamePhase trong engine src) | Code inspection |
| AC-005 | `cargo test --workspace --manifest-path monster-engine/Cargo.toml` exit 0 — engine tests không regression | `logs/test.log` |
| AC-006 | Docs FR-066 + traceability cập nhật (tiếng Việt) | Files |
| AC-007 | Final gates PASS + commit git | `logs/final-gate.log` + `git log` |

## References

- `/AGENTS.md`
- `/docs/01-product/roadmap.md`
