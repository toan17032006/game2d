//! Ray casting: `Ray2` queries against bodies and shapes ("Raycast Collider"
//! of the taxonomy). Used by vehicle wheel suspension and gameplay queries.

use monster_math::Vec2;

use crate::collision::flatten;
use crate::collision::Prim;
use crate::shape::{ColliderShape2, Transform2};
use crate::world::PhysicsWorld2;

/// A ray: origin + normalized direction + max distance.
#[derive(Debug, Clone, Copy)]
pub struct Ray2 {
    pub origin: Vec2,
    pub dir: Vec2,
    pub max_t: f32,
}

impl Ray2 {
    pub fn new(origin: Vec2, dir: Vec2, max_t: f32) -> Ray2 {
        Ray2 {
            origin,
            dir: dir.normalize_or(Vec2::DOWN),
            max_t,
        }
    }

    pub fn point_at(&self, t: f32) -> Vec2 {
        self.origin + self.dir * t
    }
}

/// A ray hit: distance along the ray, hit point and surface normal.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RayHit2 {
    pub t: f32,
    pub point: Vec2,
    pub normal: Vec2,
    /// Index of the body hit (in the queried world).
    pub body: usize,
}

/// Cast a ray against every body in the world; nearest hit wins.
pub fn raycast(world: &PhysicsWorld2, ray: Ray2) -> Option<RayHit2> {
    let mut best: Option<RayHit2> = None;
    for (index, body) in world.bodies.iter().enumerate() {
        if body.is_trigger {
            continue; // triggers are not raycast targets
        }
        if let Some(hit) = raycast_shape(&body.shape, &body.transform, ray) {
            let hit = RayHit2 { body: index, ..hit };
            if best.map(|b| hit.t < b.t).unwrap_or(true) {
                best = Some(hit);
            }
        }
    }
    best
}

/// Cast a ray against one shape.
pub fn raycast_shape(shape: &ColliderShape2, xf: &Transform2, ray: Ray2) -> Option<RayHit2> {
    if let ColliderShape2::Custom(custom) = shape {
        let t = custom.raycast(xf, ray.origin, ray.dir, ray.max_t)?;
        return Some(RayHit2 {
            t,
            point: ray.point_at(t),
            normal: -ray.dir, // conservative fallback normal
            body: usize::MAX,
        });
    }
    let mut prims = Vec::new();
    flatten(shape, xf, &mut prims);
    let mut best: Option<RayHit2> = None;
    for prim in &prims {
        if let Some(hit) = raycast_prim(prim, ray) {
            if best.map(|b| hit.t < b.t).unwrap_or(true) {
                best = Some(hit);
            }
        }
    }
    best
}

fn raycast_prim(prim: &Prim, ray: Ray2) -> Option<RayHit2> {
    match prim {
        Prim::Circle { center, radius } => ray_circle(ray, *center, *radius),
        Prim::Obb { xf, half_extents } => {
            // transform the ray into local space -> axis-aligned box test
            let local_origin = xf.to_local(ray.origin);
            let local_dir = xf.dir_to_local(ray.dir);
            let local_ray = Ray2::new(local_origin, local_dir, ray.max_t);
            let half = Vec2::new(half_extents.x.abs(), half_extents.y.abs());
            ray_aabb(local_ray, half).map(|(t, normal_local)| RayHit2 {
                t,
                point: ray.point_at(t),
                normal: xf.dir_to_world(normal_local),
                body: usize::MAX,
            })
        }
        Prim::Stadium {
            xf,
            half_length,
            radius,
        } => ray_stadium(ray, xf, *half_length, *radius),
        Prim::Plane { point, normal } => ray_plane(ray, *point, *normal),
        Prim::Poly { points } => ray_poly(ray, points),
    }
}

/// Ray vs circle: standard quadratic; nearest valid root wins.
fn ray_circle(ray: Ray2, center: Vec2, radius: f32) -> Option<RayHit2> {
    let m = ray.origin - center;
    let b = m.dot(ray.dir);
    let c = m.dot(m) - radius * radius;
    if c > 0.0 && b > 0.0 {
        return None; // outside and pointing away
    }
    let disc = b * b - c;
    if disc < 0.0 {
        return None;
    }
    let t = -b - disc.sqrt();
    let t = if t < 0.0 { -b + disc.sqrt() } else { t }; // inside: take far root
    if t < 0.0 || t > ray.max_t {
        return None;
    }
    let point = ray.point_at(t);
    let normal = (point - center).normalize_or(-ray.dir);
    Some(RayHit2 {
        t,
        point,
        normal,
        body: usize::MAX,
    })
}

