# Prompt

## Input classification

`DIRECT` — người dùng yêu cầu trực tiếp trong phiên: "Commit và thực hiện đề xuất" — tiếp tục phát triển engine + game.

## Role

Rust game developer.

## Objective

1. Character jump mechanic (input Space/W/Up → jump, gravity xử lý bởi physics step).
2. Tilemap collision layers: Solid tiles chặn Character qua move_and_slide depenetration.
3. Game polish: level design với platforms, animation states, score hiển thị via HUD text.

## Context

- 205 tests PASS trên `2d4d382`; game tách riêng tại `/game/`.

## Inputs

- Yêu cầu trực tiếp người dùng trong phiên.

## Scope

- Sandbox game improvements: jump input, platform collision, level design polish.
- Docs FR-069 + traceability + roadmap update; commit.

## Non-goals

- Không đổi engine core; không thêm features mới ngoài game polish.

## Constraints

- Zero-dep; 205 tests cũ PASS.

## Output contract

- Game improvements + tests + docs FR-069 + commit.

## Acceptance criteria

Xem `task.md`.

## Verification method

- `cargo test --workspace` exit 0; game --headless exit 0 với jump verify; final gates exit 0.

## References

- `/AGENTS.md`
- `/docs/01-product/roadmap.md`
