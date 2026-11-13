# Prompt

## Input classification

`DIRECT` — mục tiêu được người dùng đặt trực tiếp trong phiên (goal mode: tiếp tục thực hiện tới khi xong); ngôn ngữ Rust được người dùng chọn trực tiếp qua AskUserQuestion trong phiên này. Không còn input bắt buộc nào thiếu.

## Role

Senior game engine architect và Rust developer, chuyên thiết kế engine module hóa, trait-based, mở rộng được từ 2D sang 3D.

## Objective

1. Nghiên cứu và lập danh mục các thành phần bắt buộc của một game engine "code thuần" (thư viện code, không editor).
2. Xây dựng nền móng `monster-engine` bằng Rust: engine 2D chạy được, kiến trúc mở rộng, sau này thêm backend 3D mà không sửa core.

## Context

- Repository có governance `AGENTS.md`: Markdown ngoài task workspace là read-only; mọi task phải có task workspace và quality gates.
- Toolchain máy: `rustc 1.99.1` + `cargo` (có sẵn); KHÔNG có compiler C/C++, không có .NET SDK. Node.js 26 và Python 3.15 có sẵn nhưng không được chọn.
- Thư mục `/monster-engine/` hiện trống hoàn toàn; chưa có code engine nào trong repo.
- Asset 2D "Tiny Swords (Free Pack)" (PNG spritesheet) có sẵn tại `/assest/Tiny Swords (Free Pack)/`.
- Task trước (TASK-20261113-002..007) chỉ xây governance/tooling `.agent/`, chưa đụng engine.

## Inputs

- Mục tiêu người dùng (đã chuẩn hóa): "Mục tiêu xây dựng 1 engine game 2D, sau này có thể mở rộng thành game 3D ở trong monster-engine. Xây dựng 1 engine dễ mở rộng, dễ thêm hàm. Nghiên cứu những thứ cần có của 1 engine code thuần."
- Người dùng đã chọn ngôn ngữ **Rust** (AskUserQuestion, phiên hiện tại).
- `/AGENTS.md` và `/docs/00-governance/agent-execution-standard.md`.

## Scope

- `/.agent/tasks/TASK-20261113-008/` — toàn bộ artifact + nghiên cứu + ADR draft.
- `/monster-engine/` — cargo workspace Rust, nhiều crate `monster-*`, zero dependency ngoài std.
- `/.gitignore` — thêm entry build artifact `target/` (file không phải Markdown).

## Non-goals

- Không hiện thực render 3D / API GPU (chỉ chuẩn bị interface cho 3D).
- Không xây editor, GUI tool, hot-reload, scripting language, networking.
- Không sửa/xóa/tạo Markdown tài liệu dự án (`/docs/`, `/AGENTS.md`, ...).
- Không tích hợp crate ngoài (winit, wgpu, sdl2, ...).
- Không hiện thực audio playback backend phần cứng (chỉ trait + Null backend + WAV decode).

## Constraints

- Zero external crate: chỉ `std` + Win32 FFI (`user32.dll`, `gdi32.dll` qua `extern "system"`).
- Rust 2021 edition, build trên Windows 10+.
- Không ghi secrets hay dữ liệu cá nhân vào repository.
- Chỉnh sửa Markdown chỉ trong task workspace hiện tại.
- Không để TODO/placeholder/pseudo-code trong output hoàn tất.

## Execution rules

- Thực thi theo `plan.md`, cập nhật trạng thái phase sau mỗi checkpoint.
- Cập nhật `handoff.md` sau mỗi checkpoint.
- Chạy preflight gate trước `EXECUTING`; chạy final gate trước `COMPLETED`.
- Mọi kết quả kiểm tra ghi PASS/FAIL kèm evidence vào `verification.md`.

## Output contract

1. Cargo workspace `/monster-engine/` gồm các crate: `monster-math`, `monster-core`, `monster-render`, `monster-platform`, `monster-assets`, `monster-scene`, `monster-physics`, `monster-audio`, `monster-engine` (facade) và example `sandbox`.
2. `research/engine-foundation-study.md` — nghiên cứu thành phần engine code thuần.
3. `research/adr-draft-monster-engine-foundation.md` — ADR draft chờ duyệt.
4. `verification.md`, `evaluation.md`, `handoff.md` hoàn chỉnh với evidence.

## Acceptance criteria

Xem `task.md` (AC-001..AC-008).

## Verification method

- `cargo build --workspace --manifest-path monster-engine/Cargo.toml` (exit 0).
- `cargo test --workspace --manifest-path monster-engine/Cargo.toml` (exit 0).
- `cargo run --manifest-path monster-engine/crates/monster-engine/Cargo.toml --example sandbox -- --headless 120` (exit 0).
- `powershell -NoProfile -ExecutionPolicy Bypass -File .agent/scripts/run-quality-gates.ps1 -TaskId TASK-20261113-008 -Mode final` (exit 0).

## References

- `/AGENTS.md`
- `/docs/00-governance/agent-execution-standard.md`
- `/docs/03-architecture/README.md`
- `/.agent/tasks/README.md`
