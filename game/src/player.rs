//! Player control: keyboard input -> `Character` physics body velocity ->
//! engine physics bridge -> scene `Transform`; idle/run animation switching.
//!
//! Uses the newest engine components from main (physics taxonomy +
//! ECS <-> PhysicsWorld2 bridge): the player is a physics-owned
//! `BodyKind2::Character` body, so the simulation drives the player's
//! `Transform` every fixed step and the world's static colliders
//! (buildings, bounds) block movement.
//!
//! Input is abstracted as [`PlayerInput`] so headless runs can drive the
//! same pipeline with a scripted pattern (`--autodrive`) for verification.

use monster_engine::prelude::{
    log_info, EntityId, Handle, InputState, KeyCode, PhysicsWorld2, Sprite, Texture, Vec2, World,
};

use crate::anim::Animation;

/// Abstracted movement input (WASD / arrows in window mode, scripted in
/// autodrive mode).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PlayerInput {
    pub left: bool,
    pub right: bool,
    pub up: bool,
    pub down: bool,
}

impl PlayerInput {
    /// Direction vector in world axes (+x right, +y down), normalized for
    /// diagonals so speed is consistent.
    pub fn dir(self) -> Vec2 {
        let x = (self.right as i8 - self.left as i8) as f32;
        let y = (self.down as i8 - self.up as i8) as f32;
        let dir = Vec2::new(x, y);
        if dir.length_squared() > 1.0 {
            dir.normalize()
        } else {
            dir
        }
    }
}

/// Map engine input state onto player input: WASD or arrow keys.
pub fn player_input_from_state(input: &InputState) -> PlayerInput {
    PlayerInput {
        left: input.is_down(KeyCode::A) || input.is_down(KeyCode::Left),
        right: input.is_down(KeyCode::D) || input.is_down(KeyCode::Right),
        up: input.is_down(KeyCode::W) || input.is_down(KeyCode::Up),
        down: input.is_down(KeyCode::S) || input.is_down(KeyCode::Down),
    }
}

/// The controlled character: physics body index + animation sheets for the
/// idle/run swap.
#[derive(Debug, Clone, Copy)]
pub struct Player {
    pub body: usize,
    pub speed: f32,
    pub idle_sheet: Handle<Texture>,
    pub run_sheet: Handle<Texture>,
    /// (frame_w, frame_h, frames) of the idle strip.
    pub idle_geom: (u32, u32, u32),
    /// (frame_w, frame_h, frames) of the run strip.
    pub run_geom: (u32, u32, u32),
    pub fps: f32,
    /// Whether the run animation is currently playing.
    pub moving: bool,
}

/// Apply player input to the physics body and keep the animation in sync
/// with the movement state.
pub fn player_system(world: &mut World, physics: &mut PhysicsWorld2, input: PlayerInput) {
    let target: Option<(EntityId, Player)> = world
        .entities()
        .find_map(|id| world.get::<Player>(id).map(|p| (id, *p)));
    let Some((id, player)) = target else {
        return;
    };

    let dir = input.dir();
    let moving = dir.length_squared() > 0.0;
    let body = &mut physics.bodies[player.body];
    body.velocity = if moving { dir * player.speed } else { Vec2::ZERO };

    if moving != player.moving {
        let (sheet, geom) = if moving {
            (player.run_sheet, player.run_geom)
        } else {
            (player.idle_sheet, player.idle_geom)
        };
        if let Some(anim) = world.get_mut::<Animation>(id) {
            anim.set_strip(geom.0, geom.1, geom.2, player.fps);
        }
        if let Some(sprite) = world.get_mut::<Sprite>(id) {
            sprite.texture = sheet;
        }
        if let Some(p) = world.get_mut::<Player>(id) {
            p.moving = moving;
        }
    }
}

/// Scripted input sequence for headless verification: run right into the
/// east wall, walk down, run left into the west wall, walk up.
pub struct Autodrive {
    elapsed: f32,
    phase: usize,
    pub done: bool,
}

/// (duration, input, phase name) — the player starts on the clear bottom
/// lane y=190 (below all building colliders), speed 120, so each 6s run
/// reaches a wall (~720px) and the short down/up hops supply vertical
/// movement evidence.
pub const AUTODRIVE_PHASES: [(f32, PlayerInput, &str); 4] = [
    (
        0.4,
        PlayerInput { down: true, right: false, left: false, up: false },
        "down",
    ),
    (
        6.0,
        PlayerInput { right: true, left: false, up: false, down: false },
        "right",
    ),
    (
        8.0,
        PlayerInput { left: true, right: false, up: false, down: false },
        "left",
    ),
    (
        0.4,
        PlayerInput { up: true, right: false, left: false, down: false },
        "up",
    ),
];

impl Autodrive {
    pub fn new() -> Autodrive {
        Autodrive {
            elapsed: 0.0,
            phase: 0,
            done: false,
        }
    }

