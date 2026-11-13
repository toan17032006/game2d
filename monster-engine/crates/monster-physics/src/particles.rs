//! Particle-based bodies: Soft, Cloth, Particle, Fluid and Deformable.
//!
//! All five kinds are position-based dynamics (PBD) over a shared particle
//! system: gravity integration → spring/constraint satisfaction →
//! collision against world shapes → velocity update.

use monster_math::Vec2;

use crate::body::BodyKind2;
use crate::collision::collide;
use crate::shape::{ColliderShape2, Contact2, Transform2};
use crate::world::PhysicsWorld2;

/// One particle of a particle-based body.
#[derive(Debug, Clone, Copy)]
pub struct Particle2 {
    pub position: Vec2,
    pub velocity: Vec2,
    /// Pinned particles never integrate (cloth attachments).
    pub pinned: bool,
}

/// A distance constraint between two particles.
#[derive(Debug, Clone, Copy)]
pub struct Spring2 {
    pub a: usize,
    pub b: usize,
    pub rest_length: f32,
    /// 0..1 — how strongly the constraint is enforced per iteration.
    pub stiffness: f32,
}

/// Kind of the particle-based body (drives per-step behaviour).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParticleKind2 {
    /// Soft body: closed ring of particles with neighbour + cross springs.
    Soft,
    /// Cloth: WxH grid, structural + shear springs, optional pins.
    Cloth,
    /// Independent particles (cheap rain/debris).
    Particle,
    /// Fluid: SPH-lite density repulsion between neighbours.
    Fluid,
    /// Deformable: particles spring back to their rest offsets.
    Deformable,
}

/// A particle-based body: Soft / Cloth / Particle / Fluid / Deformable.
pub struct ParticleBody2 {
    pub kind: ParticleKind2,
    pub particles: Vec<Particle2>,
    pub springs: Vec<Spring2>,
    /// Rest offsets for Deformable bodies (target positions relative to center).
    pub rest_offsets: Vec<Vec2>,
    /// World anchor of the body origin (Deformable follows this).
    pub origin: Vec2,
    pub gravity_scale: f32,
    /// Radius used for particle-vs-world collisions (visual thickness).
    pub particle_radius: f32,
    /// Fluid neighbour radius (density repulsion).
    pub fluid_radius: f32,
    /// Fluid pressure strength.
    pub fluid_pressure: f32,
    /// Velocity damping per step (air drag).
    pub damping: f32,
    /// Spring constraint iterations per step.
    pub iterations: usize,
}

impl ParticleBody2 {
    fn new(kind: ParticleKind2) -> ParticleBody2 {
        ParticleBody2 {
            kind,
            particles: Vec::new(),
            springs: Vec::new(),
            rest_offsets: Vec::new(),
            origin: Vec2::ZERO,
            gravity_scale: 1.0,
            particle_radius: 3.0,
            fluid_radius: 12.0,
            fluid_pressure: 800.0,
            damping: 0.02,
            iterations: 3,
        }
    }

    /// Soft body: closed ring of `count` particles around `center`.
    pub fn soft_blob(center: Vec2, radius: f32, count: usize) -> ParticleBody2 {
        let mut body = ParticleBody2::new(ParticleKind2::Soft);
        for i in 0..count {
            let angle = (i as f32) / count as f32 * std::f32::consts::TAU;
            body.particles.push(Particle2 {
                position: center + Vec2::from_angle(angle) * radius,
                velocity: Vec2::ZERO,
                pinned: false,
            });
        }
        // neighbour springs + cross springs for volume
        for i in 0..count {
            let j = (i + 1) % count;
            let rest = (body.particles[i].position - body.particles[j].position).length();
            body.springs.push(Spring2 {
                a: i,
                b: j,
                rest_length: rest,
                stiffness: 0.9,
            });
            let k = (i + count / 2) % count;
            let rest_cross = (body.particles[i].position - body.particles[k].position).length();
            body.springs.push(Spring2 {
                a: i,
                b: k,
                rest_length: rest_cross,
                stiffness: 0.4,
            });
        }
        body
    }

