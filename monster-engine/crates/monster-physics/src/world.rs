//! The physics world: body storage, the step pipeline and contact events.
//!
//! Pipeline per `step(dt)`:
//! 1. integrate (gravity + velocity) for rigid-pipeline kinds
//! 2. broadphase (AABB pairs, brute force — uniform grid is a drop-in later)
//! 3. narrowphase (shape-level deepest contact per pair)
//! 4. trigger detection (Area overlaps → events, no response)
//! 5. impulse resolution (restitution + Coulomb friction) + positional correction
//!
//! Character bodies are moved explicitly via [`PhysicsWorld2::move_and_slide`].

use monster_math::Vec2;

use crate::body::{BodyKind2, PhysicsBody2};
use crate::collision::collide;
use crate::shape::{ColliderShape2, Contact2, Transform2};

/// Events produced by a physics step.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PhysicsEvent2 {
    /// A trigger (Area) began overlapping another body.
    TriggerEntered { area: usize, other: usize },
    /// A trigger stopped overlapping.
    TriggerExited { area: usize, other: usize },
    /// Two solid bodies began colliding this step.
    CollisionStarted { a: usize, b: usize },
}

/// The physics world: bodies + gravity + events.
pub struct PhysicsWorld2 {
    pub gravity: Vec2,
    pub bodies: Vec<PhysicsBody2>,
    /// Joints/constraints (see `joint.rs`); solved after contacts.
    pub joints: Vec<crate::joint::Joint2>,
    /// Contact persistence for CollisionStarted/Ended edge detection.
    active_contacts: Vec<(usize, usize)>,
    /// Trigger overlap persistence for Entered/Exited edges.
    active_triggers: Vec<(usize, usize)>,
    events: Vec<PhysicsEvent2>,
}

impl Default for PhysicsWorld2 {
    fn default() -> Self {
        Self::new(Vec2::new(0.0, 980.0)) // default gravity: +Y down, ~1m=100px
    }
}

impl PhysicsWorld2 {
    pub fn new(gravity: Vec2) -> PhysicsWorld2 {
        PhysicsWorld2 {
            gravity,
            bodies: Vec::new(),
            joints: Vec::new(),
            active_contacts: Vec::new(),
            active_triggers: Vec::new(),
            events: Vec::new(),
        }
    }

    /// Add a body; returns its index.
    pub fn add_body(&mut self, body: PhysicsBody2) -> usize {
        self.bodies.push(body);
        self.bodies.len() - 1
    }

    /// Drain the event queue (call once per frame after `step`).
    pub fn drain_events(&mut self) -> std::vec::IntoIter<PhysicsEvent2> {
        std::mem::take(&mut self.events).into_iter()
    }

    /// Take full ownership of a body for mutation, then push back.
    /// Convenience for game loops that mutate velocities/positions directly.
    pub fn with_body_mut<R>(
        &mut self,
        index: usize,
        f: impl FnOnce(&mut PhysicsBody2) -> R,
    ) -> Option<R> {
        let Some(body) = self.bodies.get_mut(index) else {
            return None;
        };
        Some(f(body))
    }

