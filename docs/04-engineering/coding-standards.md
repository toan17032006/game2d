# Coding Standards — monster-engine

- Owner: Engineering
- Status: `Approved`
- Last updated: `2026-11-13`
- Review date: `2027-05-13`

## Purpose

Chuẩn viết code Rust trong `monster-engine`. Mọi quy tắc dưới đây đã được áp dụng trong 9 crate hiện có và được gates/tests bảo vệ.

## Nguyên tắc nền

1. **Zero external dependency** — chỉ `std` + FFI hệ điều hành. Cần crate ngoài → phải có ADR mới bỏ qua quy tắc này.
2. **Contract tách khỏi implementation** — trait ở `monster-core`, implementation ở crate ngoài.
3. **Dependency một chiều** — crate dưới không được biết crate trên (biểu đồ trong `container-and-component.md`).
4. **`unsafe` hẹp nhất có thể** — chỉ trong `monster-platform::ffi` và caller trực tiếp, có comment giải thích.

## Rust conventions

| Mục | Quy tắc |
| --- | --- |
| Formatting | `cargo fmt` bắt buộc trước commit |
| Naming | chuẩn Rust: snake_case hàm/biến, PascalCase type, SCREAMING_SNAKE const |
| Doc comment | mọi public item có `///` (rustdoc); crate root có `//!` mô tả |
| Struct public | field private + accessor khi bất biến (ví dụ `Texture.width()`); struct dữ liệu thuần (component) cho field public |
| Derive | dữ liệu thuần: `Debug, Clone, Copy, PartialEq` khi hợp lệ; `Handle<T>` implement Clone/Copy thủ công cho mọi T |
| Error | enum riêng mỗi crate + `impl Display`; không dùng `panic!` ngoài các bất biến lập trình (`assert_eq!` trong test, `debug_assert!` cho invariant) |
| Const | hằng số Win32 đặt trong `ffi.rs` cùng nhóm API |

## Tổ chức test

- Unit test: `#[cfg(test)] mod tests` cùng file, tên test mô tả hành vi (`press_hold_release_cycle`).
- Integration test: `crates/monster-engine/tests/` cho vòng lặp engine.
- Test fixture: `crates/monster-assets/src/fixtures.rs` (`#[cfg(test)]`) — encoder PNG nhỏ cho decoder test.
- Mỗi FR (xem traceability-matrix) phải có ít nhất 1 test thật.
- Test headless: KHÔNG tạo window/audio thật; dùng NullWindow/NullAudioBackend/framebuffer assertions.

## Quy tắc riêng theo subsystem

| Subsystem | Quy tắc thêm |
| --- | --- |
| math | f32, deterministic; Mat4 column-major; test dùng `approx_eq(eps)` |
| platform | FFI trong `ffi.rs`; struct `#[repr(C)]` khớp ABI; wndproc state qua `Mutex` static |
| render | RGBA8 là định dạng duy nhất; vẽ theo call-order (sort z là việc của scene); không panic khi out-of-bounds |
| assets | decoder verify checksum (CRC32/adler32); error enum typed; registry trả `Handle<T>` |
| scene | component là struct dữ liệu thuần; system là hàm thuần `fn(&mut World, ...)`;
| engine facade | game code chỉ chạm `prelude` + contexts; không export implementation cụ thể ra ngoài trait |

## Anti-patterns cấm

- `unwrap()`/`expect()` trong code runtime (cho phép trong test và init có điều kiện).
- Dependency cycle giữa crate.
- Thêm crate ngoài không qua ADR.
- TODO/placeholder trong output hoàn tất của task.
- Logic gameplay viết trong engine core (core không biết gì về game cụ thể).

## Ví dụ tuân thủ (trích code thật)

```rust
// Trait contract ở core — implementation ở crate khác
pub trait Renderer {
    fn begin(&mut self, camera: &Camera2D);
    fn draw_sprite(&mut self, texture: &Texture, dst: Rect, src: Option<Rect>, opts: DrawOptions);
    fn draw_rect(&mut self, dst: Rect, color: Color);
    fn end(&mut self, target: &mut dyn PresentTarget);
    fn draw_count(&self) -> u32;
}

// System là hàm thuần trên ECS
pub fn movement_system(world: &mut World, dt: f32) {
    world.for_each2::<Transform, Velocity, _>(|_id, transform, velocity| {
        integrate(&mut transform.position, velocity.0, dt);
    });
}
```

## References

- `/docs/04-engineering/local-development.md`
- `/docs/04-engineering/development-workflow.md`
- `/docs/04-engineering/api-guidelines.md`
- `/docs/02-requirements/traceability-matrix.md`
- `/monster-engine/crates/` (nguồn sự thật)
