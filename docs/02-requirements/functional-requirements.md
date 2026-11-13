# Functional Requirements — monster-engine v0.1.0

- Owner: Product and Engineering
- Status: `Approved`
- Last updated: `2026-11-13`
- Review date: `2027-05-13`

## Purpose

Danh mục yêu cầu chức năng của nền móng engine, mỗi yêu cầu có ID, priority, acceptance criteria và module đảm nhiệm. Truy vết tới test xem `traceability-matrix.md`.

## Quy ước

- ID dạng `FR-NNN`; priority: `Must` (nền móng không thể thiếu), `Should`, `Could`.
- Acceptance criteria phải kiểm chứng được bằng test headless hoặc lệnh chạy thật.
- Trạng thái: ✅ = đã hiện thực + test PASS (TASK-20261113-008).

## FR — Math & Core services

| ID | Yêu cầu | Priority | Acceptance criteria | Module | Trạng thái |
| --- | --- | --- | --- | --- | --- |
| FR-001 | Vec2/Vec3/Vec4 đủ phép toán cơ bản (dot, cross, normalize, lerp, ops) | Must | normalize của (3,4) = (0.6, 0.8); ZERO normalize = ZERO | monster-math | ✅ |
| FR-002 | Mat4 column-major: identity, mul, translate, scale, rotate_z, orthographic, perspective, view_2d | Must | ortho map [0,w]→[-1,1]; view_2d đưa vị trí camera về clip (0,0) | monster-math | ✅ |
| FR-003 | Quat: from_axis_angle, mul, rotate_vec3, normalize | Must | quay +X quanh Z 90° cho +Y; composition đúng thứ tự | monster-math | ✅ |
| FR-004 | EventBus typed FIFO, drain per-frame | Must | drain đúng thứ tự push, queue rỗng sau drain | monster-core | ✅ |
| FR-005 | FixedClock accumulator: tick trả số step, alpha cho interpolation, clamp chống spiral | Must | 25ms delta với step 10ms → 2 steps + alpha 0.5; stall 10s → 10 steps | monster-core | ✅ |
| FR-006 | InputState: held/pressed/released per key + mouse + wheel, fold từ EventBus | Must | press→hold→release chu kỳ đúng; dense index duy nhất | monster-core | ✅ |
| FR-007 | Logging theo mức (error/warn/info/debug/trace) ra stderr, lọc theo MONSTER_LOG | Should | should_log lọc đúng; macro `log_info!` dùng được từ crate khác | monster-core | ✅ |
| FR-008 | Handle<T> typed index an toàn kiểu | Must | Handle<Texture> khác Handle<AudioClip> (compile-time); Copy cho mọi T | monster-core | ✅ |

## FR — Platform

| ID | Yêu cầu | Priority | Acceptance criteria | Module | Trạng thái |
| --- | --- | --- | --- | --- | --- |
| FR-009 | Cửa sổ Win32 với client size đúng qua `WindowOptions`, đóng sạch (WM_CLOSE→Destroy→WM_QUIT→open=false) | Must | compile-link user32; logic wndproc mapping đúng | monster-platform | ✅ (window thật chạy thủ công) |
| FR-010 | Map VK code → KeyCode phủ chữ/số/control; Unknown cho key lạ | Must | 0x41→A, 0x1B→Escape, 0xFF→Unknown | monster-platform | ✅ |
| FR-011 | NullWindow: headless substitute đếm present, đóng được programmatic | Must | 2 presents → count=2; sau close → ignore | monster-platform | ✅ |
| FR-012 | Present qua GDI DIB 32bpp top-down, RGBA→BGRA, recreate khi resize | Must | (compile-link; verify trực quan window mode) | monster-platform | ✅ |

## FR — Render

| ID | Yêu cầu | Priority | Acceptance criteria | Module | Trạng thái |
| --- | --- | --- | --- | --- | --- |
| FR-013 | Framebuffer RGBA8: blend_pixel "over" (255=overwrite, 0=no-op, giữa=blend có rounding), fill_rect clip biên | Must | 50% đỏ trên trắng → (255,~127,~127); fill ngoài biên không panic | monster-render | ✅ |
| FR-014 | blit_sprite: sub-rect source, tint nhân kênh, flip_x/flip_y, nearest sampling, clip dst+src, không panic out-of-bounds | Must | sprite lệch biên chỉ tô pixel giao; flip đối xứng | monster-render | ✅ |
| FR-015 | Camera2D: world_to_screen/screen_to_world + view matrix (position = tâm màn hình, zoom) | Must | roundtrip world→screen→world khớp; zoom 2 đôi offset | monster-core/render | ✅ |
| FR-016 | Renderer trait protocol begin→draw→end; begin clears + reset counter; end present | Must | draw_count reset giữa 2 frame; end submit đúng buffer | monster-core/render | ✅ |

