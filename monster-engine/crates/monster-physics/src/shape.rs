//! Collider shapes: the 15-type collider taxonomy of the engine, mapped to
//! 2D equivalents (ADR-001 is 2D-first; true 3D shapes arrive with M3).
//!
//! Mapping (user taxonomy → 2D implementation):
//! - Box → OBB (rotated rectangle)
//! - Sphere → Circle
//! - Capsule / Cylinder → Stadium (segment + radius)
//! - Cone → Cone2 (convex triangle)
//! - Plane → half-space
//! - Convex Mesh → ConvexPolygon (SAT)
//! - Mesh / Concave Mesh → ConcavePolygon (collection of convex parts)
//! - Heightmap → Heightfield (smooth segments)
//! - Terrain → Terrain (blocky height columns)
//! - Compound → Compound (child shapes with local transforms)
//! - Wheel → Wheel (circle + suspension parameters)
//! - Raycast → `Ray2` queries (see `raycast.rs`)
//! - Custom → `CustomShape2` trait object

use crate::Aabb2;
use monster_math::Vec2;

/// Rigid transform of a body (position + rotation, +Y down screen space).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform2 {
    pub position: Vec2,
    pub rotation: f32,
}

impl Transform2 {
    pub const IDENTITY: Transform2 = Transform2 {
        position: Vec2::ZERO,
        rotation: 0.0,
    };

    pub fn new(position: Vec2, rotation: f32) -> Transform2 {
        Transform2 { position, rotation }
    }

    /// Local point → world point.
    pub fn to_world(&self, local: Vec2) -> Vec2 {
        let (s, c) = self.rotation.sin_cos();
        Vec2::new(
            self.position.x + local.x * c - local.y * s,
            self.position.y + local.x * s + local.y * c,
        )
    }

    /// World point → local point.
    pub fn to_local(&self, world: Vec2) -> Vec2 {
        let d = world - self.position;
        let (s, c) = self.rotation.sin_cos();
        Vec2::new(d.x * c + d.y * s, -d.x * s + d.y * c)
    }

    /// Local direction → world direction (rotation only).
    pub fn dir_to_world(&self, dir: Vec2) -> Vec2 {
        let (s, c) = self.rotation.sin_cos();
        Vec2::new(dir.x * c - dir.y * s, dir.x * s + dir.y * c)
    }

    /// World direction → local direction.
    pub fn dir_to_local(&self, dir: Vec2) -> Vec2 {
        let (s, c) = self.rotation.sin_cos();
        Vec2::new(dir.x * c + dir.y * s, -dir.x * s + dir.y * c)
    }
}

impl Default for Transform2 {
    fn default() -> Self {
        Transform2::IDENTITY
    }
}

/// A contact between two shapes. `normal` points **from A to B**.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Contact2 {
    pub normal: Vec2,
    pub penetration: f32,
    /// Approximate contact point in world space.
    pub point: Vec2,
}

/// Custom collider: the user supplies narrowphase for their own shape.
pub trait CustomShape2 {
    /// Narrowphase against any other shape. `normal` must point from
    /// `self` (at `xf`) towards `other` (at `other_xf`).
    fn collide_with(
        &self,
        xf: &Transform2,
        other: &ColliderShape2,
        other_xf: &Transform2,
    ) -> Option<Contact2>;

    fn contains_point(&self, xf: &Transform2, p: Vec2) -> bool;

    fn bound(&self, xf: &Transform2) -> Aabb2;

    fn raycast(&self, xf: &Transform2, origin: Vec2, dir: Vec2, max_t: f32) -> Option<f32>;

    fn name(&self) -> &'static str;
}

