# Quality Strategy — monster-engine

- Owner: QA and Engineering
- Status: `Approved`
- Last updated: `2026-11-13`
- Review date: `2027-05-13`

## Purpose

Chiến lược đảm bảo chất lượng: phân tầng kiểm chứng, công cụ, tiêu chí PASS/FAIL. Đã áp dụng và kiểm chứng qua TASK-20261113-008 (105 tests PASS, gates PASS).

## Nguyên tắc

1. **Headless-first** — mọi logic kiểm chứng được không cần GUI; window thật chỉ verify trực quan.
2. **Mỗi FR có test thật** — truy vết trong `traceability-matrix.md`.
3. **Bằng chứng ghi lại** — PASS/FAIL kèm evidence trong task workspace; không báo PASS không evidence.
4. **Gates chặn thay vì con người nhớ** — preflight/final gates bắt buộc, exit code ≠ 0 = FAIL.

## Phân tầng kiểm chứng

| Tầng | Phạm vi | Công cụ | Ví dụ |
| --- | --- | --- | --- |
| L1 — Unit | hàm/cấu trúc đơn lẻ per crate | `cargo test -p <crate>` (tests cùng file) | vec normalize, blend pixel 50%, WAV mixdown |
| L2 — Tích hợp subsystem | module phối hợp | tests cùng crate | PNG fixture end-to-end (encode→decode→so pixel), Assets cache |
| L3 — Engine loop | facade + subsystems | `tests/engine_loop.rs` | CountingApp: init/update/render/shutdown đếm đúng; exit flag dừng loop |
| L4 — Smoke product | toàn bộ chạy thật | `cargo run --example sandbox -- --headless 120` | exit 0 sau ≥120 frames |
| L5 — Gates quá trình | quy trình + tài liệu + link | `run-quality-gates.ps1` (preflight/final) | doc boundaries, validator, verifier, lint, lifecycle model |
| L6 — Verify trực quan | phần phụ thuộc OS thật | chạy thủ công | `--window`: cửa sổ mở, present đúng màu |

## Bộ dữ liệu kiểm chứng

- **Fixture tự sinh** (`monster-assets/src/fixtures.rs`): PNG encoder nhỏ (stored blocks + CRC) tạo ảnh RGBA/palette+tRNS/gray — target cố định cho decoder.
- **Asset thật**: `/assest/Tiny Swords (Free Pack)/` — smoke test decode PNG thật (dynamic Huffman), cross-check kích thước IHDR vs output; skip an toàn nếu thư mục vắng.
- **Frame buffer assertions**: so mảng pixel trực tiếp — không cần file golden image.

## Tiêu chí PASS cho một release/task

```
[ ] cargo build --workspace        exit 0
[ ] cargo test --workspace         exit 0, 0 failed
[ ] cargo run --example sandbox -- --headless 120   exit 0
[ ] preflight gates                exit 0
[ ] final gates                    exit 0 (gồm product verification VERIFIED)
[ ] evidence trong logs/           đầy đủ, tham chiếu từ verification.md
```

## Giới hạn đã ghi nhận (limitations)

- Window Win32 thật và present GDI không tự động verify trong môi trường agent (L6 thủ công) — phần còn lại phủ 100% logic.
- Software renderer chỉ khẳng định đúng về pixel, chưa benchmark ngưỡng hiệu năng nghiêm ngặt (chỉ smoke fps).
- Smoke test asset thật skip (không fail) khi `/assest/` không có trong checkout.

## Xử lý khi FAIL

1. Đọc gate output (violation rõ từng dòng).
2. Sửa nguyên nhân gốc — không vá evidence.
3. Chạy lại toàn bộ tầng liên quan (không chỉ tầng fail).
4. Bug thật phát hiện → thêm regression test cho cùng lỗi (đã áp dụng: Kraft threshold, view matrix, tRNS gray, fixture bitstream).

## References

- `/docs/02-requirements/traceability-matrix.md`
- `/docs/02-requirements/non-functional-requirements.md`
- `/docs/04-engineering/local-development.md`
- `/.agent/tasks/TASK-20261113-008/logs/test.log` (bằng chứng chạy thật)
- `/AGENTS.md`
