//! WGL + OpenGL FFI and a safe [`GlContext`] — the M3 3D rendering backend
//! foundation (legacy OpenGL 1.1 fixed-function, zero external crates).
//!
//! All GL/WGL FFI lives here per the FFI rule in `coding-standards.md`.
//! The high-level `Gl3DRenderer` (implementing `monster_core::Renderer3D`)
//! lives in the facade crate and drives this context.

use std::ffi::c_void;

use crate::ffi::{self, BOOL, HDC, HWND};
use crate::window::{PlatformError, Window};

// --- WGL / OpenGL 1.1 (opengl32.dll) ---------------------------------------

#[link(name = "opengl32")]
extern "system" {
    fn wglCreateContext(hdc: HDC) -> *mut c_void;
    fn wglMakeCurrent(hdc: HDC, hglrc: *mut c_void) -> BOOL;
    fn wglDeleteContext(hglrc: *mut c_void) -> BOOL;
    fn SwapBuffers(hdc: HDC) -> BOOL;
    fn ChoosePixelFormat(hdc: HDC, pfd: *const PIXELFORMATDESCRIPTOR) -> i32;
    fn SetPixelFormat(hdc: HDC, format: i32, pfd: *const PIXELFORMATDESCRIPTOR) -> BOOL;

    fn glGetError() -> u32;
    fn glClearColor(red: f32, green: f32, blue: f32, alpha: f32);
    fn glClear(mask: u32);
    fn glEnable(cap: u32);
    fn glViewport(x: i32, y: i32, width: i32, height: i32);
    fn glMatrixMode(mode: u32);
    fn glLoadIdentity();
    fn glLoadMatrixf(m: *const f32);
    fn glFrustum(left: f64, right: f64, bottom: f64, top: f64, near_val: f64, far_val: f64);
    fn glTranslatef(x: f32, y: f32, z: f32);
    fn glRotatef(angle: f32, x: f32, y: f32, z: f32);
    fn glPushMatrix();
    fn glPopMatrix();
    fn glBegin(mode: u32);
    fn glEnd();
    fn glColor3f(red: f32, green: f32, blue: f32);
    fn glVertex3f(x: f32, y: f32, z: f32);
}

const GL_DEPTH_BUFFER_BIT: u32 = 0x0000_0100;
const GL_COLOR_BUFFER_BIT: u32 = 0x0000_4000;
const GL_DEPTH_TEST: u32 = 0x0B71;
const GL_PROJECTION: u32 = 0x1701;
const GL_MODELVIEW: u32 = 0x1700;
const GL_QUADS: u32 = 0x0007;
const GL_LINES: u32 = 0x0001;
const GL_NO_ERROR: u32 = 0;

/// PIXELFORMATDESCRIPTOR (Win32, 40 bytes).
#[repr(C)]
struct PIXELFORMATDESCRIPTOR {
    n_size: u16,
    n_version: u16,
    dw_flags: u32,
    i_pixel_type: u8,
    c_color_bits: u8,
    c_red_bits: u8,
    c_red_shift: u8,
    c_green_bits: u8,
    c_green_shift: u8,
    c_blue_bits: u8,
    c_blue_shift: u8,
    c_alpha_bits: u8,
    c_alpha_shift: u8,
    c_accum_bits: u8,
    c_accum_red_bits: u8,
    c_accum_green_bits: u8,
    c_accum_blue_bits: u8,
    c_accum_alpha_bits: u8,
    c_depth_bits: u8,
    c_stencil_bits: u8,
    c_aux_buffers: u8,
    i_layer_type: u8,
    b_reserved: u8,
    dw_layer_mask: u32,
    dw_visible_mask: u32,
    dw_damage_mask: u32,
}

