//! Built-in systems and render-order helpers.

use crate::components::{Sprite, Transform, Velocity, ZOrder};
use crate::world::{EntityId, World};
use monster_core::graphics::Camera2D;
use monster_math::Vec2;
use monster_physics::integrate;

/// Sprite culling: whether a sprite (centre + half extents) intersects the
/// camera viewport (+ `margin` pixels of slack). Pure function — the engine
/// render pass calls this before issuing sprite draws.
pub fn sprite_in_view(center: Vec2, half: Vec2, camera: &Camera2D, margin: f32) -> bool {
    let view_half = Vec2::new(camera.viewport.0, camera.viewport.1) * 0.5 + Vec2::splat(margin);
    let delta = (center - camera.position).abs();
    delta.x <= view_half.x + half.x && delta.y <= view_half.y + half.y
}

/// Smoothly move `camera` towards `target`'s position (+ `offset`).
///
/// Frame-rate independent exponential smoothing: `factor = 1 - exp(-speed*dt)`.
/// `speed` in (0, inf) - larger = snappier. The camera never teleports.
pub fn camera_follow_system(
    camera: &mut Camera2D,
    world: &World,
    target: EntityId,
    offset: Vec2,
    speed: f32,
    dt: f32,
) {
    let Some(target_transform) = world.get::<Transform>(target) else {
        return;
    };
    let desired = target_transform.position + offset;
    let speed = speed.max(0.0);
    let factor = 1.0 - (-speed * dt.max(0.0)).exp();
    camera.position += (desired - camera.position) * factor;
}

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
    use monster_math::{approx_eq, Vec2};

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
    fn sprite_culling_by_camera_viewport() {
        let camera = Camera2D::new(800.0, 600.0); // view half = (400, 300)
                                                  // inside view
        assert!(sprite_in_view(
            Vec2::new(0.0, 0.0),
            Vec2::splat(10.0),
            &camera,
            0.0
        ));
        // partially visible at the edge (sprite half counts)
        assert!(sprite_in_view(
            Vec2::new(405.0, 0.0),
            Vec2::splat(10.0),
            &camera,
            0.0
        ));
        // fully outside
        assert!(!sprite_in_view(
            Vec2::new(500.0, 0.0),
            Vec2::splat(10.0),
            &camera,
            0.0
        ));
        // margin rescues a near-edge sprite (reach = 400 + 64 + 10 = 474)
        assert!(sprite_in_view(
            Vec2::new(460.0, 0.0),
            Vec2::splat(10.0),
            &camera,
            64.0
        ));
        assert!(!sprite_in_view(
            Vec2::new(460.0, 0.0),
            Vec2::splat(10.0),
            &camera,
            0.0
        ));
    }

    #[test]
    fn camera_follow_approaches_target_smoothly() {
        let mut world = World::new();
        let target = world
            .spawn()
            .with(Transform::at(Vec2::new(500.0, 300.0)))
            .build();
        let mut camera = Camera2D::new(800.0, 600.0);
        camera.position = Vec2::new(0.0, 0.0);

        // one small step: camera moves part of the way, never teleports
        camera_follow_system(&mut camera, &world, target, Vec2::ZERO, 8.0, 1.0 / 60.0);
        let after_one = camera.position;
        assert!(
            after_one.x > 0.0 && after_one.x < 500.0,
            "partial step x={}",
            after_one.x
        );
        assert!(
            after_one.y > 0.0 && after_one.y < 300.0,
            "partial step y={}",
            after_one.y
        );

        // converge after enough steps
        for _ in 0..180 {
            camera_follow_system(&mut camera, &world, target, Vec2::ZERO, 8.0, 1.0 / 60.0);
        }
        assert!(
            approx_eq(camera.position.x, 500.0, 1.0),
            "x={}",
            camera.position.x
        );
        assert!(
            approx_eq(camera.position.y, 300.0, 1.0),
            "y={}",
            camera.position.y
        );
    }

    #[test]
    fn camera_follow_offset_and_missing_target() {
        let mut world = World::new();
        let target = world
            .spawn()
            .with(Transform::at(Vec2::new(100.0, 100.0)))
            .build();
        let mut camera = Camera2D::new(800.0, 600.0);
        camera.position = Vec2::new(100.0, 100.0);

        // offset places the camera behind/above the target
        for _ in 0..120 {
            camera_follow_system(
                &mut camera,
                &world,
                target,
                Vec2::new(0.0, -80.0),
                10.0,
                1.0 / 60.0,
            );
        }
        assert!(approx_eq(camera.position.x, 100.0, 1.0));
        assert!(approx_eq(camera.position.y, 20.0, 1.0));

        // despawned target: camera holds position, no panic
        world.despawn(target);
        camera_follow_system(&mut camera, &world, target, Vec2::ZERO, 10.0, 1.0 / 60.0);
        assert!(approx_eq(camera.position.y, 20.0, 1.0));
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
