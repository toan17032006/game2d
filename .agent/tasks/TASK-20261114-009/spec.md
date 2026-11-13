# Specification

## Inputs

- Yêu cầu người dùng: "Commit và thực hiện đề xuất" (tiếp tục phát triển game).

## Confirmed facts

- F-01: 205 tests PASS trên `28b0d42`; game tách riêng tại `/game/`.
- F-02: `move_and_slide` đã có floor/ceiling/wall detection.
- F-03: Gravity đã có trong physics step cho Character body.

## Inferences

- I-1 (inference): Jump = set velocity.y âm (lên) khi on_floor + input pressed; gravity kéo xuống tự nhiên.
- I-2 (inference): Tilemap Solid tiles đã tạo static bodies qua spawn_tilemap — move_and_slide depenetration tự chặn.

## Unknowns

- U-1: Jump height tuning cần test thủ công — headless chỉ verify velocity sign change.

## Constraints

- C-01: Zero-dep; 205 tests cũ PASS.

## Decisions

- D-01: Jump: `velocity.y = -jump_speed` khi on_floor + jump input; jump_speed = 400 px/s (tunable).
- D-02: on_floor từ move_and_slide CharacterMoveResult.floor.
- D-03: Level platforms: tilemap Solid tiles tạo platforms ở độ cao khác nhau; player jump lên platforms.

## Output contract

- Game improvements (jump + platforms) + tests + docs FR-069 + commit.

## Test Command

`cargo test --workspace --manifest-path monster-engine/Cargo.toml`

## Approved Documentation Changes

- `/docs/02-requirements/functional-requirements.md` — FR-069
- `/docs/02-requirements/traceability-matrix.md` — dòng FR-069

## References

- `/AGENTS.md`
- `/docs/01-product/roadmap.md`
