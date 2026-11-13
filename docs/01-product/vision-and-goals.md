# Vision & Goals — monster-engine

- Owner: Product
- Status: `Approved`
- Last updated: `2026-11-13`
- Review date: `2027-05-13`

## Purpose

Ghi rõ tầm nhìn và mục tiêu của sản phẩm `monster-engine`, làm căn cứ cho requirements và architecture.

## Tầm nhìn

`monster-engine` là một **game engine "code thuần"**: thư viện code mà game developer gọi API trực tiếp — không editor, không cấu hình GUI, mọi thứ đều là code có thể đọc/hiểu/sửa từng dòng. Engine bắt đầu với 2D, được thiết kế để mở rộng lên 3D mà không phải viết lại.

> "Xây dựng 1 engine game 2D, sau này có thể mở rộng thành game 3D. Engine dễ mở rộng, dễ thêm hàm."

## Mục tiêu (Goals)

| ID | Mục tiêu | Cách đo |
| --- | --- | --- |
| G-01 | Engine 2D chạy được end-to-end (window → update → render → present) | Sandbox demo chạy headless + window mode |
| G-02 | Dễ mở rộng: thêm backend mới không sửa core | Thêm implementation của trait có sẵn (`Renderer`, `AudioBackend`) |
| G-03 | Dễ thêm hàm: thêm gameplay không sửa engine | Component + system mới trong ECS |
| G-04 | "Code thuần": kiểm soát và hiểu được từng phần | Zero external crate; decoder PNG/BMP/WAV tự viết |
| G-05 | Đường lên 3D sẵn sàng | Vec3/Mat4/Quat từ đầu; `Renderer` là biên trừu tượng duy nhất |
| G-06 | Kiểm chứng được tự động | Headless test (NullWindow), 105 tests toàn workspace |

## Nguyên tắc sản phẩm

1. **Hiểu trước, tối ưu sau** — nền móng ưu tiên đúng và rõ hơn nhanh và ẩn.
2. **Contract tách khỏi implementation** — trait ở core, impl ở crate ngoài.
3. **Dữ liệu tách khỏi hành vi** — ECS-lite.
4. **Test headless-first** — không phụ thuộc GUI để kiểm chứng.
5. **Tài liệu tiếng Việt** — toàn bộ docs dự án viết tiếng Việt.

## Thành công trước mắt (đã đạt)

- Nền móng M0–M2 hoàn tất: math, core, platform, render 2D, assets (PNG thật Tiny Swords decode được), ECS, physics AABB, audio contract, facade, sandbox — 105 tests PASS, quality gates PASS (TASK-20261113-008).

## References

- `/docs/01-product/scope-and-non-goals.md`
- `/docs/01-product/roadmap.md`
- `/docs/03-architecture/adr/ADR-001-monster-engine-foundation.md`
- `/.agent/tasks/TASK-20261113-008/verification.md`