## FR — Assets

| ID | Yêu cầu | Priority | Acceptance criteria | Module | Trạng thái |
| --- | --- | --- | --- | --- | --- |
| FR-017 | Registry cache theo path: load lại trả cùng Handle, không decode lại | Must | cùng key → cùng Handle; replace giữ handle | monster-assets | ✅ |
| FR-018 | PNG decoder: bit depth 8, color type 0/2/3/4/6, tRNS (palette + gray/RGB key), filter 0-4, CRC verify, từ chối signature/interlace sai | Must | fixture RGBA/palette+tRNS/gray+tRNS/Sub-filter decode khớp; corrupt CRC → CrcMismatch; decode PNG thật Tiny Swords 192×256 | monster-assets | ✅ |
| FR-019 | BMP decoder BI_RGB 24/32bpp, top-down + bottom-up, row padding | Must | bottom-up 2×2 đọc đúng thứ tự hàng trực quan; negative height top-down | monster-assets | ✅ |
| FR-020 | Inflate DEFLATE đủ 3 block type (stored/fixed/dynamic) + zlib wrapper (header %31, adler32 verify) | Must | stored roundtrip; fixed literals/back-reference; adler sai → AdlerMismatch | monster-assets | ✅ |

## FR — Scene & Physics

| ID | Yêu cầu | Priority | Acceptance criteria | Module | Trạng thái |
| --- | --- | --- | --- | --- | --- |
| FR-021 | ECS: spawn/despawn, EntityId generational chặn stale id, component add/remove/get_mut | Must | id cũ sau despawn → get None; slot reuse đổi generation | monster-scene | ✅ |
| FR-022 | Join query for_each2 + iter_mut lọc entity chết | Must | chỉ entity có cả 2 component được visit | monster-scene | ✅ |
| FR-023 | movement_system tích phân position += vel*dt; sprite_draw_order sort theo ZOrder | Must | dt=2, vel=10 → +20; sort -2 → 0 → 5 | monster-scene | ✅ |
| FR-024 | Aabb2: intersects (edge-touch tính), separation MTV trục nhỏ nhất, collision_normal unit, bounce restitution | Must | MT V y=-1 khi overlap y=1; bounce ngược chiều đúng hệ số | monster-physics | ✅ |
| FR-025 | Broadphase find_collisions trả các cặp index chồng lấn | Must | 3 box → 1 pair đúng | monster-physics | ✅ |

## FR — Audio

| ID | Yêu cầu | Priority | Acceptance criteria | Module | Trạng thái |
| --- | --- | --- | --- | --- | --- |
| FR-026 | WAV decoder PCM 8/16/24-bit, multi-channel mixdown mono, chunk alignment, error typed | Must | stereo (1000,-500) → 250; odd chunk + pad đọc đúng data | monster-audio | ✅ |
| FR-027 | AudioBackend trait + NullAudioBackend: id tăng dần, stop/stop_all, volume clamp | Must | 2 clips → id khác nhau; volume 4.0 → 1.0 | monster-core/audio | ✅ |

## FR — Facade

| ID | Yêu cầu | Priority | Acceptance criteria | Module | Trạng thái |
| --- | --- | --- | --- | --- | --- |
| FR-028 | EngineBuilder: with_window/headless/with_fixed_fps/with_clear_color/with_audio_backend | Must | hoán đổi audio backend không sửa core (test PASS) | monster-engine | ✅ |
| FR-029 | Engine::run: init 1 lần, fixed update ×N, render mỗi frame, shutdown 1 lần, thoát qua ctx.exit() hoặc đóng window | Must | CountingApp: inits=1, shutdowns=1, frames==renders; exit → updates=1 | monster-engine | ✅ |
| FR-030 | Engine tự vẽ world sprites (z-sorted) trước app.render; resize đồng bộ camera + framebuffer | Must | renderer.draw_count ≥ 1 trong render callback | monster-engine | ✅ |
| FR-031 | Sandbox example: `--headless N` exit 0 sau ≥N frames; `--window` mở cửa sổ 960×540 | Must | --headless 120 → 122 frames, exit 0 | monster-engine (example) | ✅ |

