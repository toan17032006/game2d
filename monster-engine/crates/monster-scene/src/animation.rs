//! Sprite animation: clips sliced from spritesheet grids + the animator
//! system that drives `Sprite.src` over time.
//!
//! Tiny Swords sheets have no metadata, so clips are built by grid-slicing:
//! every frame is `frame_w x frame_h`, laid out row-major; columns =
//! `sheet_width / frame_w`. The Warrior sheets are 192x192 per frame
//! (Idle 8 frames at 1536x192, Run 4 frames at 768x192).

use crate::components::Sprite;
use crate::world::EntityId;
use crate::world::World;
use monster_math::{Rect, Vec2};

/// A sliceable animation clip: per-frame source offsets inside a texture.
#[derive(Debug, Clone, PartialEq)]
pub struct AnimationClip {
    /// Full spritesheet size in pixels.
    pub texture_size: Vec2,
    /// One frame's size in pixels.
    pub frame_size: Vec2,
    /// Per-frame top-left offsets in pixels.
    pub frames: Vec<Vec2>,
    /// Seconds per frame.
    pub frame_time: f32,
    /// Loop back to frame 0 at the end (else clamp on the last frame).
    pub looping: bool,
}

impl AnimationClip {
    /// Slice `frame_count` frames laid out row-major on a
    /// `sheet_width x sheet_height` sheet, each `frame_w x frame_h`.
    pub fn from_grid(
        sheet_width: f32,
        sheet_height: f32,
        frame_w: f32,
        frame_h: f32,
        frame_count: usize,
        fps: f32,
        looping: bool,
    ) -> AnimationClip {
        let cols = ((sheet_width / frame_w.max(1.0)) as usize).max(1);
        let frames = (0..frame_count)
            .map(|i| {
                let col = (i % cols) as f32;
                let row = (i / cols) as f32;
                Vec2::new(col * frame_w, row * frame_h)
            })
            .collect();
        AnimationClip {
            texture_size: Vec2::new(sheet_width, sheet_height),
            frame_size: Vec2::new(frame_w, frame_h),
            frames,
            frame_time: (1.0 / fps.max(0.01)).min(10.0),
            looping,
        }
    }

    pub fn frame_count(&self) -> usize {
        self.frames.len()
    }

    /// Source rect of `frame` (clamped to the valid range).
    pub fn source_rect(&self, frame: usize) -> Rect {
        let i = frame.min(self.frames.len().saturating_sub(1));
        Rect {
            position: self.frames[i],
            size: self.frame_size,
        }
    }
}

/// Component driving `Sprite.src` through an [`AnimationClip`].
#[derive(Debug, Clone)]
pub struct Animator {
    pub clip: AnimationClip,
    pub time: f32,
    pub frame: usize,
    pub playing: bool,
}

impl Animator {
    pub fn new(clip: AnimationClip) -> Animator {
        Animator {
            clip,
            time: 0.0,
            frame: 0,
            playing: true,
        }
    }

    /// Pause on the current frame.
    pub fn pause(&mut self) {
        self.playing = false;
    }

    pub fn resume(&mut self) {
        self.playing = true;
    }
}

