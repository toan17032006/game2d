//! Built-in components for the 2D foundation.

use monster_core::graphics::Texture;
use monster_core::handle::Handle;
use monster_math::{Rect, Vec2};

/// Position/rotation/scale of an entity in world space.
///
/// `rotation` is radians around the entity's center (reserved for future
/// renderer support — the software backend draws axis-aligned sprites).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform {
    pub position: Vec2,
    pub rotation: f32,
    pub scale: Vec2,
}

impl Default for Transform {
    fn default() -> Self {
        Transform {
            position: Vec2::ZERO,
            rotation: 0.0,
            scale: Vec2::ONE,
        }
    }
}

impl Transform {
    pub fn at(position: Vec2) -> Transform {
        Transform {
            position,
            ..Default::default()
        }
    }
}

/// Linear velocity, integrated by [`crate::systems::movement_system`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Velocity(pub Vec2);

/// Visual: which texture, how big on screen, optional atlas sub-rect.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sprite {
    pub texture: Handle<Texture>,
    /// Destination size in world units (before Transform scale).
    pub size: Vec2,
    /// Optional source rect in texture pixels (spritesheet frames).
    pub src: Option<Rect>,
}

/// Draw priority: higher draws later (on top).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct ZOrder(pub f32);