/// The 15-type collider taxonomy (2D mapping — see module docs).
pub enum ColliderShape2 {
    /// Sphere (2D: circle).
    Circle { radius: f32 },
    /// Box — rotated rectangle (OBB) via the body transform.
    Box { half_extents: Vec2 },
    /// Capsule / Cylinder (2D side view): segment along local X from
    /// `-half_length` to `+half_length`, swept by `radius`.
    Stadium { half_length: f32, radius: f32 },
    /// Cone (2D: convex triangle; apex points up = local -Y). Built via
    /// [`ColliderShape2::cone2`]; stored as a ConvexPolygon.
    Cone2 { base_half_width: f32, height: f32 },
    /// Plane — half-space; the solid side is **against** `normal`
    /// (normal points into the free space). `normal` is in local space.
    Plane { normal: Vec2 },
    /// Convex Mesh (2D: convex polygon, convexity validated).
    ConvexPolygon { points: Vec<Vec2> },
    /// Mesh / Concave Mesh (2D: collection of convex parts in local space).
    ConcavePolygon { convex_parts: Vec<Vec<Vec2>> },
    /// Heightmap (2D: smooth ground profile along local X; solid below).
    Heightfield {
        segment_length: f32,
        heights: Vec<f32>,
    },
    /// Terrain (2D: blocky height columns along local X — stepped ground).
    Terrain { cell_size: f32, heights: Vec<f32> },
    /// Compound — children with local offset/rotation.
    Compound { children: Vec<CompoundChild> },
    /// Wheel — behaves as a circle for collision, carries suspension
    /// parameters for vehicle bodies.
    Wheel {
        radius: f32,
        suspension_length: f32,
        spring_stiffness: f32,
        spring_damping: f32,
    },
    /// Custom — user-supplied narrowphase.
    Custom(Box<dyn CustomShape2>),
}

/// Child of a compound collider.
pub struct CompoundChild {
    pub offset: Vec2,
    pub rotation: f32,
    pub shape: ColliderShape2,
}

impl std::fmt::Debug for ColliderShape2 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ColliderShape2::Circle { radius } => write!(f, "Circle({radius})"),
            ColliderShape2::Box { half_extents } => write!(f, "Box({half_extents:?})"),
            ColliderShape2::Stadium {
                half_length,
                radius,
            } => {
                write!(f, "Stadium({half_length}, {radius})")
            }
            ColliderShape2::Cone2 {
                base_half_width,
                height,
            } => {
                write!(f, "Cone2({base_half_width}, {height})")
            }
            ColliderShape2::Plane { normal } => write!(f, "Plane({normal:?})"),
            ColliderShape2::ConvexPolygon { points } => {
                write!(f, "ConvexPolygon({} pts)", points.len())
            }
            ColliderShape2::ConcavePolygon { convex_parts } => {
                write!(f, "ConcavePolygon({} parts)", convex_parts.len())
            }
            ColliderShape2::Heightfield { heights, .. } => {
                write!(f, "Heightfield({} samples)", heights.len())
            }
            ColliderShape2::Terrain { heights, .. } => {
                write!(f, "Terrain({} columns)", heights.len())
            }
            ColliderShape2::Compound { children } => write!(f, "Compound({})", children.len()),
            ColliderShape2::Wheel { radius, .. } => write!(f, "Wheel({radius})"),
            ColliderShape2::Custom(c) => write!(f, "Custom({})", c.name()),
        }
    }
}

impl ColliderShape2 {
    /// Validate + build a convex polygon. Points may use either winding;
    /// the constructor validates convexity (SAT-safe) and keeps the order.
    pub fn convex_polygon(points: Vec<Vec2>) -> Result<ColliderShape2, String> {
        if points.len() < 3 {
            return Err("convex polygon needs >= 3 points".into());
        }
        let n = points.len();
        let mut cross_sign = 0i8;
        for i in 0..n {
            let a = points[i];
            let b = points[(i + 1) % n];
            let c = points[(i + 2) % n];
            let cross = (b - a).cross(c - b);
            if cross.abs() < 1e-9 {
                continue;
            }
            let sign = if cross > 0.0 { 1 } else { -1 };
            if cross_sign == 0 {
                cross_sign = sign;
            } else if cross_sign != sign {
                return Err("polygon is not convex".into());
            }
        }
        if cross_sign == 0 {
            return Err("polygon is degenerate".into());
        }
        Ok(ColliderShape2::ConvexPolygon { points })
    }

    /// Cone as a convex triangle: apex up (local -Y), base centered at origin.
    pub fn cone2(base_half_width: f32, height: f32) -> ColliderShape2 {
        ColliderShape2::ConvexPolygon {
            points: vec![
                Vec2::new(-base_half_width, 0.0),
                Vec2::new(base_half_width, 0.0),
                Vec2::new(0.0, -height),
            ],
        }
    }