/// Advance every `Animator` and write the current frame's source rect into
/// the entity's `Sprite`. Looping clips wrap; non-looping clips clamp on
/// the last frame and stop advancing.
pub fn sprite_animation_system(world: &mut World, dt: f32) {
    let dt = dt.max(0.0);
    let ids: Vec<EntityId> = world.entities().collect();
    for id in ids {
        let Some(animator) = world.get_mut::<Animator>(id) else {
            continue;
        };
        if !animator.playing {
            continue;
        }
        let count = animator.clip.frame_count();
        if count == 0 || animator.clip.frame_time <= 0.0 {
            continue;
        }
        animator.time += dt;
        while animator.time >= animator.clip.frame_time {
            animator.time -= animator.clip.frame_time;
            let next = animator.frame + 1;
            if next >= count {
                if animator.clip.looping {
                    animator.frame = 0;
                } else {
                    animator.frame = count - 1;
                    animator.playing = false;
                    break;
                }
            } else {
                animator.frame = next;
            }
        }
        // write the frame's source rect into the sprite
        let rect = animator.clip.source_rect(animator.frame);
        if let Some(sprite) = world.get_mut::<Sprite>(id) {
            sprite.src = Some(rect);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::Transform;
    use crate::world::World;
    use monster_core::graphics::Texture;
    use monster_core::handle::Handle;
    use monster_math::approx_eq;

    fn sheet() -> AnimationClip {
        // 2x2 grid on a 64x64 sheet, 4 frames at 2 fps
        AnimationClip::from_grid(64.0, 64.0, 32.0, 32.0, 4, 2.0, true)
    }

    #[test]
    fn from_grid_slices_row_major() {
        let clip = sheet();
        assert_eq!(clip.frame_count(), 4);
        assert_eq!(clip.frames[0], Vec2::new(0.0, 0.0));
        assert_eq!(clip.frames[1], Vec2::new(32.0, 0.0));
        assert_eq!(clip.frames[2], Vec2::new(0.0, 32.0)); // row 2
        assert_eq!(clip.frames[3], Vec2::new(32.0, 32.0));
        assert_eq!(clip.frame_time, 0.5); // 2 fps
    }

    #[test]
    fn source_rect_clamps() {
        let clip = sheet();
        let r = clip.source_rect(9); // out of range -> last frame
        assert_eq!(r.position, Vec2::new(32.0, 32.0));
        assert_eq!(r.size, Vec2::new(32.0, 32.0));
    }

    #[test]
    fn looping_animation_wraps() {
        let mut world = World::new();
        let tex = Handle::<Texture>::from_index(0);
        let entity = world
            .spawn()
            .with(Transform::at(Vec2::ZERO))
            .with(Sprite {
                texture: tex,
                size: Vec2::splat(32.0),
                src: None,
            })
            .with(Animator::new(sheet()))
            .build();

        // each step 0.6s = 1.2 frame-times: after 2 steps -> frame 2, carry 0.2
        sprite_animation_system(&mut world, 0.6);
        sprite_animation_system(&mut world, 0.6);
        let a = world.get::<Animator>(entity).unwrap();
        assert!(approx_eq(a.time, 0.2, 1e-4));
        sprite_animation_system(&mut world, 0.6);
        let a = world.get::<Animator>(entity).unwrap();
        assert_eq!(a.frame, 3);
        // sprite src updated to frame 3's rect
        let s = world.get::<Sprite>(entity).unwrap();
        assert_eq!(
            s.src,
            Some(Rect {
                position: Vec2::new(32.0, 32.0),
                size: Vec2::new(32.0, 32.0)
            })
        );

        // keep looping: 6 total steps = 3.6s = 7.2 frame-times -> frame 7 % 4 = 3
        for _ in 0..3 {
            sprite_animation_system(&mut world, 0.6);
        }
        let a = world.get::<Animator>(entity).unwrap();
        assert_eq!(a.frame, 3, "wrapped past the end back into the ring");
    }

    #[test]
    fn non_looping_clamps_and_stops() {
        let mut world = World::new();
        let tex = Handle::<Texture>::from_index(0);
        let entity = world
            .spawn()
            .with(Sprite {
                texture: tex,
                size: Vec2::splat(32.0),
                src: None,
            })
            .with(Animator::new(AnimationClip::from_grid(
                64.0, 64.0, 32.0, 32.0, 4, 2.0, false,
            )))
            .build();

        for _ in 0..10 {
            sprite_animation_system(&mut world, 0.6);
        }
        let a = world.get::<Animator>(entity).unwrap();
        assert_eq!(a.frame, 3, "clamped on the last frame");
        assert!(!a.playing, "stopped advancing");
        let frame_before = a.frame;
        sprite_animation_system(&mut world, 0.6);
        let a = world.get::<Animator>(entity).unwrap();
        assert_eq!(a.frame, frame_before);
    }

    #[test]
    fn pause_freezes_the_frame() {
        let mut world = World::new();
        let tex = Handle::<Texture>::from_index(0);
        let entity = world
            .spawn()
            .with(Sprite {
                texture: tex,
                size: Vec2::splat(32.0),
                src: None,
            })
            .with(Animator::new(sheet()))
            .build();
        world.get_mut::<Animator>(entity).unwrap().pause();

        sprite_animation_system(&mut world, 0.6);
        sprite_animation_system(&mut world, 0.6);
        let a = world.get::<Animator>(entity).unwrap();
        assert_eq!(a.frame, 0);
        assert_eq!(a.time, 0.0);
    }
}
