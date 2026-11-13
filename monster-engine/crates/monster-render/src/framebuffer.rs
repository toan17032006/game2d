//! CPU framebuffer: RGBA8 pixel buffer with blending primitives.

use monster_core::graphics::DrawOptions;
use monster_core::graphics::Texture;
use monster_math::{Color, Rect, Vec2};

/// Row-major RGBA8 framebuffer in screen space (origin top-left, +Y down).
#[derive(Debug, Clone)]
pub struct Framebuffer {
    width: u32,
    height: u32,
    pixels: Vec<u8>,
}

impl Framebuffer {
    /// Allocate a framebuffer filled with `clear_color`.
    pub fn new(width: u32, height: u32, clear_color: Color) -> Framebuffer {
        let mut fb = Framebuffer {
            width,
            height,
            pixels: Vec::new(),
        };
        fb.resize(width, height, clear_color);
        fb
    }

    /// Resize (and clear) the buffer; keeps contents uninitialized otherwise.
    pub fn resize(&mut self, width: u32, height: u32, clear_color: Color) {
        self.width = width;
        self.height = height;
        let [r, g, b, a] = clear_color.to_rgba_bytes();
        self.pixels.clear();
        self.pixels.extend(
            std::iter::repeat([r, g, b, a])
                .take((width * height) as usize)
                .flatten(),
        );
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

    /// Read the RGBA bytes at screen pixel `(x, y)`.
    pub fn get_pixel(&self, x: u32, y: u32) -> [u8; 4] {
        let i = self.index(x, y);
        [
            self.pixels[i],
            self.pixels[i + 1],
            self.pixels[i + 2],
            self.pixels[i + 3],
        ]
    }

    fn index(&self, x: u32, y: u32) -> usize {
        ((y as usize) * (self.width as usize) + x as usize) * 4
    }

    /// Blend one RGBA pixel into the buffer using the "over" operator.
    ///
    /// `a == 255` overwrites, `a == 0` is a no-op, anything between blends
    /// `src * a + dst * (1 - a)` per channel with rounding.
    pub fn blend_pixel(&mut self, x: u32, y: u32, rgba: [u8; 4]) {
        if x >= self.width || y >= self.height {
            return;
        }
        let a = rgba[3] as u32;
        if a == 0 {
            return;
        }
        let i = self.index(x, y);
        if a == 255 {
            self.pixels[i] = rgba[0];
            self.pixels[i + 1] = rgba[1];
            self.pixels[i + 2] = rgba[2];
            self.pixels[i + 3] = 255;
            return;
        }
        let inv = 255 - a;
        for c in 0..3 {
            let src = rgba[c] as u32;
            let dst = self.pixels[i + c] as u32;
            self.pixels[i + c] = ((src * a + dst * inv + 127) / 255) as u8;
        }
        // destination alpha accumulates towards opaque
        let dst_a = self.pixels[i + 3] as u32;
        self.pixels[i + 3] = ((a * 255 + (255 - a) * dst_a + 127) / 255) as u8;
    }

    /// Fill a screen-space rectangle with a solid color (blended, clipped).
    pub fn fill_rect(&mut self, rect: Rect, color: Color) {
        let min = rect.min();
        let max = rect.max();
        let x0 = min.x.max(0.0).ceil() as u32;
        let y0 = min.y.max(0.0).ceil() as u32;
        let x1 = (max.x.min(self.width as f32)).floor() as u32;
        let y1 = (max.y.min(self.height as f32)).floor() as u32;
        let bytes = color.to_rgba_bytes();
        for y in y0..y1.min(self.height) {
            for x in x0..x1.min(self.width) {
                self.blend_pixel(x, y, bytes);
            }
        }
    }

    /// Blit a (sub-)rectangle of `texture` into a screen-space destination
    /// rect with tint, flips, nearest sampling and per-pixel blending.
    /// Source and destination are clipped independently; nothing panics on
    /// out-of-bounds regions.
    pub fn blit_sprite(
        &mut self,
        texture: &Texture,
        dst: Rect,
        src: Option<Rect>,
        opts: DrawOptions,
    ) {
        let tex_w = texture.width() as f32;
        let tex_h = texture.height() as f32;
        let src_rect = src.unwrap_or(Rect::new(0.0, 0.0, tex_w, tex_h));
        if src_rect.width() <= 0.0
            || src_rect.height() <= 0.0
            || dst.width() <= 0.0
            || dst.height() <= 0.0
        {
            return;
        }

        let dst_min = dst.min();
        let dst_max = dst.max();
        let sx0 = dst_min.x.max(0.0);
        let sy0 = dst_min.y.max(0.0);
        let sx1 = dst_max.x.min(self.width as f32);
        let sy1 = dst_max.y.min(self.height as f32);
        if sx0 >= sx1 || sy0 >= sy1 {
            return;
        }

        let [tr, tg, tb, ta] = opts.tint.to_rgba_bytes();
        let tint_a = ta as f32 / 255.0;

        let dst_w = dst.width();
        let dst_h = dst.height();
        let mut y = sy0 as u32;
        while y < sy1.ceil() as u32 {
            let mut x = sx0 as u32;
            while x < sx1.ceil() as u32 {
                // pixel-center uv in destination space
                let u = ((x as f32 + 0.5) - dst_min.x) / dst_w;
                let v = ((y as f32 + 0.5) - dst_min.y) / dst_h;
                let u = if opts.flip_x { 1.0 - u } else { u };
                let v = if opts.flip_y { 1.0 - v } else { v };
                let tx = (src_rect.min().x + u * src_rect.width()).floor() as i64;
                let ty = (src_rect.min().y + v * src_rect.height()).floor() as i64;
                if tx < 0 || ty < 0 || tx >= tex_w as i64 || ty >= tex_h as i64 {
                    x += 1;
                    continue;
                }
                let [r, g, b, a] = texture.sample(tx as u32, ty as u32);
                let out_a = ((a as f32) * tint_a).round().clamp(0.0, 255.0) as u8;
                if out_a == 0 {
                    x += 1;
                    continue;
                }
                let out_r = ((r as u32 * tr as u32) / 255) as u8;
                let out_g = ((g as u32 * tg as u32) / 255) as u8;
                let out_b = ((b as u32 * tb as u32) / 255) as u8;
                self.blend_pixel(x, y, [out_r, out_g, out_b, out_a]);
                x += 1;
            }
            y += 1;
        }
    }

    /// Screen pixel a world point maps to (used by tests and the software
    /// renderer to keep camera math in one place).
    pub fn world_to_screen(camera: &monster_core::graphics::Camera2D, world: Vec2) -> Vec2 {
        camera.world_to_screen(world)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use monster_math::Color;

    #[test]
    fn clear_and_read_pixels() {
        let fb = Framebuffer::new(4, 3, Color::opaque(9, 8, 7));
        assert_eq!(fb.width(), 4);
        assert_eq!(fb.height(), 3);
        assert_eq!(fb.get_pixel(0, 0), [9, 8, 7, 255]);
        assert_eq!(fb.get_pixel(3, 2), [9, 8, 7, 255]);
    }

    #[test]
    fn blend_pixel_overwrite_and_skip() {
        let mut fb = Framebuffer::new(2, 2, Color::BLACK);
        fb.blend_pixel(0, 0, [255, 0, 0, 255]); // overwrite
        assert_eq!(fb.get_pixel(0, 0), [255, 0, 0, 255]);
        fb.blend_pixel(0, 0, [0, 255, 0, 0]); // fully transparent: no-op
        assert_eq!(fb.get_pixel(0, 0), [255, 0, 0, 255]);
        fb.blend_pixel(99, 99, [1, 2, 3, 255]); // out of bounds: no panic
    }

    #[test]
    fn blend_pixel_half() {
        let mut fb = Framebuffer::new(1, 1, Color::WHITE);
        fb.blend_pixel(0, 0, [255, 0, 0, 128]);
        let p = fb.get_pixel(0, 0);
        // 50% red over white: (255, ~127, ~127)
        assert_eq!(p[0], 255);
        assert!(p[1] >= 126 && p[1] <= 128);
        assert!(p[2] >= 126 && p[2] <= 128);
    }

    #[test]
    fn fill_rect_clips() {
        let mut fb = Framebuffer::new(4, 4, Color::BLACK);
        fb.fill_rect(Rect::new(-2.0, -2.0, 8.0, 3.0), Color::BLUE);
        for y in 0..1 {
            for x in 0..4 {
                assert_eq!(fb.get_pixel(x, y), [0, 0, 255, 255]);
            }
        }
        // below the rect stays black
        assert_eq!(fb.get_pixel(0, 2), [0, 0, 0, 255]);
    }

    #[test]
    fn blit_opaque_sprite_covers_destination() {
        let mut fb = Framebuffer::new(4, 4, Color::BLACK);
        let tex = Texture::solid(2, 2, Color::RED);
        fb.blit_sprite(
            &tex,
            Rect::new(1.0, 1.0, 2.0, 2.0),
            None,
            DrawOptions::default(),
        );
        assert_eq!(fb.get_pixel(1, 1), [255, 0, 0, 255]);
        assert_eq!(fb.get_pixel(2, 2), [255, 0, 0, 255]);
        assert_eq!(fb.get_pixel(0, 0), [0, 0, 0, 255]);
        assert_eq!(fb.get_pixel(3, 3), [0, 0, 0, 255]);
    }

    #[test]
    fn blit_src_subrect_selects_region() {
        // left half red, right half green
        let mut pixels = Vec::new();
        for y in 0..2 {
            for x in 0..2 {
                let c = if x == 0 { Color::RED } else { Color::GREEN };
                pixels.extend_from_slice(&c.to_rgba_bytes());
            }
        }
        let tex = Texture::from_rgba(2, 2, pixels);
        let mut fb = Framebuffer::new(2, 2, Color::BLACK);
        // sample only the right half of the texture
        fb.blit_sprite(
            &tex,
            Rect::new(0.0, 0.0, 1.0, 2.0),
            Some(Rect::new(1.0, 0.0, 1.0, 2.0)),
            DrawOptions::default(),
        );
        assert_eq!(fb.get_pixel(0, 0), [0, 255, 0, 255]);
        assert_eq!(fb.get_pixel(0, 1), [0, 255, 0, 255]);
    }

    #[test]
    fn blit_flip_mirrors() {
        let mut pixels = Vec::new();
        for y in 0..1 {
            for x in 0..2 {
                let c = if x == 0 { Color::RED } else { Color::GREEN };
                pixels.extend_from_slice(&c.to_rgba_bytes());
            }
        }
        let tex = Texture::from_rgba(2, 1, pixels);
        let mut fb = Framebuffer::new(2, 1, Color::BLACK);
        fb.blit_sprite(
            &tex,
            Rect::new(0.0, 0.0, 2.0, 1.0),
            None,
            DrawOptions {
                flip_x: true,
                ..DrawOptions::default()
            },
        );
        // mirrored: left pixel shows green, right shows red
        assert_eq!(fb.get_pixel(0, 0), [0, 255, 0, 255]);
        assert_eq!(fb.get_pixel(1, 0), [255, 0, 0, 255]);
    }

    #[test]
    fn blit_tint_multiplies() {
        let mut fb = Framebuffer::new(1, 1, Color::BLACK);
        let tex = Texture::solid(1, 1, Color::WHITE);
        fb.blit_sprite(
            &tex,
            Rect::new(0.0, 0.0, 1.0, 1.0),
            None,
            DrawOptions {
                tint: Color::new(128, 64, 32, 255),
                ..DrawOptions::default()
            },
        );
        assert_eq!(fb.get_pixel(0, 0), [128, 64, 32, 255]);
    }

    #[test]
    fn blit_alpha_blends_over_background() {
        let mut fb = Framebuffer::new(1, 1, Color::WHITE);
        let tex = Texture::solid(1, 1, Color::new(255, 0, 0, 128));
        fb.blit_sprite(
            &tex,
            Rect::new(0.0, 0.0, 1.0, 1.0),
            None,
            DrawOptions::default(),
        );
        let p = fb.get_pixel(0, 0);
        assert_eq!(p[0], 255);
        assert!(p[1] >= 126 && p[1] <= 128);
    }
}