    /// Build a ConcavePolygon from raw polygons (each part validated convex).
    pub fn concave_polygon(parts: Vec<Vec<Vec2>>) -> Result<ColliderShape2, String> {
        let mut validated = Vec::with_capacity(parts.len());
        for part in parts {
            match ColliderShape2::convex_polygon(part)? {
                ColliderShape2::ConvexPolygon { points } => validated.push(points),
                _ => unreachable!(),
            }
        }
        Ok(ColliderShape2::ConcavePolygon {
            convex_parts: validated,
        })
    }

    /// World-space AABB of the shape at transform `xf`.
    pub fn bound(&self, xf: &Transform2) -> Aabb2 {
        match self {
            ColliderShape2::Circle { radius } | ColliderShape2::Wheel { radius, .. } => {
                Aabb2::from_center_half(xf.position, Vec2::splat(*radius))
            }
            ColliderShape2::Box { half_extents } => {
                let he = rotated_half_extents(*half_extents, xf.rotation);
                Aabb2::from_center_half(xf.position, he)
            }
            ColliderShape2::Stadium {
                half_length: hl,
                radius: r,
            } => {
                let (hl, r) = (*hl, *r);
                let dir = xf.dir_to_world(Vec2::RIGHT);
                let a = xf.position - dir * hl;
                let b = xf.position + dir * hl;
                Aabb2 {
                    min: a.min(b) - Vec2::splat(r),
                    max: a.max(b) + Vec2::splat(r),
                }
            }
            ColliderShape2::Cone2 {
                base_half_width,
                height,
            } => {
                let he =
                    rotated_half_extents(Vec2::new(*base_half_width, *height / 2.0), xf.rotation);
                Aabb2::from_center_half(xf.position - Vec2::new(0.0, *height / 2.0), he)
            }
            ColliderShape2::Plane { .. } => {
                // infinite: generous bound (broadphase will always include it)
                Aabb2::from_center_half(xf.position, Vec2::splat(1.0e9))
            }
            ColliderShape2::ConvexPolygon { points } => bound_of_points(points, xf),
            ColliderShape2::ConcavePolygon { convex_parts } => {
                let mut b = Aabb2::EMPTY;
                for part in convex_parts {
                    let pb = bound_of_points(part, xf);
                    b.min = b.min.min(pb.min);
                    b.max = b.max.max(pb.max);
                }
                b
            }
            ColliderShape2::Heightfield {
                segment_length,
                heights,
            } => {
                let half_len = (heights.len().saturating_sub(1) as f32) * segment_length / 2.0;
                let min_y = heights.iter().cloned().fold(f32::INFINITY, f32::min);
                let max_y = heights.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
                Aabb2 {
                    min: Vec2::new(xf.position.x - half_len, xf.position.y + min_y),
                    max: Vec2::new(xf.position.x + half_len, xf.position.y + max_y),
                }
            }
            ColliderShape2::Terrain { cell_size, heights } => {
                let half_len = (heights.len() as f32) * cell_size / 2.0;
                let min_y = heights.iter().cloned().fold(f32::INFINITY, f32::min);
                Aabb2 {
                    min: Vec2::new(xf.position.x - half_len, xf.position.y + min_y),
                    // solid columns extend far downward (blocky terrain)
                    max: Vec2::new(xf.position.x + half_len, xf.position.y + 1.0e5),
                }
            }
            ColliderShape2::Compound { children } => {
                let mut b = Aabb2::EMPTY;
                for child in children {
                    let child_xf =
                        Transform2::new(xf.to_world(child.offset), xf.rotation + child.rotation);
                    let pb = child.shape.bound(&child_xf);
                    b.min = b.min.min(pb.min);
                    b.max = b.max.max(pb.max);
                }
                b
            }
            ColliderShape2::Custom(c) => c.bound(xf),
        }
    }

