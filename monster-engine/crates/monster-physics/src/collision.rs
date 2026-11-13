//! Narrowphase: every pair of collider shapes reduced to a small set of
//! collision primitives.
//!
//! Strategy: complex shapes (Compound, ConcavePolygon, Heightfield, Terrain,
//! Cone2, Wheel) are *flattened* into world-space primitives
//! (Circle / OBB / Stadium / Plane / ConvexPolygon), then every primitive
//! pair is tested and the **deepest** contact wins. `Custom` shapes bypass
//! flattening entirely — their trait supplies the narrowphase.

use monster_math::Vec2;

use crate::shape::{
    closest_point_on_segment, cone_points, point_in_convex_points, ColliderShape2, Contact2,
    Transform2,
};

/// World-space collision primitive produced by [`flatten`].
#[derive(Debug, Clone)]
pub(crate) enum Prim {
    Circle {
        center: Vec2,
        radius: f32,
    },
    Obb {
        xf: Transform2,
        half_extents: Vec2,
    },
    Stadium {
        xf: Transform2,
        half_length: f32,
        radius: f32,
    },
    Plane {
        point: Vec2,
        normal: Vec2,
    },
    /// Convex polygon with world-space points.
    Poly {
        points: Vec<Vec2>,
    },
}

/// Flatten `shape` at `xf` into world-space primitives.
pub(crate) fn flatten(shape: &ColliderShape2, xf: &Transform2, out: &mut Vec<Prim>) {
    match shape {
        ColliderShape2::Circle { radius } => {
            out.push(Prim::Circle {
                center: xf.position,
                radius: *radius,
            });
        }
        ColliderShape2::Wheel { radius, .. } => {
            out.push(Prim::Circle {
                center: xf.position,
                radius: *radius,
            });
        }
        ColliderShape2::Box { half_extents } => {
            out.push(Prim::Obb {
                xf: *xf,
                half_extents: *half_extents,
            });
        }
        ColliderShape2::Stadium {
            half_length,
            radius,
        } => {
            out.push(Prim::Stadium {
                xf: *xf,
                half_length: *half_length,
                radius: *radius,
            });
        }
        ColliderShape2::Cone2 { .. } => {
            let points = cone_points(shape);
            let world: Vec<Vec2> = points.iter().map(|p| xf.to_world(*p)).collect();
            out.push(Prim::Poly { points: world });
        }
        ColliderShape2::Plane { normal } => {
            out.push(Prim::Plane {
                point: xf.position,
                normal: xf.dir_to_world(*normal).normalize_or(Vec2::DOWN),
            });
        }
        ColliderShape2::ConvexPolygon { points } => {
            let world: Vec<Vec2> = points.iter().map(|p| xf.to_world(*p)).collect();
            out.push(Prim::Poly { points: world });
        }
        ColliderShape2::ConcavePolygon { convex_parts } => {
            for part in convex_parts {
                let world: Vec<Vec2> = part.iter().map(|p| xf.to_world(*p)).collect();
                out.push(Prim::Poly { points: world });
            }
        }
        ColliderShape2::Heightfield {
            segment_length,
            heights,
        } => {
            // each surface cell becomes a Plane whose free side is ABOVE the
            // surface (solid = below, +Y down). This gives correct push-out
            // direction for bodies resting on or sunk into the ground.
            let n = heights.len();
            let half_len = (n.saturating_sub(1) as f32) * segment_length / 2.0;
            for i in 0..n.saturating_sub(1) {
                let x0 = xf.position.x - half_len + (i as f32) * segment_length;
                let x1 = x0 + segment_length;
                let y0 = xf.position.y + heights[i];
                let y1 = xf.position.y + heights[i + 1];
                let midpoint = Vec2::new((x0 + x1) / 2.0, (y0 + y1) / 2.0);
                let d = Vec2::new(x1 - x0, y1 - y0);
                let mut normal = d.perp(); // (-dy, dx)
                if normal.y > 0.0 {
                    normal = -normal; // orient up (out of the solid)
                }
                out.push(Prim::Plane {
                    point: midpoint,
                    normal: normal.normalize_or(Vec2::new(0.0, -1.0)),
                });
            }
        }
        ColliderShape2::Terrain { cell_size, heights } => {
            let n = heights.len();
            let half_len = n as f32 * cell_size / 2.0;
            for i in 0..n {
                let x0 = xf.position.x - half_len + (i as f32) * cell_size;
                let surface = xf.position.y + heights[i];
                out.push(Prim::Obb {
                    xf: Transform2::new(Vec2::new(x0 + cell_size / 2.0, surface + 5000.0), 0.0),
                    half_extents: Vec2::new(cell_size / 2.0, 5000.0),
                });
            }
        }
        ColliderShape2::Compound { children } => {
            for child in children {
                let child_xf =
                    Transform2::new(xf.to_world(child.offset), xf.rotation + child.rotation);
                flatten(&child.shape, &child_xf, out);
            }
        }
        ColliderShape2::Custom(_) => {
            // handled by the dispatcher (trait narrowphase)
        }
    }
}

