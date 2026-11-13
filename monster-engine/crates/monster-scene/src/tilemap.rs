//! Tilemap spawning: grid of {Empty, Solid} tiles → sprite entities +
//! static physics bodies (Solid tiles only). The caller attaches entities
//! to bodies through the facade's physics bridge (returns paired indices).

use monster_core::graphics::Texture;
use monster_core::handle::Handle;
use monster_math::Vec2;
use monster_physics::body::{BodyKind2, PhysicsBody2};
use monster_physics::shape::ColliderShape2;
use monster_physics::world::PhysicsWorld2;

use crate::components::{Sprite, Transform};
use crate::world::{EntityId, World};

/// One tile of a map grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tile {
    /// Nothing spawns here.
    Empty,
    /// Sprite + static collision body.
    Solid,
}

/// Counts produced by [`spawn_tilemap`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TileSpawnResult {
    pub sprite_entities: usize,
    pub solid_bodies: usize,
}

/// Spawn a tile grid row-major (`tiles[row * cols + col]`). Solid tiles get
/// a sprite entity + a static body; Empty tiles spawn nothing. Returns the
/// counts and `(entity, body_index)` pairs for physics-bridge attachment
/// by the caller (the bridge component lives in the facade crate).
pub fn spawn_tilemap(
    world: &mut World,
    physics: &mut PhysicsWorld2,
    texture: Handle<Texture>,
    tile_size: f32,
    origin: Vec2,
    cols: usize,
    rows: usize,
    tiles: &[Tile],
) -> (TileSpawnResult, Vec<(EntityId, usize)>) {
    let mut result = TileSpawnResult::default();
    let mut pairs = Vec::new();
    for row in 0..rows {
        for col in 0..cols {
            let tile = match tiles.get(row * cols + col) {
                Some(t) => *t,
                None => continue,
            };
            if tile == Tile::Empty {
                continue;
            }
            let position = origin + Vec2::new(col as f32 * tile_size, row as f32 * tile_size);
            let body = physics.add_body(PhysicsBody2::new(
                BodyKind2::Static,
                position,
                ColliderShape2::Box {
                    half_extents: Vec2::splat(tile_size / 2.0),
                },
            ));
            let entity = world
                .spawn()
                .with(Transform::at(position))
                .with(Sprite {
                    texture,
                    size: Vec2::splat(tile_size),
                    src: None,
                })
                .build();
            result.sprite_entities += 1;
            result.solid_bodies += 1;
            pairs.push((entity, body));
        }
    }
    (result, pairs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spawns_solids_skips_empty() {
        let mut world = World::new();
        let mut physics = PhysicsWorld2::new(Vec2::ZERO);
        let tex = Handle::<Texture>::from_index(0);
        let tiles = [Tile::Solid, Tile::Empty, Tile::Empty, Tile::Solid];
        let (r, pairs) = spawn_tilemap(
            &mut world,
            &mut physics,
            tex,
            32.0,
            Vec2::new(0.0, 0.0),
            2,
            2,
            &tiles,
        );
        assert_eq!(r.sprite_entities, 2);
        assert_eq!(r.solid_bodies, 2);
        assert_eq!(pairs.len(), 2);
        assert_eq!(world.entity_count(), 2);
        assert_eq!(physics.bodies.len(), 2);
        // second solid tile sits to the right of the origin
        assert!(physics.bodies[1].transform.position.x > 0.0);
        assert!(pairs
            .iter()
            .all(|&(e, b)| world.is_alive(e) && b < physics.bodies.len()));
    }

    #[test]
    fn multi_layer_spawn_coexists() {
        let mut world = World::new();
        let mut physics = PhysicsWorld2::new(Vec2::ZERO);
        let tex = Handle::<Texture>::from_index(0);
        // layer 1: background solid tiles at origin
        let layer1 = [Tile::Solid, Tile::Solid];
        let (r1, pairs1) = spawn_tilemap(
            &mut world,
            &mut physics,
            tex,
            32.0,
            Vec2::new(-100.0, 0.0),
            2,
            1,
            &layer1,
        );
        // layer 2: foreground solid tiles offset to the right
        let layer2 = [Tile::Solid, Tile::Solid];
        let (r2, pairs2) = spawn_tilemap(
            &mut world,
            &mut physics,
            tex,
            32.0,
            Vec2::new(500.0, 100.0),
            2,
            1,
            &layer2,
        );
        assert_eq!(r1.solid_bodies, 2);
        assert_eq!(r2.solid_bodies, 2);
        assert_eq!(world.entity_count(), 4);
        assert_eq!(physics.bodies.len(), 4);
        // layers do not overlap positions
        for &(_, b1) in &pairs1 {
            for &(_, b2) in &pairs2 {
                assert_ne!(
                    physics.bodies[b1].transform.position, physics.bodies[b2].transform.position,
                    "layer tiles overlapped"
                );
            }
        }
        assert!(pairs1
            .iter()
            .chain(pairs2.iter())
            .all(|&(e, _)| world.is_alive(e)));
    }

    #[test]
    fn all_empty_spawns_nothing() {
        let mut world = World::new();
        let mut physics = PhysicsWorld2::new(Vec2::ZERO);
        let tex = Handle::<Texture>::from_index(0);
        let tiles = [Tile::Empty, Tile::Empty];
        let (r, pairs) = spawn_tilemap(
            &mut world,
            &mut physics,
            tex,
            32.0,
            Vec2::ZERO,
            2,
            1,
            &tiles,
        );
        assert_eq!(r.sprite_entities, 0);
        assert_eq!(r.solid_bodies, 0);
        assert!(pairs.is_empty());
        assert_eq!(world.entity_count(), 0);
    }
}