    /// Point-inside test (solid region).
    pub fn contains_point(&self, xf: &Transform2, p: Vec2) -> bool {
        match self {
            ColliderShape2::Circle { radius } | ColliderShape2::Wheel { radius, .. } => {
                (p - xf.position).length() <= *radius
            }
            ColliderShape2::Box { half_extents } => {
                let l = xf.to_local(p);
                l.x.abs() <= half_extents.x && l.y.abs() <= half_extents.y
            }
            ColliderShape2::Stadium {
                half_length: hl,
                radius: r,
            } => {
                let (hl, r) = (*hl, *r);
                let l = xf.to_local(p);
                let clamped_x = l.x.clamp(-hl, hl);
                (l - Vec2::new(clamped_x, 0.0)).length() <= r
            }
            ColliderShape2::Cone2 { .. } => point_in_convex_points(&cone_points(self), xf, p),
            ColliderShape2::Plane { normal } => {
                // solid side is against the normal
                xf.to_local(p).dot(*normal) <= 0.0
            }
            ColliderShape2::ConvexPolygon { points } => point_in_convex_points(points, xf, p),
            ColliderShape2::ConcavePolygon { convex_parts } => convex_parts
                .iter()
                .any(|part| point_in_convex_points(part, xf, p)),
            ColliderShape2::Heightfield {
                segment_length,
                heights,
            } => heightfield_contains(heights, *segment_length, xf.to_local(p)),
            ColliderShape2::Terrain { cell_size, heights } => {
                terrain_contains(heights, *cell_size, xf.to_local(p))
            }
            ColliderShape2::Compound { children } => {
                for child in children {
                    let child_xf =
                        Transform2::new(xf.to_world(child.offset), xf.rotation + child.rotation);
                    if child.shape.contains_point(&child_xf, p) {
                        return true;
                    }
                }
                false
            }
            ColliderShape2::Custom(c) => c.contains_point(xf, p),
        }
    }

    /// Approximate mass from a uniform density (per unit depth = 1).
    pub fn mass_from_density(&self, density: f32) -> f32 {
        let area = match self {
            ColliderShape2::Circle { radius } | ColliderShape2::Wheel { radius, .. } => {
                std::f32::consts::PI * radius * radius
            }
            ColliderShape2::Box { half_extents } => half_extents.x * half_extents.y * 4.0,
            ColliderShape2::Stadium {
                half_length,
                radius,
            } => 2.0 * half_length * radius * 2.0 + std::f32::consts::PI * radius * radius,
            ColliderShape2::Cone2 {
                base_half_width,
                height,
            } => base_half_width * 2.0 * height / 2.0,
            ColliderShape2::ConvexPolygon { points } => shoelace_area(points).abs(),
            ColliderShape2::ConcavePolygon { convex_parts } => {
                convex_parts.iter().map(|p| shoelace_area(p).abs()).sum()
            }
            ColliderShape2::Compound { children } => children
                .iter()
                .map(|c| c.shape.mass_from_density(density) / density.max(1e-6))
                .sum(),
            ColliderShape2::Custom(_) => 1.0,
            // planes/heightfields/terrain are static backdrops
            _ => 1.0,
        };
        (area * density).max(1.0e-6)
    }

    /// Human-readable kind name (tests/docs/diagnostics).
    pub fn kind_name(&self) -> &'static str {
        match self {
            ColliderShape2::Circle { .. } => "Circle",
            ColliderShape2::Box { .. } => "Box",
            ColliderShape2::Stadium { .. } => "Stadium",
            ColliderShape2::Cone2 { .. } => "Cone2",
            ColliderShape2::Plane { .. } => "Plane",
            ColliderShape2::ConvexPolygon { .. } => "ConvexPolygon",
            ColliderShape2::ConcavePolygon { .. } => "ConcavePolygon",
            ColliderShape2::Heightfield { .. } => "Heightfield",
            ColliderShape2::Terrain { .. } => "Terrain",
            ColliderShape2::Compound { .. } => "Compound",
            ColliderShape2::Wheel { .. } => "Wheel",
            ColliderShape2::Custom(_) => "Custom",
        }
    }

    /// Iterate compound/concave children as (local transform, view) pairs.
    ///
    /// Views are lightweight: for ConcavePolygon each part is surfaced as a
    /// polygon point-slice (see `collision.rs`), not a new `ColliderShape2`.
    pub fn compound_children(&self) -> Vec<(Transform2, &ColliderShape2)> {
        match self {
            ColliderShape2::Compound { children } => children
                .iter()
                .map(|c| (Transform2::new(c.offset, c.rotation), &c.shape))
                .collect(),
            _ => Vec::new(),
        }
    }
}

// -- helpers ----------------------------------------------------------------

/// Outward-aware rotated AABB half extents for an OBB.
pub(crate) fn rotated_half_extents(half_extents: Vec2, rotation: f32) -> Vec2 {
    let (s, c) = rotation.sin_cos();
    Vec2::new(
        half_extents.x * c.abs() + half_extents.y * s.abs(),
        half_extents.x * s.abs() + half_extents.y * c.abs(),
    )
}

