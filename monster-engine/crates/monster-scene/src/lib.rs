//! Scene layer for `monster-engine`: a lightweight ECS that organizes game
//! state as entities + components + systems.
//!
//! Design rules:
//! - Components are plain data structs; systems are plain functions.
//! - Adding gameplay = adding component structs + system functions; the
//!   [`World`] core never changes.
//! - [`EntityId`] is generational, so stale references after despawn are
//!   detected instead of silently hitting reused storage.

pub mod components;
pub mod systems;
pub mod world;

pub use components::{Sprite, Transform, Velocity, ZOrder};
pub use systems::{movement_system, sprite_draw_order};
pub use world::{EntityBuilder, EntityId, World};