    /// Cloth: grid of `cols x rows` particles, structural + shear springs.
    /// The top row is pinned.
    pub fn cloth(top_left: Vec2, spacing: f32, cols: usize, rows: usize) -> ParticleBody2 {
        let mut body = ParticleBody2::new(ParticleKind2::Cloth);
        for row in 0..rows {
            for col in 0..cols {
                body.particles.push(Particle2 {
                    position: top_left + Vec2::new(col as f32 * spacing, row as f32 * spacing),
                    velocity: Vec2::ZERO,
                    pinned: row == 0,
                });
            }
        }
        let idx = |col: usize, row: usize| row * cols + col;
        for row in 0..rows {
            for col in 0..cols {
                // structural: right + down
                if col + 1 < cols {
                    let a = idx(col, row);
                    let b = idx(col + 1, row);
                    let rest = (body.particles[a].position - body.particles[b].position).length();
                    body.springs.push(Spring2 {
                        a,
                        b,
                        rest_length: rest,
                        stiffness: 0.9,
                    });
                }
                if row + 1 < rows {
                    let a = idx(col, row);
                    let b = idx(col, row + 1);
                    let rest = (body.particles[a].position - body.particles[b].position).length();
                    body.springs.push(Spring2 {
                        a,
                        b,
                        rest_length: rest,
                        stiffness: 0.9,
                    });
                }
                // shear: diagonal
                if col + 1 < cols && row + 1 < rows {
                    let a = idx(col, row);
                    let b = idx(col + 1, row + 1);
                    let rest = (body.particles[a].position - body.particles[b].position).length();
                    body.springs.push(Spring2 {
                        a,
                        b,
                        rest_length: rest,
                        stiffness: 0.4,
                    });
                }
            }
        }
        body
    }

    /// Rain/debris: `count` independent particles in a box region.
    pub fn particle_cloud(region_min: Vec2, region_max: Vec2, count: usize) -> ParticleBody2 {
        let mut body = ParticleBody2::new(ParticleKind2::Particle);
        let mut seed = 0x9E3779B9u32;
        let mut rand = move || {
            seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
            (seed >> 8) as f32 / 16_777_216.0
        };
        for _ in 0..count {
            let x = region_min.x + rand() * (region_max.x - region_min.x);
            let y = region_min.y + rand() * (region_max.y - region_min.y);
            body.particles.push(Particle2 {
                position: Vec2::new(x, y),
                velocity: Vec2::ZERO,
                pinned: false,
            });
        }
        body
    }

    /// Fluid: block of particles (SPH-lite pressure keeps volume locally).
    pub fn fluid_block(top_left: Vec2, spacing: f32, cols: usize, rows: usize) -> ParticleBody2 {
        let mut body = ParticleBody2::new(ParticleKind2::Fluid);
        for row in 0..rows {
            for col in 0..cols {
                body.particles.push(Particle2 {
                    position: top_left + Vec2::new(col as f32 * spacing, row as f32 * spacing),
                    velocity: Vec2::ZERO,
                    pinned: false,
                });
            }
        }
        body
    }

    /// Deformable: a box outline whose particles spring back to rest offsets.
    pub fn deformable_box(center: Vec2, half_extents: Vec2, edge_points: usize) -> ParticleBody2 {
        let mut body = ParticleBody2::new(ParticleKind2::Deformable);
        let count = edge_points.max(4);
        for i in 0..count {
            let t = i as f32 / count as f32 * 4.0;
            let offset = match t as u32 {
                0 => Vec2::new(-half_extents.x + 2.0 * half_extents.x * t, -half_extents.y),
                1 => Vec2::new(
                    half_extents.x,
                    -half_extents.y + 2.0 * half_extents.y * (t - 1.0),
                ),
                2 => Vec2::new(
                    half_extents.x - 2.0 * half_extents.x * (t - 2.0),
                    half_extents.y,
                ),
                _ => Vec2::new(
                    -half_extents.x,
                    half_extents.y - 2.0 * half_extents.y * (t - 3.0),
                ),
            };
            body.particles.push(Particle2 {
                position: center + offset,
                velocity: Vec2::ZERO,
                pinned: false,
            });
            body.rest_offsets.push(offset);
            // spring back to the previous particle (ring shape memory)
            if i > 0 {
                let rest = (body.particles[i].position - body.particles[i - 1].position).length();
                body.springs.push(Spring2 {
                    a: i,
                    b: i - 1,
                    rest_length: rest,
                    stiffness: 0.8,
                });
            }
        }
        let first_last = (body.particles[0].position - body.particles[count - 1].position).length();
        body.springs.push(Spring2 {
            a: 0,
            b: count - 1,
            rest_length: first_last,
            stiffness: 0.8,
        });
        body.origin = center;
        body
    }

