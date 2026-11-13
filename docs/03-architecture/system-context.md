# System Context — monster-engine

- Owner: Architecture Board
- Status: `Approved`
- Last updated: `2026-11-13`
- Review date: `2027-05-13`

## Purpose

Xác định biên của `monster-engine` trong hệ thống: ai tương tác, gì vào, gì ra, và các hệ thống liên quan. Đây là bản đồ mức cao nhất — chi tiết bên trong xem `container-and-component.md`.

## Bối cảnh hệ thống

```
┌─────────────────────────┐
│  Game code (sandbox,    │  viết bằng Rust, dùng API facade
│  game thật sau này)     │
└───────────┬─────────────┘
            │ crate `monster-engine` (facade: EngineBuilder, GameApp, prelude)
            ▼
┌─────────────────────────┐        ┌──────────────────────────┐
│  monster-engine         │ I/O →  │ Hệ điều hành Windows     │
│  (thư viện engine 2D)   │        │ (user32: window/input;   │
│                         │        │  gdi32: present)         │
└───────────┬─────────────┘        └──────────────────────────┘
            │ đọc/ghi
            ▼
┌─────────────────────────┐
│  Filesystem repo        │
│  /assest/ (PNG Tiny     │
│  Swords, WAV, BMP)      │
└─────────────────────────┘
```

## Actors và tương tác

| Actor | Vai trò | Tương tác qua |
| --- | --- | --- |
| Game developer | Viết game trên engine | Trait `GameApp` (init/update/render) + `EngineBuilder` |
| Agent/AI developer | Phát triển engine theo workflow `/AGENTS.md` | Task workspace + quality gates |
| Người chơi cuối | Chạy game demo | Cửa sổ game (Win32) — chạy từ example `sandbox` |
| Asset pipeline | Cung cấp PNG/BMP/WAV | Thư mục `/assest/` — engine đọc tại runtime |

## Biên hệ thống

- **Vào:** file asset (PNG/BMP/WAV), cấu hình cửa sổ qua `WindowOptions`, event từ OS (bàn phím/chuột/resize/close).
- **Ra:** framebuffer RGBA8 present lên màn hình qua GDI; log ra stderr; kết quả chạy (`EngineRunResult`: frames/updates/fps).
- **Không phụ thuộc:** network, database, service ngoài, crate bên thứ ba (zero-dependency).

## Ràng buộc nền tảng

- Windows 10+ (Win32 FFI `user32`/`gdi32`/`kernel32`).
- Rust 2021, build bằng cargo không cần mạng.

## References

- `/docs/03-architecture/container-and-component.md`
- `/docs/03-architecture/adr/ADR-001-monster-engine-foundation.md`
- `/monster-engine/`
- `/docs/00-governance/agent-execution-standard.md`
