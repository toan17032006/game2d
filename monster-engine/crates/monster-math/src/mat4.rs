//! Column-major 4x4 matrix, matching GPU memory conventions.

use crate::vec::{Vec2, Vec3, Vec4};

/// 4x4 column-major matrix.
///
/// `cols[c]` is column `c`; element `(r, c)` is `cols[c].at(r)`.
/// Stored column-major from day one so a future GPU backend can upload
/// it without transposing.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Mat4 {
    pub cols: [Vec4; 4],
}

impl Vec4 {
    /// Access component by index (0..=3).
    #[inline]
    pub fn at(self, i: usize) -> f32 {
        match i {
            0 => self.x,
            1 => self.y,
            2 => self.z,
            3 => self.w,
            _ => 0.0,
        }
    }
}

impl Mat4 {
    pub const IDENTITY: Mat4 = Mat4 {
        cols: [
            Vec4::new(1.0, 0.0, 0.0, 0.0),
            Vec4::new(0.0, 1.0, 0.0, 0.0),
            Vec4::new(0.0, 0.0, 1.0, 0.0),
            Vec4::new(0.0, 0.0, 0.0, 1.0),
        ],
    };

    #[inline]
    pub const fn identity() -> Mat4 {
        Mat4::IDENTITY
    }

    /// Element at row `r`, column `c`.
    #[inline]
    pub fn at(&self, r: usize, c: usize) -> f32 {
        self.cols[c].at(r)
    }

    /// Matrix-matrix product `self * other` (applies `other` first when
    /// used with column vectors).
    #[inline]
    pub fn mul_mat4(&self, other: &Mat4) -> Mat4 {
        let mut out = [Vec4::ZERO; 4];
        for (c, out_c) in out.iter_mut().enumerate() {
            *out_c = Vec4::ZERO;
            for k in 0..4 {
                *out_c = *out_c + self.cols[k] * other.at(k, c);
            }
        }
        Mat4 { cols: out }
    }

    /// Transform a 4D (homogeneous) vector.
    #[inline]
    pub fn mul_vec4(&self, v: Vec4) -> Vec4 {
        Vec4::new(
            self.cols[0].x * v.x
                + self.cols[1].x * v.y
                + self.cols[2].x * v.z
                + self.cols[3].x * v.w,
            self.cols[0].y * v.x
                + self.cols[1].y * v.y
                + self.cols[2].y * v.z
                + self.cols[3].y * v.w,
            self.cols[0].z * v.x
                + self.cols[1].z * v.y
                + self.cols[2].z * v.z
                + self.cols[3].z * v.w,
            self.cols[0].w * v.x
                + self.cols[1].w * v.y
                + self.cols[2].w * v.z
                + self.cols[3].w * v.w,
        )
    }

    /// Translation matrix.
    pub fn translate(pos: Vec3) -> Mat4 {
        let mut m = Mat4::identity();
        m.cols[3] = Vec4::new(pos.x, pos.y, pos.z, 1.0);
        m
    }

    /// Uniform-ish scaling matrix.
    pub fn scale(s: Vec3) -> Mat4 {
        let mut m = Mat4::identity();
        m.cols[0].x = s.x;
        m.cols[1].y = s.y;
        m.cols[2].z = s.z;
        m
    }

    /// Rotation about the Z axis (radians) — the 2D rotation.
    pub fn rotate_z(radians: f32) -> Mat4 {
        let (s, c) = radians.sin_cos();
        let mut m = Mat4::identity();
        m.cols[0] = Vec4::new(c, s, 0.0, 0.0);
        m.cols[1] = Vec4::new(-s, c, 0.0, 0.0);
        m
    }

    /// Rotation from a quaternion.
    pub fn from_quat(q: &crate::Quat) -> Mat4 {
        let (x, y, z, w) = (q.x, q.y, q.z, q.w);
        let xx = x * x;
        let yy = y * y;
        let zz = z * z;
        let xy = x * y;
        let xz = x * z;
        let yz = y * z;
        let wx = w * x;
        let wy = w * y;
        let wz = w * z;
        let mut m = Mat4::identity();
        m.cols[0] = Vec4::new(1.0 - 2.0 * (yy + zz), 2.0 * (xy + wz), 2.0 * (xz - wy), 0.0);
        m.cols[1] = Vec4::new(2.0 * (xy - wz), 1.0 - 2.0 * (xx + zz), 2.0 * (yz + wx), 0.0);
        m.cols[2] = Vec4::new(2.0 * (xz + wy), 2.0 * (yz - wx), 1.0 - 2.0 * (xx + yy), 0.0);
        m
    }

