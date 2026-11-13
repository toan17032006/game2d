//! 2D decorative particles: emitter component + update system.
//!
//! Particles live inside the emitter component (no entity churn); the
//! engine render pass draws them as small colored rects with lifetime
//! color lerp. Bursts are spawned programmatically (e.g. coin pickup).

use crate::components::Transform;
use crate::world::{EntityId, World};
use monster_math::{Color, Vec2};

/// One live particle.
#[derive(Debug, Clone, Copy)]
pub struct Particle2D {
    pub position: Vec2,
    pub velocity: Vec2,
    pub life: f32,
    pub max_life: f32,
}

impl Particle2D {
    /// Color interpolated from start to end over the particle lifetime.
    pub fn color(&self, start: Color, end: Color) -> Color {
        let t = if self.max_life > 0.0 {
            (1.0 - self.life / self.max_life).clamp(0.0, 1.0)
        } else {
            1.0
        };
        start.lerp(end, t)
    }
}

/// Continuous/burst particle emitter attached to an entity (its Transform
/// is the emit origin).
#[derive(Debug, Clone)]
pub struct ParticleEmitter2D {
    /// Continuous emission (particles per second); 0 = bursts only.
    pub rate_per_second: f32,
    pub speed: f32,
    /// Cone half-angle (radians) around the emit direction.
    pub spread_radians: f32,
    pub direction: Vec2,
    pub lifetime: f32,
    pub size: f32,
    pub color_start: Color,
    pub color_end: Color,
    pub gravity: f32,
    pub accumulator: f32,
    pub particles: Vec<Particle2D>,
}

impl Default for ParticleEmitter2D {
    fn default() -> Self {
        ParticleEmitter2D {
            rate_per_second: 0.0,
            speed: 120.0,
            spread_radians: std::f32::consts::FRAC_PI_2,
            direction: Vec2::new(0.0, -1.0),
            lifetime: 0.6,
            size: 4.0,
            color_start: Color::opaque(255, 220, 90),
            color_end: Color::opaque(255, 90, 40),
            gravity: 300.0,
            accumulator: 0.0,
            particles: Vec::new(),
        }
    }
}

impl ParticleEmitter2D {
    /// Spawn `count` particles at `position` fanning around `direction`.
    pub fn burst(&mut self, position: Vec2, direction: Vec2, count: usize) {
        for i in 0..count {
            let step = if count > 1 {
                (i as f32 / (count - 1) as f32 - 0.5) * 2.0 * self.spread_radians
            } else {
                0.0
            };
            let angle = self.direction.y.atan2(self.direction.x) + step;
            let dir = Vec2::new(angle.cos(), angle.sin());
            self.particles.push(Particle2D {
                position,
                velocity: dir * self.speed * (0.6 + 0.4 * (i % 4) as f32 / 3.0),
                life: self.lifetime,
                max_life: self.lifetime,
            });
        }
    }

    /// Live particle count.
    pub fn live_count(&self) -> usize {
        self.particles.len()
    }
}

/// Update every emitter: continuous emission, integration, lifetime cull.
/// Particles do not collide (decorative layer).
pub fn particle_emit_system(world: &mut World, dt: f32) {
    let dt = dt.max(1.0e-6);
    let ids: Vec<EntityId> = world.entities().collect();
    for id in ids {
        let Some(origin) = world.get::<Transform>(id).map(|t| t.position) else {
            continue;
        };
        let Some(emitter) = world.get_mut::<ParticleEmitter2D>(id) else {
            continue;
        };
        // continuous emission
        emitter.accumulator += emitter.rate_per_second * dt;
        while emitter.accumulator >= 1.0 {
            emitter.accumulator -= 1.0;
            emitter.burst(origin, emitter.direction, 1);
        }
        // integrate + cull
        for p in emitter.particles.iter_mut() {
            p.velocity.y += emitter.gravity * dt;
            p.position += p.velocity * dt;
            p.life -= dt;
        }
        emitter.particles.retain(|p| p.life > 0.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::Transform;
    use monster_math::approx_eq;

    #[test]
    fn burst_spawns_and_particles_move_and_die() {
        let mut world = World::new();
        let entity = world
            .spawn()
            .with(Transform::at(Vec2::new(10.0, 10.0)))
            .with(ParticleEmitter2D {
                speed: 100.0,
                lifetime: 0.5,
                gravity: 0.0,
                ..Default::default()
            })
            .build();

        let emitter = world.get_mut::<ParticleEmitter2D>(entity).unwrap();
        emitter.burst(Vec2::new(10.0, 10.0), Vec2::new(1.0, 0.0), 8);
        assert_eq!(emitter.live_count(), 8);

        // after 0.6s (longer than lifetime 0.5) all particles are gone
        for _ in 0..12 {
            particle_emit_system(&mut world, 1.0 / 20.0);
        }
        let emitter = world.get::<ParticleEmitter2D>(entity).unwrap();
        assert_eq!(emitter.live_count(), 0);
    }

    #[test]
    fn continuous_rate_spawns_over_time() {
        let mut world = World::new();
        let entity = world
            .spawn()
            .with(Transform::at(Vec2::ZERO))
            .with(ParticleEmitter2D {
                rate_per_second: 10.0,
                lifetime: 2.0,
                gravity: 0.0,
                speed: 50.0,
                ..Default::default()
            })
            .build();

        for _ in 0..30 {
            particle_emit_system(&mut world, 1.0 / 30.0);
        }
        let emitter = world.get::<ParticleEmitter2D>(entity).unwrap();
        assert!(
            emitter.live_count() > 5,
            "continuous emission produced particles"
        );
    }

    #[test]
    fn color_lerps_over_lifetime() {
        let mut world = World::new();
        let entity = world
            .spawn()
            .with(Transform::at(Vec2::ZERO))
            .with(ParticleEmitter2D {
                color_start: Color::WHITE,
                color_end: Color::BLACK,
                lifetime: 1.0,
                gravity: 0.0,
                ..Default::default()
            })
            .build();
        let emitter = world.get_mut::<ParticleEmitter2D>(entity).unwrap();
        emitter.burst(Vec2::ZERO, Vec2::new(1.0, 0.0), 1);
        let p = emitter.particles[0];
        let fresh = p.color(Color::WHITE, Color::BLACK);
        assert_eq!(fresh, Color::WHITE);
        // half-life: grey midpoint
        let half = Particle2D {
            life: 0.5,
            max_life: 1.0,
            ..p
        };
        let c = half.color(Color::WHITE, Color::BLACK);
        assert!(approx_eq(c.r as f32, 128.0, 1.5));
    }
}
