# Specification

## Inputs

- Yêu cầu người dùng: "Commit và thực hiện đề xuất" (M4 slice 3 + M3 slice 3).
- Code hiện tại: 196 tests PASS trên `6bc904c`; winmm looped = repeat content ×N.

## Confirmed facts

- F-01: Engine render vẽ mọi sprite không culling; sprite bounds qua `shape.bound`? Sprite không có shape — culling theo sprite size/position.
- F-02: Modern shader functions nằm ngoài opengl32.lib — chỉ lấy được qua `wglGetProcAddress` khi context hiện hành.
- F-03: Compatibility profile giữ immediate mode + built-in uniforms (ftransform) — shader màu đơn giản chạy được trên legacy context nếu driver expose 2.x functions.

## Inferences

- I-1 (inference): Culling sprite theo camera viewport (position ± viewport/2 + margin) — pure fn ở scene systems, engine render gọi trước draw.
- I-2 (inference): Shader path: nếu extension functions load được (non-null) → compile "solid color shader" dùng `ftransform` + uniform color; ngược lại fallback `glColor3f`. Trạng thái lưu trên Gl3DRenderer.

## Unknowns

- U-1: Driver có expose shader functions trên legacy context không — runtime-only; code phải fallback đúng khi NULL.

## Constraints

- C-01: Zero-dep; 196 tests cũ PASS; FFI trong platform.
- C-02: Không phá vỡ fallback: mọi shader call phải guard null pointer.

## Decisions

- D-01: `sprite_in_view(center, half, camera, margin)` ở monster-scene::systems; engine render gọi; margin mặc định 64px.
- D-02: Tilemap multi-layer: gọi spawn_tilemap 2 lần (background offset -8 z; solid z 0) — là test multi-layer, không API mới.
- D-03: `monster-audio::load_wav_file(path)` = read + decode_wav; winmm looped extend buffer tới 10s max cho music.
- D-04: `monster-platform::gl`: static table `MODERN: OnceLock<ModernFns>` với 12 function pointers; `load_modern_functions()` gọi wglGetProcAddress cho từng tên; `modern_available()`; wrapper `compile_program(vs, fs) -> Result<u32, String>` guard null; Gl3DRenderer.bind → load + compile; `shader_active` flag; draw_cube dùng shader nếu active (glUniform3f màu) else glColor.
- D-05: Test Command: `cargo test --workspace --manifest-path monster-engine/Cargo.toml`.

## Output contract

- Culling + multi-layer test + music loader + extension loader/shader pipeline + docs FR-060..062 + commit.

## Test Command

`cargo test --workspace --manifest-path monster-engine/Cargo.toml`

## Approved Documentation Changes

- `/docs/02-requirements/functional-requirements.md` — FR-060..062
- `/docs/02-requirements/traceability-matrix.md` — dòng FR-060..062
- `/docs/01-product/roadmap.md` — M3/M4 partial cập nhật

## References

- `/AGENTS.md`
- `/docs/01-product/roadmap.md`
