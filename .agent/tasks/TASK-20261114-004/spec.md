# Specification

## Inputs

- Yêu cầu người dùng: "Commit và thực hiện đề xuất" (M4 slice 2).
- Code hiện tại: 184 tests PASS trên `40cb1a8`; game slice có HUD cần text, coin cần SFX, map cần tường.

## Confirmed facts

- F-01: `AudioBackend` trait + NullAudioBackend + AudioClip (mono PCM16) đã có ở monster-core/audio.
- F-02: Platform FFI rule — winmm FFI phải nằm trong monster-platform.
- F-03: Engine render pass vẽ sprites theo z-order; có `Renderer::draw_rect` sẵn.

## Inferences

- I-1 (inference): Blocky font 3×5 tự định nghĩa (digit + A-Z + punctuation gọn) — đủ HUD "SCORE n", không cần TTF.
- I-2 (inference): Particles vẽ bằng `draw_rect` trong engine render pass (không entity churn) — cần iterate component trong render.
- I-3 (inference): winmm backend single-buffer per playback; cleanup lazy trong play/stop_all; thiếu device → Err → sandbox fallback Null.

## Unknowns

- U-1: Sound output thực tế cần loa — verify thủ công bởi người dùng (giới hạn ghi rõ).

## Constraints

- C-01: Zero-dep; giữ 184 tests PASS; FFI trong platform.
- C-02: `draw_text_screen` thêm vào trait `Renderer` bằng default no-op (non-breaking).

## Decisions

- D-01: Font 3×5 bitmap dạng chuỗi pattern 15 bit/glyph trong `monster-render::text`; scale nguyên.
- D-02: TileMap helper spawn sprite entity + static body per Solid tile, dùng physics bridge; Empty bỏ qua.
- D-03: `ParticleEmitter2D` component giữ particle pool nội bộ; engine render pass vẽ `draw_rect` theo màu lerp tuổi đời.
- D-04: `WinmmAudioBackend` ở `monster-platform::winmm_audio`: waveOutOpen WAVE_MAPPER, mono PCM16 tại sample_rate của clip; cleanup lazy; trait method giữ nguyên.
- D-05: Sandbox --game: HUD "SCORE n" góc trái trên; coin burst 10 particles; tilemap tường bao quanh sân; coin blip sine ngắn qua audio backend.

## Output contract

- text.rs + tilemap.rs + particles2d.rs + winmm_audio.rs + sandbox updates + tests + docs FR-056..059 + commit.

## Test Command

`cargo test --workspace --manifest-path monster-engine/Cargo.toml`

## Approved Documentation Changes

- `/docs/02-requirements/functional-requirements.md` — FR-056..059
- `/docs/02-requirements/traceability-matrix.md` — dòng FR-056..059
- `/docs/01-product/roadmap.md` — M4 partial cập nhật

## References

- `/AGENTS.md`
- `/docs/01-product/roadmap.md`
