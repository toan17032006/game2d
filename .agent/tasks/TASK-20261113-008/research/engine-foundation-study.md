# Nghiên cứu: Thành phần cần có của một Game Engine "Code Thuần"

- Task: `TASK-20261113-008`
- Phạm vi áp dụng: `monster-engine` (2D trước, 3D sau), Rust, zero-dependency.
- Định nghĩa "code thuần": engine là **thư viện code** — game code gọi API trực tiếp (`main.rs` tạo engine, đăng ký hệ thống, chạy vòng lặp). Không editor, không file scene dựng sẵn, mọi thứ đều là code có thể đọc/hiểu/sửa từng dòng.

---

## 1. Bản đồ tổng thể — 14 subsystem bắt buộc

| # | Subsystem | Vai trò cốt lõi | Thứ tự xây dựng | Điểm mở rộng 3D |
| --- | --- | --- | --- | --- |
| 1 | Math | Nền tảng tính toán mọi hệ thống khác | M0 (đầu tiên) | Vec3/Mat4/Quat có sẵn từ đầu |
| 2 | Core services (time, events, log) | Xương sống điều phối | M0 | Không đổi |
| 3 | Platform (window/input) | Cầu tới hệ điều hành | M1 | Không đổi (3D vẫn cần window) |
| 4 | Renderer + graphics abstraction | Vẽ lên màn hình | M1 | Backend GPU mới, cùng trait |
| 5 | Assets (loader/cache) | Đưa dữ liệu vào engine | M1 | Thêm format mesh/texture 3D |
| 6 | Scene/ECS | Tổ chức trạng thái game | M2 | Component mới, không đổi core |
| 7 | Physics/collision | Vật lý đơn giản, va chạm | M2 | Swap sang 3D physics, cùng trait |
| 8 | Audio | Âm thanh | M2 | Backend đổi, trait giữ nguyên |
| 9 | Application layer (facade) | Gắn kết subsystem, vòng đời | M2 | Không đổi |
| 10 | Memory & data layout | Kiểm soát cấp phát | M3 | Bổ sung allocator 3D |
| 11 | Config/services registry | Tham số runtime | M3 | Không đổi |
| 12 | Diagnostics/profiling | Đo hiệu năng, debug | M3 | Frame stats cả 2D/3D |
| 13 | Tooling/test infrastructure | Kiểm chứng engine | M0–M3 | Golden image dùng chung |
| 14 | Extensibility mechanism | Cơ chế "dễ thêm hàm" | Dọc toàn bộ | Plugin backend |

Nguyên tắc phân tầng (mỗi tầng chỉ được gọi tầng dưới):

```
┌────────────────────────────────────────────┐
│ Game code (ví dụ sandbox, game thật)       │
├────────────────────────────────────────────┤
│ monster-engine (facade + Application)      │
├───────────┬──────────┬───────────┬─────────┤
│ scene     │ physics  │ audio     │ assets  │  ← subsystem tầng trung
├───────────┴──────────┴───────────┴─────────┤
│ render (backend impl)  platform (impl)     │
├────────────────────────────────────────────┤
│ core (traits/contracts, events, time, log) │
├────────────────────────────────────────────┤
│ math                                       │
└────────────────────────────────────────────┘
```

Quyết định kiến trúc số 1: **contract tách khỏi implementation**. Trait (`Renderer`, `AudioBackend`, `PresentTarget`, `InputSource`) nằm ở tầng core; hiện thực cụ thể nằm ở tầng ngoài. Đây chính là cơ chế "dễ thêm hàm, dễ mở rộng 3D": viết `Renderer` mới cho GPU/3D mà không chạm vào scene/game.

---

## 2. Chi tiết từng subsystem

### 2.1 Math — nền móng tính toán

**Vai trò:** mọi hệ thống khác đều dùng; sai ở đây là sai ở mọi nơi. Engine 3D sau này cần đầy đủ: vector, ma trận, quaternion, phép biến đổi không gian.

**Thành phần tối thiểu:**
- `Vec2` (2D chính), `Vec3`/`Vec4` (chuẩn bị 3D), dot/cross/normalize/lerp.
- `Mat4` — phép biến đổi affine: translate, scale, rotate, orthographic cho camera 2D, nhân chuỗi biến đổi.
- `Quat` — xoay không gặp gimbal lock, bắt buộc cho 3D sau.
- `Rect`, `Aabb` (physics/render culling), `Color` (RGBA8/LinearRGBA).
- Nội bộ: fixed-point không cần; dùng `f32`, deterministic cho test.

