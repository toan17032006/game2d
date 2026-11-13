//! Windows: the real Win32 window (production) and [`NullWindow`] (headless
//! tests). Both implement [`PlatformWindow`], the engine-facing interface.

use std::fmt;
use std::sync::Mutex;

use monster_core::events::{EngineEvent, EventBus};
use monster_core::graphics::PresentTarget;
use monster_core::input::MouseButton;

use crate::ffi;
use crate::keymap::vk_to_keycode;

/// Errors raised while creating or using a platform window.
#[derive(Debug)]
pub enum PlatformError {
    /// `RegisterClassW` failed (atom value).
    ClassRegistration(u16),
    /// `CreateWindowExW` failed.
    WindowCreation(String),
}

impl fmt::Display for PlatformError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PlatformError::ClassRegistration(atom) => {
                write!(f, "RegisterClassW failed (atom {atom})")
            }
            PlatformError::WindowCreation(what) => {
                write!(f, "CreateWindowExW failed: {what}")
            }
        }
    }
}

/// Options for the production window.
#[derive(Debug, Clone)]
pub struct WindowOptions {
    pub width: u32,
    pub height: u32,
    pub title: String,
}

impl Default for WindowOptions {
    fn default() -> Self {
        WindowOptions {
            width: 960,
            height: 540,
            title: "Monster Engine".to_string(),
        }
    }
}

/// Interface every platform window provides to the engine.
///
/// Supertrait [`PresentTarget`] lets the renderer submit frames through the
/// same object, so the engine holds one `Box<dyn PlatformWindow>` for
/// input, lifecycle and presentation.
pub trait PlatformWindow: PresentTarget {
    /// Whether the window is still open (user has not closed it).
    fn is_open(&self) -> bool;

    /// Pump OS messages and push translated [`EngineEvent`]s into `bus`.
    fn poll_events(&mut self, bus: &mut EventBus);

    /// Current client size in pixels.
    fn size(&self) -> (u32, u32);

    /// Request closure (no-op if already closed).
    fn close(&mut self);
}

// ---------------------------------------------------------------------------
// Shared wndproc state
// ---------------------------------------------------------------------------

#[derive(Default)]
struct WndProcState {
    events: Vec<EngineEvent>,
    closed: bool,
}

static WINDOW_STATE: Mutex<WndProcState> = Mutex::new(WndProcState {
    events: Vec::new(),
    closed: false,
});

fn push_event(event: EngineEvent) {
    if let Ok(mut state) = WINDOW_STATE.lock() {
        state.events.push(event);
    }
}

