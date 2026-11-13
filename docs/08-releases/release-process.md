# Release Process — monster-engine

- Owner: Release Manager
- Status: `Approved`
- Last updated: `2026-11-13`
- Review date: `2027-05-13`

## Purpose

Quy trình phát hành một phiên bản `monster-engine` (đánh version, kiểm chứng, ghi release notes, commit).

## Phiên bản

- Định dạng SemVer `MAJOR.MINOR.PATCH` — nguồn sự thật là `version` trong `monster-engine/Cargo.toml` (workspace.package).
- Nền móng hiện tại: `0.1.0`.
- Đổi breaking API public → tăng MAJOR (hoặc MINOR khi 0.x); thêm tính năng → MINOR; fix → PATCH.

## Quy trình phát hành

```
1. Chốt scope release (milestone từ /docs/01-product/roadmap.md)
2. Task workspace + preflight gate (theo /AGENTS.md)
3. Hiện thực + test: cargo test --workspace PASS
4. Chạy bộ tiêu chí PASS (xem /docs/05-quality/quality-strategy.md)
5. Viết release notes: /docs/08-releases/release-notes/release-X.Y.Z.md
6. Cập nhật version trong Cargo.toml (nếu đổi)
7. Final gate PASS
8. Commit: "release: vX.Y.Z — <tóm tắt>"
9. Cập nhật handoff; báo người dùng
```

## Tiêu chí bắt buộc trước release

| Hạng mục | Tiêu chí |
| --- | --- |
| Build | `cargo build --workspace` exit 0 |
| Test | `cargo test --workspace` exit 0, 0 failed |
| Smoke | sandbox headless exit 0 |
| Gates | preflight + final PASS |
| Docs | release notes mới; docs khớp code (traceability cập nhật) |
| ADR | quyết định kiến trúc mới đã có ADR |
| Evidence | logs đầy đủ trong task workspace |

## Rollback

- Release là commit git — `git revert` commit release là quay về phiên bản trước.
- Cargo workspace không có migration runtime; dữ liệu game (asset) không đổi format khi rollback (RGBA8 cố định).
- Nếu release kèm thay đổi format asset/data: MUST có migration guide trong `/docs/08-releases/migration-guides/` trước khi commit.

## Checklist release

```
[ ] Milestone scope hoàn tất (không TODO treo)
[ ] 105+ tests PASS; không test bị ignore lặng lẽ
[ ] Release notes viết đủ: what's new, limitations, hướng dẫn chạy
[ ] Traceability matrix cập nhật
[ ] Final gates PASS
[ ] Commit message theo "release: vX.Y.Z"
```

## References

- `/docs/08-releases/release-notes/release-0.1.0.md`
- `/docs/05-quality/quality-strategy.md`
- `/docs/01-product/roadmap.md`
- `/AGENTS.md`