    /// Left-handed orthographic projection used by the 2D camera.
    ///
    /// Maps the world rectangle `[left, right] x [bottom, top]` to
    /// `[-1, 1] x [-1, 1]` clip space (standard orientation: `bottom`
    /// maps to clip `-1`, `top` maps to clip `+1`). The engine's world
    /// space uses +Y pointing down, so world Y grows towards the bottom
    /// of the screen.
    pub fn orthographic(left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) -> Mat4 {
        let mut m = Mat4::identity();
        let inv_w = 1.0 / (right - left);
        let inv_h = 1.0 / (top - bottom);
        let inv_d = 1.0 / (far - near);
        m.cols[0] = Vec4::new(2.0 * inv_w, 0.0, 0.0, 0.0);
        m.cols[1] = Vec4::new(0.0, 2.0 * inv_h, 0.0, 0.0);
        m.cols[2] = Vec4::new(0.0, 0.0, inv_d, 0.0);
        m.cols[3] = Vec4::new(
            -(left + right) * inv_w,
            -(top + bottom) * inv_h,
            -near * inv_d,
            1.0,
        );
        m
    }

    /// Left-handed perspective projection (for the future 3D backend).
    pub fn perspective(fov_y_radians: f32, aspect: f32, near: f32, far: f32) -> Mat4 {
        let f = 1.0 / fov_y_radians.tan();
        let inv_d = 1.0 / (far - near);
        let mut m = Mat4::identity();
        m.cols[0] = Vec4::new(f / aspect, 0.0, 0.0, 0.0);
        m.cols[1] = Vec4::new(0.0, f, 0.0, 0.0);
        m.cols[2] = Vec4::new(0.0, 0.0, far * inv_d, 1.0);
        m.cols[3] = Vec4::new(0.0, 0.0, -near * far * inv_d, 0.0);
        m
    }

    /// Right-handed look-at view matrix (OpenGL convention: camera looks
    /// down its -Z axis; world +Y is up for 3D).
    pub fn look_at(eye: Vec3, center: Vec3, up: Vec3) -> Mat4 {
        let f = (center - eye).normalize(); // forward
        let up_n = up.normalize();
        let s = f.cross(up_n).normalize(); // right
        let u = s.cross(f); // camera up
        let mut m = Mat4::identity();
        m.cols[0] = Vec4::new(s.x, u.x, -f.x, 0.0);
        m.cols[1] = Vec4::new(s.y, u.y, -f.y, 0.0);
        m.cols[2] = Vec4::new(s.z, u.z, -f.z, 0.0);
        m.cols[3] = Vec4::new(-s.dot(eye), -u.dot(eye), f.dot(eye), 1.0);
        m
    }

