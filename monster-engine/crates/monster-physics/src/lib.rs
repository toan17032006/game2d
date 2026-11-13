//! Physics foundations for `monster-engine`: axis-aligned bounding boxes,
//! Euler integration and collision resolution.
//!
//! Geometry is generic over plain math types, so the physics crate has no
//! dependency on scene/render — the facade (or game code) glues components
//! to these functions. A 3D upgrade adds `Aabb3` alongside; the broadphase
//! and resolution structure stays the same.

use monster_math::Vec2;

/// Axis-aligned bounding box defined by min/max corners (+Y down).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Aabb2 {
    pub min: Vec2,
    pub max: Vec2,
}

impl Aabb2 {
    pub const EMPTY: Aabb2 = Aabb2 {
        min: Vec2::ZERO,
        max: Vec2::ZERO,
    };

    pub fn new(min: Vec2, max: Vec2) -> Aabb2 {
        debug_assert!(max.x >= min.x && max.y >= min.y, "Aabb2 min must be <= max");
        Aabb2 { min, max }
    }

    /// Box from a center point and half extents.
    pub fn from_center_half(center: Vec2, half: Vec2) -> Aabb2 {
        Aabb2 {
            min: center - half,
            max: center + half,
        }
    }

    pub fn center(&self) -> Vec2 {
        (self.min + self.max) * 0.5
    }

    pub fn half_extents(&self) -> Vec2 {
        (self.max - self.min) * 0.5
    }

    pub fn contains(&self, p: Vec2) -> bool {
        p.x >= self.min.x && p.x <= self.max.x && p.y >= self.min.y && p.y <= self.max.y
    }

    pub fn intersects(&self, other: &Aabb2) -> bool {
        self.min.x <= other.max.x
            && self.max.x >= other.min.x
            && self.min.y <= other.max.y
            && self.max.y >= other.min.y
    }

    /// Minimal translation vector (MTV) that moves `self` out of `other`.
    ///
    /// Returns `Some(offset)` where adding `offset` to self's position
    /// resolves the overlap with the smallest movement, or `None` when the
    /// boxes do not overlap.
    pub fn separation(&self, other: &Aabb2) -> Option<Vec2> {
        let dx_left = other.min.x - self.max.x; // negative while overlapping
        let dx_right = other.max.x - self.min.x; // positive while overlapping
        let dy_up = other.min.y - self.max.y;
        let dy_down = other.max.y - self.min.y;

        if dx_left > 0.0 || dx_right < 0.0 || dy_up > 0.0 || dy_down < 0.0 {
            return None; // separated
        }

        let overlap_right = dx_right; // distance to move self right
        let overlap_left = -dx_left; // distance to move self left
        let overlap_down = dy_down; // distance to move self down
        let overlap_up = -dy_up; // distance to move self up

        let mut best = (overlap_right, Vec2::RIGHT);
        if overlap_left < best.0 {
            best = (overlap_left, -Vec2::RIGHT);
        }
        if overlap_down < best.0 {
            best = (overlap_down, Vec2::DOWN);
        }
        if overlap_up < best.0 {
            best = (overlap_up, -Vec2::DOWN);
        }
        Some(best.1 * best.0)
    }

    /// Collision normal of the MTV axis (unit vector), if overlapping.
    pub fn collision_normal(&self, other: &Aabb2) -> Option<Vec2> {
        self.separation(other).map(|mtv| {
            if mtv.x.abs() >= mtv.y.abs() {
                Vec2::new(mtv.x.signum(), 0.0)
            } else {
                Vec2::new(0.0, mtv.y.signum())
            }
        })
    }
}

/// Euler integration step: `pos += vel * dt`.
pub fn integrate(pos: &mut Vec2, vel: Vec2, dt: f32) {
    *pos += vel * dt;
}

/// Apply gravity to a velocity: `vel += gravity * dt`.
pub fn apply_gravity(vel: &mut Vec2, gravity: Vec2, dt: f32) {
    *vel += gravity * dt;
}

/// Reflect velocity against a unit collision normal with restitution
/// (`0.0` = no bounce, `1.0` = perfect bounce).
pub fn bounce(vel: Vec2, normal: Vec2, restitution: f32) -> Vec2 {
    let n = normal.normalize();
    let along = vel.dot(n);
    if along < 0.0 {
        vel - n * (along * (1.0 + restitution))
    } else {
        // already separating: keep velocity
        vel
    }
}

/// A pair of colliding shapes, reported as indices into the input slice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CollisionPair {
    pub a: usize,
    pub b: usize,
}

/// Brute-force broadphase: all overlapping pairs among `boxes`.
///
/// O(n²) — intentionally simple for the foundation; a uniform grid slots in
/// behind the same return type when entity counts grow.
pub fn find_collisions(boxes: &[Aabb2]) -> Vec<CollisionPair> {
    let mut pairs = Vec::new();
    for i in 0..boxes.len() {
        for j in (i + 1)..boxes.len() {
            if boxes[i].intersects(&boxes[j]) {
                pairs.push(CollisionPair { a: i, b: j });
            }
        }
    }
    pairs
}

