# Prompt

## Input classification

`DIRECT` — người dùng yêu cầu trực tiếp trong phiên: "Commit & Tách hoàn toàn game ra khỏi engine, engine chỉ làm nhiệm vụ engine, game code thì import engine vô. Sau đó commit và làm việc phát triển engine kế tiếp."

## Role

Rust engine architect — tách biệt engine/game boundary đúng chuẩn.

## Objective

1. Tách game ra khỏi `monster-engine` thành crate/binary riêng tại `/game/` — game code import `monster-engine` qua path dependency.
2. Engine facade chỉ expose API (prelude, traits, components, systems) — KHÔNG chứa game logic.
3. Xóa sandbox example khỏi engine (cube_3d example giữ lại — là engine demo).
4. Phát triển engine tiếp: cải thiện API surface (Input trait cho game override, GameApp lifecycle docs).

## Context

- Repo sạch trên `28b0d42`; 205 tests PASS; sandbox example chứa game code trong engine facade.
- ADR-001: engine 2D-first; ADR-002: GL legacy backend.

## Inputs

- Yêu cầu trực tiếp người dùng trong phiên.
- Code `monster-engine/` hiện tại.

## Scope

- Tạo `/game/` — Cargo project riêng (binary crate `monster-game`).
- Di chuyển sandbox/game logic từ `monster-engine/examples/sandbox.rs` sang `/game/src/main.rs`.
- Dọn `monster-engine` facade: giữ prelude + traits, bỏ game-specific code.
- `/game/Cargo.toml` phụ thuộc `monster-engine` qua path.
- Workspace Cargo.toml KHÔNG thêm game (game độc lập — import engine như thư viện).

## Non-goals

- Không đổi monster-physics/monster-core/etc internals.
- Không thêm features mới vào engine (chỉ refactor API surface).

## Constraints

- Zero external crate cho engine; game crate cũng zero external crate (chỉ phụ thuộc monster-engine path).
- 205 tests cũ PASS (trừ sandbox example tests chuyển sang game crate).
- Game build được độc lập: `cargo build` từ `/game/`.

## Execution rules

- Preflight trước EXECUTING; final gates trước COMPLETED; commit sau gates.

## Output contract

- `/game/` binary crate build + chạy được 3 mode (--game/--physics/classic).
- `monster-engine` facade sạch game code.
- `cargo test --workspace` (engine tests) exit 0.
- `cargo build` từ `/game/` exit 0.
- Docs FR-066 + traceability + commit.

## Acceptance criteria

Xem `task.md` (AC-001..AC-007).

## Verification method

- `cargo test --workspace --manifest-path monster-engine/Cargo.toml` exit 0.
- `cargo build` từ `/game/` exit 0.
- `cargo run` từ `/game/` với `--headless 120 --game` exit 0.
- Final gates exit 0.

## References

- `/AGENTS.md`
- `/docs/03-architecture/adr/ADR-001-monster-engine-foundation.md`
- `/docs/03-architecture/adr/ADR-002-monster-engine-3d-gl-backend.md`
