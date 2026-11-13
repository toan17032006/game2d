//! `Gl3DRenderer` — the M3 3D renderer: legacy OpenGL 1.1 fixed-function
//! driven through `monster_platform::gl`'s `GlContext`.
//!
//! Implements `monster_core::Renderer3D`. The renderer works in two modes:
//! - **Bound** (`bind(&GlContext)`): GL commands issue for real (window demo).
//! - **Unbound** (headless): state-only recorder — tests and CI can drive the
//!   full frame protocol without a desktop session (NullWindow philosophy).

use monster_core::graphics::{Camera3D, Renderer3D};
use monster_core::mesh::Mesh3D;
use monster_math::{Color, Vec3};
use monster_platform::gl;
use monster_platform::GlContext;

/// Legacy-GL 3D renderer.
pub struct Gl3DRenderer {
    viewport: (u32, u32),
    clear_color: [f32; 4],
    draw_count: u32,
    /// True once bound to a live `GlContext`; false = state-only recorder.
    bound: bool,
    /// Compiled shader program id when the modern-GL extension functions
    /// are available on the bound context (0 = fixed-function fallback).
    shader_program: u32,
}

impl Gl3DRenderer {
    /// Headless/state-only renderer: full frame protocol without GL calls.
    pub fn new_unbound(width: u32, height: u32) -> Gl3DRenderer {
        Gl3DRenderer {
            viewport: (width, height),
            clear_color: [0.08, 0.09, 0.13, 1.0],
            draw_count: 0,
            bound: false,
            shader_program: 0,
        }
    }

    /// Bind to a live GL context: enables depth testing and issues GL from
    /// now on. Requires an interactive desktop session.
    pub fn bind(&mut self, context: &GlContext) {
        let _ = context;
        gl::enable_depth_test();
        // M3 slice 3: attempt the shader pipeline; fall back to
        // fixed-function when the extension functions are unavailable.
        if gl::load_modern_functions() {
            if let Ok(program) = gl::compile_program(gl::SHADER_VS, gl::SHADER_FS) {
                self.shader_program = program;
            }
        }
        self.bound = true;
    }

    /// Whether the shader pipeline is active (0 = fixed-function fallback).
    pub fn shader_program(&self) -> u32 {
        self.shader_program
    }

    pub fn is_bound(&self) -> bool {
        self.bound
    }

    /// Resize the viewport (window resize).
    pub fn resize(&mut self, width: u32, height: u32) {
        self.viewport = (width, height);
    }

    /// Set the clear color (0..1 floats, default dark blue-grey).
    pub fn set_clear_color(&mut self, r: f32, g: f32, b: f32, a: f32) {
        self.clear_color = [r, g, b, a];
    }

    fn issue(&self) -> bool {
        self.bound
    }
}

impl Renderer3D for Gl3DRenderer {
    fn begin(&mut self, camera: &Camera3D) {
        if self.issue() {
            gl::set_viewport(self.viewport.0, self.viewport.1);
            gl::clear_color_buffer(self.clear_color);
            gl::enable_depth_test();
            if self.shader_program != 0 {
                gl::use_program(self.shader_program);
                gl::uniform3f(self.shader_program, "u_tint", 1.0, 1.0, 1.0);
            }

            // projection: perspective frustum derived from fov/aspect
            let top = camera.near * (camera.fov_y_radians / 2.0).tan();
            let bottom = -top;
            let right = top * camera.aspect;
            let left = -right;
            gl::load_projection_frustum(
                left as f64,
                right as f64,
                bottom as f64,
                top as f64,
                camera.near as f64,
                camera.far as f64,
            );

            // view: look_at matrix (column-major 4x4)
            let view = camera.view_matrix();
            let mut m = [0.0f32; 16];
            for col in 0..4 {
                m[col * 4] = view.cols[col].x;
                m[col * 4 + 1] = view.cols[col].y;
                m[col * 4 + 2] = view.cols[col].z;
                m[col * 4 + 3] = view.cols[col].w;
            }
            gl::load_modelview(&m);
        }
        self.draw_count = 0;
    }

    fn draw_cube(&mut self, position: Vec3, half_size: f32, color: Color, rotation_y: f32) {
        if self.issue() {
            gl::push_matrix();
            gl::translate(position.x, position.y, position.z);
            gl::rotate_y(rotation_y);
            let (r, g, b, _) = color.to_f32_tuple();
            let s = half_size;

            gl::begin_quads();
            // front
            gl::color3(r, g, b);
            gl::vertex3(-s, -s, s);
            gl::vertex3(s, -s, s);
            gl::vertex3(s, s, s);
            gl::vertex3(-s, s, s);
            // back
            gl::color3(r * 0.6, g * 0.6, b * 0.6);
            gl::vertex3(s, -s, -s);
            gl::vertex3(-s, -s, -s);
            gl::vertex3(-s, s, -s);
            gl::vertex3(s, s, -s);
            // right
            gl::color3(r * 0.8, g * 0.8, b * 0.8);
            gl::vertex3(s, -s, s);
            gl::vertex3(s, -s, -s);
            gl::vertex3(s, s, -s);
            gl::vertex3(s, s, s);
            // left
            gl::color3(r * 0.5, g * 0.5, b * 0.5);
            gl::vertex3(-s, -s, -s);
            gl::vertex3(-s, -s, s);
            gl::vertex3(-s, s, s);
            gl::vertex3(-s, s, -s);
            // top
            gl::color3((r * 1.2).min(1.0), (g * 1.2).min(1.0), (b * 1.2).min(1.0));
            gl::vertex3(-s, s, s);
            gl::vertex3(s, s, s);
            gl::vertex3(s, s, -s);
            gl::vertex3(-s, s, -s);
            // bottom
            gl::color3(r * 0.35, g * 0.35, b * 0.35);
            gl::vertex3(-s, -s, -s);
            gl::vertex3(s, -s, -s);
            gl::vertex3(s, -s, s);
            gl::vertex3(-s, -s, s);
            gl::end();
            gl::pop_matrix();
        }
        self.draw_count += 1;
    }

