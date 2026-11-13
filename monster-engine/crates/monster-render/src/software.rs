//! [`Software2DRenderer`] — the default [`Renderer`] backend.

use monster_core::graphics::{Camera2D, DrawOptions, PresentTarget, Renderer, Texture};
use monster_math::Vec2;
use monster_math::{Color, Rect};

use crate::framebuffer::Framebuffer;

/// Immediate-mode software 2D renderer.
///
/// Each frame: `begin(camera)` clears the framebuffer and installs the
/// camera; draw calls rasterize immediately; `end(target)` submits the
/// finished buffer to the present target.
pub struct Software2DRenderer {
    framebuffer: Framebuffer,
    clear_color: Color,
    camera: Option<Camera2D>,
    draw_count: u32,
}

impl Software2DRenderer {
    pub fn new(width: u32, height: u32, clear_color: Color) -> Software2DRenderer {
        Software2DRenderer {
            framebuffer: Framebuffer::new(width, height, clear_color),
            clear_color,
            camera: None,
            draw_count: 0,
        }
    }

    /// Pixel buffer of the current/last frame (headless tests read this).
    pub fn framebuffer(&self) -> &Framebuffer {
        &self.framebuffer
    }

    /// Resize the backing framebuffer (e.g. on `WindowResized`).
    pub fn resize(&mut self, width: u32, height: u32) {
        self.framebuffer.resize(width, height, self.clear_color);
    }

    pub fn set_clear_color(&mut self, color: Color) {
        self.clear_color = color;
    }

    /// Convert a world-space rect into a screen-space rect for the camera.
    fn world_rect_to_screen(camera: &Camera2D, dst: Rect) -> Rect {
        let a = camera.world_to_screen(dst.min());
        let b = camera.world_to_screen(dst.max());
        let min = Vec2::new(a.x.min(b.x), a.y.min(b.y));
        let size = Vec2::new((a.x - b.x).abs(), (a.y - b.y).abs());
        Rect {
            position: min,
            size,
        }
    }
}

impl Renderer for Software2DRenderer {
    fn begin(&mut self, camera: &Camera2D) {
        if self.framebuffer.width() != camera.viewport.0 as u32
            || self.framebuffer.height() != camera.viewport.1 as u32
        {
            self.framebuffer.resize(
                camera.viewport.0 as u32,
                camera.viewport.1 as u32,
                self.clear_color,
            );
        } else {
            self.framebuffer.resize(
                self.framebuffer.width(),
                self.framebuffer.height(),
                self.clear_color,
            );
        }
        self.camera = Some(camera.clone());
        self.draw_count = 0;
    }

    fn draw_sprite(&mut self, texture: &Texture, dst: Rect, src: Option<Rect>, opts: DrawOptions) {
        let Some(camera) = &self.camera else {
            return;
        };
        let screen = Self::world_rect_to_screen(camera, dst);
        self.framebuffer.blit_sprite(texture, screen, src, opts);
        self.draw_count += 1;
    }

    fn draw_rect(&mut self, dst: Rect, color: Color) {
        let Some(camera) = &self.camera else {
            return;
        };
        let screen = Self::world_rect_to_screen(camera, dst);
        self.framebuffer.fill_rect(screen, color);
        self.draw_count += 1;
    }

    fn end(&mut self, target: &mut dyn PresentTarget) {
        target.present(
            self.framebuffer.width(),
            self.framebuffer.height(),
            self.framebuffer.pixels(),
        );
    }