/// Ray vs axis-aligned box (local space): slab method. Returns (t, normal).
fn ray_aabb(ray: Ray2, half: Vec2) -> Option<(f32, Vec2)> {
    let mut t_min = 0.0f32;
    let mut t_max = ray.max_t;
    let mut hit_normal = Vec2::ZERO;
    for axis in 0..2 {
        let o = if axis == 0 {
            ray.origin.x
        } else {
            ray.origin.y
        };
        let d = if axis == 0 { ray.dir.x } else { ray.dir.y };
        let h = if axis == 0 { half.x } else { half.y };
        if d.abs() < 1.0e-9 {
            if o.abs() > h {
                return None;
            }
            continue;
        }
        let inv = 1.0 / d;
        let mut t1 = (-h - o) * inv;
        let mut t2 = (h - o) * inv;
        let mut face_sign = -1.0f32; // entering through the -face
        if t1 > t2 {
            std::mem::swap(&mut t1, &mut t2);
            face_sign = 1.0;
        }
        if t1 > t_min {
            t_min = t1;
            hit_normal = if axis == 0 {
                Vec2::new(face_sign, 0.0)
            } else {
                Vec2::new(0.0, face_sign)
            };
        }
        t_max = t_max.min(t2);
        if t_min > t_max {
            return None;
        }
    }
    if t_min <= 0.0 || t_min > ray.max_t {
        return None;
    }
    Some((t_min, hit_normal))
}

/// Ray vs stadium (capsule): both end circles + the two parallel side lines.
fn ray_stadium(ray: Ray2, xf: &Transform2, hl: f32, r: f32) -> Option<RayHit2> {
    let dir = xf.dir_to_world(Vec2::RIGHT);
    let a = xf.position - dir * hl;
    let b = xf.position + dir * hl;
    let mut best: Option<RayHit2> = None;
    for center in [a, b] {
        if let Some(hit) = ray_circle(ray, center, r) {
            if best.map(|bh| hit.t < bh.t).unwrap_or(true) {
                best = Some(hit);
            }
        }
    }
    // the straight sides: two lines offset by ±r·perp(dir), range-checked
    let perp = dir.perp();
    for offset in [perp * r, -perp * r] {
        let a2 = a + offset;
        let b2 = b + offset;
        let line_dir = (b2 - a2).normalize_or(Vec2::RIGHT);
        let denom = ray.dir.cross(line_dir);
        if denom.abs() <= 1.0e-9 {
            continue;
        }
        let t = (a2 - ray.origin).cross(line_dir) / denom;
        if t < 0.0 || t > ray.max_t {
            continue;
        }
        let point = ray.point_at(t);
        let u = (point - a2).dot(line_dir);
        if !(0.0..=(b2 - a2).length()).contains(&u) {
            continue;
        }
        let side = (point - ray.origin).dot(perp) > 0.0;
        let normal = if side { -perp } else { perp };
        if best.map(|bh| t < bh.t).unwrap_or(true) {
            best = Some(RayHit2 {
                t,
                point,
                normal,
                body: usize::MAX,
            });
        }
    }
    best
}

/// Ray vs plane (half-space): the crossing of the surface line.
fn ray_plane(ray: Ray2, point: Vec2, normal: Vec2) -> Option<RayHit2> {
    let denom = ray.dir.dot(normal);
    if denom.abs() < 1.0e-9 {
        return None;
    }
    let t = (point - ray.origin).dot(normal) / denom;
    if t < 0.0 || t > ray.max_t {
        return None;
    }
    Some(RayHit2 {
        t,
        point: ray.point_at(t),
        normal,
        body: usize::MAX,
    })
}

