# Non-functional Requirements — monster-engine v0.1.0

- Owner: Product and Engineering
- Status: `Approved`
- Last updated: `2026-11-13`
- Review date: `2027-05-13`

## Purpose

Các yêu cầu phi chức năng: kiến trúc, kiểm chứng, hiệu năng, bảo trì, môi trường. Mỗi NFR có cách đo và trạng thái so với nền móng v0.1.0.

| ID | Yêu cầu | Priority | Cách đo / Acceptance criteria | Trạng thái |
| --- | --- | --- | --- | --- |
| NFR-01 | **Zero external dependency** — chỉ `std` + FFI hệ điều hành | Must | Mọi `Cargo.toml` chỉ có dependency path nội bộ | ✅ |
| NFR-02 | **Kiểm chứng headless** — toàn bộ test không cần GUI/màn hình | Must | `cargo test --workspace` exit 0, 105 tests | ✅ |
| NFR-03 | **Hiệu năng render nền móng** — sandbox headless ≥ 60 fps | Should | sandbox headless 122 frames / 0.2s → avg ~660 fps (debug build) | ✅ |
| NFR-04 | **Deterministic simulation** — update fixed timestep, kết quả lặp lại được | Must | FixedClock step cố định; clamp 250ms; test accumulator chính xác | ✅ |
| NFR-05 | **An toàn bộ nhớ** — không UB trong safe code; biên unsafe hẹp | Must | unsafe chỉ trong `ffi.rs` + caller trực tiếp; test không panic | ✅ |
| NFR-06 | **Khả mở rộng** — thêm backend không sửa core | Must | test hoán đổi `AudioBackend` PASS; trait ở `monster-core` | ✅ |
| NFR-07 | **Khả bảo trì** — crate biên cứng, dependency một chiều, tên rõ | Must | biểu đồ dependency trong container-and-component.md; doc comment trên public API | ✅ |
| NFR-08 | **Thời gian build** — workspace debug build nhanh (< 10s tăng dần sau lần đầu) | Should | build tăng dần ~0.02-0.8s mỗi crate trên máy tham chiếu | ✅ |
| NFR-09 | **Chất lượng code tự động** — rustfmt chuẩn, gates chặn lỗi lặp lại | Should | `cargo fmt` sạch; preflight/final gates PASS | ✅ |
| NFR-10 | **Tài liệu song hành** — public API có doc comment; docs tiếng Việt khớp code | Must | rustdoc trên mọi public item; traceability matrix | ✅ |
| NFR-11 | **Bảo mật dữ liệu** — không secrets/credentials trong repo | Must | review staged files; .gitignore loại build artifacts | ✅ |
| NFR-12 | **Tương thích nền tảng** — Windows 10+ x64, Rust stable 2021 edition | Must | rustc 1.99.1 stable-x86_64-pc-windows-msvc build + link OK | ✅ |

## Ghi chú hiệu năng

- Số liệu NFR-03 đo trên build debug (`opt-level = 1` trong `[profile.dev]`), máy tham chiếu là máy phát hiện tại. Software renderer vơỉ blit per-pixel — profile cho thấy cần tối ưu (SIMD/GPU) thì làm ở M3+.
- Không đặt SLO loại production cho thư viện chưa có game release.

## References

- `/docs/02-requirements/functional-requirements.md`
- `/docs/02-requirements/traceability-matrix.md`
- `/docs/05-quality/quality-strategy.md`
- `/docs/03-architecture/adr/ADR-001-monster-engine-foundation.md`