/// Main narrowphase entry: deepest contact between two shapes,
/// `normal` pointing **from A to B**.
pub fn collide(
    a: &ColliderShape2,
    xa: &Transform2,
    b: &ColliderShape2,
    xb: &Transform2,
) -> Option<Contact2> {
    // Custom shapes run their own narrowphase first.
    if let ColliderShape2::Custom(custom) = a {
        return custom.collide_with(xa, b, xb);
    }
    if let ColliderShape2::Custom(custom) = b {
        return custom.collide_with(xb, a, xa).map(|c| Contact2 {
            normal: -c.normal,
            ..c
        });
    }

    let mut prims_a = Vec::new();
    let mut prims_b = Vec::new();
    flatten(a, xa, &mut prims_a);
    flatten(b, xb, &mut prims_b);

    let mut best: Option<Contact2> = None;
    for pa in &prims_a {
        for pb in &prims_b {
            if let Some(c) = prim_collide(pa, pb) {
                let better = match &best {
                    Some(existing) => c.penetration > existing.penetration,
                    None => true,
                };
                if better {
                    best = Some(c);
                }
            }
        }
    }
    best
}

fn flip(mut c: Contact2) -> Contact2 {
    c.normal = -c.normal;
    c
}

/// Narrowphase between two primitives; normal A→B.
fn prim_collide(pa: &Prim, pb: &Prim) -> Option<Contact2> {
    match (pa, pb) {
        (
            Prim::Circle {
                center: ca,
                radius: ra,
            },
            Prim::Circle {
                center: cb,
                radius: rb,
            },
        ) => circle_circle(*ca, *ra, *cb, *rb),
        (
            Prim::Circle {
                center: c,
                radius: r,
            },
            Prim::Obb { xf, half_extents },
        ) => circle_obb(*c, *r, xf, half_extents),
        (
            Prim::Obb { xf, half_extents },
            Prim::Circle {
                center: c,
                radius: r,
            },
        ) => circle_obb(*c, *r, xf, half_extents).map(flip),
        (
            Prim::Circle {
                center: c,
                radius: r,
            },
            Prim::Stadium {
                xf,
                half_length,
                radius,
            },
        ) => circle_stadium(*c, *r, xf, *half_length, *radius),
        (
            Prim::Stadium {
                xf,
                half_length,
                radius,
            },
            Prim::Circle {
                center: c,
                radius: r,
            },
        ) => circle_stadium(*c, *r, xf, *half_length, *radius).map(flip),
        (
            Prim::Circle {
                center: c,
                radius: r,
            },
            Prim::Plane { point, normal },
        ) => circle_plane(*c, *r, *point, *normal),
        (
            Prim::Plane { point, normal },
            Prim::Circle {
                center: c,
                radius: r,
            },
        ) => circle_plane(*c, *r, *point, *normal).map(flip),
        (
            Prim::Circle {
                center: c,
                radius: r,
            },
            Prim::Poly { points },
        ) => circle_poly(*c, *r, points),
        (
            Prim::Poly { points },
            Prim::Circle {
                center: c,
                radius: r,
            },
        ) => circle_poly(*c, *r, points).map(flip),
        (
            Prim::Obb {
                xf: xa,
                half_extents: ha,
            },
            Prim::Obb {
                xf: xb,
                half_extents: hb,
            },
        ) => obb_obb(xa, ha, xb, hb),
        (
            Prim::Obb {
                xf: xf_a,
                half_extents,
            },
            Prim::Stadium {
                xf: xf_b,
                half_length,
                radius,
            },
        ) => obb_stadium(xf_a, half_extents, xf_b, *half_length, *radius),
        (
            Prim::Stadium {
                xf: xf_b,
                half_length,
                radius,
            },
            Prim::Obb {
                xf: xf_a,
                half_extents,
            },
        ) => obb_stadium(xf_a, half_extents, xf_b, *half_length, *radius).map(flip),
        (Prim::Obb { xf, half_extents }, Prim::Plane { point, normal }) => {
            obb_plane(xf, half_extents, *point, *normal)
        }
        (Prim::Plane { point, normal }, Prim::Obb { xf, half_extents }) => {
            obb_plane(xf, half_extents, *point, *normal).map(flip)
        }
        (Prim::Obb { xf, half_extents }, Prim::Poly { points }) => {
            let corners = obb_corners(xf, half_extents);
            poly_poly(&corners, points)
        }
        (Prim::Poly { points }, Prim::Obb { xf, half_extents }) => {
            let corners = obb_corners(xf, half_extents);
            poly_poly(points, &corners).map(flip)
        }
        (
            Prim::Stadium {
                xf: xa,
                half_length: la,
                radius: ra,
            },
            Prim::Stadium {
                xf: xb,
                half_length: lb,
                radius: rb,
            },
        ) => stadium_stadium(xa, *la, *ra, xb, *lb, *rb),
        (
            Prim::Stadium {
                xf,
                half_length,
                radius,
            },
            Prim::Plane { point, normal },
        ) => stadium_plane(xf, *half_length, *radius, *point, *normal),
        (
            Prim::Plane { point, normal },
            Prim::Stadium {
                xf,
                half_length,
                radius,
            },
        ) => stadium_plane(xf, *half_length, *radius, *point, *normal).map(flip),
        (
            Prim::Stadium {
                xf,
                half_length,
                radius,
            },
            Prim::Poly { points },
        ) => stadium_poly(xf, *half_length, *radius, points),
        (
            Prim::Poly { points },
            Prim::Stadium {
                xf,
                half_length,
                radius,
            },
        ) => stadium_poly(xf, *half_length, *radius, points).map(flip),
        (Prim::Plane { .. }, Prim::Plane { .. }) => None, // two infinite planes
        (Prim::Plane { point, normal }, Prim::Poly { points }) => {
            plane_poly(*point, *normal, points)
        }
        (Prim::Poly { points }, Prim::Plane { point, normal }) => {
            plane_poly(*point, *normal, points).map(flip)
        }
        (Prim::Poly { points: pts_a }, Prim::Poly { points: pts_b }) => poly_poly(pts_a, pts_b),
    }
}

