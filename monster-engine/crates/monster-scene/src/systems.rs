//! Built-in systems and render-order helpers.

use crate::components::{Sprite, Transform, Velocity, ZOrder};
use crate::world::{EntityId, World};
use monster_math::Vec2;
use monster_physics::integrate;

/// Integrate every `Transform + Velocity` pair: `position += velocity * dt`.
///
/// Requires the physics crate's pure function so the integration formula
/// stays defined in exactly one place.
pub fn movement_system(world: &mut World, dt: f32) {
    world.for_each2::<Transform, Velocity, _>(|_id, transform, velocity| {
        integrate(&mut transform.position, velocity.0, dt);
    });
}

/// One drawable entity flattened for the render pass.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpriteDraw {
    pub id: EntityId,
    pub texture: monster_core::handle::Handle<monster_core::graphics::Texture>,
    /// Destination center in world space.
    pub position: Vec2,
    /// Destination size in world units (Transform scale applied).
    pub size: Vec2,
    pub src: Option<monster_math::Rect>,
}

/// Flatten all `Transform + Sprite (+ ZOrder)` entities, sorted by z
/// ascending (far first, near last). The renderer draws in call order, so
/// this list IS the draw order.
pub fn sprite_draw_order(world: &World) -> Vec<SpriteDraw> {
    let mut zipped: Vec<(f32, SpriteDraw)> = Vec::new();
    for id in world.entities() {
        let Some(sprite) = world.get::<Sprite>(id) else {
            continue;
        };
        let Some(transform) = world.get::<Transform>(id) else {
            continue;
        };
        let z = world.get::<ZOrder>(id).map(|z| z.0).unwrap_or(0.0);
        zipped.push((
            z,
            SpriteDraw {
                id,
                texture: sprite.texture,
                position: transform.position,
                size: Vec2::new(
                    sprite.size.x * transform.scale.x,
                    sprite.size.y * transform.scale.y,
                ),
                src: sprite.src,
            },
        ));
    }
    zipped.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    zipped.into_iter().map(|(_, draw)| draw).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::Sprite;
    use monster_core::graphics::Texture;
    use monster_core::handle::Handle;
    use monster_math::Vec2;

    #[test]
    fn movement_integrates_positions() {
        let mut world = World::new();
        let a = world
            .spawn()
            .with(Transform::at(Vec2::new(0.0, 0.0)))
            .with(Velocity(Vec2::new(10.0, -5.0)))
            .build();
        let b = world
            .spawn()
            .with(Transform::at(Vec2::new(1.0, 1.0)))
            .build();
        movement_system(&mut world, 2.0);
        let ta = world.get::<Transform>(a).unwrap();
        assert_eq!(ta.position, Vec2::new(20.0, -10.0));
        let tb = world.get::<Transform>(b).unwrap(); // no velocity: unchanged
        assert_eq!(tb.position, Vec2::new(1.0, 1.0));
    }

    #[test]
    fn draw_order_sorts_by_z() {
        let mut world = World::new();
        let tex = Handle::<Texture>::from_index(0);
        let low = world
            .spawn()
            .with(Transform::at(Vec2::new(0.0, 0.0)))
            .with(Sprite {
                texture: tex,
                size: Vec2::splat(10.0),
                src: None,
            })
            .with(ZOrder(5.0))
            .build();
        let high = world
            .spawn()
            .with(Transform::at(Vec2::new(1.0, 0.0)))
            .with(Sprite {
                texture: tex,
                size: Vec2::splat(10.0),
                src: None,
            })
            .with(ZOrder(-2.0))
            .build();
        let default_z = world
            .spawn()
            .with(Transform::at(Vec2::new(2.0, 0.0)))
            .with(Sprite {
                texture: tex,
                size: Vec2::splat(10.0),
                src: None,
            })
            .build();
        let order = sprite_draw_order(&world);
        let ids: Vec<EntityId> = order.iter().map(|d| d.id).collect();
        // -2.0 first, then default 0.0, then 5.0
        assert_eq!(ids, vec![high, default_z, low]);
        // size scaling applied
        assert_eq!(order[0].size, Vec2::splat(10.0));
    }
}