    /// Step the particle system: integrate → constraints → collide.
    pub fn step(&mut self, dt: f32, gravity: Vec2, world: &PhysicsWorld2) {
        let dt = dt.max(1.0e-6);

        // 1. integrate velocities + positions
        // Anti-tunneling: cap the per-step displacement below the particle
        // radius so the discrete collision probe cannot skip a surface.
        let max_speed = self.particle_radius * 0.8 / dt;
        for p in self.particles.iter_mut() {
            if p.pinned {
                continue;
            }
            p.velocity += gravity * self.gravity_scale * dt;
            p.velocity = p.velocity * (1.0 - self.damping);
            if p.velocity.length() > max_speed {
                p.velocity = p.velocity.normalize_or(Vec2::DOWN) * max_speed;
            }
            p.position += p.velocity * dt;
        }

        // snapshot positions before constraints — total per-step displacement
        // is bounded afterwards (anti-tunneling for constraint pushes too)
        let before: Vec<Vec2> = self.particles.iter().map(|p| p.position).collect();

        // 2. kind-specific constraints (pre-collision)
        match self.kind {
            ParticleKind2::Fluid => self.apply_fluid_pressure(),
            ParticleKind2::Deformable => self.apply_deform_memory(),
            _ => {}
        }

        // 3. spring constraint satisfaction (PBD, several iterations)
        for _ in 0..self.iterations {
            for spring in &self.springs {
                let (pa, pb) = {
                    let a = &self.particles[spring.a];
                    let b = &self.particles[spring.b];
                    (a.position, b.position)
                };
                let delta = pb - pa;
                let dist = delta.length();
                if dist <= 1.0e-9 {
                    continue;
                }
                let diff = (dist - spring.rest_length) / dist;
                let move_vec = delta * (diff * spring.stiffness * 0.5);
                let pinned_a = self.particles[spring.a].pinned;
                let pinned_b = self.particles[spring.b].pinned;
                match (pinned_a, pinned_b) {
                    (false, false) => {
                        self.particles[spring.a].position += move_vec;
                        self.particles[spring.b].position -= move_vec;
                    }
                    (false, true) => {
                        self.particles[spring.a].position += move_vec * 2.0;
                    }
                    (true, false) => {
                        self.particles[spring.b].position -= move_vec * 2.0;
                    }
                    (true, true) => {}
                }
            }
        }

        // 4. bound the CONSTRAINT-induced displacement from the start of the
        // step (fluid pressure / springs must not teleport particles) —
        // collision push is applied after and is never clamped, so solids
        // always win
        let max_displacement = self.particle_radius * 1.5;
        for (i, p) in self.particles.iter_mut().enumerate() {
            if p.pinned {
                continue;
            }
            let delta = p.position - before[i];
            if delta.length() > max_displacement {
                p.position = before[i] + delta.normalize_or(Vec2::DOWN) * max_displacement;
                p.velocity = p.velocity * 0.5;
            }
        }

        // 5. collide particles against world shapes (final say, unclamped)
        for p in self.particles.iter_mut() {
            if p.pinned {
                continue;
            }
            let push = particle_vs_world(p.position, self.particle_radius, world);
            if let Some((normal, penetration)) = push {
                p.position += normal * penetration;
                // kill velocity into the surface (+ slight friction)
                let vel_n = p.velocity.dot(-normal);
                if vel_n > 0.0 {
                    p.velocity += normal * vel_n * 0.9; // remove most normal velocity
                }
                p.velocity = p.velocity * 0.9; // surface friction
            }
        }
        let _ = dt;
    }
}

