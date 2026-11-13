//! Math foundations for `monster-engine`.
//!
//! Contains the linear-algebra surface needed by every other subsystem.
//! 2D is the primary use case today, but [`Vec3`], [`Vec4`], [`Mat4`] and
//! [`Quat`] are provided from day one so a future 3D backend does not need
//! to change this API. [`Mat4`] is column-major, matching GPU conventions.

pub mod color;
pub mod mat4;
pub mod quat;
pub mod rect;
pub mod vec;

pub use color::Color;
pub use mat4::Mat4;
pub use quat::Quat;
pub use rect::Rect;
pub use vec::{Vec2, Vec3, Vec4};

/// Linear interpolation for scalar values.
#[inline]
pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// Clamp `v` into the inclusive range `[min, max]`.
#[inline]
pub fn clamp(v: f32, min: f32, max: f32) -> f32 {
    if v < min {
        min
    } else if v > max {
        max
    } else {
        v
    }
}

/// Compare two floats with an absolute epsilon tolerance.
#[inline]
pub fn approx_eq(a: f32, b: f32, eps: f32) -> bool {
    (a - b).abs() <= eps
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scalar_helpers() {
        assert!(approx_eq(lerp(1.0, 3.0, 0.5), 2.0, 1e-6));
        assert_eq!(clamp(2.0, 0.0, 1.0), 1.0);
        assert_eq!(clamp(-1.0, 0.0, 1.0), 0.0);
        assert_eq!(clamp(0.5, 0.0, 1.0), 0.5);
    }
}
