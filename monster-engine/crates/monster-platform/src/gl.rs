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
const GL_TRIANGLES: u32 = 0x0004;
const GL_VERTEX_SHADER: u32 = 0x8B31;
const GL_FRAGMENT_SHADER: u32 = 0x8B30;
const GL_COMPILE_STATUS: u32 = 0x8B81;
const GL_LINK_STATUS: u32 = 0x8B82;
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

pub fn begin_triangles() {
    unsafe {
        glBegin(GL_TRIANGLES);
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

// --- M3 slice 3: modern GL via extension loading (shader pipeline) ---------
//
// Functions beyond GL 1.1 are not in opengl32.lib - they must be fetched
// with wglGetProcAddress while a context is current. Every pointer is
// null-guarded; when anything is missing the renderer falls back to
// fixed-function (glColor path).

pub(crate) type WglGetProcAddressFn = unsafe extern "system" fn(*const u8) -> *mut c_void;

#[link(name = "opengl32")]
extern "system" {
    fn wglGetProcAddress(name: *const u8) -> *mut c_void;
}

#[repr(C)]
#[derive(Default, Clone, Copy)]
pub(crate) struct ModernFns {
    pub create_shader: usize,
    pub shader_source: usize,
    pub compile_shader: usize,
    pub get_shader_iv: usize,
    pub get_shader_info_log: usize,
    pub create_program: usize,
    pub attach_shader: usize,
    pub link_program: usize,
    pub get_program_iv: usize,
    pub use_program: usize,
    pub get_uniform_location: usize,
    pub uniform3f: usize,
}

static MODERN: std::sync::OnceLock<Option<ModernFns>> = std::sync::OnceLock::new();

/// Load the modern-GL function pointers through `wglGetProcAddress`.
/// Returns false when any required pointer is missing (no context, or the
/// driver does not support shaders) - callers must fall back.
pub fn load_modern_functions() -> bool {
    MODERN
        .get_or_init(|| unsafe {
            let fetch = |name: &str| -> usize {
                let mut cname = name.as_bytes().to_vec();
                cname.push(0);
                unsafe { wglGetProcAddress(cname.as_ptr()) as usize }
            };
            let fns = ModernFns {
                create_shader: fetch("glCreateShader"),
                shader_source: fetch("glShaderSource"),
                compile_shader: fetch("glCompileShader"),
                get_shader_iv: fetch("glGetShaderiv"),
                get_shader_info_log: fetch("glGetShaderInfoLog"),
                create_program: fetch("glCreateProgram"),
                attach_shader: fetch("glAttachShader"),
                link_program: fetch("glLinkProgram"),
                get_program_iv: fetch("glGetProgramiv"),
                use_program: fetch("glUseProgram"),
                get_uniform_location: fetch("glGetUniformLocation"),
                uniform3f: fetch("glUniform3f"),
            };
            let complete = fns.create_shader != 0
                && fns.shader_source != 0
                && fns.compile_shader != 0
                && fns.get_shader_iv != 0
                && fns.get_shader_info_log != 0
                && fns.create_program != 0
                && fns.attach_shader != 0
                && fns.link_program != 0
                && fns.get_program_iv != 0
                && fns.use_program != 0
                && fns.get_uniform_location != 0
                && fns.uniform3f != 0;
            if complete {
                Some(fns)
            } else {
                None
            }
        })
        .is_some()
}

/// Whether the modern shader functions are loaded and usable.
pub fn modern_available() -> bool {
    load_modern_functions()
}

pub const SHADER_VS: &str =
    "#version 120\nvoid main() { gl_Position = ftransform(); gl_FrontColor = gl_Color; }";
pub const SHADER_FS: &str = "#version 120\nuniform vec3 u_tint; void main() { gl_FragColor = vec4(gl_Color.rgb * u_tint, gl_Color.a); }";

type GlCreateShader = unsafe extern "system" fn(u32) -> u32;
type GlShaderSource = unsafe extern "system" fn(u32, i32, *const *const u8, *const i32);
type GlCompileShader = unsafe extern "system" fn(u32);
type GlGetShaderIv = unsafe extern "system" fn(u32, u32, *mut i32);
type GlGetShaderInfoLog = unsafe extern "system" fn(u32, i32, *mut i32, *mut u8);
type GlCreateProgram = unsafe extern "system" fn() -> u32;
type GlAttachShader = unsafe extern "system" fn(u32, u32);
type GlLinkProgram = unsafe extern "system" fn(u32);
type GlGetProgramIv = unsafe extern "system" fn(u32, u32, *mut i32);
type GlUseProgram = unsafe extern "system" fn(u32);
type GlGetUniformLocation = unsafe extern "system" fn(u32, *const u8) -> i32;
type GlUniform3f = unsafe extern "system" fn(i32, f32, f32, f32);

/// Compile + link a vertex/fragment shader pair (GLSL 120). Returns the
/// program id or a descriptive error with the info log.
pub fn compile_program(vs_src: &str, fs_src: &str) -> Result<u32, String> {
    let fns = MODERN
        .get()
        .and_then(|m| m.as_ref())
        .ok_or_else(|| "modern GL not available".to_string())?;
    unsafe {
        let create_shader: GlCreateShader = std::mem::transmute(fns.create_shader);
        let shader_source: GlShaderSource = std::mem::transmute(fns.shader_source);
        let compile_shader: GlCompileShader = std::mem::transmute(fns.compile_shader);
        let get_shader_iv: GlGetShaderIv = std::mem::transmute(fns.get_shader_iv);
        let get_shader_info_log: GlGetShaderInfoLog = std::mem::transmute(fns.get_shader_info_log);
        let create_program: GlCreateProgram = std::mem::transmute(fns.create_program);
        let attach_shader: GlAttachShader = std::mem::transmute(fns.attach_shader);
        let link_program: GlLinkProgram = std::mem::transmute(fns.link_program);
        let get_program_iv: GlGetProgramIv = std::mem::transmute(fns.get_program_iv);

        let compile_stage = |stage: u32, src: &str| -> Result<u32, String> {
            let shader = create_shader(stage);
            let mut bytes = src.as_bytes().to_vec();
            bytes.push(0);
            let ptr: *const u8 = bytes.as_ptr();
            let len = (bytes.len() - 1) as i32;
            shader_source(shader, 1, &ptr, &len);
            compile_shader(shader);
            let mut ok = 0i32;
            get_shader_iv(shader, GL_COMPILE_STATUS, &mut ok);
            if ok == 0 {
                let mut log = vec![0u8; 1024];
                let mut written = 0i32;
                get_shader_info_log(shader, 1024, &mut written, log.as_mut_ptr());
                let log = String::from_utf8_lossy(&log[..written.max(0) as usize]).into_owned();
                return Err(format!("shader compile failed: {log}"));
            }
            Ok(shader)
        };

        let vs = compile_stage(GL_VERTEX_SHADER, vs_src)?;
        let fs = compile_stage(GL_FRAGMENT_SHADER, fs_src)?;
        let program = create_program();
        attach_shader(program, vs);
        attach_shader(program, fs);
        link_program(program);
        let mut ok = 0i32;
        get_program_iv(program, GL_LINK_STATUS, &mut ok);
        if ok == 0 {
            return Err("program link failed".into());
        }
        Ok(program)
    }
}

/// Bind a shader program (0 = fixed-function).
pub fn use_program(program: u32) {
    if let Some(fns) = MODERN.get().and_then(|m| m.as_ref()) {
        let use_program: GlUseProgram = unsafe { std::mem::transmute(fns.use_program) };
        unsafe { use_program(program) };
    }
}

/// Set a vec3 uniform on the current program.
pub fn uniform3f(program: u32, name: &str, x: f32, y: f32, z: f32) {
    if let Some(fns) = MODERN.get().and_then(|m| m.as_ref()) {
        let get_uniform_location: GlGetUniformLocation =
            unsafe { std::mem::transmute(fns.get_uniform_location) };
        let uniform3f: GlUniform3f = unsafe { std::mem::transmute(fns.uniform3f) };
        let mut cname = name.as_bytes().to_vec();
        cname.push(0);
        let location = unsafe { get_uniform_location(program, cname.as_ptr()) };
        if location >= 0 {
            unsafe { uniform3f(location, x, y, z) };
        }
    }
}

#[cfg(test)]
mod modern_tests {
    use super::*;

    #[test]
    fn extension_loader_without_context_falls_back() {
        // No current context in headless runs: pointers come back null and
        // the loader must report "not available" instead of crashing.
        let _ = load_modern_functions();
        assert!(!modern_available());
    }

    #[test]
    fn shader_sources_are_glsl_120_pass_through() {
        assert!(SHADER_VS.contains("#version 120"));
        assert!(SHADER_VS.contains("ftransform"));
        assert!(SHADER_FS.contains("#version 120"));
        assert!(SHADER_FS.contains("u_tint"));
        assert!(SHADER_FS.contains("gl_Color"));
    }

    #[test]
    fn compile_program_requires_modern_gl() {
        // without a current context the loader reports unavailable -> Err
        if !modern_available() {
            assert!(compile_program(SHADER_VS, SHADER_FS).is_err());
        }
    }
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