// -- primitive narrowphase ---------------------------------------------------

fn circle_circle(ca: Vec2, ra: f32, cb: Vec2, rb: f32) -> Option<Contact2> {
    let d = cb - ca;
    let dist = d.length();
    let pen = ra + rb - dist;
    if pen <= 0.0 {
        return None;
    }
    let normal = if dist > 1.0e-9 { d / dist } else { Vec2::RIGHT };
    let point = ca + normal * (ra - pen / 2.0);
    Some(Contact2 {
        normal,
        penetration: pen,
        point,
    })
}

fn circle_obb(c: Vec2, r: f32, xf: &Transform2, he: &Vec2) -> Option<Contact2> {
    let local = xf.to_local(c);
    let clamped = Vec2::new(local.x.clamp(-he.x, he.x), local.y.clamp(-he.y, he.y));
    let inside = local.x.abs() <= he.x && local.y.abs() <= he.y;
    if !inside {
        let closest_world = xf.to_world(clamped);
        let delta = closest_world - c;
        let dist = delta.length();
        let pen = r - dist;
        if pen <= 0.0 {
            return None;
        }
        // normal circle→box: from circle centre towards the box surface
        let normal = if dist > 1.0e-9 {
            delta / dist
        } else {
            Vec2::RIGHT
        };
        Some(Contact2 {
            normal,
            penetration: pen,
            point: closest_world,
        })
    } else {
        // centre inside the box: push out through the nearest face. The
        // contact normal points INTO the box (circle→box), so resolution
        // (-normal) moves the circle out through that face.
        let dx = he.x - local.x.abs();
        let dy = he.y - local.y.abs();
        let (normal_local, pen) = if dx < dy {
            (Vec2::new(-local.x.signum(), 0.0), dx + r)
        } else {
            (Vec2::new(0.0, -local.y.signum()), dy + r)
        };
        let normal = xf.dir_to_world(normal_local);
        let point = xf.to_world(clamped);
        Some(Contact2 {
            normal,
            penetration: pen,
            point,
        })
    }
}

fn circle_stadium(c: Vec2, r: f32, xf: &Transform2, hl: f32, sr: f32) -> Option<Contact2> {
    let dir = xf.dir_to_world(Vec2::RIGHT);
    let a = xf.position - dir * hl;
    let b = xf.position + dir * hl;
    let cp = closest_point_on_segment(c, a, b);
    let delta = cp - c;
    let dist = delta.length();
    let pen = r + sr - dist;
    if pen <= 0.0 {
        return None;
    }
    let normal = if dist > 1.0e-9 {
        delta / dist
    } else {
        (cp - xf.position).normalize_or(Vec2::DOWN)
    };
    let point = c + normal * (r - pen / 2.0);
    Some(Contact2 {
        normal,
        penetration: pen,
        point,
    })
}

/// Signed distance of `p` from the plane surface, positive on the free side.
fn plane_distance(p: Vec2, plane_point: Vec2, normal: Vec2) -> f32 {
    (p - plane_point).dot(normal)
}

fn circle_plane(c: Vec2, r: f32, plane_point: Vec2, normal: Vec2) -> Option<Contact2> {
    // A=circle, B=plane → normal A→B = -plane normal (into the solid)
    let d = plane_distance(c, plane_point, normal);
    let pen = r - d;
    if pen <= 0.0 {
        return None;
    }
    let point = c - normal * (d - pen / 2.0).max(0.0);
    Some(Contact2 {
        normal: -normal,
        penetration: pen,
        point,
    })
}