    /// One simulation step.
    pub fn step(&mut self, dt: f32) {
        let dt = dt.max(1.0e-6);

        // 1. integrate
        for body in self.bodies.iter_mut() {
            if !body.kind.is_rigid_pipeline() {
                continue;
            }
            body.transform.rotation += body.angular_velocity * dt;
            if body.responds_to_collision() && body.kind.uses_gravity() {
                body.velocity += self.gravity * body.gravity_scale * dt;
            }
            // kinematic/animatable bodies move by their set velocity
            body.transform.position += body.velocity * dt;
        }

        // 2+3. broadphase + narrowphase over rigid-pipeline bodies
        let mut contacts: Vec<(usize, usize, Contact2)> = Vec::new();
        let mut triggers: Vec<(usize, usize)> = Vec::new();
        for i in 0..self.bodies.len() {
            for j in (i + 1)..self.bodies.len() {
                let (a, b) = (&self.bodies[i], &self.bodies[j]);
                let bound_a = a.shape.bound(&a.transform);
                let bound_b = b.shape.bound(&b.transform);
                if !bound_a.intersects(&bound_b) {
                    continue;
                }
                match collide(&a.shape, &a.transform, &b.shape, &b.transform) {
                    Some(contact) => {
                        if a.is_trigger || b.is_trigger {
                            triggers.push((i, j));
                        } else {
                            contacts.push((i, j, contact));
                        }
                    }
                    None => {}
                }
            }
        }

        // 4. trigger events (entered/exited edges)
        for &(area, other) in &triggers {
            let key = (area.min(other), area.max(other));
            if !self.active_triggers.contains(&key) {
                self.active_triggers.push(key);
                self.events.push(PhysicsEvent2::TriggerEntered {
                    area: key.0,
                    other: key.1,
                });
            }
        }
        self.active_triggers.retain(|&key| {
            let still = triggers.iter().any(|&(a, b)| (a.min(b), a.max(b)) == key);
            if !still {
                self.events.push(PhysicsEvent2::TriggerExited {
                    area: key.0,
                    other: key.1,
                });
            }
            still
        });

        // collision started edges
        for &(a, b, _) in &contacts {
            let key = (a, b);
            if !self.active_contacts.contains(&key) {
                self.active_contacts.push(key);
                self.events.push(PhysicsEvent2::CollisionStarted { a, b });
            }
        }
        self.active_contacts
            .retain(|&key| contacts.iter().any(|&(a, b, _)| (a, b) == key));

        // 5. resolve contacts (impulse + positional correction)
        const VELOCITY_ITERATIONS: usize = 4;
        const POSITION_ITERATIONS: usize = 2;
        const BAUMGARTE: f32 = 0.4;
        const SLOP: f32 = 0.05;

        for _ in 0..VELOCITY_ITERATIONS {
            for &(ia, ib, contact) in &contacts {
                let (inv_a, inv_b, rest_a, rest_b, fr_a, fr_b) = {
                    let a = &self.bodies[ia];
                    let b = &self.bodies[ib];
                    (
                        a.inv_mass(),
                        b.inv_mass(),
                        a.restitution,
                        b.restitution,
                        a.friction,
                        b.friction,
                    )
                };
                if inv_a + inv_b <= 0.0 {
                    continue;
                }
                // kinematic bodies act as infinite-mass movers: they respond
                // with 0 but push others; area/trigger skipped earlier
                let (resp_a, resp_b) = {
                    let a = &self.bodies[ia];
                    let b = &self.bodies[ib];
                    (a.responds_to_collision(), b.responds_to_collision())
                };
                let eff_inv_a = if resp_a { inv_a } else { 0.0 };
                let eff_inv_b = if resp_b { inv_b } else { 0.0 };
                if eff_inv_a + eff_inv_b <= 0.0 {
                    continue;
                }

                let n = contact.normal; // A→B
                let rv = self.bodies[ib].velocity - self.bodies[ia].velocity;
                let vel_n = rv.dot(n);
                // kinematic movers impose their velocity on dynamic bodies:
                // treat approaching relative velocity as-is (B approaches A)
                if vel_n >= 0.0 {
                    continue; // separating
                }
                let e = rest_a.max(rest_b).min(0.9);
                let j = -(1.0 + e) * vel_n / (eff_inv_a + eff_inv_b);
                if resp_a {
                    self.bodies[ia].velocity -= n * (j * eff_inv_a);
                }
                if resp_b {
                    self.bodies[ib].velocity += n * (j * eff_inv_b);
                }
                // Coulomb friction along the tangent
                let tangent = n.perp();
                let rv_t = (self.bodies[ib].velocity - self.bodies[ia].velocity).dot(tangent);
                let mut jt = -rv_t / (eff_inv_a + eff_inv_b);
                let mu = (fr_a + fr_b) * 0.5;
                let max_friction = mu * j.abs();
                jt = jt.clamp(-max_friction, max_friction);
                if resp_a {
                    self.bodies[ia].velocity -= tangent * (jt * eff_inv_a);
                }
                if resp_b {
                    self.bodies[ib].velocity += tangent * (jt * eff_inv_b);
                }
            }
        }

        for _ in 0..POSITION_ITERATIONS {
            for &(ia, ib, contact) in &contacts {
                let (eff_a, eff_b) = {
                    let a = &self.bodies[ia];
                    let b = &self.bodies[ib];
                    (
                        if a.responds_to_collision() {
                            a.inv_mass()
                        } else {
                            0.0
                        },
                        if b.responds_to_collision() {
                            b.inv_mass()
                        } else {
                            0.0
                        },
                    )
                };
                let total = eff_a + eff_b;
                if total <= 0.0 {
                    continue;
                }
                let corr = (contact.penetration - SLOP).max(0.0) * BAUMGARTE;
                let n = contact.normal;
                self.bodies[ia].transform.position -= n * (corr * eff_a / total);
                self.bodies[ib].transform.position += n * (corr * eff_b / total);
            }
        }

        // 6. joints
        crate::joint::solve_joints(self, dt);
    }

