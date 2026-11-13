//! Bridge between the scene ECS (`monster-scene`) and `PhysicsWorld2`.
//!
//! Sync direction:
//! - **Physics-owned kinds** (Rigid, Character, Bone, Vehicle, Articulated):
//!   the simulation wins — body transforms are copied into scene `Transform`.
//! - **Authored kinds** (Static, Kinematic, Animatable, Area): the scene wins
//!   — `Transform` is copied into the body so placement stays in ECS.
//!
//! Entities attach to bodies with the [`PhysicsLink`] component.

use monster_physics::body::BodyKind2;
use monster_physics::world::PhysicsWorld2;
use monster_scene::{EntityId, Transform, World};

/// Component linking an entity to a physics body (by index).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PhysicsLink {
    pub body: usize,
}

/// Which side owns the transform for this body kind.
fn physics_owns_transform(kind: BodyKind2) -> bool {
    matches!(
        kind,
        BodyKind2::Rigid
            | BodyKind2::Character
            | BodyKind2::Bone
            | BodyKind2::Vehicle
            | BodyKind2::Articulated
    )
}

/// Attach an entity to a physics body (by index in the world's body list).
pub fn attach(world: &mut World, entity: EntityId, body: usize) -> bool {
    world.insert(entity, PhysicsLink { body })
}

/// Physics-owned bodies: copy body transforms into scene `Transform`s.
pub fn sync_dynamic_to_scene(physics: &PhysicsWorld2, world: &mut World) {
    let ids: Vec<EntityId> = world.entities().collect();
    for id in ids {
        let Some(link) = world.get::<PhysicsLink>(id).copied() else {
            continue;
        };
        let Some(body) = physics.bodies.get(link.body) else {
            continue;
        };
        if !physics_owns_transform(body.kind) {
            continue;
        }
        if let Some(t) = world.get_mut::<Transform>(id) {
            t.position = body.transform.position;
            t.rotation = body.transform.rotation;
        }
    }
}

/// Authored bodies: copy scene `Transform`s into body transforms.
pub fn sync_static_to_physics(world: &World, physics: &mut PhysicsWorld2) {
    let ids: Vec<EntityId> = world.entities().collect();
    for id in ids {
        let Some(link) = world.get::<PhysicsLink>(id).copied() else {
            continue;
        };
        let Some(t) = world.get::<Transform>(id).copied() else {
            continue;
        };
        let Some(body) = physics.bodies.get_mut(link.body) else {
            continue;
        };
        if physics_owns_transform(body.kind) {
            continue;
        }
        body.transform.position = t.position;
        body.transform.rotation = t.rotation;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use monster_math::Vec2;
    use monster_physics::body::BodyKind2;
    use monster_physics::body::PhysicsBody2;
    use monster_physics::shape::ColliderShape2;

    #[test]
    fn dynamic_body_drives_scene_transform() {
        let mut world = World::new();
        let mut physics = PhysicsWorld2::new(Vec2::new(0.0, 980.0));
        let body = physics.add_body(
            PhysicsBody2::new(
                BodyKind2::Rigid,
                Vec2::new(10.0, 20.0),
                ColliderShape2::Circle { radius: 5.0 },
            )
            .with_gravity_scale(0.0)
            .with_velocity(Vec2::new(50.0, 0.0)),
        );
        let entity = world.spawn().with(Transform::at(Vec2::ZERO)).build();
        assert!(attach(&mut world, entity, body));

        physics.step(1.0 / 60.0); // body moves +x
        sync_dynamic_to_scene(&physics, &mut world);

        let t = world.get::<Transform>(entity).unwrap();
        assert!(t.position.x > 10.0, "entity follows physics body");
        assert_eq!(t.position.y, 20.0);
    }

    #[test]
    fn static_scene_transform_drives_body() {
        let mut world = World::new();
        let mut physics = PhysicsWorld2::new(Vec2::ZERO);
        let body = physics.add_body(PhysicsBody2::new(
            BodyKind2::Static,
            Vec2::new(0.0, 0.0),
            ColliderShape2::Box {
                half_extents: Vec2::new(50.0, 5.0),
            },
        ));
        let entity = world
            .spawn()
            .with(Transform::at(Vec2::new(7.0, 9.0)))
            .build();
        attach(&mut world, entity, body);

        sync_static_to_physics(&world, &mut physics);

        assert_eq!(physics.bodies[body].transform.position, Vec2::new(7.0, 9.0));
    }

    #[test]
    fn unlinked_entities_are_untouched() {
        let mut world = World::new();
        let mut physics = PhysicsWorld2::new(Vec2::ZERO);
        let entity = world
            .spawn()
            .with(Transform::at(Vec2::new(1.0, 1.0)))
            .build();
        sync_dynamic_to_scene(&physics, &mut world);
        assert_eq!(
            world.get::<Transform>(entity).unwrap().position,
            Vec2::new(1.0, 1.0)
        );
    }
}
