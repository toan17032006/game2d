//! RGBA color stored as 8-bit channels.

/// 8-bit RGBA color.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const WHITE: Color = Color {
        r: 255,
        g: 255,
        b: 255,
        a: 255,
    };
    pub const BLACK: Color = Color {
        r: 0,
        g: 0,
        b: 0,
        a: 255,
    };
    pub const RED: Color = Color {
        r: 255,
        g: 0,
        b: 0,
        a: 255,
    };
    pub const GREEN: Color = Color {
        r: 0,
        g: 255,
        b: 0,
        a: 255,
    };
    pub const BLUE: Color = Color {
        r: 0,
        g: 0,
        b: 255,
        a: 255,
    };
    pub const YELLOW: Color = Color {
        r: 255,
        g: 255,
        b: 0,
        a: 255,
    };
    pub const TRANSPARENT: Color = Color {
        r: 0,
        g: 0,
        b: 0,
        a: 0,
    };

    #[inline]
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Color {
        Color { r, g, b, a }
    }

    #[inline]
    pub const fn opaque(r: u8, g: u8, b: u8) -> Color {
        Color { r, g, b, a: 255 }
    }

    /// Channels as `[r, g, b, a]` bytes — the engine's texture layout.
    #[inline]
    pub const fn to_rgba_bytes(&self) -> [u8; 4] {
        [self.r, self.g, self.b, self.a]
    }

    /// Channels normalized to `0.0..=1.0` floats.
    #[inline]
    pub fn to_f32_tuple(&self) -> (f32, f32, f32, f32) {
        (
            self.r as f32 / 255.0,
            self.g as f32 / 255.0,
            self.b as f32 / 255.0,
            self.a as f32 / 255.0,
        )
    }

    /// Componentwise linear interpolation (useful for fades).
    pub fn lerp(self, other: Color, t: f32) -> Color {
        let lerp_u8 = |a: u8, b: u8| -> u8 {
            let fa = a as f32;
            let fb = b as f32;
            crate::lerp(fa, fb, t).round().clamp(0.0, 255.0) as u8
        };
        Color::new(
            lerp_u8(self.r, other.r),
            lerp_u8(self.g, other.g),
            lerp_u8(self.b, other.b),
            lerp_u8(self.a, other.a),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::approx_eq;

    #[test]
    fn constants_and_conversions() {
        assert_eq!(Color::WHITE.to_rgba_bytes(), [255, 255, 255, 255]);
        let (r, g, b, a) = Color::new(255, 0, 128, 64).to_f32_tuple();
        assert!(approx_eq(r, 1.0, 1e-6));
        assert!(approx_eq(g, 0.0, 1e-6));
        assert!(approx_eq(b, 128.0 / 255.0, 1e-6));
        assert!(approx_eq(a, 64.0 / 255.0, 1e-6));
    }

    #[test]
    fn lerp_endpoints_and_midpoint() {
        let a = Color::BLACK;
        let b = Color::WHITE;
        assert_eq!(a.lerp(b, 0.0), Color::BLACK);
        assert_eq!(a.lerp(b, 1.0), Color::WHITE);
        let mid = a.lerp(b, 0.5);
        assert_eq!(mid.r, 128);
        assert_eq!(mid.a, 255);
    }
}
