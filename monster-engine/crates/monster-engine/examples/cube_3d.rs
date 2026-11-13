//! Monster Engine 3D demo (M3 slice): a rotating cube over a reference
//! grid, rendered with the legacy-GL backend through the physics-free raw
//! loop. ESC or closing the window exits.
//!
//! Requires an interactive desktop session (a real window + GL context).

use monster_engine::gl3d::Gl3DRenderer;
use monster_engine::prelude::*;
use monster_platform::GlContext;

fn main() {
    set_max_level(Level::Info);

    // 1. window
    let mut window = match Window::new(&WindowOptions {
        width: 960,
        height: 540,
        title: "Monster Engine — 3D Cube (M3 slice)".to_string(),
    }) {
        Ok(w) => w,
        Err(e) => {
            log_error!("cannot open a window in this session: {e}");
            std::process::exit(1);
        }
    };

    // 2. GL context + renderer
    let context = match GlContext::new(&window) {
        Ok(c) => c,
        Err(e) => {
            log_error!("cannot create an OpenGL context: {e}");
            std::process::exit(1);
        }
    };
    let (w, h) = window.size();
    let mut renderer = Gl3DRenderer::new_unbound(w, h);
    renderer.bind(&context);
    renderer.set_clear_color(0.08, 0.09, 0.13, 1.0);

    // 3. camera orbit state
    let mut orbit_angle: f32 = 0.0;
    let mut events = EventBus::new();

    log_info!("cube_3d running — ESC or close the window to exit");

    // 4. raw loop (M3 slice demo; the 2D Engine loop stays untouched)
    while window.is_open() {
        window.poll_events(&mut events);
        if !window.is_open() {
            break;
        }
        for event in events.drain() {
            if event == EngineEvent::WindowClosed {
                std::mem::take(&mut events);
                window.close();
            }
        }

        orbit_angle += 0.8f32.to_radians() * (1.0 / 60.0);
        let camera = Camera3D::new(
            Vec3::new(orbit_angle.sin() * 6.0, 3.0, orbit_angle.cos() * 6.0),
            Vec3::new(0.0, 0.5, 0.0),
            w as f32 / h.max(1) as f32,
        );

        renderer.begin(&camera);
        renderer.draw_grid(10.0, 20, Color::opaque(70, 90, 70));
        renderer.draw_cube(
            Vec3::new(0.0, 0.6, 0.0),
            0.6,
            Color::opaque(146, 110, 205),
            orbit_angle * 3.0,
        );
        renderer.draw_cube(
            Vec3::new(2.5, 0.4, -1.5),
            0.35,
            Color::opaque(90, 165, 230),
            -orbit_angle * 2.0,
        );
        renderer.draw_cube(
            Vec3::new(-2.5, 0.4, 1.5),
            0.35,
            Color::opaque(230, 165, 90),
            orbit_angle * 2.0,
        );
        renderer.end();
        context.swap_buffers();
    }

    log_info!("cube_3d exited cleanly");
}
