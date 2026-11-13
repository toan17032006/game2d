//! Joints / constraints: Distance, Pin (revolute), Weld (fixed) and Spring.
//!
//! Also provides the builders for **Physical Bone** chains and
//! **Articulated** assemblies — both are rigid bodies (capsule segments)
//! linked by Pin joints to a static anchor.

use monster_math::Vec2;

use crate::body::{BodyKind2, PhysicsBody2};
use crate::shape::{ColliderShape2, Transform2};
use crate::world::PhysicsWorld2;

/// Constraint between two bodies (indices into `PhysicsWorld2.bodies`).
#[derive(Debug, Clone)]
pub enum Joint2 {
    /// Fixed distance between two anchors.
    Distance {
        a: usize,
        b: usize,
        anchor_a: Vec2,
        anchor_b: Vec2,
        length: f32,
    },
    /// Anchors pinned together (revolute in 3D terms; point constraint in 2D).
    Pin {
        a: usize,
        b: usize,
        anchor_a: Vec2,
        anchor_b: Vec2,
    },
    /// Weld: anchors pinned AND relative angle locked.
    Weld {
        a: usize,
        b: usize,
        anchor_a: Vec2,
        anchor_b: Vec2,
    },
    /// Spring: Hooke force + damping towards `rest_length`.
    Spring {
        a: usize,
        b: usize,
        anchor_a: Vec2,
        anchor_b: Vec2,
        rest_length: f32,
        stiffness: f32,
        damping: f32,
    },
}

/// Solve all joints for one step (called by `PhysicsWorld2::step`).
pub fn solve_joints(world: &mut PhysicsWorld2, dt: f32) {
    const ITERATIONS: usize = 3;
    let joints = world.joints.clone();
    let dt = dt.max(1.0e-6);

    for _ in 0..ITERATIONS {
        for joint in &joints {
            match joint {
                Joint2::Spring {
                    a,
                    b,
                    anchor_a,
                    anchor_b,
                    rest_length,
                    stiffness,
                    damping,
                } => {
                    solve_spring(
                        world,
                        *a,
                        *b,
                        *anchor_a,
                        *anchor_b,
                        *rest_length,
                        *stiffness,
                        *damping,
                        dt,
                    );
                }
                Joint2::Distance {
                    a,
                    b,
                    anchor_a,
                    anchor_b,
                    length,
                } => {
                    solve_distance(world, *a, *b, *anchor_a, *anchor_b, *length, dt);
                }
                Joint2::Pin {
                    a,
                    b,
                    anchor_a,
                    anchor_b,
                } => {
                    solve_pin_vector(world, *a, *b, *anchor_a, *anchor_b, dt);
                }
                Joint2::Weld {
                    a,
                    b,
                    anchor_a,
                    anchor_b,
                } => {
                    solve_pin_vector(world, *a, *b, *anchor_a, *anchor_b, dt);
                    lock_angles(world, *a, *b);
                }
            }
        }
    }
}

fn world_anchor(world: &PhysicsWorld2, index: usize, anchor_local: Vec2) -> Option<Vec2> {
    world
        .bodies
        .get(index)
        .map(|b| b.transform.to_world(anchor_local))
}

/// Distance constraint: cancel approach velocity along the axis + bounded
/// positional correction (sign-verified: stretched pulls bodies TOGETHER).
fn solve_distance(
    world: &mut PhysicsWorld2,
    a: usize,
    b: usize,
    anchor_a: Vec2,
    anchor_b: Vec2,
    length: f32,
    _dt: f32,
) {
    let Some(wa) = world_anchor(world, a, anchor_a) else {
        return;
    };
    let Some(wb) = world_anchor(world, b, anchor_b) else {
        return;
    };
    let (inv_a, inv_b) = {
        let (ba, bb) = (&world.bodies[a], &world.bodies[b]);
        (ba.inv_mass(), bb.inv_mass())
    };
    let total = inv_a + inv_b;
    if total <= 0.0 {
        return;
    }
    let delta = wb - wa;
    let dist = delta.length();
    if dist <= 1.0e-9 {
        return;
    }
    let axis = delta / dist;
    let diff = dist - length; // > 0 stretched, < 0 compressed
                              // positional: split correction by inverse mass, clamped per step
    let corr = (diff * 0.8).clamp(-50.0, 50.0);
    let ba = &mut world.bodies[a];
    ba.transform.position += axis * (corr * inv_a / total);
    let bb = &mut world.bodies[b];
    bb.transform.position -= axis * (corr * inv_b / total);
    // velocity: cancel relative approach along the axis (no bias injection)
    let rv = world.bodies[b].velocity - world.bodies[a].velocity;
    let vel_n = rv.dot(axis);
    // j = vel_n / total applied: vB -= axis*j*invB, vA += axis*j*invA
    // (falling weight: vel_n > 0 -> vB moves back UP, cancelling the fall)
    let imp = vel_n / total;
    let ba = &mut world.bodies[a];
    ba.velocity += axis * (imp * inv_a);
    let bb = &mut world.bodies[b];
    bb.velocity -= axis * (imp * inv_b);
}