    /// Character-body movement: move along `velocity` for `dt` seconds,
    /// depenetrating against solid bodies. Returns (landed on floor?, hit
    /// ceiling?, hit wall?) with `on_floor` computed from the last contacts.
    pub fn move_and_slide(
        &mut self,
        character: usize,
        velocity: Vec2,
        dt: f32,
    ) -> CharacterMoveResult {
        let mut result = CharacterMoveResult::default();
        let Some(index) = self
            .bodies
            .get(character)
            .map(|b| b.kind == BodyKind2::Character)
        else {
            return result;
        };
        if !index {
            return result;
        }

        // axis-separated movement (classic platformer style): x then y
        let dx = velocity.x * dt;
        let dy = velocity.y * dt;
        let up_normal = Vec2::new(0.0, -1.0); // floor pushes up

        // horizontal pass
        self.bodies[character].transform.position.x += dx;
        result.wall = self.depenetrate_one(character, None);

        // vertical pass
        self.bodies[character].transform.position.y += dy;
        if dy > 0.0 {
            // moving down: floor contacts stop us
            result.floor = self.depenetrate_one(character, Some(up_normal));
            if result.floor {
                self.bodies[character].velocity.y = 0.0;
            }
        } else {
            result.ceiling = self.depenetrate_one(character, Some(-up_normal));
            if result.ceiling && dy < 0.0 {
                self.bodies[character].velocity.y = 0.0;
            }
        }
        result
    }

    /// Push `index` out of any overlap. `floor_normal` (Some) filters
    /// contacts by normal direction for on-floor/ceiling detection.
    fn depenetrate_one(&mut self, index: usize, floor_normal: Option<Vec2>) -> bool {
        let mut hit = false;
        for _ in 0..3 {
            let mut any = false;
            let xf = self.bodies[index].transform;
            for j in 0..self.bodies.len() {
                if j == index {
                    continue;
                }
                let other = &self.bodies[j];
                if other.is_trigger || other.kind == BodyKind2::Area {
                    continue;
                }
                let Some(contact) = collide(
                    &self.bodies[index].shape,
                    &xf,
                    &other.shape,
                    &other.transform,
                ) else {
                    continue;
                };
                any = true;
                hit = true;
                // contact.normal points from self → other; push self out along -normal
                let mut n = -contact.normal;
                if let Some(want) = floor_normal {
                    if n.dot(want) < 0.5 {
                        continue; // not the direction we asked about
                    }
                }
                self.bodies[index].transform.position += n * contact.penetration;
            }
            if !any {
                break;
            }
        }
        hit
    }

