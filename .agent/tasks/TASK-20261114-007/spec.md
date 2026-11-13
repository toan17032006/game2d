# Specification

## Inputs

- Yêu cầu người dùng: "Commit & Tách hoàn toàn game ra khỏi engine, engine chỉ làm nhiệm vụ engine, game code thì import engine vô."

## Confirmed facts

- F-01: Repo sạch trên `28b0d42`; 205 tests PASS.
- F-02: Game logic nằm trong `monster-engine/crates/monster-engine/examples/sandbox.rs` (~560 dòng).
- F-03: Facade `monster-engine` crate có prelude exports + GameApp trait — đủ cho game code import.
- F-04: Facade cũng chứa GamePhase, animation switch logic, coin collect logic — tất cả là GAME logic nằm trong engine.

## Inferences

- I-1 (inference): Game crate đặt tại `/game/` (repo root level, không trong workspace) — tạo binary crate độc lập phụ thuộc `monster-engine` path.
- I-2 (inference): Engine giữ cube_3d example (engine demo), bỏ sandbox example.

## Unknowns

- U-1: Cargo workspace boundary — `/game/` ngoài workspace nên cargo build độc lập; cần test.

## Constraints

- C-01: Zero external crate cho cả engine và game.
- C-02: Giữ 205 engine tests PASS (sandbox example tests chuyển sang game crate).
- C-03: Game build độc lập: `cargo build` từ `/game/` không cần `--manifest-path`.

## Decisions

- D-01: `/game/Cargo.toml` tên `monster-game`, phụ thuộc `monster-engine = { path = "../monster-engine" }`.
- D-02: Game code di chuyển nguyên trạng từ sandbox.rs sang `/game/src/main.rs`; internal test tương tự.
- D-03: Engine facade bỏ examples/sandbox.rs + tests/physics_bridge.rs giữ lại (đã là engine tests).
- D-04: Facade dọn: giữ prelude + traits + systems; bỏ game-specific imports trong examples.
- D-05: FR-066 ghi kiến trúc tách game.

## Output contract

- `/game/` crate build+run được; engine sạch game code; docs FR-066; commit.

## Test Command

`cargo test --workspace --manifest-path monster-engine/Cargo.toml`

## Approved Documentation Changes

- `/docs/02-requirements/functional-requirements.md` — FR-066
- `/docs/02-requirements/traceability-matrix.md` — dòng FR-066
- `/docs/03-architecture/system-context.md` — cập nhật game crate tách riêng

## References

- `/AGENTS.md`
- `/docs/03-architecture/adr/ADR-001-monster-engine-foundation.md`