/// Local-space triangle points of a Cone2 (used by narrowphase).
pub(crate) fn cone_points(shape: &ColliderShape2) -> Vec<Vec2> {
    if let ColliderShape2::Cone2 {
        base_half_width,
        height,
    } = shape
    {
        vec![
            Vec2::new(-base_half_width, 0.0),
            Vec2::new(*base_half_width, 0.0),
            Vec2::new(0.0, -height),
        ]
    } else {
        Vec::new()
    }
}

fn bound_of_points(points: &[Vec2], xf: &Transform2) -> Aabb2 {
    let world: Vec<Vec2> = points.iter().map(|p| xf.to_world(*p)).collect();
    let min = world
        .iter()
        .copied()
        .fold(Vec2::splat(f32::INFINITY), Vec2::min);
    let max = world
        .iter()
        .copied()
        .fold(Vec2::splat(f32::NEG_INFINITY), Vec2::max);
    Aabb2 { min, max }
}

/// Point-in-convex-polygon (world space): the point must be behind every
/// outward face. Normals are oriented away from the centroid, so winding
/// does not matter.
pub(crate) fn point_in_convex_points(points: &[Vec2], xf: &Transform2, p: Vec2) -> bool {
    let n = points.len();
    if n < 3 {
        return false;
    }
    let centroid = xf.to_world(centroid_of(points));
    for i in 0..n {
        let a = xf.to_world(points[i]);
        let b = xf.to_world(points[(i + 1) % n]);
        let mut outward = (b - a).perp();
        if outward.dot(centroid - a) > 0.0 {
            outward = -outward;
        }
        if outward.dot(p - a) > 0.0 {
            return false;
        }
    }
    true
}

pub(crate) fn centroid_of(points: &[Vec2]) -> Vec2 {
    let sum = points.iter().copied().fold(Vec2::ZERO, |a, b| a + b);
    sum / points.len() as f32
}

fn shoelace_area(points: &[Vec2]) -> f32 {
    let n = points.len();
    let mut area = 0.0;
    for i in 0..n {
        let a = points[i];
        let b = points[(i + 1) % n];
        area += a.cross(b);
    }
    area / 2.0
}

pub(crate) fn heightfield_contains(heights: &[f32], segment_length: f32, local: Vec2) -> bool {
    let n = heights.len();
    if n < 2 {
        return false;
    }
    let half_len = (n - 1) as f32 * segment_length / 2.0;
    if local.x < -half_len || local.x > half_len {
        return false;
    }
    let t = (local.x + half_len) / segment_length;
    let i = (t as usize).min(n - 2);
    let frac = t - i as f32;
    let surface_y = heights[i] * (1.0 - frac) + heights[i + 1] * frac;
    local.y >= surface_y // solid below the surface (+Y down)
}

pub(crate) fn terrain_contains(heights: &[f32], cell_size: f32, local: Vec2) -> bool {
    let n = heights.len();
    if n == 0 {
        return false;
    }
    let half_len = n as f32 * cell_size / 2.0;
    if local.x < -half_len || local.x > half_len {
        return false;
    }
    let col = ((local.x + half_len) / cell_size) as usize;
    local.y >= heights[col.min(n - 1)]
}

/// Surface Y (local) of a heightfield at local x, if within range.
//[used by raycast.rs in P5]
#[allow(dead_code)]
pub(crate) fn heightfield_surface_y(
    heights: &[f32],
    segment_length: f32,
    local_x: f32,
) -> Option<f32> {
    let n = heights.len();
    if n < 2 {
        return None;
    }
    let half_len = (n - 1) as f32 * segment_length / 2.0;
    if local_x < -half_len || local_x > half_len {
        return None;
    }
    let t = (local_x + half_len) / segment_length;
    let i = (t as usize).min(n - 2);
    let frac = t - i as f32;
    Some(heights[i] * (1.0 - frac) + heights[i + 1] * frac)
}

/// Surface Y (local) of a terrain column at local x, if within range.
//[used by raycast.rs in P5]
#[allow(dead_code)]
pub(crate) fn terrain_surface_y(heights: &[f32], cell_size: f32, local_x: f32) -> Option<f32> {
    let n = heights.len();
    if n == 0 {
        return None;
    }
    let half_len = n as f32 * cell_size / 2.0;
    if local_x < -half_len || local_x > half_len {
        return None;
    }
    let col = ((local_x + half_len) / cell_size) as usize;
    Some(heights[col.min(n - 1)])
}