impl Default for PIXELFORMATDESCRIPTOR {
    fn default() -> Self {
        PIXELFORMATDESCRIPTOR {
            n_size: std::mem::size_of::<PIXELFORMATDESCRIPTOR>() as u16,
            n_version: 1,
            dw_flags: 0,
            i_pixel_type: 0,
            c_color_bits: 0,
            c_red_bits: 0,
            c_red_shift: 0,
            c_green_bits: 0,
            c_green_shift: 0,
            c_blue_bits: 0,
            c_blue_shift: 0,
            c_alpha_bits: 0,
            c_alpha_shift: 0,
            c_accum_bits: 0,
            c_accum_red_bits: 0,
            c_accum_green_bits: 0,
            c_accum_blue_bits: 0,
            c_accum_alpha_bits: 0,
            c_depth_bits: 0,
            c_stencil_bits: 0,
            c_aux_buffers: 0,
            i_layer_type: 0,
            b_reserved: 0,
            dw_layer_mask: 0,
            dw_visible_mask: 0,
            dw_damage_mask: 0,
        }
    }
}

const PFD_DRAW_TO_WINDOW: u32 = 0x0000_0004;
const PFD_SUPPORT_OPENGL: u32 = 0x0000_0020;
const PFD_DOUBLEBUFFER: u32 = 0x0000_0002;
const PFD_TYPE_RGBA: u8 = 0;
const PFD_MAIN_PLANE: u8 = 0;

/// OpenGL rendering context bound to a window.
///
/// Created from a real [`Window`] (legacy WGL context — GL 1.1 fixed
/// function). Single-threaded use: the context is current from creation
/// until drop.
pub struct GlContext {
    hwnd: HWND,
    hdc: HDC,
    hglrc: *mut c_void,
}

impl GlContext {
    /// Create an OpenGL context for `window` and make it current.
    ///
    /// Requires an interactive desktop session (a real window + DC); fails
    /// gracefully with [`PlatformError::GlContext`] otherwise.
    pub fn new(window: &Window) -> Result<GlContext, PlatformError> {
        let hwnd = window.raw_hwnd();
        if hwnd.is_null() {
            return Err(PlatformError::GlContext("null window handle".into()));
        }
        unsafe {
            let hdc = ffi::GetDC(hwnd);
            if hdc.is_null() {
                return Err(PlatformError::GlContext("GetDC failed".into()));
            }

            let pfd = PIXELFORMATDESCRIPTOR {
                dw_flags: PFD_DRAW_TO_WINDOW | PFD_SUPPORT_OPENGL | PFD_DOUBLEBUFFER,
                i_pixel_type: PFD_TYPE_RGBA,
                c_color_bits: 32,
                c_depth_bits: 24,
                i_layer_type: PFD_MAIN_PLANE,
                ..Default::default()
            };
            let format = ChoosePixelFormat(hdc, &pfd);
            if format == 0 {
                ffi::ReleaseDC(hwnd, hdc);
                return Err(PlatformError::GlContext("ChoosePixelFormat failed".into()));
            }
            if SetPixelFormat(hdc, format, &pfd) == 0 {
                ffi::ReleaseDC(hwnd, hdc);
                return Err(PlatformError::GlContext("SetPixelFormat failed".into()));
            }

            let hglrc = wglCreateContext(hdc);
            if hglrc.is_null() {
                ffi::ReleaseDC(hwnd, hdc);
                return Err(PlatformError::GlContext("wglCreateContext failed".into()));
            }
            if wglMakeCurrent(hdc, hglrc) == 0 {
                wglDeleteContext(hglrc);
                ffi::ReleaseDC(hwnd, hdc);
                return Err(PlatformError::GlContext("wglMakeCurrent failed".into()));
            }

            Ok(GlContext { hwnd, hdc, hglrc })
        }
    }

    /// Present the back buffer.
    pub fn swap_buffers(&self) {
        unsafe {
            SwapBuffers(self.hdc);
        }
    }

    /// Last GL error state (0 = GL_NO_ERROR); diagnostics only.
    pub fn last_error(&self) -> u32 {
        unsafe { glGetError() }
    }

    pub fn is_no_error(&self) -> bool {
        self.last_error() == GL_NO_ERROR
    }
}

impl Drop for GlContext {
    fn drop(&mut self) {
        unsafe {
            wglMakeCurrent(std::ptr::null_mut(), std::ptr::null_mut());
            wglDeleteContext(self.hglrc);
            ffi::ReleaseDC(self.hwnd, self.hdc);
        }
    }
}

// --- Raw GL commands used by `Gl3DRenderer` ---------------------------------
// Public: the facade (`monster_engine::gl3d`) drives them.

