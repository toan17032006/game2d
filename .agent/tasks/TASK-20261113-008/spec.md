# Specification

## Inputs

- Mục tiêu người dùng (chuẩn hóa): engine 2D trước / 3D sau trong `monster-engine`; dễ mở rộng, dễ thêm hàm; nghiên cứu thành phần cần có của engine code thuần.
- Lựa chọn ngôn ngữ của người dùng trong phiên: **Rust** (AskUserQuestion).
- `/AGENTS.md`, `/docs/00-governance/agent-execution-standard.md`.

## Confirmed facts

- F-01: `rustc 1.99.1` + `cargo` đã cài; không có compiler C/C++, không có .NET SDK; Node.js 26 và Python 3.15 có sẵn.
- F-02: `/monster-engine/` tồn tại nhưng trống hoàn toàn (0 file).
- F-03: Asset "Tiny Swords (Free Pack)" PNG có sẵn tại `/assest/`.
- F-04: Markdown ngoài task workspace là read-only trừ khi được duyệt trực tiếp (`check-doc-boundaries.ps1`).
- F-05: Quality gate final yêu cầu product verification command khai báo trong `spec.md` và chạy exit 0.
- F-06: Các task TASK-20261113-002..007 đã COMPLETED, chỉ đụng governance/tooling, chưa có code engine.

## Inferences

- I-1 (inference): "Engine code thuần" = engine dạng thư viện code, không editor; game code gọi API engine trực tiếp. Căn cứ: người dùng nhấn "code thuần" và muốn "dễ thêm hàm".
- I-2 (inference): Engine cần decode PNG thuần Rust để dùng được asset Tiny Swords sau này; không có crate ngoài nên phải tự viết inflate + PNG decoder.
- I-3 (inference): Window/input qua Win32 API FFI là đủ cho demo 2D trên Windows; không cần SDL.
- I-4 (inference): Vec3/Mat4/Quat nên có sẵn từ đầu trong math để đường lên 3D không phải rewrite API.

## Unknowns

- U-1: Không xác nhận được môi trường CI/desktop session có cho phép tạo Win32 window lúc chạy test tự động. Giải pháp: mọi test chạy headless (framebuffer mềm + Null backend); window thật chỉ dùng khi người dùng chạy example thủ công.
- U-2: Không kiểm tra được khả năng truy cập crates.io. Giải pháp: zero external dependency — không cần network.

## Constraints

- C-01: Zero external crate; chỉ `std` và FFI `user32.dll`/`gdi32.dll` (khai báo `extern "system"` tự viết).
- C-02: Rust 2021 edition; Windows 10+.
- C-03: Không sửa Markdown ngoài `/.agent/tasks/TASK-20261113-008/`.
- C-04: Không secrets/credentials trong repo.
- C-05: Không TODO/placeholder trong output hoàn tất; mọi subsystem có hiện thực thật (Null backend là hiện thực có chủ đích, được test, không phải placeholder).

## Decisions

- D-01: Rust 2021, cargo workspace với crate nhỏ theo boundary: `monster-math`, `monster-core`, `monster-render`, `monster-platform`, `monster-assets`, `monster-scene`, `monster-physics`, `monster-audio`, `monster-engine` (facade). Crate = biên cứng cho "dễ mở rộng".
- D-02: Kiến trúc trait-based: `Renderer` (vẽ + present), `AudioBackend`, `PresentTarget` định nghĩa trong `monster-core`; hiện thực cụ thể (Software2DRenderer, NullWindow, NullAudioBackend) đăng ký qua `EngineBuilder` — thêm backend mới không sửa core.
- D-03: 2D trước. Software renderer 2D: framebuffer RGBA8, blit sprite có alpha/tint/flip, camera offset+zoom, z-order; present qua GDI (`CreateDIBSection` + `BitBlt`). Backend GPU/3D sau này implement cùng trait `Renderer`.
- D-04: Math đầy đủ từ đầu: Vec2/Vec3/Vec4, Mat4, Quat, Rect, Color — phục vụ 3D sau.
- D-05: Assets: registry/cache + decoder BMP thuần + PNG decoder thuần (zlib inflate + PNG chunk/filter) để dùng Tiny Swords.
- D-06: ECS-lite trong `monster-scene`: EntityId generational, component store theo type, system chạy theo `World`; không dùng archetype-ECS phức tạp ở nền móng.
- D-07: Product verification command (headless, deterministic): `cargo test --workspace --manifest-path monster-engine/Cargo.toml`.

## Output contract

- Cargo workspace `/monster-engine/` build được với `cargo build --workspace`, test được với `cargo test --workspace`, example `sandbox` chạy `--headless N` exit 0 và chạy window thật khi người dùng tự chạy.
- API surface: `EngineBuilder::new().with_window(...).with_audio_backend(...).run(app)`; `GameApp` trait {init/update/render/shutdown}; `World` ECS; `Renderer` trait; `Texture`, `Sprite`; `Aabb`, `Transform`.
- Tài liệu nghiên cứu + ADR draft trong `research/` của task workspace.

## Test Command

`cargo test --workspace --manifest-path monster-engine/Cargo.toml`

## Approved Documentation Changes

- None. Task này KHÔNG sửa tài liệu dự án ngoài task workspace. ADR được draft trong task workspace và chỉ chuyển vào `/docs/03-architecture/adr/` khi người dùng phê duyệt trực tiếp trong phiên khác.

## References

- `/AGENTS.md`
- `/docs/00-governance/agent-execution-standard.md`
- `/docs/03-architecture/README.md`
- `/.agent/scripts/check-doc-boundaries.ps1`