/// Closest point on segment [a, b] to p.
pub fn closest_point_on_segment(p: Vec2, a: Vec2, b: Vec2) -> Vec2 {
    let ab = b - a;
    let len2 = ab.length_squared();
    if len2 <= 1.0e-9 {
        return a;
    }
    let t = ((p - a).dot(ab) / len2).clamp(0.0, 1.0);
    a + ab * t
}

/// Segment-segment closest distance (0 when intersecting).
pub fn segment_segment_distance(a1: Vec2, a2: Vec2, b1: Vec2, b2: Vec2) -> f32 {
    let d1 = a2 - a1;
    let d2 = b2 - b1;
    let r = b1 - a1;
    let denom = d1.cross(d2);
    if denom.abs() > 1.0e-12 {
        let t = (r.cross(d2)) / denom;
        let u = (r.cross(d1)) / denom;
        if (0.0..=1.0).contains(&t) && (0.0..=1.0).contains(&u) {
            return 0.0;
        }
    }
    closest_point_on_segment(a1, b1, b2)
        .distance(a1)
        .min(closest_point_on_segment(a2, b1, b2).distance(a2))
        .min(closest_point_on_segment(b1, a1, a2).distance(b1))
        .min(closest_point_on_segment(b2, a1, a2).distance(b2))
}

#[cfg(test)]
mod tests {
    use super::*;
    use monster_math::approx_eq;

    #[test]
    fn transform_roundtrip() {
        let xf = Transform2::new(Vec2::new(10.0, 5.0), 1.234);
        let p = Vec2::new(3.0, -2.0);
        let w = xf.to_world(p);
        let l = xf.to_local(w);
        assert!(approx_eq(l.x, p.x, 1e-4));
        assert!(approx_eq(l.y, p.y, 1e-4));
        // direction rotation is orthogonal
        let d = xf.dir_to_local(xf.dir_to_world(Vec2::RIGHT));
        assert!(approx_eq(d.x, 1.0, 1e-4) && approx_eq(d.y, 0.0, 1e-4));
    }

    #[test]
    fn convex_polygon_validates() {
        let square = vec![
            Vec2::new(-1.0, -1.0),
            Vec2::new(1.0, -1.0),
            Vec2::new(1.0, 1.0),
            Vec2::new(-1.0, 1.0),
        ];
        assert!(ColliderShape2::convex_polygon(square.clone()).is_ok());
        let reversed: Vec<Vec2> = square.iter().rev().cloned().collect();
        assert!(ColliderShape2::convex_polygon(reversed).is_ok());
        let concave = vec![
            Vec2::new(-1.0, -1.0),
            Vec2::new(1.0, 1.0),
            Vec2::new(1.0, -1.0),
            Vec2::new(-1.0, 1.0),
        ];
        assert!(ColliderShape2::convex_polygon(concave).is_err());
        assert!(ColliderShape2::convex_polygon(vec![Vec2::ZERO]).is_err());
    }

    #[test]
    fn contains_point_shapes() {
        let xf = Transform2::new(Vec2::new(5.0, 5.0), 0.0);
        assert!(ColliderShape2::Circle { radius: 2.0 }.contains_point(&xf, Vec2::new(6.0, 5.0)));
        assert!(!ColliderShape2::Circle { radius: 2.0 }.contains_point(&xf, Vec2::new(8.0, 5.0)));

        let box_shape = ColliderShape2::Box {
            half_extents: Vec2::new(2.0, 1.0),
        };
        assert!(box_shape.contains_point(&xf, Vec2::new(6.5, 5.5)));
        assert!(!box_shape.contains_point(&xf, Vec2::new(8.0, 5.0)));

        let rotated = Transform2::new(Vec2::new(0.0, 0.0), std::f32::consts::FRAC_PI_2);
        assert!(box_shape.contains_point(&rotated, Vec2::new(0.5, -1.5)));

        let stadium = ColliderShape2::Stadium {
            half_length: 2.0,
            radius: 1.0,
        };
        assert!(stadium.contains_point(&xf, Vec2::new(7.0, 5.0))); // inside cap
        assert!(!stadium.contains_point(&xf, Vec2::new(8.5, 5.0))); // 1.5 beyond cap > radius

        let cone = ColliderShape2::cone2(2.0, 4.0);
        assert!(cone.contains_point(&xf, Vec2::new(5.0, 2.0)));
        assert!(!cone.contains_point(&xf, Vec2::new(5.0, 5.5)));
    }