**Interface đề xuất (Rust):**

```rust
pub struct Vec2 { pub x: f32, pub y: f32 }
impl Vec2 {
    pub fn dot(self, o: Vec2) -> f32;
    pub fn normalize(self) -> Vec2;
    pub fn lerp(self, o: Vec2, t: f32) -> Vec2;
}
pub struct Mat4 { pub cols: [Vec4; 4] }
impl Mat4 {
    pub fn orthographic(left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) -> Mat4;
    pub fn translate(scale: Vec2, rot: f32, pos: Vec2) -> Mat4; // 2D compose
    pub fn mul_mat4(self, o: Mat4) -> Mat4;
}
```

**Lưu ý mở rộng 3D:** giữ `Mat4` là column-major (chuẩn GPU) ngay từ đầu để backend GPU sau này không phải transpose.

### 2.2 Core services — time, event, log

**Vai trò:** xương sống điều phối. Mọi subsystem giao tiếp qua đây thay vì gọi chéo nhau.

**(a) Time / game loop — trái tim của engine:**
- Mô hình chuẩn công nghiệp: **fixed timestep update + variable render** ("Fix Your Timestep!", Glenn Fiedler):
  - `update` chạy đúng `dt = 1/60` giây mô phỏng → physics deterministic, gameplay ổn.
  - `render` mỗi frame hiển thị theo `alpha = accumulator/dt` (interpolation).
- Đồng hồ dùng `std::time::Instant`, `delta_seconds()` bọc an toàn (clamp max 250ms tránh "spiral of death" khi treo máy).

```rust
pub struct FixedClock { /* accumulator */ }
impl FixedClock {
    pub fn tick(&mut self, now: Instant) -> usize; // số bước update phải chạy
    pub fn alpha(&self) -> f32;                    // tỉ lệ cho render
}
```

**(b) Event bus:** queue sự kiện typed, producer/consumer tách rời. Input, window, gameplay đều là event.

```rust
pub enum EngineEvent {
    WindowClosed, WindowResized { width: u32, height: u32 },
    KeyPressed(KeyCode), KeyReleased(KeyCode), MouseMoved { x: f32, y: f32 },
    MouseButton { button: MouseButton, pressed: bool },
}
pub struct EventBus { /* drain từng frame */ }
impl EventBus { pub fn push(&mut self, e: EngineEvent); pub fn drain(&mut self) -> std::vec::IntoIter<EngineEvent>; }
```

**(c) Logging:** có cấu trúc mức độ (`error/warn/info/debug/trace`), tắt được khi release, zero-dep (ghi stderr).

### 2.3 Platform — window, input, thời gian hệ thống

**Vai trò:** phần duy nhất chạm hệ điều hành. Phải bị nhốt sau một biên mỏng để 2D/3D phía trên không biết gì về Win32.

**Thành phần:** tạo window (resize/close/focus), pump tin nhắn OS, bàn phím/chuột (state held/pressed/released), present framebuffer lên màn hình, DPI.

**Interface đề xuất:** `Window` (Win32: `CreateWindowExW` + `PeekMessageW` cho game loop), `Window::poll_events(&mut self, &mut EventBus)`, `Window::present(...)`, trait `PresentTarget` để headless test không cần window thật (NullWindow).

**Lưu ý 3D:** 3D vẫn dùng đúng window này; chỉ thêm backend lấy HWND nạp vào renderer GPU.

### 2.4 Renderer — trừu tượng đồ họa + 2D renderer

**Vai trò:** vẽ hình. Biên trừu tượng quyết định có mở rộng 3D được hay không.

**Định nghĩa biên đúng:**

```rust
pub trait Renderer {
    fn begin(&mut self, camera: &Camera2D);
    fn draw_sprite(&mut self, texture: &Texture, dst: Rect, src: Option<Rect>, opts: DrawOptions);
    fn draw_rect(&mut self, dst: Rect, color: Color);
    fn end(&mut self, target: &dyn PresentTarget);          // flush + present
}
pub struct Texture { width: u32, height: u32, pixels: Vec<u8> } // RGBA8
```

- **2D hiện tại:** software renderer — framebuffer RGBA8, blit sprite có alpha blend, tint, flip, camera offset + zoom, sort z. Present qua GDI (`CreateDIBSection` + `BitBlt`).
- **3D sau này:** backend mới (GPU) implement cùng trait `Renderer`; scene/game code không đổi. Thêm `draw_mesh(camera3d)` vào trait khi lên 3D — đúng 1 điểm mở rộng.
- Khái niệm cần có ở cả 2D/3D: camera (view transform), batch/group theo texture (2D: giảm cache miss; 3D: batch draw call), viewport, clear color.

