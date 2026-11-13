//! Graphics contracts: texture, camera, draw options and the backend traits.
//!
//! These interfaces are the engine's extensibility boundary. `Renderer` is
//! implemented today by the software 2D backend (`monster-render`); a future
//! GPU/3D backend implements the same trait, so scene and game code never
//! changes. `PresentTarget` decouples "what was drawn" (a pixel buffer) from
//! "where it shows up" (a Win32 window, or a headless capture in tests).

use monster_math::{Mat4, Rect, Vec2};

/// CPU-side RGBA8 image usable as a sprite source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Texture {
    width: u32,
    height: u32,
    /// Row-major RGBA8 bytes, `width * height * 4` long.
    pixels: Vec<u8>,
}

impl Texture {
    /// Create a texture from RGBA8 bytes; `pixels.len()` must match the size.
    pub fn from_rgba(width: u32, height: u32, pixels: Vec<u8>) -> Texture {
        let expected = (width as usize) * (height as usize) * 4;
        assert_eq!(
            pixels.len(),
            expected,
            "texture pixel buffer must be width*height*4 bytes"
        );
        Texture {
            width,
            height,
            pixels,
        }
    }

    /// Solid-color texture of the given size.
    pub fn solid(width: u32, height: u32, color: monster_math::Color) -> Texture {
        let [r, g, b, a] = color.to_rgba_bytes();
        Texture::from_rgba(
            width,
            height,
            vec![r, g, b, a].repeat((width as usize) * (height as usize)),
        )
    }

    /// Checkerboard texture — handy for tests and demos.
    pub fn checker(
        size: u32,
        cell: u32,
        a: monster_math::Color,
        b: monster_math::Color,
    ) -> Texture {
        let mut pixels = Vec::with_capacity((size * size * 4) as usize);
        for y in 0..size {
            for x in 0..size {
                let cell_color = if ((x / cell) + (y / cell)) % 2 == 0 {
                    a
                } else {
                    b
                };
                pixels.extend_from_slice(&cell_color.to_rgba_bytes());
            }
        }
        Texture::from_rgba(size, size, pixels)
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn pixels(&self) -> &[u8] {
        &self.pixels
    }

    /// Sample a texel with clamped coordinates.
    pub fn sample(&self, x: u32, y: u32) -> [u8; 4] {
        let x = x.min(self.width.saturating_sub(1));
        let y = y.min(self.height.saturating_sub(1));
        let i = ((y as usize) * (self.width as usize) + x as usize) * 4;
        [
            self.pixels[i],
            self.pixels[i + 1],
            self.pixels[i + 2],
            self.pixels[i + 3],
        ]
    }
}

/// 2D camera: world position, zoom and target viewport.
///
/// The view matrix maps world coordinates to clip space; the software
/// renderer additionally derives a world-to-screen helper from it.
#[derive(Debug, Clone, PartialEq)]
pub struct Camera2D {
    pub position: Vec2,
    pub zoom: f32,
    pub viewport: (f32, f32),
}

impl Camera2D {
    pub fn new(viewport_width: f32, viewport_height: f32) -> Camera2D {
        Camera2D {
            position: Vec2::ZERO,
            zoom: 1.0,
            viewport: (viewport_width, viewport_height),
        }
    }

    /// Column-major view matrix for this camera.
    pub fn view_matrix(&self) -> Mat4 {
        Mat4::view_2d(self.position, 0.0, self.zoom, self.viewport)
    }

    /// Map a world-space point to screen pixels.
    pub fn world_to_screen(&self, world: Vec2) -> Vec2 {
        let (w, h) = self.viewport;
        let center = Vec2::new(w, h) * 0.5;
        center + (world - self.position) * self.zoom
    }

    /// Map screen pixels back to world coordinates.
    pub fn screen_to_world(&self, screen: Vec2) -> Vec2 {
        let (w, h) = self.viewport;
        let center = Vec2::new(w, h) * 0.5;
        self.position + (screen - center) / self.zoom
    }
}

/// Per-sprite drawing parameters.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DrawOptions {
    /// Multiplied against source colors (255 = no tint).
    pub tint: monster_math::Color,
    /// Mirror the source horizontally.
    pub flip_x: bool,
    /// Mirror the source vertically.
    pub flip_y: bool,
}

impl Default for DrawOptions {
    fn default() -> Self {
        DrawOptions {
            tint: monster_math::Color::WHITE,
            flip_x: false,
            flip_y: false,
        }
    }
}

/// Destination a renderer submits its finished frame to.
pub trait PresentTarget {
    /// Submit `rgba` (row-major RGBA8, `width*height*4` bytes).
    fn present(&mut self, width: u32, height: u32, rgba: &[u8]);
}

/// Backend-agnostic drawing interface.
///
/// Frame protocol: `begin(camera)` → any number of `draw_*` → `end(target)`.
/// Draw calls are issued in call order; sorting by layer/z is the scene
/// layer's job before it hands batches to the renderer.
pub trait Renderer {
    /// Start a frame; implementations may clear to their configured color.
    fn begin(&mut self, camera: &Camera2D);

    /// Draw a sprite. `src` selects a sub-rectangle of the texture
    /// (`None` = the whole texture); `dst` is the destination rect in
    /// world space.
    fn draw_sprite(&mut self, texture: &Texture, dst: Rect, src: Option<Rect>, opts: DrawOptions);

    /// Draw a solid-color rectangle in world space.
    fn draw_rect(&mut self, dst: Rect, color: monster_math::Color);

    /// Finish the frame and submit it to `target`.
    fn end(&mut self, target: &mut dyn PresentTarget);

    /// Number of draw items issued in the current/last frame (for stats).
    fn draw_count(&self) -> u32;
}

#[cfg(test)]
mod tests {
    use super::*;
    use monster_math::Color;

    #[test]
    fn texture_solid_and_sample() {
        let t = Texture::solid(3, 2, Color::opaque(10, 20, 30));
        assert_eq!(t.width(), 3);
        assert_eq!(t.height(), 2);
        assert_eq!(t.pixels().len(), 3 * 2 * 4);
        assert_eq!(t.sample(2, 1), [10, 20, 30, 255]);
        // clamped sampling beyond bounds
        assert_eq!(t.sample(99, 99), [10, 20, 30, 255]);
    }

    #[test]
    fn checker_alternates() {
        let t = Texture::checker(4, 2, Color::BLACK, Color::WHITE);
        assert_eq!(t.sample(0, 0), [0, 0, 0, 255]);
        assert_eq!(t.sample(2, 0), [255, 255, 255, 255]);
        assert_eq!(t.sample(0, 2), [255, 255, 255, 255]);
    }

    #[test]
    fn camera_roundtrip_and_zoom() {
        let mut cam = Camera2D::new(800.0, 600.0);
        cam.zoom = 2.0;
        cam.position = Vec2::new(100.0, 50.0);
        let s = cam.world_to_screen(Vec2::new(150.0, 100.0));
        // (150-100)*2 + 400 = 500 ; (100-50)*2 + 300 = 400
        assert_eq!(s, Vec2::new(500.0, 400.0));
        let w = cam.screen_to_world(s);
        assert_eq!(w, Vec2::new(150.0, 100.0));
    }
}