## FR — Physics v2 (TASK-20261113-010)

Yêu cầu hệ vật lý đầy đủ theo taxonomy người dùng đặt ra. Quyết định ADR-001 là 2D-first: các shape 3D được ánh xạ tương đương 2D có chủ đích (Sphere→Circle, Capsule/Cylinder→Stadium, Cone→triangle, Mesh→polygon, Terrain→cột khối). Phiên bản 3D thật thuộc M3.

| ID | Yêu cầu | Priority | Acceptance criteria | Module | Trạng thái |
| --- | --- | --- | --- | --- | --- |
| FR-032 | `PhysicsWorld2` pipeline: integrate → broadphase → narrowphase → impulse resolution (restitution + Coulomb friction) + positional correction (Baumgarte) | Must | bi rơi 120 bước đọng trên ground tại y≈180, vy≈0; restitution 0.9 nảy lên > 40px | monster-physics::world | ✅ |
| FR-033 | 15 loại collider trong `ColliderShape2` (mapping 2D ghi rõ) + narrowphase thật từng cặp qua flatten-to-primitives | Must | test từng loại: circle/box/stadium/cone/plane/convex/concave/heightfield/terrain/compound/wheel/custom đều có contact đúng hướng + độ xuyên đúng | monster-physics::shape/collision | ✅ |
| FR-034 | Static Body (không di chuyển, inv_mass=0) + Rigid Body (gravity + response) | Must | static không đổi; rigid đọng trên ground | monster-physics::world | ✅ |
| FR-035 | Kinematic Body + Animatable Body: di chuyển bằng velocity, không bị gravity/response đẩy, ĐẨY được dynamic body | Must | kinematic đẩy rigid > 29px; animatable không rơi | monster-physics::world | ✅ |
| FR-036 | Area / Trigger Body: phát `TriggerEntered`/`TriggerExited`, không response | Must | visitor đi xuyên area: entered rồi exited; vị trí xuyên suốt không bị chặn | monster-physics::world | ✅ |
| FR-037 | Character Body: `move_and_slide` tách trục, on-floor/on-ceiling/wall | Must | character đi bộ đọng trên sàn (y≈75); tường chặn x < 85 | monster-physics::world | ✅ |
| FR-038 | Physical Bone: segment stadium giữa 2 điểm | Must | bone tạo đúng stadium (half_length, radius) | monster-physics::joint | ✅ |
| FR-039 | Articulated Body: chuỗi bone nối Pin joint với static anchor | Must | chuỗi 4 bone sau 90 bước vẫn dính nhau (khoảng cách < 45px) | monster-physics::joint | ✅ |
| FR-040 | Soft Body: ring PBD + neighbour/cross springs | Must | blob rơi 120 bước: không lọt ground, không vỡ (spread < 120px) | monster-physics::particles | ✅ |
| FR-041 | Cloth Body: grid structural+shear springs, pin hàng đầu | Must | pin đứng yên; phần dưới rơi treo > 60px | monster-physics::particles | ✅ |
| FR-042 | Vehicle Body: chassis + wheel raycast suspension (Hooke + damper) + drive | Must | chassis đọng trên suspension (bottom < ground−5); drive di chuyển > 30px | monster-physics::vehicle | ✅ |
| FR-043 | Particle Body: point particles độc lập, có velocity clamp chống xuyên | Must | 40 hạt rơi 180 bước đọng trên ground (150..193) | monster-physics::particles | ✅ |
| FR-044 | Fluid Body: SPH-lite pressure repulsion | Must | khối fluid rơi đọng trên ground và loang rộng hơn ban đầu | monster-physics::particles | ✅ |
| FR-045 | Deformable Body: shape memory spring về rest offsets | Must | rơi đọng trên ground, hình ring phục hồi ≥ 60% rest width | monster-physics::particles | ✅ |
| FR-046 | Joints/Constraints: Distance, Pin, Weld, Spring (impulse + Baumgarte, clamp chống nổ) | Must | distance giữ đúng 100px ± 6; pin giữ anchor < 8px; weld khóa góc < 0.3 rad; spring đọng ~rest length | monster-physics::joint | ✅ |
| FR-047 | Raycast API: `Ray2`/`RayHit2` quét world, nearest hit, mọi shape (kể cả heightfield) | Must | ray trúng circle t=80; box face t=90; heightfield t=50; nearest thắng; quá ngắn → None | monster-physics::raycast | ✅ |
| FR-048 | Custom Collider: trait `CustomShape2` (narrowphase/contains/bound/raycast do user viết) | Must | test custom half-plane: penetration + normal đúng | monster-physics::shape/collision | ✅ |

