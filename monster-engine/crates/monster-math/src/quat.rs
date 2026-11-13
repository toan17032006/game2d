//! Quaternion — rotation without gimbal lock, ready for the 3D backend.

use crate::vec::Vec3;

/// Unit quaternion `(x, y, z, w)`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Quat {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Quat {
    pub const IDENTITY: Quat = Quat {
        x: 0.0,
        y: 0.0,
        z: 0.0,
        w: 1.0,
    };

    #[inline]
    pub const fn identity() -> Quat {
        Quat::IDENTITY
    }

    /// Rotation of `radians` around a (normalized internally) axis.
    pub fn from_axis_angle(axis: Vec3, radians: f32) -> Quat {
        let len = axis.length();
        if len <= f32::EPSILON {
            return Quat::identity();
        }
        let (s, c) = (radians * 0.5).sin_cos();
        let n = axis / len;
        Quat {
            x: n.x * s,
            y: n.y * s,
            z: n.z * s,
            w: c,
        }
    }

    /// Quaternion product: applying `self` then `other` (for unit quats).
    pub fn mul(&self, other: &Quat) -> Quat {
        Quat {
            w: self.w * other.w - self.x * other.x - self.y * other.y - self.z * other.z,
            x: self.w * other.x + self.x * other.w + self.y * other.z - self.z * other.y,
            y: self.w * other.y - self.x * other.z + self.y * other.w + self.z * other.x,
            z: self.w * other.z + self.x * other.y - self.y * other.x + self.z * other.w,
        }
    }

    pub fn conjugate(&self) -> Quat {
        Quat {
            x: -self.x,
            y: -self.y,
            z: -self.z,
            w: self.w,
        }
    }

    pub fn norm(&self) -> f32 {
        (self.x * self.x + self.y * self.y + self.z * self.z + self.w * self.w).sqrt()
    }

    /// Normalize to unit length; identity fallback for near-zero quats.
    pub fn normalize(&self) -> Quat {
        let n = self.norm();
        if n <= f32::EPSILON {
            return Quat::identity();
        }
        Quat {
            x: self.x / n,
            y: self.y / n,
            z: self.z / n,
            w: self.w / n,
        }
    }

    /// Rotate a 3D vector: `q * v * conj(q)` via the optimized form.
    pub fn rotate_vec3(&self, v: Vec3) -> Vec3 {
        let qv = Vec3::new(self.x, self.y, self.z);
        let t = qv.cross(v) * 2.0;
        v + qv.cross(t) + t * self.w
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::approx_eq;

    #[test]
    fn identity_rotates_nothing() {
        let q = Quat::identity();
        let v = q.rotate_vec3(Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(v, Vec3::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn z_axis_90_maps_x_to_y() {
        let q = Quat::from_axis_angle(Vec3::new(0.0, 0.0, 1.0), std::f32::consts::FRAC_PI_2);
        let v = q.rotate_vec3(Vec3::new(1.0, 0.0, 0.0));
        assert!(approx_eq(v.x, 0.0, 1e-5));
        assert!(approx_eq(v.y, 1.0, 1e-5));
        assert!(approx_eq(v.z, 0.0, 1e-5));
    }

    #[test]
    fn composition_order() {
        let qz = Quat::from_axis_angle(Vec3::new(0.0, 0.0, 1.0), std::f32::consts::FRAC_PI_2);
        let qy = Quat::from_axis_angle(Vec3::new(0.0, 1.0, 0.0), std::f32::consts::FRAC_PI_2);
        // `qy.mul(&qz)` applies qz first: +X -> +Y, and the Y rotation
        // leaves +Y fixed. Result: +Y.
        let v = qy.mul(&qz).rotate_vec3(Vec3::new(1.0, 0.0, 0.0));
        assert!(approx_eq(v.x, 0.0, 1e-5));
        assert!(approx_eq(v.y, 1.0, 1e-5));
        assert!(approx_eq(v.z, 0.0, 1e-5));
        // Reversed order applies qy first: +X -> -Z, and the Z rotation
        // leaves -Z fixed. Result: -Z.
        let v2 = qz.mul(&qy).rotate_vec3(Vec3::new(1.0, 0.0, 0.0));
        assert!(approx_eq(v2.x, 0.0, 1e-5));
        assert!(approx_eq(v2.y, 0.0, 1e-5));
        assert!(approx_eq(v2.z, -1.0, 1e-5));
    }

    #[test]
    fn normalize_unnormalized() {
        let q = Quat::from_axis_angle(Vec3::new(0.0, 0.0, 1.0), 1.234);
        let scaled = Quat {
            x: q.x * 3.0,
            y: q.y * 3.0,
            z: q.z * 3.0,
            w: q.w * 3.0,
        };
        let n = scaled.normalize();
        assert!(approx_eq(n.norm(), 1.0, 1e-5));
        let v1 = q.rotate_vec3(Vec3::new(0.0, 1.0, 0.0));
        let v2 = n.rotate_vec3(Vec3::new(0.0, 1.0, 0.0));
        assert!(
            approx_eq(v1.x, v2.x, 1e-4)
                && approx_eq(v1.y, v2.y, 1e-4)
                && approx_eq(v1.z, v2.z, 1e-4)
        );
    }
}
