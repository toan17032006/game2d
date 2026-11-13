//! Sprite-sheet animation: component + advance system.
//!
//! Strips are single-row sheets; frame N occupies the texture rect
//! `(N * frame_w, 0, frame_w, frame_h)`. The system writes the current
//! frame rect into [`Sprite::src`] every fixed step.

use monster_engine::prelude::{Rect, Sprite, World};

/// Plays one strip animation (loops forever — every demo animation loops).
/// The paired `Sprite` already carries the texture handle; `Animation`
/// only tracks playback state and publishes the frame rect.
#[derive(Debug, Clone, Copy)]
pub struct Animation {
    pub frame_w: f32,
    pub frame_h: f32,
    pub frames: u32,
    pub fps: f32,
    timer: f32,
    frame: u32,
}

impl Animation {
    pub fn new(frame_w: u32, frame_h: u32, frames: u32, fps: f32) -> Animation {
        Animation {
            frame_w: frame_w as f32,
            frame_h: frame_h as f32,
            frames: frames.max(1),
            fps: fps.max(0.01),
            timer: 0.0,
            frame: 0,
        }
    }

    /// Swap to another strip (e.g. idle <-> run): new geometry and timing,
    /// playback restarts from frame 0.
    pub fn set_strip(&mut self, frame_w: u32, frame_h: u32, frames: u32, fps: f32) {
        self.frame_w = frame_w as f32;
        self.frame_h = frame_h as f32;
        self.frames = frames.max(1);
        self.fps = fps.max(0.01);
        self.timer = 0.0;
        self.frame = 0;
    }

    /// Texture-space rect of the current frame.
    pub fn src_rect(&self) -> Rect {
        Rect::new(
            self.frame as f32 * self.frame_w,
            0.0,
            self.frame_w,
            self.frame_h,
        )
    }
}

/// Advance every `Animation` and publish its frame rect into the paired
/// `Sprite`.
pub fn animation_system(world: &mut World, dt: f32) {
    world.for_each2::<Sprite, Animation, _>(|_id, sprite, anim| {
        anim.timer += dt;
        let step = 1.0 / anim.fps;
        while anim.timer >= step {
            anim.timer -= step;
            anim.frame += 1;
            if anim.frame >= anim.frames {
                anim.frame = 0; // loop
            }
        }
        sprite.src = Some(anim.src_rect());
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use monster_engine::prelude::{Handle, Transform, Vec2};

    #[test]
    fn animation_advances_frames_and_updates_sprite() {
        let mut world = World::new();
        let sheet = Handle::from_index(0);
        world
            .spawn()
            .with(Transform::at(Vec2::ZERO))
            .with(Sprite {
                texture: sheet,
                size: Vec2::splat(64.0),
                src: None,
            })
            .with(Animation::new(64, 64, 4, 2.0))
            .build();

        // At 2 fps, one frame per 0.5s of fixed dt.
        animation_system(&mut world, 0.5);
        animation_system(&mut world, 0.5);
        animation_system(&mut world, 0.5);
        animation_system(&mut world, 0.25); // partial step keeps frame 3
        for id in world.entities() {
            let anim = world.get::<Animation>(id).expect("anim");
            assert_eq!(anim.frame, 3);
            let sprite = world.get::<Sprite>(id).expect("sprite");
            let src = sprite.src.expect("src set");
            assert_eq!((src.position.x, src.position.y, src.size.x, src.size.y), (192.0, 0.0, 64.0, 64.0));
        }
        animation_system(&mut world, 0.25); // wraps to frame 0
        for id in world.entities() {
            let anim = world.get::<Animation>(id).expect("anim");
            assert_eq!(anim.frame, 0);
        }
    }
}
