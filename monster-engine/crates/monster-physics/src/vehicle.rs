//! Vehicle body: rigid chassis + raycast wheel suspension.
//!
//! Each wheel casts a suspension ray down from its chassis anchor; the
//! spring force (Hooke + damping) pushes the chassis up when compressed.
//! Drive force is applied along the chassis forward axis at the wheels.

use monster_math::Vec2;

use crate::body::{BodyKind2, PhysicsBody2};
use crate::raycast::{raycast, Ray2};
use crate::shape::{ColliderShape2, Transform2};
use crate::world::PhysicsWorld2;

/// One wheel of a vehicle.
#[derive(Debug, Clone)]
pub struct Wheel2 {
    /// Anchor on the chassis (local space).
    pub local_offset: Vec2,
    pub radius: f32,
    pub suspension_length: f32,
    pub spring_stiffness: f32,
    pub spring_damping: f32,
    /// Drive force along chassis forward, per wheel.
    pub drive_force: f32,
    /// Current compression (0..suspension_length), updated each step.
    pub compression: f32,
}

impl Wheel2 {
    pub fn new(local_offset: Vec2) -> Wheel2 {
        Wheel2 {
            local_offset,
            radius: 12.0,
            suspension_length: 20.0,
            spring_stiffness: 30.0,
            spring_damping: 4.0,
            drive_force: 0.0,
            compression: 0.0,
        }
    }
}

/// A vehicle: chassis body index + wheels.
pub struct Vehicle2 {
    /// Index of the chassis body (BodyKind2::Vehicle) in the world.
    pub chassis: usize,
    pub wheels: Vec<Wheel2>,
}

impl Vehicle2 {
    /// Create a chassis + two wheels. Suspension is scaled by the chassis
    /// mass so the vehicle actually hovers under gravity.
    pub fn new(world: &mut PhysicsWorld2, position: Vec2) -> Vehicle2 {
        let chassis = world.add_body(
            PhysicsBody2::new(
                BodyKind2::Vehicle,
                position,
                ColliderShape2::Box {
                    half_extents: Vec2::new(30.0, 10.0),
                },
            )
            .with_gravity_scale(1.0),
        );
        let mass = world.bodies[chassis].mass.max(1.0);
        let mut wheels = vec![
            Wheel2::new(Vec2::new(-20.0, 8.0)),
            Wheel2::new(Vec2::new(20.0, 8.0)),
        ];
        for wheel in wheels.iter_mut() {
            // hover at ~5px compression: k*5 ~= weight (gravity*mass)/2 wheels
            wheel.spring_stiffness = mass * 98.0;
            wheel.spring_damping = mass * 4.0;
        }
        Vehicle2 { chassis, wheels }
    }

    /// Set the drive force (positive = forward / +X of the chassis).
    pub fn drive(&mut self, force: f32) {
        for wheel in self.wheels.iter_mut() {
            wheel.drive_force = force;
        }
    }

    /// One vehicle step: wheel suspension + drive forces on the chassis.
    pub fn step(&mut self, world: &mut PhysicsWorld2, dt: f32) {
        let dt = dt.max(1.0e-6);
        let chassis_xf = world.bodies[self.chassis].transform;
        let chassis_inv = world.bodies[self.chassis].inv_mass();

        for wheel in self.wheels.iter_mut() {
            let anchor_world = chassis_xf.to_world(wheel.local_offset);
            let down = chassis_xf.dir_to_world(Vec2::new(0.0, 1.0));
            let ray = Ray2::new(anchor_world, down, wheel.suspension_length + wheel.radius);

            let compression = match raycast(world, ray) {
                Some(hit) if hit.t <= wheel.suspension_length + wheel.radius => {
                    (wheel.suspension_length + wheel.radius - hit.t)
                        .clamp(0.0, wheel.suspension_length)
                }
                _ => 0.0,
            };
            wheel.compression = compression;

            // spring + damper along chassis-up proportional to compression;
            // damper opposes compression VELOCITY (moving down pushes up harder)
            if compression > 0.0 && chassis_inv > 0.0 {
                let up = -down;
                let compression_rate = -world.bodies[self.chassis].velocity.dot(up);
                let force = (wheel.spring_stiffness * compression
                    + wheel.spring_damping * compression_rate)
                    .max(0.0);
                world.bodies[self.chassis].velocity += up * (force * chassis_inv * dt);
            }

            // drive force along chassis forward (+X local)
            if wheel.drive_force != 0.0 && compression > 0.0 {
                let forward = chassis_xf.dir_to_world(Vec2::new(1.0, 0.0));
                world.bodies[self.chassis].velocity +=
                    forward * (wheel.drive_force * chassis_inv * dt);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use monster_math::approx_eq;

    fn track() -> PhysicsWorld2 {
        let mut world = PhysicsWorld2::new(Vec2::new(0.0, 980.0));
        world.add_body(PhysicsBody2::new(
            BodyKind2::Static,
            Vec2::new(0.0, 200.0),
            ColliderShape2::Box {
                half_extents: Vec2::new(2000.0, 10.0),
            },
        ));
        world
    }

    #[test]
    fn vehicle_settles_on_suspension() {
        let mut world = track();
        let mut vehicle = Vehicle2::new(&mut world, Vec2::new(0.0, 100.0));
        for _ in 0..240 {
            vehicle.step(&mut world, 1.0 / 60.0);
            world.step(1.0 / 60.0);
        }
        let chassis = &world.bodies[vehicle.chassis];
        // wheels (anchor y=+8, suspension 20, radius 12) keep the chassis
        // hovering above the ground surface (y=190) — not resting on it
        let ground_surface = 190.0;
        let chassis_bottom = chassis.transform.position.y + 10.0;
        assert!(
            chassis_bottom < ground_surface - 5.0,
            "chassis should hover on suspension, bottom={}",
            chassis_bottom
        );
        // and it should be roughly level (no crazy rotation)
        assert!(chassis.transform.rotation.abs() < 0.5);
    }

    #[test]
    fn vehicle_drives_forward() {
        let mut world = track();
        let mut vehicle = Vehicle2::new(&mut world, Vec2::new(0.0, 100.0));
        // settle first
        for _ in 0..120 {
            vehicle.step(&mut world, 1.0 / 60.0);
            world.step(1.0 / 60.0);
        }
        let x0 = world.bodies[vehicle.chassis].transform.position.x;
        let mass = world.bodies[vehicle.chassis].mass;
        vehicle.drive(mass * 800.0);
        for _ in 0..90 {
            vehicle.step(&mut world, 1.0 / 60.0);
            world.step(1.0 / 60.0);
        }
        let x1 = world.bodies[vehicle.chassis].transform.position.x;
        assert!(x1 > x0 + 30.0, "vehicle moved {} -> {}", x0, x1);
    }

    #[test]
    fn suspension_compression_tracks_ground() {
        let mut world = track();
        let mut vehicle = Vehicle2::new(&mut world, Vec2::new(0.0, 100.0));
        vehicle.step(&mut world, 1.0 / 60.0);
        // in the air: no compression
        assert!(approx_eq(vehicle.wheels[0].compression, 0.0, 1e-3));
        for _ in 0..200 {
            vehicle.step(&mut world, 1.0 / 60.0);
            world.step(1.0 / 60.0);
        }
        // on the ground: compression > 0
        assert!(
            vehicle.wheels[0].compression > 0.0,
            "wheels compressed on ground"
        );
    }
}