    /// Camera view matrix for 2D: maps world space to clip space so that a
    /// world point at `position` lands on the viewport center (clip 0,0).
    ///
    /// Chain (rightmost applied first): translate the world by the negative
    /// camera position, rotate by the negative camera rotation, scale by
    /// `scale` (zoom), then project through a viewport-centered ortho box.
    /// Matches `Camera2D::world_to_screen` exactly.
    pub fn view_2d(position: Vec2, rotation: f32, zoom: f32, viewport: (f32, f32)) -> Mat4 {
        let (w, h) = viewport;
        let ortho = Mat4::orthographic(-w * 0.5, w * 0.5, -h * 0.5, h * 0.5, -1.0, 1.0);
        let t = Mat4::translate(Vec3::new(-position.x, -position.y, 0.0));
        let r = Mat4::rotate_z(-rotation);
        let s = Mat4::scale(Vec3::new(zoom, zoom, 1.0));
        ortho.mul_mat4(&s).mul_mat4(&r).mul_mat4(&t)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const EPS: f32 = 1e-5;

    #[test]
    fn identity_and_associativity() {
        let a = Mat4::translate(Vec3::new(1.0, 2.0, 3.0));
        let i = Mat4::identity();
        assert_eq!(i.mul_mat4(&a), a);
        assert_eq!(a.mul_mat4(&i), a);
        let v = Vec4::new(1.0, 1.0, 1.0, 1.0);
        let r = i.mul_mat4(&a).mul_vec4(v);
        assert!(crate::approx_eq(r.x, 2.0, EPS));
        assert!(crate::approx_eq(r.y, 3.0, EPS));
        assert!(crate::approx_eq(r.z, 4.0, EPS));
    }

    #[test]
    fn orthographic_maps_corners() {
        let m = Mat4::orthographic(0.0, 100.0, 0.0, 50.0, -1.0, 1.0);
        let c = m.mul_vec4(Vec4::new(0.0, 0.0, 0.0, 1.0));
        assert!(crate::approx_eq(c.x, -1.0, EPS));
        assert!(crate::approx_eq(c.y, -1.0, EPS)); // world bottom maps to clip -1
        let c2 = m.mul_vec4(Vec4::new(100.0, 50.0, 0.0, 1.0));
        assert!(crate::approx_eq(c2.x, 1.0, EPS));
        assert!(crate::approx_eq(c2.y, 1.0, EPS));
    }

    #[test]
    fn rotate_z_90_moves_x_to_y() {
        let m = Mat4::rotate_z(std::f32::consts::FRAC_PI_2);
        let v = m.mul_vec4(Vec4::new(1.0, 0.0, 0.0, 1.0));
        assert!(crate::approx_eq(v.x, 0.0, EPS));
        assert!(crate::approx_eq(v.y, 1.0, EPS));
    }

    #[test]
    fn from_quat_matches_rotate_z() {
        let q = crate::Quat::from_axis_angle(Vec3::new(0.0, 0.0, 1.0), std::f32::consts::FRAC_PI_2);
        let m = Mat4::from_quat(&q);
        let v = m.mul_vec4(Vec4::new(1.0, 0.0, 0.0, 1.0));
        assert!(crate::approx_eq(v.x, 0.0, EPS));
        assert!(crate::approx_eq(v.y, 1.0, EPS));
    }

    #[test]
    fn look_at_maps_camera_axis_to_view_axis() {
        // camera at origin looking down -Z (standard GL pose), up +Y:
        // a point straight ahead stays on the -Z view axis.
        let eye = Vec3::new(0.0, 0.0, 0.0);
        let center = Vec3::new(0.0, 0.0, -10.0);
        let m = Mat4::look_at(eye, center, Vec3::new(0.0, 1.0, 0.0));
        let ahead = m.mul_vec4(Vec4::new(0.0, 0.0, -5.0, 1.0));
        assert!(crate::approx_eq(ahead.x, 0.0, EPS));
        assert!(crate::approx_eq(ahead.y, 0.0, EPS));
        assert!(crate::approx_eq(ahead.z, -5.0, EPS));

        // a point to the camera's right (+X world when facing -Z) is +X in view
        let right = m.mul_vec4(Vec4::new(3.0, 0.0, -5.0, 1.0));
        assert!(crate::approx_eq(right.x, 3.0, EPS));

        // camera offset: world point at the eye maps to view origin
        let m2 = Mat4::look_at(
            Vec3::new(5.0, 3.0, 8.0),
            Vec3::new(5.0, 3.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
        );
        let at_eye = m2.mul_vec4(Vec4::new(5.0, 3.0, 8.0, 1.0));
        assert!(crate::approx_eq(at_eye.x, 0.0, EPS));
        assert!(crate::approx_eq(at_eye.y, 0.0, EPS));
        assert!(crate::approx_eq(at_eye.z, 0.0, EPS));
    }

    #[test]
    fn perspective_points_into_screen() {
        let m = Mat4::perspective(std::f32::consts::FRAC_PI_2, 1.0, 0.1, 100.0);
        let near_clip = m.mul_vec4(Vec4::new(0.0, 0.0, 0.1, 1.0));
        let far_clip = m.mul_vec4(Vec4::new(0.0, 0.0, 100.0, 1.0));
        // NDC z = clip.z / clip.w (perspective divide happens outside).
        assert!(crate::approx_eq(near_clip.z / near_clip.w, 0.0, EPS));
        assert!(crate::approx_eq(far_clip.z / far_clip.w, 1.0, EPS));
    }

    #[test]
    fn view_2d_centers_camera_position() {
        // Camera at (10, 5) over a 100x100 viewport: world (10, 5) lands on
        // the viewport center => clip space (0, 0).
        let m = Mat4::view_2d(Vec2::new(10.0, 5.0), 0.0, 1.0, (100.0, 100.0));
        let c = m.mul_vec4(Vec4::new(10.0, 5.0, 0.0, 1.0));
        assert!(crate::approx_eq(c.x, 0.0, EPS));
        assert!(crate::approx_eq(c.y, 0.0, EPS));
        // Zoom 2 doubles screen offsets: 50 world units right of the camera
        // becomes 100 screen pixels => clip x = 2*100/100 - 0 = 2.
        let m2 = Mat4::view_2d(Vec2::new(10.0, 5.0), 0.0, 2.0, (100.0, 100.0));
        let c2 = m2.mul_vec4(Vec4::new(60.0, 5.0, 0.0, 1.0));
        assert!(crate::approx_eq(c2.x, 2.0, EPS));
        assert!(crate::approx_eq(c2.y, 0.0, EPS));
    }
}