    fn draw_count(&self) -> u32 {
        self.draw_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use monster_math::Color;

    /// Headless present target capturing the submitted frame.
    struct CapturingTarget {
        frames: Vec<(u32, u32, Vec<u8>)>,
    }

    impl CapturingTarget {
        fn new() -> CapturingTarget {
            CapturingTarget { frames: Vec::new() }
        }
    }

    impl PresentTarget for CapturingTarget {
        fn present(&mut self, width: u32, height: u32, rgba: &[u8]) {
            self.frames.push((width, height, rgba.to_vec()));
        }
    }

    #[test]
    fn begin_clears_and_counts() {
        let mut r = Software2DRenderer::new(4, 4, Color::BLACK);
        let cam = Camera2D::new(4.0, 4.0);
        r.begin(&cam);
        r.draw_rect(Rect::new(0.0, 0.0, 2.0, 2.0), Color::RED);
        assert_eq!(r.draw_count(), 1);
        r.begin(&cam); // new frame: clear + counter reset
        assert_eq!(r.draw_count(), 0);
        assert_eq!(r.framebuffer().get_pixel(0, 0), [0, 0, 0, 255]);
    }

    #[test]
    fn camera_transform_maps_world_to_screen() {
        // Camera (2,1) sits at the screen CENTER: world rect (1,0)-(3,2)
        // spans the whole 2x2 viewport; a 1x1 rect at the camera position
        // covers only the bottom-right pixel.
        let mut cam = Camera2D::new(2.0, 2.0);
        cam.position = Vec2::new(2.0, 1.0);
        let mut r = Software2DRenderer::new(2, 2, Color::BLACK);
        r.begin(&cam);
        r.draw_rect(Rect::new(2.0, 1.0, 1.0, 1.0), Color::RED);
        assert_eq!(r.framebuffer().get_pixel(1, 1), [255, 0, 0, 255]);
        assert_eq!(r.framebuffer().get_pixel(0, 0), [0, 0, 0, 255]);
        r.begin(&cam);
        r.draw_rect(Rect::new(1.0, 0.0, 2.0, 2.0), Color::RED);
        assert_eq!(r.framebuffer().get_pixel(0, 0), [255, 0, 0, 255]);
        assert_eq!(r.framebuffer().get_pixel(1, 1), [255, 0, 0, 255]);
    }

    #[test]
    fn zoom_shrinks_draw_area() {
        // Camera at world (0,0) = screen center (2,2). Zoom 2: a 2x2 world
        // rect starting at the camera covers screen pixels (2,2)-(6,6),
        // clipped to (2,2)..(3,3) on a 4x4 framebuffer.
        let mut cam = Camera2D::new(4.0, 4.0);
        cam.zoom = 2.0;
        let mut r = Software2DRenderer::new(4, 4, Color::BLACK);
        r.begin(&cam);
        r.draw_rect(Rect::new(0.0, 0.0, 2.0, 2.0), Color::GREEN);
        assert_eq!(r.framebuffer().get_pixel(2, 2), [0, 255, 0, 255]);
        assert_eq!(r.framebuffer().get_pixel(3, 3), [0, 255, 0, 255]);
        assert_eq!(r.framebuffer().get_pixel(0, 0), [0, 0, 0, 255]);
        assert_eq!(r.framebuffer().get_pixel(1, 1), [0, 0, 0, 255]);
    }

    #[test]
    fn end_presents_frame_to_target() {
        let mut r = Software2DRenderer::new(2, 2, Color::BLUE);
        let cam = Camera2D::new(2.0, 2.0);
        r.begin(&cam);
        let mut target = CapturingTarget::new();
        r.end(&mut target);
        assert_eq!(target.frames.len(), 1);
        let (w, h, bytes) = &target.frames[0];
        assert_eq!((*w, *h), (2, 2));
        assert_eq!(bytes.len(), 2 * 2 * 4);
        assert_eq!(&bytes[0..4], &[0, 0, 255, 255]);
    }

    #[test]
    fn sprite_via_camera_and_clipping_no_panic() {
        let mut r = Software2DRenderer::new(4, 4, Color::BLACK);
        let cam = Camera2D::new(4.0, 4.0);
        let tex = Texture::solid(2, 2, Color::RED);
        r.begin(&cam);
        // far off-screen (camera center maps world (0,0) to screen (2,2)):
        // clipped away entirely
        r.draw_sprite(
            &tex,
            Rect::new(-10.0, -10.0, 2.0, 2.0),
            None,
            DrawOptions::default(),
        );
        assert_eq!(r.framebuffer().get_pixel(0, 0), [0, 0, 0, 255]);
        // world rect (1,1)-(3,3) -> screen (3,3)-(5,5): only pixel (3,3) lands
        r.draw_sprite(
            &tex,
            Rect::new(1.0, 1.0, 2.0, 2.0),
            None,
            DrawOptions::default(),
        );
        assert_eq!(r.framebuffer().get_pixel(3, 3), [255, 0, 0, 255]);
        assert_eq!(r.draw_count(), 2);
    }
}