## FR — Physics Bridge (TASK-20261114-001)

| ID | Yêu cầu | Priority | Acceptance criteria | Module | Trạng thái |
| --- | --- | --- | --- | --- | --- |
| FR-049 | PhysicsBridge đồng bộ ECS ↔ `PhysicsWorld2` qua facade: component `PhysicsLink`, `EngineBuilder::with_physics(gravity)`, `UpdateContext.physics`/`InitContext.physics`; hướng sync: dynamic body → scene Transform, static/kinematic/area → scene → body | Must | integration test: entity rơi theo physics qua `engine.run` (y đọng ≈ body); entity không link không bị đụng; engine không bật physics → ctx.physics = None; sandbox `--physics` exit 0 (240 bước, 6 bóng đọng, sprites khớp body < 1px, 12 trigger events) | monster-engine::physics_bridge | ✅ |

## FR — Camera follow + M3 3D slice (TASK-20261114-002)

| ID | Yêu cầu | Priority | Acceptance criteria | Module | Trạng thái |
| --- | --- | --- | --- | --- | --- |
| FR-050 | Camera follow: `camera_follow_system` theo mượt target (exponential smoothing theo speed*dt, không teleport) + offset; target despawn → camera giữ vị trí | Must | 1 bước camera đi một phần đường; 180 bước đạt target ±1px; offset đặt camera sau/lưng target; target despawn không panic | monster-scene::systems | ✅ |
| FR-051 | 3D camera + trait: `Mat4::look_at` (right-handed) + `Camera3D` (position/look_target/fov/aspect) + `view_projection` + trait `Renderer3D` (begin/draw_cube/draw_grid/end) | Must | look_at đưa eye về gốc tọa độ view, trục +X giữ bên phải; Camera3D defaults hợp lệ (fov 60°, near < far); mock renderer chạy đủ protocol | monster-math/monster-core | ✅ |
| FR-052 | GL backend (M3 slice): WGL legacy context (`GlContext` 32-bit màu/24-bit depth/double-buffer) + `Gl3DRenderer` implement `Renderer3D` (cube 6 mặt + grid + depth test + perspective frustum); hoạt động bound (GL thật) và unbound (headless recorder); example `cube_3d` | Must | link opengl32 thành công; unbound renderer chạy đủ frame protocol headless; pixel format descriptor 40 bytes đúng layout; demo window chạy thủ công | monster-platform::gl + monster-engine::gl3d | ✅ |

## FR — Animation + Mesh + Game slice (TASK-20261114-003)

| ID | Yêu cầu | Priority | Acceptance criteria | Module | Trạng thái |
| --- | --- | --- | --- | --- | --- |
| FR-053 | Sprite animation: `AnimationClip` (grid-slice row-major) + component `Animator` + `sprite_animation_system` (advance, loop wrap, non-loop clamp + stop, pause) viết frame rect vào `Sprite.src` | Must | from_grid offsets đúng row-major; loop wrap (7 % 4 = 3); non-loop clamp frame cuối + playing=false; pause đứng yên | monster-scene::animation | ✅ |
| FR-054 | Mesh loading (M3 slice 2): `Mesh3D` (vertices/indices, cube builder) + OBJ loader thuần (v/f, fan triangulation, negative indices, lỗi có mô tả) + `Renderer3D::draw_mesh` (mock đếm + GL immediate triangles) | Must | cube 8 verts/12 tris; OBJ fixture parse indices đúng; lỗi OBJ (thiếu tọa độ, face 2 index, index ngoài range) → Err mô tả; draw_mesh unbound đếm 1 | monster-core::mesh + gl3d | ✅ |
| FR-055 | Tiny Swords game slice: sandbox `--game` — Character player (move_and_slide, input A/D hoặc auto-walk headless) + Warrior_Run animation (grid-slice từ PNG thật) + camera follow + 3 coin Area (score) | Must | headless exit 0: player đi > 150px, animation đổi frame > 2 lần, score = 3/3 coins, camera follow (di chuyển theo player), demo gốc + --physics không regression | monster-engine::examples | ✅ |