### 2.5 Assets — nạp và quản lý tài nguyên

**Vai trò:** biến file trên đĩa thành dữ liệu engine (texture, âm thanh, sau này là mesh/material/scene). Mọi engine "code thuần" đều cần hệ thống resource riêng vì không có editor import sẵn.

**Thành phần:**
- Registry + cache theo đường dẫn (load 1 lần, dùng lại; đếm tham chiếu sau này).
- Handles (`Handle<Texture>`) thay vì raw pointer — an toàn bộ nhớ theo cách Rust.
- Decoder thuần: BMP (đơn giản, test được), **PNG** (zlib inflate + chunk + filter — bắt buộc để dùng Tiny Swords), WAV cho audio.
- Vòng đời: load → decode → upload vào renderer (tạo `Texture`) → cache; hot-reload là phần mở rộng.

```rust
pub struct Assets { cache: HashMap<String, CachedResource> }
impl Assets {
    pub fn load_texture(&mut self, path: &str, renderer: &mut dyn Renderer) -> Handle<Texture>;
    pub fn get(&self, h: Handle<Texture>) -> &Texture;
}
```

### 2.6 Scene / ECS — tổ chức trạng thái game

**Vai trò:** nơi game code "sống". Hai mô hình phổ biến: GameObject–Component (Unity) và ECS thuần (Bevy/EnTT). Nền móng nên chọn **ECS-lite**: entity = id, component = dữ liệu thuần (struct), system = hàm xử lý theo query.

**Lý do:** dữ liệu tách khỏi hành vi → thêm component mới không sửa core; system sắp xếp lại được; hướng tới SoA khi cần hiệu năng; đây là mô hình dễ mở rộng nhất cho cả 2D và 3D.

```rust
pub struct World { /* entity generations + component stores theo TypeId */ }
impl World {
    pub fn spawn(&mut self) -> EntityBuilder;
    pub fn despawn(&mut self, e: EntityId) -> bool;
    pub fn get<T: Component>(&self, e: EntityId) -> Option<&T>;
    pub fn iter_mut<T: Component>(&mut self) -> impl Iterator<Item = (EntityId, &mut T)>;
}
pub fn movement_system(world: &mut World, dt: f32) { /* Transform + Velocity */ }
```

Đi kèm: `Camera2D`, layer/z-order (thứ tự vẽ), scene graph nhẹ (parent→transform sau này cho 3D).

### 2.7 Physics / collision

**Vai trò:** tích phân chuyển động (đọc input/velocity → cập nhật vị trí), phát hiện và giải quyết va chạm.

**Thành phần nền móng:** `Aabb` overlap, tích phân Euler cố định bước, broadphase brute-force (nâng cấp uniform grid sau), giải va chạm đẩy tách theo trục penetration nhỏ nhất, gravity/lực đơn giản.

**Mở rộng 3D:** `Aabb` → `Aabb3` + cùng cấu trúc broadphase/narrowphase; engine chỉ đổi kiểu hình học.

### 2.8 Audio

**Vai trò:** phát nhạc/SFX. Nền móng tách rõ 2 phần: **decode format** (WAV PCM thuần Rust, có test) và **playback backend** (trait `AudioBackend`, hiện thực đầu tiên = Null — mute; sau này waveOut/DirectSound/XAudio2 qua FFI, hoặc backend WebAudio khi port web).

```rust
pub trait AudioBackend {
    fn play_clip(&mut self, clip: &AudioClip, volume: f32, looped: bool) -> AudioPlaybackId;
    fn stop_all(&mut self);
}
```

### 2.9 Application layer — facade gắn kết mọi thứ

**Vai trò:** đúng thứ user yêu cầu "dễ thêm hàm": game code chỉ implement 1 trait, engine lo phần còn lại. Builder pattern cấu hình subsystem như lắp lego.

```rust
pub trait GameApp {
    fn init(&mut self, ctx: &mut EngineContext);
    fn update(&mut self, ctx: &mut EngineContext, dt: f32);
    fn render(&mut self, ctx: &mut EngineContext, renderer: &mut dyn Renderer);
    fn shutdown(&mut self, ctx: &mut EngineContext) {}
}

let engine = EngineBuilder::new()
    .with_window(WindowOptions { width: 960, height: 540, title: "Monster" })
    .with_audio_backend(NullAudioBackend::default())
    .build();
engine.run(&mut MyApp);
```