/// Push a point particle out of every world shape. Returns the strongest
/// push (normal out of the solid, penetration depth).
pub(crate) fn particle_vs_world(
    point: Vec2,
    radius: f32,
    world: &PhysicsWorld2,
) -> Option<(Vec2, f32)> {
    let mut best: Option<(Vec2, f32)> = None;
    let probe = ColliderShape2::Circle { radius };
    let xf = Transform2::new(point, 0.0);
    for body in &world.bodies {
        if body.is_trigger || body.kind == BodyKind2::Area {
            continue;
        }
        let bound_a = probe.bound(&xf);
        let bound_b = body.shape.bound(&body.transform);
        if !bound_a.intersects(&bound_b) {
            continue;
        }
        let Some(contact) = collide(&probe, &xf, &body.shape, &body.transform) else {
            continue;
        };
        let Contact2 {
            normal,
            penetration,
            ..
        } = contact;
        // contact.normal A(particle)→B(solid): particle is pushed along -normal
        let push = -normal;
        let depth = penetration;
        if best.map(|(_bn, bd)| depth > bd).unwrap_or(true) {
            best = Some((push, depth));
        }
    }
    best
}

impl ParticleBody2 {
    /// SPH-lite: repel particles that are closer than `fluid_radius`.
    fn apply_fluid_pressure(&mut self) {
        let n = self.particles.len();
        let mut pushes = vec![Vec2::ZERO; n];
        for i in 0..n {
            for j in (i + 1)..n {
                let d = self.particles[j].position - self.particles[i].position;
                let dist = d.length();
                if dist > 1.0e-6 && dist < self.fluid_radius {
                    let strength = self.fluid_pressure * (1.0 - dist / self.fluid_radius);
                    let push = (d / dist) * strength * 0.5;
                    pushes[i] -= push;
                    pushes[j] += push;
                }
            }
        }
        for (i, push) in pushes.into_iter().enumerate() {
            self.particles[i].position += push * 0.016; // scaled to a step
        }
    }