fn circle_poly(c: Vec2, r: f32, points: &[Vec2]) -> Option<Contact2> {
    let identity = Transform2::IDENTITY;
    let inside = point_in_convex_points(points, &identity, c);
    if inside {
        // deepest face push-out; normal circle→poly = into the polygon
        let mut best: Option<(f32, Vec2, Vec2)> = None; // (pen, normal, point)
        let n = points.len();
        let centroid = crate::shape::centroid_of(points);
        for i in 0..n {
            let a = points[i];
            let b = points[(i + 1) % n];
            let mut outward = (b - a).perp();
            if outward.dot(centroid - a) > 0.0 {
                outward = -outward;
            }
            let outward = outward.normalize_or(Vec2::DOWN);
            let sep = (c - a).dot(outward); // negative inside
            let pen = r + sep;
            if best.map(|(bp, _, _)| pen < bp).unwrap_or(true) {
                let point = c - outward * sep;
                best = Some((pen, -outward, point));
            }
        }
        let (pen, normal, point) = best?;
        if pen <= 0.0 {
            return None;
        }
        Some(Contact2 {
            normal,
            penetration: pen,
            point,
        })
    } else {
        let mut cp_best: Option<(f32, Vec2)> = None;
        let n = points.len();
        for i in 0..n {
            let a = points[i];
            let b = points[(i + 1) % n];
            let cp = closest_point_on_segment(c, a, b);
            let d = (c - cp).length();
            if cp_best.map(|(bd, _)| d < bd).unwrap_or(true) {
                cp_best = Some((d, cp));
            }
        }
        let (dist, cp) = cp_best?;
        let pen = r - dist;
        if pen <= 0.0 {
            return None;
        }
        let delta = cp - c;
        let normal = if dist > 1.0e-9 {
            delta / dist
        } else {
            Vec2::DOWN
        };
        Some(Contact2 {
            normal,
            penetration: pen,
            point: cp,
        })
    }
}

fn obb_corners(xf: &Transform2, he: &Vec2) -> Vec<Vec2> {
    vec![
        xf.to_world(Vec2::new(-he.x, -he.y)),
        xf.to_world(Vec2::new(he.x, -he.y)),
        xf.to_world(Vec2::new(he.x, he.y)),
        xf.to_world(Vec2::new(-he.x, he.y)),
    ]
}

fn project_points(points: &[Vec2], axis: Vec2) -> (f32, f32) {
    let mut min = f32::INFINITY;
    let mut max = f32::NEG_INFINITY;
    for p in points {
        let d = p.dot(axis);
        min = min.min(d);
        max = max.max(d);
    }
    (min, max)
}

fn mid_of(points: &[Vec2]) -> Vec2 {
    crate::shape::centroid_of(points)
}

/// SAT between two convex polygons (world-space points).
fn poly_poly(a: &[Vec2], b: &[Vec2]) -> Option<Contact2> {
    let mut best: Option<(f32, Vec2)> = None; // (penetration, normal A→B)
    let mut separated = false;

    let mid_a = mid_of(a);
    let mid_b = mid_of(b);

    let mut check_faces = |faces: &[Vec2], other: &[Vec2]| {
        let n = faces.len();
        let centroid = crate::shape::centroid_of(faces);
        for i in 0..n {
            let pa = faces[i];
            let pb = faces[(i + 1) % n];
            let mut axis = (pb - pa).perp();
            if axis.dot(centroid - pa) > 0.0 {
                axis = -axis; // outward
            }
            let axis = axis.normalize_or(Vec2::DOWN);
            let (min_f, max_f) = project_points(faces, axis);
            let (min_o, max_o) = project_points(other, axis);
            let overlap = max_f.min(max_o) - min_f.max(min_o);
            if overlap <= 0.0 {
                // SAT: one separating axis means NO contact at all — the
                // stale candidate from an earlier axis must be discarded.
                separated = true;
                return;
            }
            let mut normal = axis;
            if normal.dot(mid_b - mid_a) < 0.0 {
                normal = -normal; // orient A→B
            }
            if best.map(|(bp, _)| overlap < bp).unwrap_or(true) {
                best = Some((overlap, normal));
            }
        }
    };

    check_faces(a, b);
    check_faces(b, a);

    if separated {
        return None;
    }
    let (pen, normal) = best?;
    let point = mid_a + normal * ((mid_b - mid_a).dot(normal) / 2.0);
    Some(Contact2 {
        normal,
        penetration: pen,
        point,
    })
}

fn obb_obb(xa: &Transform2, ha: &Vec2, xb: &Transform2, hb: &Vec2) -> Option<Contact2> {
    let corners_a = obb_corners(xa, ha);
    let corners_b = obb_corners(xb, hb);
    poly_poly(&corners_a, &corners_b)
}