unsafe extern "system" fn wnd_proc(
    hwnd: ffi::HWND,
    msg: u32,
    wparam: ffi::WPARAM,
    lparam: ffi::LPARAM,
) -> ffi::LRESULT {
    match msg {
        ffi::WM_CLOSE => {
            push_event(EngineEvent::WindowClosed);
            ffi::DestroyWindow(hwnd);
            0
        }
        ffi::WM_DESTROY => {
            ffi::PostQuitMessage(0);
            0
        }
        ffi::WM_SIZE => {
            let w = ffi::low_word(lparam) as u32;
            let h = ffi::high_word(lparam) as u32;
            if w > 0 && h > 0 {
                push_event(EngineEvent::WindowResized {
                    width: w,
                    height: h,
                });
            }
            0
        }
        ffi::WM_SETFOCUS => {
            push_event(EngineEvent::WindowFocused);
            0
        }
        ffi::WM_KILLFOCUS => {
            push_event(EngineEvent::WindowFocusLost);
            0
        }
        ffi::WM_KEYDOWN | ffi::WM_SYSKEYDOWN => {
            // Let Alt+F4 through so the OS can produce WM_CLOSE.
            if msg == ffi::WM_SYSKEYDOWN && wparam == 0x73 {
                return ffi::DefWindowProcW(hwnd, msg, wparam, lparam);
            }
            push_event(EngineEvent::KeyPressed(vk_to_keycode(wparam as u32)));
            0
        }
        ffi::WM_KEYUP | ffi::WM_SYSKEYUP => {
            push_event(EngineEvent::KeyReleased(vk_to_keycode(wparam as u32)));
            0
        }
        ffi::WM_MOUSEMOVE => {
            let x = ffi::low_word(lparam) as i16 as f32;
            let y = ffi::high_word(lparam) as i16 as f32;
            push_event(EngineEvent::MouseMoved { x, y });
            0
        }
        ffi::WM_LBUTTONDOWN | ffi::WM_LBUTTONUP => {
            push_event(mouse_button_event(MouseButton::Left, msg));
            0
        }
        ffi::WM_RBUTTONDOWN | ffi::WM_RBUTTONUP => {
            push_event(mouse_button_event(MouseButton::Right, msg));
            0
        }
        ffi::WM_MBUTTONDOWN | ffi::WM_MBUTTONUP => {
            push_event(mouse_button_event(MouseButton::Middle, msg));
            0
        }
        ffi::WM_PAINT => {
            // We redraw every frame ourselves; just validate to avoid a
            // busy WM_PAINT loop.
            ffi::ValidateRect(hwnd, std::ptr::null());
            0
        }
        _ => ffi::DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

fn mouse_button_event(button: MouseButton, msg: u32) -> EngineEvent {
    let pressed = matches!(
        msg,
        ffi::WM_LBUTTONDOWN | ffi::WM_RBUTTONDOWN | ffi::WM_MBUTTONDOWN
    );
    EngineEvent::MouseButton { button, pressed }
}

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

// ---------------------------------------------------------------------------
// Real Win32 window
// ---------------------------------------------------------------------------

const WINDOW_CLASS_NAME: &str = "MonsterEngineWindowClass";

/// GDI resources cached across presents.
struct GdiPresent {
    mem_dc: ffi::HDC,
    bitmap: ffi::HBITMAP,
    old_bitmap: ffi::HGDIOBJ,
    bits: *mut std::ffi::c_void,
    size: (u32, u32),
}

/// Production Win32 window.
pub struct Window {
    hwnd: ffi::HWND,
    width: u32,
    height: u32,
    open: bool,
    gdi: Option<GdiPresent>,
}

impl Window {
    /// Create and show a Win32 window with the requested client size.
    pub fn new(options: &WindowOptions) -> Result<Window, PlatformError> {
        unsafe {
            let hinstance = ffi::GetModuleHandleW(std::ptr::null());
            if hinstance.is_null() {
                return Err(PlatformError::WindowCreation("GetModuleHandleW".into()));
            }

            let class_name = wide(WINDOW_CLASS_NAME);
            let wc = ffi::WNDCLASSW {
                style: ffi::CS_HREDRAW | ffi::CS_VREDRAW,
                lpfnWndProc: wnd_proc,
                cbClsExtra: 0,
                cbWndExtra: 0,
                hInstance: hinstance,
                hIcon: std::ptr::null_mut(),
                hCursor: std::ptr::null_mut(),
                hbrBackground: std::ptr::null_mut(),
                lpszMenuName: std::ptr::null(),
                lpszClassName: class_name.as_ptr(),
            };
            let atom = ffi::RegisterClassW(&wc);
            if atom == 0 {
                return Err(PlatformError::ClassRegistration(atom));
            }

            let title = wide(&options.title);
            // Adjust an overlapped window so the CLIENT area matches options.
            let mut rect = ffi::RECT {
                left: 0,
                top: 0,
                right: options.width as i32,
                bottom: options.height as i32,
            };
            ffi::AdjustWindowRect(&mut rect, ffi::WS_OVERLAPPEDWINDOW, 0);

            let hwnd = ffi::CreateWindowExW(
                0,
                class_name.as_ptr(),
                title.as_ptr(),
                ffi::WS_OVERLAPPEDWINDOW | ffi::WS_VISIBLE,
                80,
                60,
                rect.right - rect.left,
                rect.bottom - rect.top,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                hinstance,
                std::ptr::null_mut(),
            );
            if hwnd.is_null() {
                return Err(PlatformError::WindowCreation("null hwnd".into()));
            }
            ffi::ShowWindow(hwnd, ffi::SW_SHOW);

            Ok(Window {
                hwnd,
                width: options.width,
                height: options.height,
                open: true,
                gdi: None,
            })
        }
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        unsafe {
            if let Some(gdi) = self.gdi.take() {
                ffi::SelectObject(gdi.mem_dc, gdi.old_bitmap);
                ffi::DeleteObject(gdi.bitmap);
                ffi::DeleteDC(gdi.mem_dc);
            }
            if !self.hwnd.is_null() {
                ffi::DestroyWindow(self.hwnd);
            }
        }
    }
}

impl PlatformWindow for Window {
    fn is_open(&self) -> bool {
        self.open
    }

    fn poll_events(&mut self, bus: &mut EventBus) {
        unsafe {
            let mut msg = ffi::MSG::default();
            loop {
                let has = ffi::PeekMessageW(&mut msg, std::ptr::null_mut(), 0, 0, ffi::PM_REMOVE);
                if has == 0 {
                    break;
                }
                if msg.message == ffi::WM_QUIT {
                    self.open = false;
                } else {
                    ffi::TranslateMessage(&msg);
                    ffi::DispatchMessageW(&msg);
                }
            }
        }
        if let Ok(mut state) = WINDOW_STATE.lock() {
            for event in state.events.drain(..) {
                if let EngineEvent::WindowResized { width, height } = event {
                    self.width = width;
                    self.height = height;
                }
                if let EngineEvent::WindowClosed = event {
                    self.open = false;
                }
                bus.push(event);
            }
            state.closed = false;
        }
    }

    fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    fn close(&mut self) {
        self.open = false;
        unsafe {
            if !self.hwnd.is_null() {
                ffi::DestroyWindow(self.hwnd);
            }
        }
    }
}

impl PresentTarget for Window {
    fn present(&mut self, width: u32, height: u32, rgba: &[u8]) {
        if !self.open || width == 0 || height == 0 || self.hwnd.is_null() {
            return;
        }
        unsafe {
            // (Re)create the DIB when the size changes.
            let needs_new = match &self.gdi {
                Some(gdi) => gdi.size != (width, height),
                None => true,
            };
            if needs_new {
                if let Some(gdi) = self.gdi.take() {
                    ffi::SelectObject(gdi.mem_dc, gdi.old_bitmap);
                    ffi::DeleteObject(gdi.bitmap);
                    ffi::DeleteDC(gdi.mem_dc);
                }
                let window_dc = ffi::GetDC(self.hwnd);
                let mem_dc = ffi::CreateCompatibleDC(window_dc);
                ffi::ReleaseDC(self.hwnd, window_dc);
                if mem_dc.is_null() {
                    return;
                }
                let bi = ffi::BITMAPINFO {
                    bmiHeader: ffi::BITMAPINFOHEADER {
                        biSize: std::mem::size_of::<ffi::BITMAPINFOHEADER>() as u32,
                        biWidth: width as i32,
                        // negative height = top-down rows, matching our layout
                        biHeight: -(height as i32),
                        biPlanes: 1,
                        biBitCount: 32,
                        biCompression: 0, // BI_RGB
                        ..Default::default()
                    },
                    bmiColors: [0; 1],
                };
                let mut bits: *mut std::ffi::c_void = std::ptr::null_mut();
                let bitmap = ffi::CreateDIBSection(
                    mem_dc,
                    &bi,
                    ffi::DIB_RGB_COLORS,
                    &mut bits,
                    std::ptr::null_mut(),
                    0,
                );
                if bitmap.is_null() || bits.is_null() {
                    ffi::DeleteDC(mem_dc);
                    return;
                }
                let old_bitmap = ffi::SelectObject(mem_dc, bitmap);
                self.gdi = Some(GdiPresent {
                    mem_dc,
                    bitmap,
                    old_bitmap,
                    bits,
                    size: (width, height),
                });
            }

            let Some(gdi) = &self.gdi else { return };
            // RGBA -> BGRA copy into the DIB.
            let px_count = (width as usize) * (height as usize);
            let expected = px_count * 4;
            if rgba.len() < expected {
                return;
            }
            let dst = gdi.bits as *mut u8;
            for i in 0..px_count {
                let s = i * 4;
                let d = i * 4;
                *dst.add(d) = rgba[s + 2]; // B
                *dst.add(d + 1) = rgba[s + 1]; // G
                *dst.add(d + 2) = rgba[s]; // R
                *dst.add(d + 3) = rgba[s + 3]; // A
            }

            let window_dc = ffi::GetDC(self.hwnd);
            ffi::BitBlt(
                window_dc,
                0,
                0,
                width as i32,
                height as i32,
                gdi.mem_dc,
                0,
                0,
                ffi::SRCCOPY,
            );
            ffi::ReleaseDC(self.hwnd, window_dc);
        }
    }
}

// ---------------------------------------------------------------------------
// Headless window
// ---------------------------------------------------------------------------

/// Window substitute for headless runs and tests: never opens a real OS
/// window, counts presents, and can be closed programmatically.
pub struct NullWindow {
    open: bool,
    presents: u64,
    last_present_size: Option<(u32, u32)>,
}

impl Default for NullWindow {
    fn default() -> Self {
        Self::new()
    }
}

impl NullWindow {
    pub fn new() -> NullWindow {
        NullWindow {
            open: true,
            presents: 0,
            last_present_size: None,
        }
    }

    /// Number of frames presented so far.
    pub fn presents(&self) -> u64 {
        self.presents
    }

    /// Size recorded by the most recent present.
    pub fn last_present_size(&self) -> Option<(u32, u32)> {
        self.last_present_size
    }
}

impl PlatformWindow for NullWindow {
    fn is_open(&self) -> bool {
        self.open
    }

    fn poll_events(&mut self, _bus: &mut EventBus) {
        // Headless: no OS messages; game code may close via `close()`.
    }

    fn size(&self) -> (u32, u32) {
        self.last_present_size.unwrap_or((0, 0))
    }

    fn close(&mut self) {
        self.open = false;
    }
}

impl PresentTarget for NullWindow {
    fn present(&mut self, width: u32, height: u32, _rgba: &[u8]) {
        if !self.open {
            return;
        }
        self.presents += 1;
        self.last_present_size = Some((width, height));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use monster_core::events::EventBus;
    use monster_core::input::MouseButton;

    #[test]
    fn wide_encoding_null_terminated() {
        let w = wide("Ab");
        assert_eq!(w, vec![65, 98, 0]);
    }

    #[test]
    fn null_window_counts_presents() {
        let mut win = NullWindow::new();
        let mut bus = EventBus::new();
        assert!(win.is_open());
        win.poll_events(&mut bus);
        assert!(win.is_open());
        assert_eq!(win.presents(), 0);
        win.present(4, 4, &[0; 64]);
        win.present(4, 4, &[0; 64]);
        assert_eq!(win.presents(), 2);
        assert_eq!(win.size(), (4, 4));
        win.close();
        assert!(!win.is_open());
        win.present(4, 4, &[0; 64]); // ignored after close
        assert_eq!(win.presents(), 2);
    }

    #[test]
    fn null_window_bus_stays_empty() {
        let mut win = NullWindow::new();
        let mut bus = EventBus::new();
        win.poll_events(&mut bus);
        assert!(bus.is_empty());
    }

    #[test]
    fn mouse_event_mapping() {
        let down = mouse_button_event(MouseButton::Left, ffi::WM_LBUTTONDOWN);
        assert_eq!(
            down,
            EngineEvent::MouseButton {
                button: MouseButton::Left,
                pressed: true
            }
        );
        let up = mouse_button_event(MouseButton::Right, ffi::WM_RBUTTONUP);
        assert_eq!(
            up,
            EngineEvent::MouseButton {
                button: MouseButton::Right,
                pressed: false
            }
        );
    }
}