#[cfg(test)]
mod tests {
    use super::*;
    use monster_math::approx_eq;

    #[test]
    fn intersects_variants() {
        // a spans (-5,-5)-(5,5)
        let a = Aabb2::from_center_half(Vec2::new(0.0, 0.0), Vec2::new(5.0, 5.0));
        // b spans (4,-2)-(8,2): overlaps a on x in [4,5]
        let b = Aabb2::from_center_half(Vec2::new(6.0, 0.0), Vec2::new(2.0, 2.0));
        assert!(a.intersects(&b));
        let far = Aabb2::from_center_half(Vec2::new(50.0, 50.0), Vec2::new(1.0, 1.0));
        assert!(!a.intersects(&far));
        // touching box spans (5,-2)-(9,2): edge exactly at x=5
        let touching = Aabb2::from_center_half(Vec2::new(7.0, 0.0), Vec2::new(2.0, 2.0));
        assert!(a.intersects(&touching)); // edge touch counts
    }

    #[test]
    fn separation_picks_smallest_axis() {
        // a spans (-10,-2)-(10,2); b spans (-10,1)-(10,5): y overlap is 1,
        // x overlap is 20 -> smallest move is 1 unit up (screen-down space).
        let a = Aabb2::from_center_half(Vec2::new(0.0, 0.0), Vec2::new(10.0, 2.0));
        let b = Aabb2::from_center_half(Vec2::new(0.0, 3.0), Vec2::new(10.0, 2.0));
        let mtv = a.separation(&b).unwrap();
        assert!(approx_eq(mtv.y, -1.0, 1e-5));
        assert!(approx_eq(mtv.x, 0.0, 1e-5));
        // applying it (with a hair of slack past the touching edge) resolves
        let moved =
            Aabb2::from_center_half(Vec2::new(0.0, 0.0) + mtv * 1.001, Vec2::new(10.0, 2.0));
        assert!(!moved.intersects(&b));
    }

    #[test]
    fn separation_none_when_apart() {
        let a = Aabb2::from_center_half(Vec2::new(0.0, 0.0), Vec2::new(2.0, 2.0));
        let b = Aabb2::from_center_half(Vec2::new(10.0, 10.0), Vec2::new(2.0, 2.0));
        assert!(a.separation(&b).is_none());
        assert!(a.collision_normal(&b).is_none());
    }

    #[test]
    fn collision_normal_is_unit_axis() {
        // b sits to the right of a: minimal separation pushes a LEFT.
        let a = Aabb2::from_center_half(Vec2::new(0.0, 0.0), Vec2::new(2.0, 2.0));
        let b = Aabb2::from_center_half(Vec2::new(3.0, 0.0), Vec2::new(2.0, 2.0));
        let n = a.collision_normal(&b).unwrap();
        assert!(approx_eq(n.length(), 1.0, 1e-5));
        assert!(approx_eq(n.x, -1.0, 1e-5)); // away from b
                                             // velocity moving right (+x) against normal (-1,0) bounces to left
        let bounced = bounce(Vec2::new(3.0, 0.0), n, 1.0);
        assert!(approx_eq(bounced.x, -3.0, 1e-5));
    }

    #[test]
    fn integrate_and_gravity() {
        let mut pos = Vec2::new(0.0, 0.0);
        integrate(&mut pos, Vec2::new(10.0, 0.0), 0.5);
        assert!(approx_eq(pos.x, 5.0, 1e-5));
        let mut vel = Vec2::new(0.0, 0.0);
        apply_gravity(&mut vel, Vec2::new(0.0, 100.0), 0.25);
        assert!(approx_eq(vel.y, 25.0, 1e-5));
    }

    #[test]
    fn bounce_reflects_and_restitution() {
        let n = Vec2::new(0.0, -1.0); // floor normal pointing up (screen-down space)
        let v = Vec2::new(3.0, 10.0); // falling (moving +Y)
        let bounced = bounce(v, n, 1.0);
        assert!(approx_eq(bounced.x, 3.0, 1e-5));
        assert!(approx_eq(bounced.y, -10.0, 1e-5));
        let damped = bounce(v, n, 0.5);
        assert!(approx_eq(damped.y, -5.0, 1e-4));
        // separating velocity passes through unchanged
        let up = Vec2::new(3.0, -10.0);
        assert_eq!(bounce(up, n, 1.0), up);
    }

    #[test]
    fn broadphase_reports_pairs() {
        let boxes = vec![
            Aabb2::from_center_half(Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0)),
            Aabb2::from_center_half(Vec2::new(1.5, 0.0), Vec2::new(1.0, 1.0)), // overlaps 0
            Aabb2::from_center_half(Vec2::new(50.0, 50.0), Vec2::new(1.0, 1.0)),
        ];
        let pairs = find_collisions(&boxes);
        assert_eq!(pairs.len(), 1);
        assert_eq!(pairs[0], CollisionPair { a: 0, b: 1 });
    }
}