fn obb_stadium(
    xf_a: &Transform2,
    he: &Vec2,
    xf_b: &Transform2,
    hl: f32,
    r: f32,
) -> Option<Contact2> {
    // SAT with rounding: box faces + stadium segment axis
    let dir = xf_b.dir_to_world(Vec2::RIGHT);
    let sa = xf_b.position - dir * hl;
    let sb = xf_b.position + dir * hl;
    let corners = obb_corners(xf_a, he);

    let mut best: Option<(f32, Vec2)> = None;
    let mut separated = false;

    let mut consider = |axis: Vec2,
                        min_a: f32,
                        max_a: f32,
                        min_b: f32,
                        max_b: f32,
                        center_a: Vec2,
                        center_b: Vec2| {
        let overlap = max_a.min(max_b) - min_a.max(min_b);
        if overlap <= 0.0 {
            separated = true; // one separating axis = no contact
            return;
        }
        let mut normal = axis;
        if normal.dot(center_b - center_a) < 0.0 {
            normal = -normal;
        }
        if best.map(|(bp, _)| overlap < bp).unwrap_or(true) {
            best = Some((overlap, normal));
        }
    };

    // box face axes (2)
    let (s, c) = xf_a.rotation.sin_cos();
    for axis in [Vec2::new(c, s), Vec2::new(-s, c)] {
        let axis = axis.normalize_or(Vec2::RIGHT);
        let (min_a, max_a) = project_points(&corners, axis);
        let p_a = sa.dot(axis);
        let p_b = sb.dot(axis);
        consider(
            axis,
            min_a,
            max_a,
            p_a.min(p_b) - r,
            p_a.max(p_b) + r,
            xf_a.position,
            xf_b.position,
        );
    }
    // stadium segment axis
    let seg_axis = (sb - sa).normalize_or(Vec2::RIGHT);
    if seg_axis.length_squared() > 0.5 {
        let (min_a, max_a) = project_points(&corners, seg_axis);
        let (min_b, max_b) = project_points(&[sa, sb], seg_axis);
        consider(
            seg_axis,
            min_a,
            max_a,
            min_b - r,
            max_b + r,
            xf_a.position,
            xf_b.position,
        );
    }

    if separated {
        return None;
    }
    let (pen, normal) = best?;
    let cp = closest_point_on_segment(xf_a.position, sa, sb);
    Some(Contact2 {
        normal,
        penetration: pen,
        point: cp,
    })
}

fn obb_plane(xf: &Transform2, he: &Vec2, plane_point: Vec2, normal: Vec2) -> Option<Contact2> {
    // A=obb, B=plane: normal A→B = -plane normal (into solid)
    let corners = obb_corners(xf, he);
    let mut deepest: Option<(f32, Vec2)> = None;
    for corner in corners {
        let depth = -plane_distance(corner, plane_point, normal);
        if depth > 0.0 && deepest.map(|(bd, _)| depth > bd).unwrap_or(true) {
            deepest = Some((depth, corner));
        }
    }
    let (pen, point) = deepest?;
    Some(Contact2 {
        normal: -normal,
        penetration: pen,
        point,
    })
}

fn stadium_stadium(
    xa: &Transform2,
    la: f32,
    ra: f32,
    xb: &Transform2,
    lb: f32,
    rb: f32,
) -> Option<Contact2> {
    let dir_a = xa.dir_to_world(Vec2::RIGHT);
    let dir_b = xb.dir_to_world(Vec2::RIGHT);
    let a1 = xa.position - dir_a * la;
    let a2 = xa.position + dir_a * la;
    let b1 = xb.position - dir_b * lb;
    let b2 = xb.position + dir_b * lb;
    let dist = crate::shape::segment_segment_distance(a1, a2, b1, b2);
    let pen = ra + rb - dist;
    if pen <= 0.0 {
        return None;
    }
    let d = xb.position - xa.position;
    let normal = d.normalize_or(Vec2::DOWN);
    let point = xa.position + normal * (la + ra - pen / 2.0);
    Some(Contact2 {
        normal,
        penetration: pen,
        point,
    })
}

fn stadium_plane(
    xf: &Transform2,
    hl: f32,
    r: f32,
    plane_point: Vec2,
    normal: Vec2,
) -> Option<Contact2> {
    let dir = xf.dir_to_world(Vec2::RIGHT);
    let a = xf.position - dir * hl;
    let b = xf.position + dir * hl;
    let mut min_d = f32::INFINITY;
    let mut deep_point = a;
    for p in [a, b, xf.position] {
        let d = plane_distance(p, plane_point, normal);
        if d < min_d {
            min_d = d;
            deep_point = p;
        }
    }
    let pen = r - min_d;
    if pen <= 0.0 {
        return None;
    }
    // A=stadium, B=plane: normal A→B = -plane normal
    let point = deep_point - normal * min_d.max(0.0);
    Some(Contact2 {
        normal: -normal,
        penetration: pen,
        point,
    })
}