## FR — M4 slice 2: HUD + tilemap + particles + audio (TASK-20261114-004)

| ID | Yêu cầu | Priority | Acceptance criteria | Module | Trạng thái |
| --- | --- | --- | --- | --- | --- |
| FR-056 | Text HUD: blocky bitmap font 3×5 (0-9, A-Z, punctuation gọn) + `draw_text_screen` (screen-space, integer scale, uppercase-insensitive, ký tự lạ → space); `Renderer` trait default no-op | Must | glyphs ≤ 3 bits/hàng; text_width/height đúng; draw_text ghi pixel trong glyph box; scale + advance đúng | monster-render::text | ✅ |
| FR-057 | Tilemap: `spawn_tilemap` grid {Empty, Solid} → sprite entity + static body per Solid (pairs trả về cho bridge), Empty bỏ qua | Must | 2/4 solid → 2 entity + 2 body đúng vị trí; all-empty → 0 | monster-scene::tilemap | ✅ |
| FR-058 | Particles 2D: `ParticleEmitter2D` (burst + continuous rate, cone spread, gravity, lifetime, color lerp) + `particle_emit_system` + engine render pass vẽ draw_rect world-space | Must | burst 8 hạt → di chuyển → chết hết sau lifetime; continuous rate 10/s sinh > 5; color lerp trắng→đen midpoint xám | monster-scene::particles2d | ✅ |
| FR-059 | Audio playback thật: winmm FFI (`waveOut*`) + `WinmmAudioBackend` implement `AudioBackend` (mono PCM16, volume, lazy device open, reset/unprepare cleanup); thiếu device → constructor lỗi mô tả → fallback Null | Must | link winmm; construction báo device count; play/stop/stop_all state machine PASS (hoặc skip-graceful khi không device); sandbox --game dùng winmm/Null | monster-platform::winmm_audio | ✅ |

## FR — M4 slice 3 + M3 slice 3 (TASK-20261114-005)

| ID | Yêu cầu | Priority | Acceptance criteria | Module | Trạng thái |
| --- | --- | --- | --- | --- | --- |
| FR-060 | Sprite culling: `sprite_in_view` (viewport camera + margin) — engine render bỏ qua sprite ngoài view | Must | trong view → true; mép (half tính vào) → true; ngoài → false; margin cứu sprite sát mép | monster-scene::systems | ✅ |
| FR-061 | Tilemap multi-layer: 2 layer (background + foreground) trên cùng texture — entities/bodies cộng dồn, không đè vị trí | Must | 2 layer × 2 solid → 4 entity + 4 body, positions khác nhau | monster-scene::tilemap | ✅ |
| FR-062 | M3 slice 3: extension loader (`wglGetProcAddress`, guard null từng function) + shader compile/link (GLSL 120 pass-through + `u_tint` uniform, info-log errors) + Gl3DRenderer shader path với fallback fixed-function khi functions không khả dụng | Must | loader không context → `modern_available` false → compile_program Err → Gl3DRenderer fallback; shader sources GLSL 120 pass-through | monster-platform::gl + monster-engine::gl3d | ✅ |
| FR-062a | Music WAV: `load_wav_file` đọc + decode file WAV thành AudioClip (looped playback qua winmm đã có) | Should | fixture WAV temp file → decode khớp samples + sample_rate | monster-audio | ✅ |

## FR — Kiến trúc tách game/engine (TASK-20261114-007)

| ID | Yêu cầu | Priority | Acceptance criteria | Module | Trạng thái |
| --- | --- | --- | --- | --- | --- |
| FR-066 | Game tách hoàn toàn khỏi engine: `/game/` binary crate phụ thuộc `monster-engine` path dependency; engine facade KHÔNG chứa game logic (không GamePhase, không coin collect, không level flow); game import qua prelude-only | Must | `cargo build` từ /game/ exit 0; `cargo run -- --headless 120 --game` exit 0; engine tests 205 PASS không regression; engine src không chứa GamePhase/sandbox | /game/ + /monster-engine/ | ✅ |

## References

- `/docs/02-requirements/non-functional-requirements.md`
- `/docs/02-requirements/traceability-matrix.md`
- `/docs/03-architecture/container-and-component.md`
- `/.agent/tasks/TASK-20261113-008/verification.md`