Vòng lặp chuẩn: `poll_events → input → fixed_update×N → render(alpha) → present`, thoát sạch khi đóng window.

### 2.10 Memory & data layout

**Vai trò:** engine thật kiểm soát cấp phát thay vì để mặc định. Nền móng: mọi bộ đệm là `Vec` có capacity dựa trước (framebuffer, batch), pooling cho object nhỏ (particle sau này), tránh cấp phát trong hot loop (per-frame buffers tái sử dụng). Sau này: arena allocator cho frame data, pool cho entity, SoA cho component store lớn. Ghi nhận là **khu vực tối ưu tiếp theo**, không làm sớm.

### 2.11 Config / services registry

**Vai trò:** tham số runtime (kích thước window, volume, seed, đường dẫn asset) không nhét cứng vào code. Nền móng: struct `EngineConfig` + `Services` struct truyền qua `EngineContext` (đủ dùng, không cần DI framework).

### 2.12 Diagnostics & profiling

**Vai trò:** đo được mới tối ưu được. Nền móng: frame counter, fps trung bình, thời gian update/render (đo bằng `Instant`), log mức per-frame tắt được; assert engine (`debug_assert`) cho invariant. Sau này: overlays HUD, CPU/GPU timeline.

### 2.13 Tooling & test infrastructure — bảo đảm engine không hỏng

**Vai trò:** engine là thư viện dài hạn → không có test là không dám refactor. Chiến lược zero-dep:
- Test thuần logic (math/ECS/physics/inflate) — không cần window.
- **Headless rendering:** software renderer xuất framebuffer → so pixel trực tiếp trong test (golden image nhỏ, không cần file).
- `NullWindow` + `NullAudioBackend` → toàn vòng lặp engine chạy không GUI trong CI.
- Smoke test PNG thật từ `/assest/` (bắt sai lệch decoder sớm).

### 2.14 Extensibility mechanism — cơ chế "dễ thêm hàm"

Tổng hợp các cơ chế được áp dụng xuyên suốt:
1. **Trait contracts ở core** — thêm backend mới = thêm crate/crate module mới, core không đổi (đã chứng minh ở §2.4/§2.8).
2. **Component as data** — thêm feature gameplay = thêm struct component + system, không sửa `World`.
3. **Builder pattern** — cấu hình engine khai báo, không sửa nguồn engine.
4. **Crate biên cứng** — dependency một chiều (biểu đồ §1), cấm cycle.
5. **Event bus** — hệ thống mới nghe sự kiện có sẵn mà không chạm hệ thống cũ.
6. **Không để subsystem gọi chéo ngang** — đều đi qua service/trait ở core.

---

## 3. Những thứ KHÔNG cần ở nền móng (tránh over-engineering)

- Editor, asset pipeline GUI, hot-reload — chỉ cần khi team tăng lên.
- Scripting language (Lua/AngelScript) — Rust trait đủ cho giai đoạn này.
- Multi-thread job system, render thread tách rời — chỉ đáng khi profile cho thấy nút cổ chai.
- GPU renderer ngay từ đầu — software 2D đủ cho nền móng, test headless được, và buộc biên `Renderer` tách sạch trước khi thêm GPU.
- Archetype-ECS phức tạp, allocator tùy chỉnh, async runtime.

## 4. Lộ trình gợi ý sau nền móng (M0–M2 của task này)

- **M3:** GPU backend (render 3D: mesh, depth buffer, shader qua trait `Renderer` mở rộng) + physics 3D.
- **M4:** sprite animation (spritesheet Tiny Swords), tilemap, particle, text, audio playback thật, camera follow/shake, UI-less menu flow.
- **M5:** đóng gói thành game hoàn chỉnh trên asset Tiny Swords.

## 5. Kết luận

Một engine code thuần tối thiểu = **math + core services + platform + render abstraction + assets + ECS + physics + audio + application facade**, được giữ bằng 3 cơ chế: contract/tách implementation (trait), dữ liệu tách hành vi (ECS), biên crate một chiều. Cả 3 đều được hiện thực trong `monster-engine` nền móng này; tài liệu ADR draft kèm theo ghi quyết định chính thức.

## References

- `Fix Your Timestep!` — Glenn Fiedler (fixed timestep pattern).
- `Game Programming Patterns` — Robert Nystrom (game loop, event queue, object pool, update method).
- `Handmade Hero` (Casey Muratori) — mô hình platform layer tách biên với engine.
- Hoạt động thực tế: Bevy (ECS), Macroquad/raylib (2D API bề mặt), SDL (platform layer).
