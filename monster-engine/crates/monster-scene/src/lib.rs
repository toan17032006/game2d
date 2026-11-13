//! Scene layer for `monster-engine`: a lightweight ECS that organizes game
//! state as entities + components + systems.
//!
//! Design rules:
//! - Components are plain data structs; systems are plain functions.
//! - Adding gameplay = adding component structs + system functions; the
//!   [`World`] core never changes.
//! - [`EntityId`] is generational, so stale references after despawn are
//!   detected instead of silently hitting reused storage.

pub mod animation;
pub mod components;
pub mod particles2d;
pub mod systems;
pub mod tilemap;
pub mod world;

pub use animation::{sprite_animation_system, AnimationClip, Animator};
pub use components::{Sprite, Transform, Velocity, ZOrder};
pub use particles2d::{particle_emit_system, Particle2D, ParticleEmitter2D};
pub use systems::{camera_follow_system, movement_system, sprite_draw_order, sprite_in_view};
pub use tilemap::{spawn_tilemap, Tile, TileSpawnResult};
pub use world::{EntityBuilder, EntityId, World};