/// 2D point constraint (Pin): full-vector impulse form.
fn solve_pin_vector(
    world: &mut PhysicsWorld2,
    a: usize,
    b: usize,
    anchor_a: Vec2,
    anchor_b: Vec2,
    dt: f32,
) {
    let Some(wa) = world_anchor(world, a, anchor_a) else {
        return;
    };
    let Some(wb) = world_anchor(world, b, anchor_b) else {
        return;
    };
    let (inv_a, inv_b) = {
        let (ba, bb) = (&world.bodies[a], &world.bodies[b]);
        (ba.inv_mass(), bb.inv_mass())
    };
    let total = inv_a + inv_b;
    if total <= 0.0 {
        return;
    }
    let cv = wb - wa; // constraint violation vector
    let rv = world.bodies[b].velocity - world.bodies[a].velocity;
    let beta = 0.3;
    let bias = cv * (beta / dt.max(1.0e-4));
    let lambda = -(rv + bias) * (1.0 / total);
    let mag = lambda.length();
    let lambda = if mag > 5000.0 {
        lambda * (5000.0 / mag)
    } else {
        lambda
    };
    let ba = &mut world.bodies[a];
    ba.velocity -= lambda * inv_a;
    let bb = &mut world.bodies[b];
    bb.velocity += lambda * inv_b;
    // positional stabilization: A moves TOWARDS B's anchor (+cv), B towards A's
    let fix = cv * 0.2;
    let ba = &mut world.bodies[a];
    ba.transform.position += fix * (inv_a / total);
    let bb = &mut world.bodies[b];
    bb.transform.position -= fix * (inv_b / total);
}

fn lock_angles(world: &mut PhysicsWorld2, a: usize, b: usize) {
    let (inv_a, inv_b) = {
        let (ba, bb) = (&world.bodies[a], &world.bodies[b]);
        (ba.inv_mass(), bb.inv_mass())
    };
    if inv_a + inv_b <= 0.0 {
        return;
    }
    let total = inv_a + inv_b;
    let delta = world.bodies[a].angular_velocity - world.bodies[b].angular_velocity;
    let ba = &mut world.bodies[a];
    ba.angular_velocity -= delta * (inv_a / total);
    let bb = &mut world.bodies[b];
    bb.angular_velocity += delta * (inv_b / total);
    // positional angle correction (half each)
    let angle_diff = world.bodies[a].transform.rotation - world.bodies[b].transform.rotation;
    let ba = &mut world.bodies[a];
    ba.transform.rotation -= angle_diff * 0.2 * (inv_a / total);
    let bb = &mut world.bodies[b];
    bb.transform.rotation += angle_diff * 0.2 * (inv_b / total);
}

fn solve_spring(
    world: &mut PhysicsWorld2,
    a: usize,
    b: usize,
    anchor_a: Vec2,
    anchor_b: Vec2,
    rest_length: f32,
    stiffness: f32,
    damping: f32,
    dt: f32,
) {
    let (Some(wa), Some(wb)) = (
        world_anchor(world, a, anchor_a),
        world_anchor(world, b, anchor_b),
    ) else {
        return;
    };
    let (inv_a, inv_b) = {
        let (ba, bb) = (&world.bodies[a], &world.bodies[b]);
        (ba.inv_mass(), bb.inv_mass())
    };
    if inv_a + inv_b <= 0.0 {
        return;
    }
    let delta = wb - wa;
    let dist = delta.length();
    if dist <= 1.0e-9 {
        return;
    }
    let axis = delta / dist;
    let rv = world.bodies[b].velocity - world.bodies[a].velocity;
    let vel_n = rv.dot(axis);
    // Hooke + damping (force pulling towards rest_length)
    let force = stiffness * (dist - rest_length) + damping * vel_n;
    // apply as impulses along the axis
    let impulse = force * dt;
    let ba = &mut world.bodies[a];
    ba.velocity += axis * (impulse * inv_a);
    let bb = &mut world.bodies[b];
    bb.velocity -= axis * (impulse * inv_b);
}

