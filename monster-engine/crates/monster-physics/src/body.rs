//! Physics bodies: the 14-kind taxonomy, mapped to 2D behaviour.
//!
//! Rigid-body kinds (Static/Rigid/Kinematic/Character/Area/Animatable/Bone)
//! run on the impulse pipeline in [`crate::world`]. Particle-based kinds
//! (Soft/Cloth/Fluid/Particle/Deformable) run position-based dynamics in
//! [`crate::particles`]. Vehicle combines a rigid chassis with wheel
//! suspension ([`crate::vehicle`]).

use monster_math::Vec2;

use crate::shape::{ColliderShape2, Transform2};

/// The 14 body kinds of the engine taxonomy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BodyKind2 {
    /// Never moves, infinite mass, anchors the world.
    Static,
    /// Dynamic body: gravity + forces + collision response.
    Rigid,
    /// Moved by code (velocity set explicitly), unaffected by gravity,
    /// pushes dynamic bodies, unaffected by them.
    Kinematic,
    /// Platformer-style body: `move_and_slide`, no rotation, on-floor state.
    Character,
    /// Detects overlaps and fires trigger events; no collision response.
    Area,
    /// Kinematic body driven by animation; pushes dynamic bodies.
    Animatable,
    /// One bone segment of a ragdoll: capsule rigid body linked by joints.
    Bone,
    /// Soft body: ring/blob of mass-spring particles.
    Soft,
    /// Cloth: grid of particles with structural/shear springs + pins.
    Cloth,
    /// Vehicle: rigid chassis + wheel suspension (see `vehicle.rs`).
    Vehicle,
    /// Articulated: rigid bodies linked by joints (chain/ragdoll assembly).
    Articulated,
    /// Independent point particles (cheap, no inter-particle forces).
    Particle,
    /// Fluid: SPH-lite particle pressure (density repulsion).
    Fluid,
    /// Deformable: shape points spring back to their rest offsets.
    Deformable,
}

impl BodyKind2 {
    /// Whether this kind participates in the rigid-body pipeline.
    pub fn is_rigid_pipeline(self) -> bool {
        matches!(
            self,
            BodyKind2::Static
                | BodyKind2::Rigid
                | BodyKind2::Kinematic
                | BodyKind2::Character
                | BodyKind2::Area
                | BodyKind2::Animatable
                | BodyKind2::Bone
                | BodyKind2::Vehicle
                | BodyKind2::Articulated
        )
    }

    /// Whether gravity is applied by the default integrator.
    pub fn uses_gravity(self) -> bool {
        matches!(
            self,
            BodyKind2::Rigid | BodyKind2::Character | BodyKind2::Bone | BodyKind2::Vehicle
        )
    }

    /// Human-readable name.
    pub fn name(self) -> &'static str {
        match self {
            BodyKind2::Static => "Static",
            BodyKind2::Rigid => "Rigid",
            BodyKind2::Kinematic => "Kinematic",
            BodyKind2::Character => "Character",
            BodyKind2::Area => "Area",
            BodyKind2::Animatable => "Animatable",
            BodyKind2::Bone => "Bone",
            BodyKind2::Soft => "Soft",
            BodyKind2::Cloth => "Cloth",
            BodyKind2::Vehicle => "Vehicle",
            BodyKind2::Articulated => "Articulated",
            BodyKind2::Particle => "Particle",
            BodyKind2::Fluid => "Fluid",
            BodyKind2::Deformable => "Deformable",
        }
    }
}

/// A physics body: kind + transform + shape + material properties.
pub struct PhysicsBody2 {
    pub kind: BodyKind2,
    pub transform: Transform2,
    pub shape: ColliderShape2,
    /// Linear velocity (world space, px/s).
    pub velocity: Vec2,
    /// Angular velocity (rad/s) — integrates the collider orientation.
    pub angular_velocity: f32,
    /// Mass (0 = infinite for Static/Kinematic; computed from density otherwise).
    pub mass: f32,
    /// Bounciness 0..1.
    pub restitution: f32,
    /// Coulomb friction coefficient.
    pub friction: f32,
    /// Multiplier on gravity for this body.
    pub gravity_scale: f32,
    /// Trigger bodies (Area, or trigger flag) detect overlaps without response.
    pub is_trigger: bool,
    /// User tag (gameplay association).
    pub tag: u32,
}

impl PhysicsBody2 {
    /// Builder-style constructor with sane defaults (density 1.0 mass).
    pub fn new(kind: BodyKind2, position: Vec2, shape: ColliderShape2) -> PhysicsBody2 {
        let mass = match kind {
            BodyKind2::Static | BodyKind2::Kinematic => 0.0, // infinite
            _ => shape.mass_from_density(1.0),
        };
        PhysicsBody2 {
            kind,
            transform: Transform2::new(position, 0.0),
            shape,
            velocity: Vec2::ZERO,
            angular_velocity: 0.0,
            mass,
            restitution: 0.0,
            friction: 0.3,
            gravity_scale: 1.0,
            is_trigger: kind == BodyKind2::Area,
            tag: 0,
        }
    }