/// Ray vs convex polygon (world points): nearest edge crossing.
fn ray_poly(ray: Ray2, points: &[Vec2]) -> Option<RayHit2> {
    let mut best: Option<RayHit2> = None;
    let n = points.len();
    let centroid = crate::shape::centroid_of(points);
    for i in 0..n {
        let a = points[i];
        let b = points[(i + 1) % n];
        let edge = b - a;
        let mut normal = edge.perp().normalize_or(Vec2::DOWN);
        if normal.dot(centroid - a) > 0.0 {
            normal = -normal; // outward
        }
        let denom = ray.dir.dot(normal);
        if denom.abs() < 1.0e-9 {
            continue;
        }
        let t = (a - ray.origin).dot(normal) / denom;
        if t < 0.0 || t > ray.max_t {
            continue;
        }
        let point = ray.point_at(t);
        let u = (point - a).dot(edge) / edge.length_squared().max(1e-9);
        if !(0.0..=1.0).contains(&u) {
            continue;
        }
        if best.map(|bh| t < bh.t).unwrap_or(true) {
            best = Some(RayHit2 {
                t,
                point,
                normal,
                body: usize::MAX,
            });
        }
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::body::{BodyKind2, PhysicsBody2};
    use monster_math::approx_eq;

    #[test]
    fn ray_hits_circle() {
        let mut world = PhysicsWorld2::new(Vec2::ZERO);
        world.add_body(PhysicsBody2::new(
            BodyKind2::Static,
            Vec2::new(0.0, 100.0),
            ColliderShape2::Circle { radius: 20.0 },
        ));
        let ray = Ray2::new(Vec2::new(0.0, 0.0), Vec2::new(0.0, 1.0), 500.0);
        let hit = raycast(&world, ray).expect("hits circle");
        assert!(approx_eq(hit.t, 80.0, 1e-3));
        assert!(approx_eq(hit.point.y, 80.0, 1e-3));
        assert!(approx_eq(hit.normal.y, -1.0, 1e-3)); // surface faces the ray origin
        assert_eq!(hit.body, 0);
    }

    #[test]
    fn ray_misses_when_too_short() {
        let mut world = PhysicsWorld2::new(Vec2::ZERO);
        world.add_body(PhysicsBody2::new(
            BodyKind2::Static,
            Vec2::new(0.0, 100.0),
            ColliderShape2::Circle { radius: 20.0 },
        ));
        let ray = Ray2::new(Vec2::new(0.0, 0.0), Vec2::new(0.0, 1.0), 50.0);
        assert!(raycast(&world, ray).is_none());
    }

    #[test]
    fn ray_hits_box_face() {
        let mut world = PhysicsWorld2::new(Vec2::ZERO);
        world.add_body(PhysicsBody2::new(
            BodyKind2::Static,
            Vec2::new(0.0, 100.0),
            ColliderShape2::Box {
                half_extents: Vec2::new(30.0, 10.0),
            },
        ));
        let ray = Ray2::new(Vec2::new(0.0, 0.0), Vec2::new(0.0, 1.0), 500.0);
        let hit = raycast(&world, ray).expect("hits box top");
        assert!(approx_eq(hit.t, 90.0, 1e-3)); // top face at y=90
        assert!(approx_eq(hit.normal.y, -1.0, 1e-3));
    }

    #[test]
    fn ray_hits_heightfield() {
        let mut world = PhysicsWorld2::new(Vec2::ZERO);
        world.add_body(PhysicsBody2::new(
            BodyKind2::Static,
            Vec2::new(0.0, 50.0),
            ColliderShape2::Heightfield {
                segment_length: 10.0,
                heights: vec![0.0, 0.0, 0.0],
            },
        ));
        let ray = Ray2::new(Vec2::new(-25.0, 0.0), Vec2::new(0.0, 1.0), 500.0);
        let hit = raycast(&world, ray).expect("hits heightfield surface");
        assert!(approx_eq(hit.t, 50.0, 1e-2));
    }

    #[test]
    fn ray_nearest_hit_wins() {
        let mut world = PhysicsWorld2::new(Vec2::ZERO);
        world.add_body(PhysicsBody2::new(
            BodyKind2::Static,
            Vec2::new(0.0, 150.0),
            ColliderShape2::Circle { radius: 20.0 },
        ));
        let near = world.add_body(PhysicsBody2::new(
            BodyKind2::Static,
            Vec2::new(0.0, 60.0),
            ColliderShape2::Circle { radius: 10.0 },
        ));
        let ray = Ray2::new(Vec2::new(0.0, 0.0), Vec2::new(0.0, 1.0), 500.0);
        let hit = raycast(&world, ray).expect("hits near circle");
        assert_eq!(hit.body, near);
        assert!(approx_eq(hit.t, 50.0, 1e-3));
    }

    #[test]
    fn character_move_and_slide_walks_on_floor() {
        let mut world = PhysicsWorld2::new(Vec2::new(0.0, 980.0));
        world.add_body(PhysicsBody2::new(
            BodyKind2::Static,
            Vec2::new(0.0, 100.0),
            ColliderShape2::Box {
                half_extents: Vec2::new(500.0, 10.0),
            },
        ));
        let character = world.add_body(PhysicsBody2::new(
            BodyKind2::Character,
            Vec2::new(0.0, 50.0),
            ColliderShape2::Box {
                half_extents: Vec2::new(10.0, 15.0),
            },
        ));

        // walk right while falling onto the ground
        let mut landed = false;
        for _ in 0..120 {
            let res = world.move_and_slide(character, Vec2::new(50.0, 200.0), 1.0 / 60.0);
            if res.floor {
                landed = true;
            }
        }
        assert!(landed, "character landed on the floor");
        let body = &world.bodies[character];
        // rests with its bottom (y + 15) on the ground surface (y = 90)
        assert!(
            (body.transform.position.y - 75.0).abs() < 3.0,
            "rest y={}",
            body.transform.position.y
        );
        assert!(
            body.transform.position.x > 50.0,
            "walked right: x={}",
            body.transform.position.x
        );
    }

    #[test]
    fn character_stops_at_wall() {
        let mut world = PhysicsWorld2::new(Vec2::ZERO);
        world.add_body(PhysicsBody2::new(
            BodyKind2::Static,
            Vec2::new(100.0, 0.0),
            ColliderShape2::Box {
                half_extents: Vec2::new(10.0, 500.0),
            },
        ));
        let character = world.add_body(PhysicsBody2::new(
            BodyKind2::Character,
            Vec2::new(0.0, 0.0),
            ColliderShape2::Box {
                half_extents: Vec2::new(10.0, 10.0),
            },
        ));
        for _ in 0..60 {
            world.move_and_slide(character, Vec2::new(120.0, 0.0), 1.0 / 60.0);
        }
        let x = world.bodies[character].transform.position.x;
        // wall left face at x=90; character right edge stops at ~90
        assert!(x < 85.0, "blocked at x={}", x);
        assert!(x > 60.0, "but moved some: x={}", x);
    }
}
