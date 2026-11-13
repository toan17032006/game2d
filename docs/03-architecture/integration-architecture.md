# Integration Architecture — monster-engine

- Owner: Architecture Board
- Status: `Approved`
- Last updated: `2026-11-13`
- Review date: `2027-05-13`

## Purpose

Mô tả các điểm tích hợp của engine với thế giới bên ngoài: biên Win32 FFI, đường present framebuffer lên màn hình, đường dẫn event từ OS vào engine, và cơ chế cắm backend mới.

## 1. Biên Win32 FFI (monster-platform::ffi)

Tất cả lời gọi hệ điều hành gom về một module — phần duy nhất của engine chạm nền tảng.

| Nhóm API | DLL | Dùng cho |
| --- | --- | --- |
| RegisterClassW, CreateWindowExW, DefWindowProcW, PeekMessageW, TranslateMessage, DispatchMessageW, ShowWindow, DestroyWindow, PostQuitMessage, ValidateRect, AdjustWindowRect, GetDC/ReleaseDC | user32 | Vòng đời cửa sổ + message pump |
| CreateDIBSection, SelectObject, DeleteObject, CreateCompatibleDC, DeleteDC, BitBlt | gdi32 | Present framebuffer |
| GetModuleHandleW | kernel32 | HInstance cho đăng ký window class |

- Struct `#[repr(C)]` tự khai báo khớp ABI x64 (WNDCLASSW, MSG đầy đủ 44 bytes gồm lPrivate, BITMAPINFOHEADER, RECT, POINT).
- Wndproc là `unsafe extern "system" fn` — state chia sẻ qua `static Mutex<WndProcState>` (event queue + closed flag), wndproc push, `poll_events` drain.
- Bàn phím: WM_KEYDOWN/UP + WM_SYSKEYDOWN/UP (Alt+F4 được truyền `DefWindowProcW` để OS sinh WM_CLOSE); map VK→KeyCode trong `keymap.rs` (hàm thuần, có unit test).
- Chuột: WM_MOUSEMOVE (tọa độ 16-bit có dấu từ lParam), WM_L/R/MBUTTONDOWN/UP.

## 2. Đường present (renderer → màn hình)

```
Software2DRenderer.end(target)
    │ framebuffer RGBA8 (row-major, top-down)
    ▼
Window::present(width, height, rgba)         [implement PresentTarget]
    │ (re)create DIB 32bpp top-down khi kích thước đổi
    │ copy RGBA → BGRA vào DIB bits (vòng lặp per-pixel)
    ▼
GetDC(hwnd) → BitBlt(mem_dc → window_dc, SRCCOPY) → ReleaseDC
```

`NullWindow::present` thay đường GDI bằng bộ đếm frames — headless test không cần màn hình.

## 3. Đường event (OS → engine)

```
WndProc (WM_*) → push EngineEvent vào static Mutex queue
    ▼
Window::poll_events: PeekMessageW loop → WM_QUIT ⇒ open=false
    ▼
Engine::run: phân route EventBus
  ├─ WindowResized → camera.viewport + renderer.resize
  ├─ WindowClosed  → thoát loop
  └─ KeyPressed/Released, MouseMoved/Button/Wheel → InputState.begin_frame
```

## 4. Cơ chế cắm backend mới (điểm tích hợp tương lai)

| Mở rộng | Cách làm | Chạm core? |
| --- | --- | --- |
| GPU/3D renderer | Crate mới implement `Renderer`; thêm `draw_mesh` vào trait khi cần; nạp HWND từ `PlatformWindow` | Chỉ thêm method trait (1 điểm) |
| Audio playback thật (waveOut/DirectSound) | Crate mới implement `AudioBackend`; `EngineBuilder::with_audio_backend` thay Null | Không |
| Platform khác (Linux/macOS) | Crate platform mới implement `PlatformWindow` | Không |
| Asset format khác (sprite atlas JSON, mesh 3D) | Module mới trong `monster-assets` trả `ImageData`/mesh | Không |

## 5. Tích hợp asset (filesystem)

- Đường dẫn asset: `/assest/` (repo) — engine đọc tại runtime qua `Assets::load_file`.
- Format nhận diện theo extension: `.png`/`.bmp`/`.dib` → texture; WAV qua `monster-audio::decode_wav`.
- Không có hot-reload ở nền móng (registry cache theo key — mở rộng sau).

## Quy tắc tích hợp

- MUST gom mọi FFI vào `monster-platform::ffi` — cấm rải lời gọi Win32 ở crate khác.
- MUST giữ biên `unsafe` nhỏ nhất có thể; hàm unsafe chỉ trong `ffi.rs` và các caller trực tiếp.
- MUST implement backend mới qua trait ở core — không gọi chéo implementation.
- MUST test headless (NullWindow/NullAudioBackend) cho mọi logic không phụ thuộc OS thật.

## References

- `/docs/03-architecture/container-and-component.md`
- `/docs/03-architecture/data-architecture.md`
- `/docs/03-architecture/adr/ADR-001-monster-engine-foundation.md`
- `/monster-engine/crates/monster-platform/src/ffi.rs` (nguồn sự thật FFI)
- Win32 API — Microsoft Learn: <https://learn.microsoft.com/en-us/windows/win32/api/> (truy cập 2026-11-13)