fn plane_poly(plane_point: Vec2, normal: Vec2, points: &[Vec2]) -> Option<Contact2> {
    let mut deepest: Option<(f32, Vec2)> = None;
    for v in points {
        let depth = -plane_distance(*v, plane_point, normal);
        if depth > 0.0 && deepest.map(|(bd, _)| depth > bd).unwrap_or(true) {
            deepest = Some((depth, *v));
        }
    }
    let (pen, point) = deepest?;
    Some(Contact2 {
        normal: -normal,
        penetration: pen,
        point,
    })
}

fn stadium_poly(xf: &Transform2, hl: f32, r: f32, points: &[Vec2]) -> Option<Contact2> {
    // SAT with rounding: stadium segment axis + polygon faces
    let dir = xf.dir_to_world(Vec2::RIGHT);
    let sa = xf.position - dir * hl;
    let sb = xf.position + dir * hl;
    let poly_centroid = crate::shape::centroid_of(points);

    let mut best: Option<(f32, Vec2)> = None;

    let n = points.len();
    for i in 0..n {
        let pa = points[i];
        let pb = points[(i + 1) % n];
        let mut axis = (pb - pa).perp();
        if axis.dot(poly_centroid - pa) > 0.0 {
            axis = -axis; // outward from polygon
        }
        let axis = axis.normalize_or(Vec2::DOWN);
        let d_a = (sa - pa).dot(axis);
        let d_b = (sb - pa).dot(axis);
        let sep = d_a.min(d_b); // stadium surface = min - r
        let pen = r - sep;
        if pen <= 0.0 {
            return None;
        }
        let mut normal = -axis; // stadium→poly: away from polygon face
        if normal.dot(poly_centroid - xf.position) < 0.0 {
            normal = -normal;
        }
        if best.map(|(bp, _)| pen < bp).unwrap_or(true) {
            best = Some((pen, normal));
        }
    }
    // stadium segment axis
    let seg_axis = (sb - sa).normalize_or(Vec2::RIGHT);
    if seg_axis.length_squared() > 0.5 {
        let (min_p, max_p) = project_points(points, seg_axis);
        let (min_s, max_s) = project_points(&[sa, sb], seg_axis);
        let overlap = max_p.min(max_s + r) - min_p.max(min_s - r);
        if overlap > 0.0 && best.map(|(bp, _)| overlap > bp).unwrap_or(true) {
            let mut normal = seg_axis;
            if normal.dot(poly_centroid - xf.position) < 0.0 {
                normal = -normal;
            }
            best = Some((overlap, normal));
        }
    }

    let (pen, normal) = best?;
    let cp = closest_point_on_segment(xf.position, sa, sb);
    Some(Contact2 {
        normal,
        penetration: pen,
        point: cp,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shape::{CompoundChild, CustomShape2};
    use crate::Aabb2;
    use monster_math::approx_eq;

    const EPS: f32 = 1e-3;

    fn circle(r: f32, x: f32, y: f32) -> (ColliderShape2, Transform2) {
        (
            ColliderShape2::Circle { radius: r },
            Transform2::new(Vec2::new(x, y), 0.0),
        )
    }

    #[test]
    fn circle_circle_contact_and_separation() {
        let (sa, xa) = circle(2.0, 0.0, 0.0);
        // overlap by 1: centres 2.5 apart, radii sum 3
        let (sb, xb) = circle(1.0, 2.5, 0.0);
        let c = collide(&sa, &xa, &sb, &xb).expect("overlap");
        assert!(approx_eq(c.penetration, 0.5, EPS));
        assert!(approx_eq(c.normal.x, 1.0, EPS)); // A→B is +X
                                                  // deeper overlap
        let (sb2, xb2) = circle(1.0, 2.0, 0.0);
        let c2 = collide(&sa, &xa, &sb2, &xb2).expect("overlap");
        assert!(approx_eq(c2.penetration, 1.0, EPS));
        // exactly touching (pen == 0) or separated: no contact
        let (sb3, xb3) = circle(1.0, 3.0, 0.0);
        assert!(collide(&sa, &xa, &sb3, &xb3).is_none());
        let (sb4, xb4) = circle(1.0, 5.0, 0.0);
        assert!(collide(&sa, &xa, &sb4, &xb4).is_none());
    }

    #[test]
    fn circle_box_contact_normal_flips_with_order() {
        let sb = ColliderShape2::Box {
            half_extents: Vec2::new(2.0, 2.0),
        };
        let xb = Transform2::new(Vec2::new(0.0, 0.0), 0.0);
        let (sc, xc) = circle(1.0, 2.5, 0.0); // box edge at x=2, circle centre 0.5 past it
        let c = collide(&sb, &xb, &sc, &xc).expect("overlap");
        assert!(approx_eq(c.normal.x, 1.0, EPS)); // A=box → B=circle is +X
        assert!(approx_eq(c.penetration, 0.5, EPS));
        let c2 = collide(&sc, &xc, &sb, &xb).expect("overlap");
        assert!(approx_eq(c2.normal.x, -1.0, EPS)); // reversed: A=circle → B=box is -X
    }

    #[test]
    fn box_box_sat() {
        let ba = ColliderShape2::Box {
            half_extents: Vec2::new(2.0, 2.0),
        };
        let bb = ColliderShape2::Box {
            half_extents: Vec2::new(1.0, 3.0),
        };
        let xa = Transform2::new(Vec2::new(0.0, 0.0), 0.0);
        let xb = Transform2::new(Vec2::new(2.5, 0.0), 0.0);
        let c = collide(&ba, &xa, &bb, &xb).expect("overlap");
        assert!(approx_eq(c.normal.x, 1.0, EPS));
        assert!(approx_eq(c.penetration, 0.5, EPS)); // (2+1) - 2.5
        let xrot = Transform2::new(Vec2::new(2.4, 0.0), std::f32::consts::FRAC_PI_4);
        assert!(collide(&ba, &xa, &bb, &xrot).is_some());
        let xfar = Transform2::new(Vec2::new(10.0, 0.0), 0.0);
        assert!(collide(&ba, &xa, &bb, &xfar).is_none());
    }

    #[test]
    fn circle_plane_from_free_side() {
        let plane = ColliderShape2::Plane { normal: Vec2::DOWN };
        let xp = Transform2::new(Vec2::new(0.0, 10.0), 0.0);
        let (sc, xc) = circle(1.0, 0.0, 10.5); // centre 0.5 into the free side
        let c = collide(&sc, &xc, &plane, &xp).expect("penetrating");
        assert!(approx_eq(c.normal.y, -1.0, EPS)); // A→B = up (into free space is +n; A→B = -n = -Y?? see below)
                                                   // A=circle below the surface... circle centre y=10.5 (free side is +Y beyond 10):
                                                   // normal A→B should point towards the plane solid side = -normal = UP (-Y in screen = up)
        assert!(approx_eq(c.penetration, 0.5, EPS));
    }

    #[test]
    fn circle_convex_polygon_and_cone() {
        let cone = ColliderShape2::cone2(3.0, 4.0);
        let xc = Transform2::new(Vec2::new(0.0, 0.0), 0.0);
        let (sc, xs) = circle(1.0, 0.0, 0.5); // just under the base face (y=0)
        let c = collide(&sc, &xs, &cone, &xc).expect("circle overlaps cone base");
        assert!(c.penetration > 0.0);
        let (sc2, xs2) = circle(1.0, 0.0, -6.0); // above the apex
        assert!(collide(&sc2, &xs2, &cone, &xc).is_none());
    }

    #[test]
    fn concave_polygon_via_parts() {
        let leg = |x: f32| {
            vec![
                Vec2::new(x, 0.0),
                Vec2::new(x + 2.0, 0.0),
                Vec2::new(x + 2.0, 10.0),
                Vec2::new(x, 10.0),
            ]
        };
        let bottom = vec![
            Vec2::new(-1.0, 8.0),
            Vec2::new(15.0, 8.0),
            Vec2::new(15.0, 10.0),
            Vec2::new(-1.0, 10.0),
        ];
        let concave = ColliderShape2::concave_polygon(vec![leg(0.0), leg(13.0), bottom]).unwrap();
        let xu = Transform2::new(Vec2::new(0.0, 0.0), 0.0);
        let (sc, xs) = circle(2.0, 7.0, 10.5); // inside the bottom bar (y 8..10)
        let c = collide(&sc, &xs, &concave, &xu).expect("hits bottom bar");
        assert!(c.penetration > 0.0);
        let (sc2, xs2) = circle(1.5, 7.0, 4.0); // inside the U cavity: no contact
        assert!(collide(&sc2, &xs2, &concave, &xu).is_none());
    }

    #[test]
    fn heightfield_smooth_ground() {
        let hf = ColliderShape2::Heightfield {
            segment_length: 10.0,
            heights: vec![0.0, 0.0, 0.0],
        };
        let xh = Transform2::new(Vec2::new(0.0, 10.0), 0.0); // surface at y=10
        let (sc, xs) = circle(2.0, 5.0, 11.0); // centre 1 into the ground
        let c = collide(&sc, &xs, &hf, &xh).expect("circle in ground");
        assert!(c.penetration > 0.0);
        // normal A(circle)→B(ground) points down (+Y)
        assert!(c.normal.y > 0.5);
        let (sc2, xs2) = circle(2.0, 5.0, 5.0); // well above
        assert!(collide(&sc2, &xs2, &hf, &xh).is_none());
    }

    #[test]
    fn terrain_blocky_columns() {
        let terrain = ColliderShape2::Terrain {
            cell_size: 10.0,
            heights: vec![0.0, 5.0],
        };
        let xt = Transform2::new(Vec2::new(0.0, 0.0), 0.0);
        // column 1 (x in [-0..10]) surface at y=5
        let (sc, xs) = circle(2.0, 5.0, 6.0); // centre 1 into column 1
        let c = collide(&sc, &xs, &terrain, &xt).expect("hits raised column");
        assert!(c.penetration > 0.0);
        let (sc2, xs2) = circle(2.0, 5.0, -3.0); // above flat column surface (y=0)
        assert!(collide(&sc2, &xs2, &terrain, &xt).is_none());
    }

    #[test]
    fn stadium_stadium_and_plane() {
        let st = ColliderShape2::Stadium {
            half_length: 2.0,
            radius: 1.0,
        };
        let xa = Transform2::new(Vec2::new(0.0, 0.0), 0.0);
        let xb = Transform2::new(Vec2::new(4.5, 0.0), 0.0);
        // segments span ±2 → gap between segments = 0.5, radii sum 2 → pen 1.5
        let c = collide(&st, &xa, &st, &xb).expect("overlapping capsules");
        assert!(approx_eq(c.penetration, 1.5, EPS));

        let plane = ColliderShape2::Plane { normal: Vec2::DOWN };
        let xp = Transform2::new(Vec2::new(0.0, 5.0), 0.0); // solid above y=5
        let xs = Transform2::new(Vec2::new(0.0, 7.0), 0.0); // capsule centre 2 below plane
                                                            // capsule surface extends r=1 further → pen = 1 - 2 = -1?? centre at y=7, plane at 5,
                                                            // free side is +Y: min_d = 2 → pen = 1 - 2 = -1 → no contact
        assert!(collide(&st, &xs, &plane, &xp).is_none());
        let xs2 = Transform2::new(Vec2::new(0.0, 5.5), 0.0); // min_d = 0.5 → pen = 0.5
        let c2 = collide(&st, &xs2, &plane, &xp).expect("capsule into plane");
        assert!(approx_eq(c2.penetration, 0.5, EPS));
    }

    #[test]
    fn compound_vs_circle_deepest_contact() {
        let compound = ColliderShape2::Compound {
            children: vec![
                CompoundChild {
                    offset: Vec2::new(-3.0, 0.0),
                    rotation: 0.0,
                    shape: ColliderShape2::Circle { radius: 2.0 },
                },
                CompoundChild {
                    offset: Vec2::new(3.0, 0.0),
                    rotation: 0.0,
                    shape: ColliderShape2::Circle { radius: 2.0 },
                },
            ],
        };
        let xc = Transform2::IDENTITY;
        let (sc, xs) = circle(1.5, 6.0, 0.0); // overlaps right child (centre 3, r 2) by 0.5
        let c = collide(&sc, &xs, &compound, &xc).expect("hits compound child");
        assert!(approx_eq(c.penetration, 0.5, EPS));
        assert!(approx_eq(c.normal.x, -1.0, EPS)); // A(circle)→B(compound) is -X
    }

    #[test]
    fn custom_shape_trait_narrowphase() {
        struct HalfPlaneRight;
        impl CustomShape2 for HalfPlaneRight {
            fn collide_with(
                &self,
                xf: &Transform2,
                _other: &ColliderShape2,
                other_xf: &Transform2,
            ) -> Option<Contact2> {
                // solid on the +X side: push the other body back -X
                let d = other_xf.position.x - xf.position.x;
                if d < 0.0 {
                    Some(Contact2 {
                        normal: Vec2::new(1.0, 0.0),
                        penetration: -d,
                        point: xf.position,
                    })
                } else {
                    None
                }
            }
            fn contains_point(&self, xf: &Transform2, p: Vec2) -> bool {
                p.x > xf.position.x
            }
            fn bound(&self, _xf: &Transform2) -> Aabb2 {
                Aabb2::from_center_half(Vec2::ZERO, Vec2::splat(1.0e9))
            }
            fn raycast(&self, _xf: &Transform2, _o: Vec2, _d: Vec2, _t: f32) -> Option<f32> {
                None
            }
            fn name(&self) -> &'static str {
                "HalfPlaneRight"
            }
        }
        let custom = ColliderShape2::Custom(Box::new(HalfPlaneRight));
        let xc = Transform2::new(Vec2::new(0.0, 0.0), 0.0);
        let (sc, xs) = circle(1.0, -2.0, 0.0); // circle centre 2 into the solid side... solid is +X: d = -2 → pen 2
        let c = collide(&custom, &xc, &sc, &xs).expect("custom narrowphase");
        assert!(approx_eq(c.penetration, 2.0, EPS));
        assert!(approx_eq(c.normal.x, 1.0, EPS));
    }

    #[test]
    fn wheel_behaves_like_circle() {
        let wheel = ColliderShape2::Wheel {
            radius: 2.0,
            suspension_length: 10.0,
            spring_stiffness: 100.0,
            spring_damping: 5.0,
        };
        let xw = Transform2::new(Vec2::new(0.0, 0.0), 0.0);
        let (sb, xb) = circle(1.0, 2.5, 0.0);
        let c = collide(&wheel, &xw, &sb, &xb).expect("wheel collides like a circle");
        assert!(approx_eq(c.penetration, 0.5, EPS));
    }
}