// -- builders: Physical Bone chains & Articulated assemblies -----------------

/// Add one Physical Bone (capsule segment between `from` and `to`).
pub fn add_bone(world: &mut PhysicsWorld2, from: Vec2, to: Vec2) -> usize {
    let center = (from + to) * 0.5;
    let half_length = (to - from).length() / 2.0;
    let rotation = (to.y - from.y).atan2(to.x - from.x);
    let mut body = PhysicsBody2::new(
        BodyKind2::Bone,
        center,
        ColliderShape2::Stadium {
            half_length,
            radius: 4.0,
        },
    );
    body.transform = Transform2::new(center, rotation);
    world.add_body(body)
}

/// Build an **Articulated** assembly: a static anchor + a chain of bones
/// connected by Pin joints. Returns the bone body indices.
///
/// This is the ragdoll spine/limb primitive: Physical Bone + Articulated.
pub fn build_articulated_chain(
    world: &mut PhysicsWorld2,
    origin: Vec2,
    segment_count: usize,
    segment_length: f32,
) -> Vec<usize> {
    let mut bones = Vec::new();
    let mut prev_end = origin;
    for _ in 0..segment_count {
        let start = prev_end;
        let end = start + Vec2::new(segment_length, 0.0);
        let bone = add_bone(world, start, end);
        bones.push(bone);
        // pin joint at the shared endpoint between consecutive bones
        if let Some(&prev) = bones.iter().rev().nth(1) {
            world.joints.push(Joint2::Pin {
                a: prev,
                b: bone,
                anchor_a: Vec2::new(segment_length / 2.0, 0.0),
                anchor_b: Vec2::new(-segment_length / 2.0, 0.0),
            });
        }
        prev_end = end;
    }
    // anchor the first bone's start point to the world via a static body
    let anchor = PhysicsBody2::new(
        BodyKind2::Static,
        origin,
        ColliderShape2::Circle { radius: 2.0 },
    );
    let anchor_index = world.add_body(anchor);
    world.joints.push(Joint2::Pin {
        a: anchor_index,
        b: bones[0],
        anchor_a: Vec2::ZERO,
        anchor_b: Vec2::new(-segment_length / 2.0, 0.0),
    });
    bones
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gravity() -> Vec2 {
        Vec2::new(0.0, 980.0)
    }

    #[test]
    fn distance_joint_holds_length_under_gravity() {
        let mut world = PhysicsWorld2::new(gravity());
        let anchor = world.add_body(PhysicsBody2::new(
            BodyKind2::Static,
            Vec2::new(0.0, 0.0),
            ColliderShape2::Circle { radius: 2.0 },
        ));
        let weight = world.add_body(
            PhysicsBody2::new(
                BodyKind2::Rigid,
                Vec2::new(0.0, 100.0),
                ColliderShape2::Circle { radius: 6.0 },
            )
            .with_gravity_scale(1.0),
        );
        world.joints.push(Joint2::Distance {
            a: anchor,
            b: weight,
            anchor_a: Vec2::ZERO,
            anchor_b: Vec2::ZERO,
            length: 100.0,
        });

        for _ in 0..90 {
            world.step(1.0 / 60.0);
        }
        let dist = (world.bodies[weight].transform.position
            - world.bodies[anchor].transform.position)
            .length();
        assert!((dist - 100.0).abs() < 6.0, "distance {}", dist);
    }

    #[test]
    fn pin_joint_keeps_anchors_together() {
        let mut world = PhysicsWorld2::new(gravity());
        let a = world.add_body(
            PhysicsBody2::new(
                BodyKind2::Rigid,
                Vec2::new(-20.0, 0.0),
                ColliderShape2::Box {
                    half_extents: Vec2::new(10.0, 5.0),
                },
            )
            .with_gravity_scale(1.0),
        );
        let b = world.add_body(
            PhysicsBody2::new(
                BodyKind2::Rigid,
                Vec2::new(20.0, 0.0),
                ColliderShape2::Box {
                    half_extents: Vec2::new(10.0, 5.0),
                },
            )
            .with_gravity_scale(1.0),
        );
        world.joints.push(Joint2::Pin {
            a,
            b,
            anchor_a: Vec2::new(10.0, 0.0),  // right edge of A
            anchor_b: Vec2::new(-10.0, 0.0), // left edge of B
        });
        for _ in 0..60 {
            world.step(1.0 / 60.0);
        }
        let wa = world.bodies[a].transform.to_world(Vec2::new(10.0, 0.0));
        let wb = world.bodies[b].transform.to_world(Vec2::new(-10.0, 0.0));
        assert!(
            (wa - wb).length() < 8.0,
            "anchors apart by {}",
            (wa - wb).length()
        );
    }

    #[test]
    fn weld_locks_relative_angle() {
        let mut world = PhysicsWorld2::new(Vec2::ZERO);
        let a = world.add_body(
            PhysicsBody2::new(
                BodyKind2::Rigid,
                Vec2::new(0.0, 0.0),
                ColliderShape2::Circle { radius: 5.0 },
            )
            .with_gravity_scale(0.0),
        );
        let b = world.add_body(
            PhysicsBody2::new(
                BodyKind2::Rigid,
                Vec2::new(15.0, 0.0),
                ColliderShape2::Circle { radius: 5.0 },
            )
            .with_gravity_scale(0.0),
        );
        world.bodies[a].angular_velocity = 2.0;
        world.joints.push(Joint2::Weld {
            a,
            b,
            anchor_a: Vec2::new(10.0, 0.0),
            anchor_b: Vec2::new(-5.0, 0.0),
        });
        for _ in 0..30 {
            world.step(1.0 / 60.0);
        }
        let diff = (world.bodies[a].transform.rotation - world.bodies[b].transform.rotation).abs();
        assert!(diff < 0.3, "angle drift {}", diff);
        let dv = (world.bodies[a].angular_velocity - world.bodies[b].angular_velocity).abs();
        assert!(dv < 0.3, "angular velocity mismatch {}", dv);
    }

    #[test]
    fn spring_pulls_towards_rest_length() {
        let mut world = PhysicsWorld2::new(Vec2::ZERO);
        let anchor = world.add_body(PhysicsBody2::new(
            BodyKind2::Static,
            Vec2::new(0.0, 0.0),
            ColliderShape2::Circle { radius: 1.0 },
        ));
        let mass = world.add_body(
            PhysicsBody2::new(
                BodyKind2::Rigid,
                Vec2::new(250.0, 0.0), // stretched far beyond rest 100
                ColliderShape2::Circle { radius: 4.0 },
            )
            .with_gravity_scale(0.0),
        );
        world.joints.push(Joint2::Spring {
            a: anchor,
            b: mass,
            anchor_a: Vec2::ZERO,
            anchor_b: Vec2::ZERO,
            rest_length: 100.0,
            stiffness: 40.0,
            damping: 20.0,
        });
        for _ in 0..240 {
            world.step(1.0 / 60.0);
        }
        let dist = world.bodies[mass].transform.position.x;
        // heavily damped spring settles near the rest length
        assert!(dist > 60.0 && dist < 160.0, "settled at {}", dist);
    }

    #[test]
    fn articulated_chain_hangs_together() {
        let mut world = PhysicsWorld2::new(gravity());
        let bones = build_articulated_chain(&mut world, Vec2::new(0.0, 0.0), 4, 30.0);
        assert_eq!(bones.len(), 4);
        for _ in 0..90 {
            world.step(1.0 / 60.0);
        }
        // consecutive bones stay near their rest separation (30px)
        for i in 0..bones.len() - 1 {
            let d = (world.bodies[bones[i]].transform.position
                - world.bodies[bones[i + 1]].transform.position)
                .length();
            assert!(d < 45.0, "bones {}-{} drifted to {}", i, i + 1, d);
        }
    }

    #[test]
    fn bone_body_uses_stadium() {
        let mut world = PhysicsWorld2::new(Vec2::ZERO);
        let idx = add_bone(&mut world, Vec2::new(0.0, 0.0), Vec2::new(40.0, 0.0));
        match &world.bodies[idx].shape {
            ColliderShape2::Stadium {
                half_length,
                radius,
            } => {
                assert!((half_length - 20.0).abs() < 1e-4);
                assert!((radius - 4.0).abs() < 1e-4);
            }
            other => panic!("bone should be a stadium, got {:?}", other),
        }
        assert_eq!(world.bodies[idx].kind, BodyKind2::Bone);
    }
}