    #[test]
    fn plane_solid_side() {
        let xf = Transform2::new(Vec2::new(0.0, 10.0), 0.0);
        let plane = ColliderShape2::Plane { normal: Vec2::DOWN };
        assert!(plane.contains_point(&xf, Vec2::new(0.0, 5.0))); // above = against normal
        assert!(!plane.contains_point(&xf, Vec2::new(0.0, 15.0))); // below = free side
    }

    #[test]
    fn heightfield_and_terrain_containment() {
        let xf = Transform2::new(Vec2::new(0.0, 0.0), 0.0);
        let hf = ColliderShape2::Heightfield {
            segment_length: 10.0,
            heights: vec![0.0, 0.0, 5.0, 5.0],
        };
        // heights are surface Y offsets (+Y down): solid BELOW (y >= surface)
        assert!(hf.contains_point(&xf, Vec2::new(5.0, 8.0))); // below the raised surface (y=5)
        assert!(!hf.contains_point(&xf, Vec2::new(5.0, 3.0))); // above that surface
        assert!(!hf.contains_point(&xf, Vec2::new(-15.0, -1.0))); // above flat part (y=0)
        assert!(hf.contains_point(&xf, Vec2::new(-15.0, 2.0)));

        let terrain = ColliderShape2::Terrain {
            cell_size: 10.0,
            heights: vec![0.0, 0.0, 8.0],
        };
        // range is local x in [-15, 15] (3 columns x 10)
        assert!(terrain.contains_point(&xf, Vec2::new(-12.0, 4.0))); // column 0 (height 0)
        assert!(terrain.contains_point(&xf, Vec2::new(12.0, 10.0))); // column 2 (height 8)
        assert!(!terrain.contains_point(&xf, Vec2::new(12.0, 4.0))); // above column 2 surface
        assert!(!terrain.contains_point(&xf, Vec2::new(25.0, 10.0))); // outside range
    }

    #[test]
    fn bound_reflects_rotation() {
        let xf = Transform2::new(Vec2::new(0.0, 0.0), std::f32::consts::FRAC_PI_4);
        let b = ColliderShape2::Box {
            half_extents: Vec2::new(1.0, 1.0),
        }
        .bound(&xf);
        assert!(approx_eq(b.max.x, 2.0f32.sqrt(), 1e-4));
    }

    #[test]
    fn compound_bound_and_contains() {
        let shape = ColliderShape2::Compound {
            children: vec![
                CompoundChild {
                    offset: Vec2::new(-10.0, 0.0),
                    rotation: 0.0,
                    shape: ColliderShape2::Circle { radius: 2.0 },
                },
                CompoundChild {
                    offset: Vec2::new(10.0, 0.0),
                    rotation: 0.0,
                    shape: ColliderShape2::Circle { radius: 2.0 },
                },
            ],
        };
        let b = shape.bound(&Transform2::IDENTITY);
        assert!(approx_eq(b.min.x, -12.0, 1e-5));
        assert!(approx_eq(b.max.x, 12.0, 1e-5));
        assert!(shape.contains_point(&Transform2::IDENTITY, Vec2::new(10.0, 0.0)));
        assert!(!shape.contains_point(&Transform2::IDENTITY, Vec2::new(0.0, 0.0)));
    }

    #[test]
    fn mass_from_density_sane() {
        let m = ColliderShape2::Circle { radius: 1.0 }.mass_from_density(2.0);
        assert!(approx_eq(m, std::f32::consts::PI * 2.0, 1e-3));
    }

    #[test]
    fn segment_helpers() {
        let cp = closest_point_on_segment(
            Vec2::new(5.0, 1.0),
            Vec2::new(0.0, 0.0),
            Vec2::new(10.0, 0.0),
        );
        assert!(approx_eq(cp.x, 5.0, 1e-5));
        assert!(approx_eq(cp.y, 0.0, 1e-5));
        assert!(approx_eq(
            segment_segment_distance(
                Vec2::new(-1.0, 0.0),
                Vec2::new(1.0, 0.0),
                Vec2::new(0.0, -1.0),
                Vec2::new(0.0, 1.0)
            ),
            0.0,
            1e-6
        ));
    }
}