    fn draw_mesh(&mut self, mesh: &Mesh3D, position: Vec3, rotation_y: f32, color: Color) {
        if self.issue() {
            gl::push_matrix();
            gl::translate(position.x, position.y, position.z);
            gl::rotate_y(rotation_y);
            let (r, g, b, _) = color.to_f32_tuple();
            gl::color3(r, g, b);
            gl::begin_triangles();
            for &index in &mesh.indices {
                let v = mesh.vertices[index as usize];
                gl::vertex3(v.x, v.y, v.z);
            }
            gl::end();
            gl::pop_matrix();
        }
        self.draw_count += 1;
    }

    fn draw_grid(&mut self, half_size: f32, divisions: u32, color: Color) {
        if self.issue() {
            let (r, g, b, _) = color.to_f32_tuple();
            gl::color3(r, g, b);
            gl::begin_lines();
            let step = half_size * 2.0 / divisions.max(1) as f32;
            for i in 0..=divisions {
                let x = -half_size + step * i as f32;
                gl::vertex3(x, 0.0, -half_size);
                gl::vertex3(x, 0.0, half_size);
                gl::vertex3(-half_size, 0.0, x);
                gl::vertex3(half_size, 0.0, x);
            }
            gl::end();
        }
        self.draw_count += 1;
    }

    fn end(&mut self) {
        if self.issue() && self.shader_program != 0 {
            gl::use_program(0); // back to fixed-function
        }
        // present is performed by the caller through GlContext::swap_buffers
    }

    fn draw_count(&self) -> u32 {
        self.draw_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unbound_renderer_drives_full_frame_protocol() {
        let mut renderer = Gl3DRenderer::new_unbound(800, 600);
        renderer.set_clear_color(0.2, 0.1, 0.0, 1.0);
        assert!(!renderer.is_bound());
        let camera = Camera3D::new(Vec3::new(0.0, 3.0, 8.0), Vec3::ZERO, 800.0 / 600.0);
        renderer.begin(&camera);
        renderer.draw_cube(Vec3::ZERO, 1.0, Color::GREEN, 0.25);
        renderer.draw_grid(10.0, 10, Color::WHITE);
        renderer.end();
        assert_eq!(renderer.draw_count(), 2);
        // resize tracked
        renderer.resize(1920, 1080);
        renderer.begin(&camera);
        assert_eq!(renderer.draw_count(), 0); // counter resets per frame
        assert!(renderer.is_bound() == false);
    }

    #[test]
    fn draw_mesh_counts_in_unbound_mode() {
        let mut renderer = Gl3DRenderer::new_unbound(1, 1);
        let camera = Camera3D::new(Vec3::new(0.0, 0.0, 5.0), Vec3::ZERO, 1.0);
        let mesh = Mesh3D::cube(0.5);
        renderer.begin(&camera);
        renderer.draw_mesh(&mesh, Vec3::new(1.0, 0.0, 0.0), 0.25, Color::BLUE);
        renderer.end();
        assert_eq!(renderer.draw_count(), 1);
    }

    #[test]
    fn camera3d_drives_begin_state() {
        let mut renderer = Gl3DRenderer::new_unbound(640, 480);
        let camera = Camera3D::new(
            Vec3::new(5.0, 4.0, 9.0),
            Vec3::new(0.0, 1.0, 0.0),
            4.0 / 3.0,
        );
        renderer.begin(&camera);
        // state-only: no GL calls, protocol still runs
        assert_eq!(renderer.draw_count(), 0);
    }

    #[test]
    fn cube_rotation_is_accepted_parameter() {
        let mut renderer = Gl3DRenderer::new_unbound(1, 1);
        let camera = Camera3D::new(Vec3::new(0.0, 0.0, 5.0), Vec3::ZERO, 1.0);
        renderer.begin(&camera);
        for angle in [0.0f32, 1.0, 2.5, std::f32::consts::PI] {
            renderer.draw_cube(Vec3::new(0.0, 0.0, 0.0), 0.5, Color::RED, angle);
        }
        assert_eq!(renderer.draw_count(), 4);
    }
}