pub fn clear_color_buffer(color: [f32; 4]) {
    unsafe {
        glClearColor(color[0], color[1], color[2], color[3]);
        glClear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT);
    }
}

pub fn enable_depth_test() {
    unsafe {
        glEnable(GL_DEPTH_TEST);
    }
}

pub fn set_viewport(width: u32, height: u32) {
    unsafe {
        glViewport(0, 0, width as i32, height as i32);
    }
}

pub fn load_projection_frustum(
    left: f64,
    right: f64,
    bottom: f64,
    top: f64,
    near_val: f64,
    far_val: f64,
) {
    unsafe {
        glMatrixMode(GL_PROJECTION);
        glLoadIdentity();
        glFrustum(left, right, bottom, top, near_val, far_val);
        glMatrixMode(GL_MODELVIEW);
    }
}

/// Upload a column-major 4x4 matrix into the modelview stack.
pub fn load_modelview(matrix: &[f32; 16]) {
    unsafe {
        glMatrixMode(GL_MODELVIEW);
        glLoadMatrixf(matrix.as_ptr());
    }
}

pub fn translate(x: f32, y: f32, z: f32) {
    unsafe {
        glTranslatef(x, y, z);
    }
}

pub fn rotate_y(angle_radians: f32) {
    unsafe {
        glRotatef(angle_radians.to_degrees(), 0.0, 1.0, 0.0);
    }
}

pub fn push_matrix() {
    unsafe {
        glPushMatrix();
    }
}

pub fn pop_matrix() {
    unsafe {
        glPopMatrix();
    }
}

pub fn begin_quads() {
    unsafe {
        glBegin(GL_QUADS);
    }
}

pub fn begin_lines() {
    unsafe {
        glBegin(GL_LINES);
    }
}

pub fn end() {
    unsafe {
        glEnd();
    }
}

pub fn color3(r: f32, g: f32, b: f32) {
    unsafe {
        glColor3f(r, g, b);
    }
}

pub fn vertex3(x: f32, y: f32, z: f32) {
    unsafe {
        glVertex3f(x, y, z);
    }
}

pub fn no_gl_error() -> bool {
    unsafe { glGetError() == GL_NO_ERROR }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pixel_format_descriptor_layout() {
        // Win32 PIXELFORMATDESCRIPTOR is exactly 40 bytes
        assert_eq!(std::mem::size_of::<PIXELFORMATDESCRIPTOR>(), 40);
        let pfd = PIXELFORMATDESCRIPTOR {
            dw_flags: PFD_DRAW_TO_WINDOW | PFD_SUPPORT_OPENGL | PFD_DOUBLEBUFFER,
            i_pixel_type: PFD_TYPE_RGBA,
            c_color_bits: 32,
            c_depth_bits: 24,
            i_layer_type: PFD_MAIN_PLANE,
            ..Default::default()
        };
        assert_eq!(pfd.n_size, 40);
        assert_eq!(pfd.c_depth_bits, 24);
    }

    #[test]
    fn gl_constants_match_registry() {
        // values from gl.h (OpenGL 1.1)
        assert_eq!(GL_DEPTH_BUFFER_BIT, 0x0100);
        assert_eq!(GL_COLOR_BUFFER_BIT, 0x4000);
        assert_eq!(GL_DEPTH_TEST, 0x0B71);
        assert_eq!(GL_PROJECTION, 0x1701);
        assert_eq!(GL_MODELVIEW, 0x1700);
        assert_eq!(GL_QUADS, 0x0007);
        assert_eq!(GL_LINES, 0x0001);
        assert_eq!(GL_NO_ERROR, 0);
    }

    /// Creating a real GL context needs an interactive desktop session.
    /// Kept `#[ignore]` so automated runs never flash a window; run with
    /// `cargo test -p monster-platform -- --ignored` on a desktop session.
    #[test]
    #[ignore = "requires an interactive desktop session (creates a real window)"]
    fn gl_context_creates_and_destroys() {
        let window = crate::window::Window::new(&crate::window::WindowOptions {
            width: 320,
            height: 240,
            title: "GL context test".into(),
        })
        .expect("window");
        let context = GlContext::new(&window).expect("gl context");
        assert!(context.is_no_error());
        drop(context);
    }
}
