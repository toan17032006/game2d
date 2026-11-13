import io

p = 'crates/monster-platform/src/gl.rs'
with io.open(p, 'r', encoding='utf-8') as f:
    c = f.read()

anchor = '#[cfg(test)]\nmod tests {'
assert anchor in c, 'anchor missing'

modern = '''// --- M3 slice 3: modern GL via extension loading (shader pipeline) ---------
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
                wglGetProcAddress(cname.as_ptr()) as usize
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
            if complete { Some(fns) } else { None }
        })
        .is_some()
}

/// Whether the modern shader functions are loaded and usable.
pub fn modern_available() -> bool {
    load_modern_functions()
}

pub(crate) const SHADER_VS: &str = "#version 120\\nvoid main() { gl_Position = ftransform(); gl_FrontColor = gl_Color; }";
pub(crate) const SHADER_FS: &str = "#version 120\\nuniform vec3 u_tint; void main() { gl_FragColor = vec4(gl_Color.rgb * u_tint, gl_Color.a); }";

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
        use_program(program);
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
        let location = get_uniform_location(program, cname.as_ptr());
        if location >= 0 {
            uniform3f(location, x, y, z);
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

'''
c = c.replace(anchor, modern + anchor, 1)

c = c.replace('''const GL_QUADS: u32 = 0x0007;''', '''const GL_QUADS: u32 = 0x0007;
const GL_TRIANGLES: u32 = 0x0004;
const GL_VERTEX_SHADER: u32 = 0x8B31;
const GL_FRAGMENT_SHADER: u32 = 0x8B30;
const GL_COMPILE_STATUS: u32 = 0x8B81;
const GL_LINK_STATUS: u32 = 0x8B82;''')

with io.open(p, 'w', encoding='utf-8') as f:
    f.write(c)
print('gl.rs modern ok:', 'load_modern_functions' in c)