    /// All trigger/area overlaps for `index` this step (post-step query).
    pub fn overlapping(&self, index: usize) -> Vec<usize> {
        let xf = self.bodies[index].transform;
        let shape = &self.bodies[index].shape;
        let mut out = Vec::new();
        for j in 0..self.bodies.len() {
            if j == index {
                continue;
            }
            let bound_a = shape.bound(&xf);
            let bound_b = self.bodies[j].shape.bound(&self.bodies[j].transform);
            if !bound_a.intersects(&bound_b) {
                continue;
            }
            if collide(shape, &xf, &self.bodies[j].shape, &self.bodies[j].transform).is_some() {
                out.push(j);
            }
        }
        out
    }
}

/// Result of [`PhysicsWorld2::move_and_slide`].
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct CharacterMoveResult {
    pub floor: bool,
    pub ceiling: bool,
    pub wall: bool,
}

/// Static helper retained for direct contact queries.
pub fn contact_between(a: &PhysicsBody2, b: &PhysicsBody2) -> Option<Contact2> {
    collide(&a.shape, &a.transform, &b.shape, &b.transform)
}

/// Convenience constructor bridging to `Transform2`.
pub fn transform_at(x: f32, y: f32) -> Transform2 {
    Transform2::new(Vec2::new(x, y), 0.0)
}

#[allow(unused_imports)]
use ColliderShape2 as _ColliderShape2KeepImport;

#[cfg(test)]
mod tests {
    use super::*;

    fn box_shape(w: f32, h: f32) -> ColliderShape2 {
        ColliderShape2::Box {
            half_extents: Vec2::new(w, h),
        }
    }

    fn ground(world: &mut PhysicsWorld2, y: f32) -> usize {
        world.add_body(
            PhysicsBody2::new(
                BodyKind2::Static,
                Vec2::new(0.0, y),
                box_shape(1000.0, 10.0),
            )
            .with_material(0.0, 0.6),
        )
    }

    #[test]
    fn rigid_falls_and_lands_on_static_ground() {
        let mut world = PhysicsWorld2::new(Vec2::new(0.0, 980.0));
        ground(&mut world, 200.0);
        let ball = world.add_body(
            PhysicsBody2::new(
                BodyKind2::Rigid,
                Vec2::new(0.0, 50.0),
                ColliderShape2::Circle { radius: 10.0 },
            )
            .with_material(0.0, 0.5),
        );

        let dt = 1.0 / 60.0;
        for _ in 0..120 {
            world.step(dt);
        }
        let b = &world.bodies[ball];
        // ground surface at y=190 (200-10); ball rests at 190-10=180 (+ slop tolerance)
        assert!(
            b.transform.position.y > 170.0 && b.transform.position.y < 185.0,
            "resting y={}",
            b.transform.position.y
        );
        // vertical velocity settled ~0
        assert!(b.velocity.y.abs() < 60.0, "vy={}", b.velocity.y);
        assert!(b.transform.position.x.abs() < 1.0, "no drift");
    }

    #[test]
    fn bouncing_ball_with_restitution() {
        let mut world = PhysicsWorld2::new(Vec2::new(0.0, 980.0));
        ground(&mut world, 200.0);
        world.add_body(
            PhysicsBody2::new(
                BodyKind2::Rigid,
                Vec2::new(0.0, 100.0),
                ColliderShape2::Circle { radius: 10.0 },
            )
            .with_material(0.9, 0.0),
        );
        let dt = 1.0 / 60.0;
        let mut max_rebound_up = 0.0f32;
        let mut prev_y = 100.0f32;
        for _ in 0..180 {
            world.step(dt);
            let y = world.bodies[1].transform.position.y;
            if y < prev_y - 0.5 && y < 190.0 {
                max_rebound_up = max_rebound_up.max(190.0 - y);
            }
            prev_y = y;
        }
        // restitution 0.9: ball must bounce back up noticeably
        assert!(max_rebound_up > 40.0, "rebound height {}", max_rebound_up);
    }