    /// Current input for this fixed step and whether the whole pattern has
    /// finished.
    pub fn tick(&mut self, dt: f32) -> PlayerInput {
        if self.done {
            return PlayerInput::default();
        }
        self.elapsed += dt;
        let (duration, _input, name) = AUTODRIVE_PHASES[self.phase];
        if self.elapsed >= duration {
            self.elapsed -= duration;
            self.phase += 1;
            if self.phase >= AUTODRIVE_PHASES.len() {
                self.done = true;
                return PlayerInput::default();
            }
            let next_name = AUTODRIVE_PHASES[self.phase].2;
            log_phase(name, "finished");
            log_phase(next_name, "starting");
        }
        AUTODRIVE_PHASES[self.phase].1
    }
}

fn log_phase(name: &str, what: &str) {
    log_info!("autodrive: phase '{name}' {what}");
}

#[cfg(test)]
mod tests {
    use super::*;
    use monster_engine::prelude::{BodyKind2, ColliderShape2, PhysicsBody2, Transform, ZOrder};

    fn spawn_player_world(physics: &mut PhysicsWorld2) -> (World, usize) {
        let body = physics.add_body(
            PhysicsBody2::new(
                BodyKind2::Character,
                Vec2::new(100.0, 50.0),
                ColliderShape2::Box {
                    half_extents: Vec2::new(20.0, 14.0),
                },
            )
            .with_gravity_scale(0.0),
        );
        let mut world = World::new();
        world
            .spawn()
            .with(Transform::at(Vec2::new(100.0, 50.0)))
            .with(Sprite {
                texture: Handle::from_index(0),
                size: Vec2::splat(192.0),
                src: None,
            })
            .with(ZOrder(10.0))
            .with(Animation::new(192, 192, 8, 10.0))
            .with(Player {
                body,
                speed: 120.0,
                idle_sheet: Handle::from_index(0),
                run_sheet: Handle::from_index(1),
                idle_geom: (192, 192, 8),
                run_geom: (192, 192, 6),
                fps: 10.0,
                moving: false,
            })
            .build();
        (world, body)
    }

    #[test]
    fn input_sets_body_velocity() {
        let mut physics = PhysicsWorld2::new(Vec2::ZERO);
        let (mut world, body) = spawn_player_world(&mut physics);

        player_system(
            &mut world,
            &mut physics,
            PlayerInput { right: true, left: false, up: false, down: false },
        );
        assert_eq!(physics.bodies[body].velocity, Vec2::new(120.0, 0.0));

        player_system(&mut world, &mut physics, PlayerInput::default());
        assert_eq!(physics.bodies[body].velocity, Vec2::ZERO);
    }

    #[test]
    fn diagonal_input_is_normalized() {
        let mut physics = PhysicsWorld2::new(Vec2::ZERO);
        let (mut world, body) = spawn_player_world(&mut physics);
        player_system(
            &mut world,
            &mut physics,
            PlayerInput { right: true, down: true, left: false, up: false },
        );
        let v = physics.bodies[body].velocity;
        assert!((v.length() - 120.0).abs() < 0.5, "diagonal speed capped, got {v:?}");
    }

    #[test]
    fn moving_switches_to_run_animation() {
        let mut physics = PhysicsWorld2::new(Vec2::ZERO);
        let (mut world, _) = spawn_player_world(&mut physics);
        let id = world.entities().next().expect("player entity");

        player_system(
            &mut world,
            &mut physics,
            PlayerInput { right: true, left: false, up: false, down: false },
        );
        let anim = world.get::<Animation>(id).expect("anim");
        assert_eq!(anim.frames, 6, "run strip has 6 frames");
        let sprite = world.get::<Sprite>(id).expect("sprite");
        assert_eq!(sprite.texture.index(), 1, "run sheet handle");
        let player = world.get::<Player>(id).expect("player");
        assert!(player.moving);

        player_system(&mut world, &mut physics, PlayerInput::default());
        let anim = world.get::<Animation>(id).expect("anim");
        assert_eq!(anim.frames, 8, "idle strip has 8 frames");
        assert_eq!(world.get::<Player>(id).unwrap().moving, false);
    }

    #[test]
    fn physics_step_moves_character_body() {
        let mut physics = PhysicsWorld2::new(Vec2::ZERO);
        let (mut world, body) = spawn_player_world(&mut physics);
        player_system(
            &mut world,
            &mut physics,
            PlayerInput { right: true, left: false, up: false, down: false },
        );
        physics.step(1.0 / 60.0);
        assert!(
            physics.bodies[body].transform.position.x > 101.0,
            "body integrated velocity"
        );
    }

    #[test]
    fn autodrive_pattern_sequence() {
        let mut ad = Autodrive::new();
        let mut saw_right = false;
        let mut saw_left = false;
        let mut total = 0.0f32;
        let ticks = (0.4f32 + 6.0 + 8.0 + 0.4 + 1.0).ceil() as usize;
        for _ in 0..ticks {
            let input = ad.tick(1.0);
            saw_right |= input.right;
            saw_left |= input.left;
            total += 1.0;
        }
        assert!(saw_right && saw_left, "pattern visits both directions");
        assert!(ad.done, "pattern finished after ~14 ticks");
        assert_eq!(ad.tick(1.0), PlayerInput::default(), "done stays done");
        let _ = total;
    }
}