    /// Deformable memory: particles spring back towards origin + rest offset.
    fn apply_deform_memory(&mut self) {
        for (i, p) in self.particles.iter_mut().enumerate() {
            if p.pinned {
                continue;
            }
            let Some(rest) = self.rest_offsets.get(i).copied() else {
                continue;
            };
            let target = self.origin + rest;
            let delta = target - p.position;
            p.position += delta * 0.1; // shape memory pull-back
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::body::PhysicsBody2;
    use monster_math::approx_eq;

    fn flat_ground() -> PhysicsWorld2 {
        let mut world = PhysicsWorld2::new(Vec2::new(0.0, 980.0));
        world.add_body(PhysicsBody2::new(
            BodyKind2::Static,
            Vec2::new(0.0, 200.0),
            ColliderShape2::Box {
                half_extents: Vec2::new(1000.0, 10.0),
            },
        ));
        world
    }

    #[test]
    fn soft_blob_falls_stays_connected() {
        let mut world = flat_ground();
        let mut blob = ParticleBody2::soft_blob(Vec2::new(0.0, 50.0), 30.0, 16);
        for _ in 0..120 {
            blob.step(1.0 / 60.0, world.gravity, &world);
        }
        // all particles above the ground surface (y=190) and still a ring
        for p in &blob.particles {
            assert!(p.position.y < 195.0, "particle sank: {}", p.position.y);
        }
        // ring stayed roughly intact: spread (max-min distance) reasonable
        let mut max_d = 0.0f32;
        for i in 0..blob.particles.len() {
            for j in (i + 1)..blob.particles.len() {
                max_d =
                    max_d.max((blob.particles[i].position - blob.particles[j].position).length());
            }
        }
        assert!(max_d < 120.0, "blob exploded apart: spread {}", max_d);
    }

    #[test]
    fn cloth_pinned_top_hangs() {
        let mut world = PhysicsWorld2::new(Vec2::new(0.0, 980.0));
        let mut cloth = ParticleBody2::cloth(Vec2::new(0.0, 0.0), 10.0, 8, 12);
        let pinned_positions: Vec<Vec2> = cloth
            .particles
            .iter()
            .filter(|p| p.pinned)
            .map(|p| p.position)
            .collect();
        for _ in 0..120 {
            cloth.step(1.0 / 60.0, world.gravity, &world);
        }
        // pins never moved
        let pins_now: Vec<Vec2> = cloth
            .particles
            .iter()
            .filter(|p| p.pinned)
            .map(|p| p.position)
            .collect();
        for (a, b) in pinned_positions.iter().zip(pins_now.iter()) {
            assert!(approx_eq(a.x, b.x, 1e-3) && approx_eq(a.y, b.y, 1e-3));
        }
        // cloth hangs: bottom row fell well below the top
        let first_row_y = cloth.particles[0].position.y;
        let last_row_y = cloth.particles[cloth.particles.len() - 1].position.y;
        assert!(last_row_y > first_row_y + 60.0, "cloth did not hang");
    }

    #[test]
    fn particle_cloud_lands_on_ground() {
        let mut world = flat_ground();
        let mut cloud =
            ParticleBody2::particle_cloud(Vec2::new(-50.0, 0.0), Vec2::new(50.0, 20.0), 40);
        for _ in 0..180 {
            cloud.step(1.0 / 60.0, world.gravity, &world);
        }
        for p in &cloud.particles {
            assert!(
                p.position.y < 193.0,
                "particle fell through: {}",
                p.position.y
            );
            assert!(
                p.position.y > 150.0,
                "particle still falling: {}",
                p.position.y
            );
        }
    }

    #[test]
    fn fluid_stays_on_ground_and_spreads() {
        let mut world = flat_ground();
        let mut fluid = ParticleBody2::fluid_block(Vec2::new(-30.0, 120.0), 8.0, 8, 6);
        let start_width = {
            let min = fluid
                .particles
                .iter()
                .map(|p| p.position.x)
                .fold(f32::INFINITY, f32::min);
            let max = fluid
                .particles
                .iter()
                .map(|p| p.position.x)
                .fold(f32::NEG_INFINITY, f32::max);
            max - min
        };
        for _ in 0..150 {
            fluid.step(1.0 / 60.0, world.gravity, &world);
        }
        for p in &fluid.particles {
            assert!(p.position.y < 193.0, "fluid particle fell through");
        }
        // pressure spreads the block wider than it started
        let min = fluid
            .particles
            .iter()
            .map(|p| p.position.x)
            .fold(f32::INFINITY, f32::min);
        let max = fluid
            .particles
            .iter()
            .map(|p| p.position.x)
            .fold(f32::NEG_INFINITY, f32::max);
        let width = max - min;
        assert!(
            width > start_width,
            "fluid did not spread: {} -> {}",
            start_width,
            width
        );
    }

    #[test]
    fn deformable_squashes_then_recovers_shape() {
        let mut world = flat_ground();
        let mut deformable =
            ParticleBody2::deformable_box(Vec2::new(0.0, 150.0), Vec2::new(30.0, 20.0), 16);
        let rest_width = {
            let min = deformable
                .particles
                .iter()
                .map(|p| p.position.x)
                .fold(f32::INFINITY, f32::min);
            let max = deformable
                .particles
                .iter()
                .map(|p| p.position.x)
                .fold(f32::NEG_INFINITY, f32::max);
            max - min
        };
        // drop it
        for _ in 0..90 {
            deformable.step(1.0 / 60.0, world.gravity, &world);
        }
        // it rests above the ground without falling through
        for p in &deformable.particles {
            assert!(p.position.y < 193.0, "particle fell through ground");
        }
        // and the ring width recovered towards the rest width (shape memory)
        let min = deformable
            .particles
            .iter()
            .map(|p| p.position.x)
            .fold(f32::INFINITY, f32::min);
        let max = deformable
            .particles
            .iter()
            .map(|p| p.position.x)
            .fold(f32::NEG_INFINITY, f32::max);
        let width = max - min;
        assert!(
            width > rest_width * 0.6,
            "shape lost: {} vs rest {}",
            width,
            rest_width
        );
    }
}