    #[test]
    fn kinematic_pushes_dynamic() {
        let mut world = PhysicsWorld2::default();
        let pusher = world.add_body(
            PhysicsBody2::new(
                BodyKind2::Kinematic,
                Vec2::new(-50.0, 0.0),
                box_shape(10.0, 10.0),
            )
            .with_velocity(Vec2::new(60.0, 0.0)),
        );
        let pushed = world.add_body(
            PhysicsBody2::new(BodyKind2::Rigid, Vec2::new(0.0, 0.0), box_shape(10.0, 10.0))
                .with_gravity_scale(0.0),
        );

        for _ in 0..60 {
            world.step(1.0 / 60.0);
        }
        // the dynamic box got pushed to +X by the kinematic mover
        let pushed_x = world.bodies[pushed].transform.position.x;
        assert!(pushed_x > 29.0, "pushed x={}", pushed_x);
        assert!(world.bodies[pusher].transform.position.x > pushed_x - 22.0);
    }

    #[test]
    fn area_trigger_events_fire_and_exit() {
        let mut world = PhysicsWorld2::new(Vec2::ZERO);
        let area = world.add_body(PhysicsBody2::new(
            BodyKind2::Area,
            Vec2::new(0.0, 0.0),
            ColliderShape2::Circle { radius: 20.0 },
        ));
        let visitor = world.add_body(
            PhysicsBody2::new(
                BodyKind2::Rigid,
                Vec2::new(60.0, 0.0),
                ColliderShape2::Circle { radius: 5.0 },
            )
            .with_gravity_scale(0.0)
            .with_velocity(Vec2::new(-30.0, 0.0)),
        );

        let mut entered = false;
        let mut exited = false;
        for _ in 0..200 {
            world.step(1.0 / 60.0);
            for event in world.drain_events() {
                match event {
                    PhysicsEvent2::TriggerEntered { area: a, other: o }
                        if a == area && o == visitor =>
                    {
                        entered = true;
                    }
                    PhysicsEvent2::TriggerExited { area: a, other: o }
                        if a == area && o == visitor =>
                    {
                        exited = true;
                    }
                    _ => {}
                }
            }
        }
        assert!(entered, "trigger entered");
        assert!(exited, "trigger exited after passing through");
        // visitor passed straight through (no response)
        assert!(world.bodies[visitor].transform.position.x < -35.0);
    }

    #[test]
    fn animatable_pushes_and_ignores_gravity() {
        let mut world = PhysicsWorld2::new(Vec2::new(0.0, 980.0));
        let anim = world.add_body(
            PhysicsBody2::new(
                BodyKind2::Animatable,
                Vec2::new(-40.0, 0.0),
                box_shape(10.0, 10.0),
            )
            .with_velocity(Vec2::new(40.0, 0.0)),
        );
        let rigid = world.add_body(
            PhysicsBody2::new(BodyKind2::Rigid, Vec2::new(0.0, 0.0), box_shape(10.0, 10.0))
                .with_gravity_scale(0.0),
        );
        for _ in 0..40 {
            world.step(1.0 / 60.0);
        }
        assert!(world.bodies[rigid].transform.position.x > 5.0, "pushed");
        assert!(
            world.bodies[anim].transform.position.y.abs() < 1.0,
            "no gravity on animatable"
        );
    }

    #[test]
    fn contact_between_helper() {
        let a = PhysicsBody2::new(BodyKind2::Static, Vec2::new(0.0, 0.0), box_shape(5.0, 5.0));
        let b = PhysicsBody2::new(BodyKind2::Rigid, Vec2::new(8.0, 0.0), box_shape(5.0, 5.0));
        let c = contact_between(&a, &b).expect("overlapping boxes");
        assert!(approx(c.penetration, 2.0));
    }

    fn approx(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-3
    }
}