    /// Set material properties (chained).
    pub fn with_material(mut self, restitution: f32, friction: f32) -> PhysicsBody2 {
        self.restitution = restitution.clamp(0.0, 1.0);
        self.friction = friction.max(0.0);
        self
    }

    pub fn with_velocity(mut self, velocity: Vec2) -> PhysicsBody2 {
        self.velocity = velocity;
        self
    }

    pub fn with_gravity_scale(mut self, scale: f32) -> PhysicsBody2 {
        self.gravity_scale = scale;
        self
    }

    pub fn with_tag(mut self, tag: u32) -> PhysicsBody2 {
        self.tag = tag;
        self
    }

    /// Mark as trigger (Area bodies are triggers by default).
    pub fn as_trigger(mut self) -> PhysicsBody2 {
        self.is_trigger = true;
        self
    }

    /// Inverse mass (0 for infinite-mass bodies).
    pub fn inv_mass(&self) -> f32 {
        if self.mass <= 0.0 {
            0.0
        } else {
            1.0 / self.mass
        }
    }

    /// Whether this body is moved by collision response.
    pub fn responds_to_collision(&self) -> bool {
        !matches!(
            self.kind,
            BodyKind2::Static | BodyKind2::Kinematic | BodyKind2::Animatable | BodyKind2::Area
        )
    }

    /// Whether this body applies response to OTHERS (kinematic pushes).
    pub fn applies_response(&self) -> bool {
        !matches!(self.kind, BodyKind2::Static | BodyKind2::Area)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn body_kinds_cover_taxonomy() {
        let kinds = [
            BodyKind2::Static,
            BodyKind2::Rigid,
            BodyKind2::Kinematic,
            BodyKind2::Character,
            BodyKind2::Area,
            BodyKind2::Animatable,
            BodyKind2::Bone,
            BodyKind2::Soft,
            BodyKind2::Cloth,
            BodyKind2::Vehicle,
            BodyKind2::Articulated,
            BodyKind2::Particle,
            BodyKind2::Fluid,
            BodyKind2::Deformable,
        ];
        assert_eq!(kinds.len(), 14);
        for k in kinds {
            assert!(!k.name().is_empty());
        }
    }

    #[test]
    fn static_body_infinite_mass_no_response() {
        let b = PhysicsBody2::new(
            BodyKind2::Static,
            Vec2::ZERO,
            ColliderShape2::Box {
                half_extents: Vec2::new(10.0, 1.0),
            },
        );
        assert_eq!(b.inv_mass(), 0.0);
        assert!(!b.responds_to_collision());
        assert!(!b.applies_response()); // static does not push
    }

    #[test]
    fn kinematic_pushes_but_not_pushed() {
        let b = PhysicsBody2::new(
            BodyKind2::Kinematic,
            Vec2::ZERO,
            ColliderShape2::Circle { radius: 1.0 },
        );
        assert_eq!(b.mass, 0.0);
        assert!(!b.responds_to_collision());
        assert!(b.applies_response());
    }

    #[test]
    fn rigid_defaults_and_builder() {
        let b = PhysicsBody2::new(
            BodyKind2::Rigid,
            Vec2::new(1.0, 2.0),
            ColliderShape2::Circle { radius: 2.0 },
        )
        .with_material(0.8, 0.1)
        .with_velocity(Vec2::new(3.0, 0.0))
        .with_gravity_scale(0.5)
        .with_tag(7);
        assert!(b.mass > 0.0);
        assert_eq!(b.restitution, 0.8);
        assert_eq!(b.friction, 0.1);
        assert_eq!(b.velocity, Vec2::new(3.0, 0.0));
        assert_eq!(b.gravity_scale, 0.5);
        assert_eq!(b.tag, 7);
        assert!(b.responds_to_collision());
        assert!(b.applies_response());
        assert!(b.inv_mass() > 0.0);
    }

    #[test]
    fn area_is_trigger_by_default() {
        let b = PhysicsBody2::new(
            BodyKind2::Area,
            Vec2::ZERO,
            ColliderShape2::Circle { radius: 5.0 },
        );
        assert!(b.is_trigger);
        let r = PhysicsBody2::new(
            BodyKind2::Rigid,
            Vec2::ZERO,
            ColliderShape2::Circle { radius: 5.0 },
        );
        assert!(!r.is_trigger);
        assert!(r.as_trigger().is_trigger);
    }
}
