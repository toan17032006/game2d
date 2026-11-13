# Specification

## Inputs

- Yêu cầu người dùng: "Commit và thực hiện đề xuất" (M3 slice 2 mesh loading + M4 sprite animation/Tiny Swords).
- Asset thật: Tiny Swords Warrior sheets 192×192/frame (Idle 8 frames 1536×192).

## Confirmed facts

- F-01: Repo sạch trên `1e6dcae`; 173 tests PASS.
- F-02: Warrior sheets: Idle 1536×192 (8 frame), Attack1/Run 768×192 (4 frame), Guard 1152×192 (6 frame) — cols = width/192.
- F-03: Physics Character body + move_and_slide + camera_follow_system + physics_attach đã có.

## Inferences

- I-1 (inference): Grid-slice animation (không metadata) là cách chuẩn cho Tiny Swords vì mọi sheet cùng chiều cao 192.
- I-2 (inference): OBJ là format mesh text tối giản phù hợp "mesh loading" M3 slice 2 (v + f, tam giác/quad).

## Unknowns

- U-1: Tọa độ frame trong sheet Tiny Swords có padding lạ không — grid-slice chuẩn; nếu lệch, demo vẫn hoạt động (visual check thủ công).

## Constraints

- C-01: Zero-dep; 173 tests cũ PASS.
- C-02: `Renderer3D::draw_mesh` thêm vào trait → phải implement cho Gl3DRenderer + mock (không break).

## Decisions

- D-01: `AnimationClip` lưu frames là offsets (px); `Animator` component giữ clip + time + frame + playing; system set `Sprite.src`.
- D-02: `AnimationClip::from_grid(sheet_w, sheet_h, frame_w, frame_h, count, fps, looping)` row-major.
- D-03: `Mesh3D { vertices: Vec<Vec3>, indices: Vec<u32>, colors: Option }` + `Mesh3D::cube(half)`; OBJ loader chỉ parse `v`/`f` (tam giác hoát quad), trả Mesh3D; lỗi → Err(String).
- D-04: `draw_mesh(mesh, position, rotation_y, color)` thêm vào trait `Renderer3D`; GL emit tam giác qua GL_TRIANGLES immediate.
- D-05: Sandbox `--game`: mặt đất + player Character (auto-walk right headless / A-D window) + Warrior_Run animation (nếu tìm thấy sheet, fallback procedural checker) + 3 coin Area + camera follow target=player; score = coin collected; headless exit khi player đi hết đường hoặc đủ steps; assert: player.x tiến, animation frame > 0 thay đổi, score ≥ 1, camera đã dịch theo.

## Output contract

- animation.rs + Mesh3D/OBJ + draw_mesh + sandbox --game + docs FR-053..055 + commit.

## Test Command

`cargo test --workspace --manifest-path monster-engine/Cargo.toml`

## Approved Documentation Changes

- `/docs/02-requirements/functional-requirements.md` — FR-053..055
- `/docs/02-requirements/traceability-matrix.md` — dòng FR-053..055
- `/docs/01-product/roadmap.md` — M4 partial ✅

## References

- `/AGENTS.md`
- `/docs/01-product/roadmap.md`
