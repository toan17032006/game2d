//! Monster Engine sandbox demo.
//!
//! Modes:
//! - `--physics`: physics demo — a static ground, 6 falling balls attached
//!   to sprites via the physics bridge, and a trigger area that counts
//!   overlaps. Works headless (`--headless N`) and in a window.
//! - default (no flags): the original bouncing-grid sprite demo.
//!
//! Headless runs exit 0 after `[frames]` rendered frames (default 120) with
//! the verification checks applied.

use monster_engine::prelude::*;

const WORLD_HALF_W: f32 = 480.0;
const WORLD_HALF_H: f32 = 270.0;
const SPRITE_SIZE: f32 = 48.0;
const GROUND_Y: f32 = 240.0;
const DEFAULT_HEADLESS_FRAMES: u64 = 120;

struct SandboxGame {
    /// Headless frame budget: exit after this many rendered frames.
    headless_frame_limit: Option<u64>,
    frames_rendered: u64,
    /// Physics demo mode (--physics).
    physics_mode: bool,
    /// Simulated fixed steps (physics mode exit condition).
    sim_steps: u32,
    ball_bodies: Vec<usize>,
    trigger_events: u32,
    /// Entity ids of the physics balls (sprites follow via the bridge).
    ball_entities: Vec<EntityId>,
}

impl SandboxGame {
    /// 32x32 solid square with a darker border, tinted per color.
    fn make_sprite_texture(assets: &mut Assets, key: &str, c: Color) -> Handle<Texture> {
        let border = Color::new(c.r / 2, c.g / 2, c.b / 2, 255);
        let mut rgba = Vec::with_capacity(32 * 32 * 4);
        for y in 0..32u32 {
            for x in 0..32u32 {
                let edge = x == 0 || y == 0 || x == 31 || y == 31;
                let px = if edge { border } else { c };
                rgba.extend_from_slice(&px.to_rgba_bytes());
            }
        }
        assets.insert_rgba(key, 32, 32, rgba)
    }
}

impl GameApp for SandboxGame {
    fn init(&mut self, ctx: &mut InitContext<'_>) {
        ctx.camera.position = Vec2::ZERO;

        let textures = [
            ("tex_purple", Color::opaque(146, 110, 205)),
            ("tex_green", Color::opaque(110, 205, 146)),
            ("tex_orange", Color::opaque(230, 165, 90)),
            ("tex_blue", Color::opaque(90, 165, 230)),
        ];
        let handles: Vec<_> = textures
            .iter()
            .map(|(key, c)| Self::make_sprite_texture(ctx.assets, key, *c))
            .collect();

        if self.physics_mode {
            if ctx.physics.is_none() {
                log_error!("--physics requested but physics was not enabled: add with_physics");
                return;
            }
            let physics = ctx.physics.as_mut().unwrap();

            // static ground, authored in the scene and synced to physics
            let ground = physics.add_body(PhysicsBody2::new(
                BodyKind2::Static,
                Vec2::new(0.0, GROUND_Y),
                ColliderShape2::Box {
                    half_extents: Vec2::new(1000.0, 10.0),
                },
            ));
            let ground_entity = ctx
                .world
                .spawn()
                .with(Transform::at(Vec2::new(0.0, GROUND_Y)))
                .with(Sprite {
                    texture: handles[0],
                    size: Vec2::new(2000.0, 20.0),
                    src: None,
                })
                .build();
            physics_attach(&mut ctx.world, ground_entity, ground);

            // trigger area mid-air: balls pass through and fire events
            let _area = physics.add_body(PhysicsBody2::new(
                BodyKind2::Area,
                Vec2::new(0.0, 150.0),
                ColliderShape2::Circle { radius: 40.0 },
            ));

            // falling balls attached to sprites through the bridge
            for i in 0..6 {
                let x = -75.0 + i as f32 * 30.0;
                let body = physics.add_body(
                    PhysicsBody2::new(
                        BodyKind2::Rigid,
                        Vec2::new(x, -40.0 - i as f32 * 18.0),
                        ColliderShape2::Circle { radius: 10.0 },
                    )
                    .with_material(0.6, 0.4),
                );
                self.ball_bodies.push(body);
                let entity = ctx
                    .world
                    .spawn()
                    .with(Transform::at(Vec2::new(x, -40.0 - i as f32 * 18.0)))
                    .with(Sprite {
                        texture: handles[1 + (i % 3)],
                        size: Vec2::splat(20.0),
                        src: None,
                    })
                    .build();
                physics_attach(&mut ctx.world, entity, body);
                self.ball_entities.push(entity);
            }
            log_info!("sandbox physics: ground + 6 balls + trigger area");
            return;
        }

        // original demo: grid of bouncing sprites driven by movement_system
        let cols = 8;
        let rows = 5;
        for row in 0..rows {
            for col in 0..cols {
                let i = row * cols + col;
                let x = (col as f32 - (cols - 1) as f32 / 2.0) * 110.0;
                let y = (row as f32 - (rows - 1) as f32 / 2.0) * 110.0;
                let speed = 60.0 + (i % 7) as f32 * 15.0;
                let dir = Vec2::from_angle(i as f32 * 0.7);
                ctx.world
                    .spawn()
                    .with(Transform::at(Vec2::new(x, y)))
                    .with(Velocity(dir * speed))
                    .with(Sprite {
                        texture: handles[i % handles.len()],
                        size: Vec2::splat(SPRITE_SIZE),
                        src: None,
                    })
                    .with(ZOrder((i % 3) as f32))
                    .build();
            }
        }

        // exercise the audio backend (Null backend in headless/CI: mute)
        let clip = AudioClip::sine(22050, 220.0, 0.4, 6000);
        ctx.audio.play_clip(&clip, 0.5, false);

        log_info!(
            "sandbox initialized: {} textures, {} sprites",
            handles.len(),
            cols * rows
        );
    }

    fn update(&mut self, ctx: &mut UpdateContext<'_>) {
        if ctx.input.was_pressed(KeyCode::Escape) {
            ctx.exit();
        }
        // headless budget: once enough frames were rendered, stop
        if let Some(limit) = self.headless_frame_limit {
            if self.frames_rendered >= limit {
                ctx.exit();
                return;
            }
        }

        if self.physics_mode {
            // collect trigger events from the physics world
            if let Some(physics) = ctx.physics.as_mut() {
                for event in physics.drain_events() {
                    if matches!(
                        event,
                        monster_physics::world::PhysicsEvent2::TriggerEntered { .. }
                    ) {
                        self.trigger_events += 1;
                    }
                }
            }
            // physics progress is measured in simulated steps (4s of sim),
            // not frames — headless renders far faster than the 60Hz clock
            self.sim_steps += 1;
            if self.sim_steps >= 240 {
                ctx.exit();
            }
            return; // sprites follow physics via the bridge (no manual motion)
        }

        movement_system(ctx.world, ctx.dt);

        // bounce sprites off the world bounds: read state first, compute,
        // then write back — keeps the ECS borrows clean
        let half = Vec2::new(
            WORLD_HALF_W - SPRITE_SIZE / 2.0,
            WORLD_HALF_H - SPRITE_SIZE / 2.0,
        );
        struct Bounce {
            id: EntityId,
            position: Vec2,
            velocity: Vec2,
        }
        let mut updates: Vec<Bounce> = Vec::new();
        for id in ctx.world.entities() {
            let Some(transform) = ctx.world.get::<Transform>(id) else {
                continue;
            };
            let Some(vel) = ctx.world.get::<Velocity>(id) else {
                continue;
            };
            let mut v = vel.0;
            let p = transform.position;
            if (p.x < -half.x && v.x < 0.0) || (p.x > half.x && v.x > 0.0) {
                v.x = -v.x;
            }
            if (p.y < -half.y && v.y < 0.0) || (p.y > half.y && v.y > 0.0) {
                v.y = -v.y;
            }
            updates.push(Bounce {
                id,
                position: Vec2::new(p.x.clamp(-half.x, half.x), p.y.clamp(-half.y, half.y)),
                velocity: v,
            });
        }
        for b in updates {
            if let Some(transform) = ctx.world.get_mut::<Transform>(b.id) {
                transform.position = b.position;
            }
            if let Some(vel) = ctx.world.get_mut::<Velocity>(b.id) {
                vel.0 = b.velocity;
            }
        }
    }

    fn render(&mut self, _ctx: &mut RenderContext<'_>) {
        self.frames_rendered += 1;
    }
}

fn main() {
    set_max_level(Level::Info);
    let args: Vec<String> = std::env::args().collect();

    let mut open_window = false;
    let mut headless_frames: Option<u64> = None;
    let mut physics_mode = false;
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--headless" => {
                headless_frames = Some(match args.get(i + 1).and_then(|s| s.parse::<u64>().ok()) {
                    Some(n) => {
                        i += 1; // consumed the count argument
                        n
                    }
                    None => DEFAULT_HEADLESS_FRAMES,
                });
            }
            "--window" => open_window = true,
            "--physics" => physics_mode = true,
            _ => {}
        }
        i += 1;
    }

    let mut builder = if open_window {
        EngineBuilder::new()
            .with_window(WindowOptions {
                width: 960,
                height: 540,
                title: "Monster Engine Sandbox".to_string(),
            })
            .with_physics(Vec2::new(0.0, 980.0))
    } else {
        let mut b = EngineBuilder::new().headless();
        if physics_mode {
            b = b.with_physics(Vec2::new(0.0, 980.0));
        }
        b
    };
    builder = builder.with_fixed_fps(60);

    let mut engine = builder.build().expect("engine build");
    let mut game = SandboxGame {
        headless_frame_limit: if physics_mode { None } else { headless_frames },
        frames_rendered: 0,
        physics_mode,
        sim_steps: 0,
        ball_bodies: Vec::new(),
        trigger_events: 0,
        ball_entities: Vec::new(),
    };

    let result = engine.run(&mut game);

    // verification: headless physics runs must land every ball near the
    // ground surface (y = GROUND_Y - 10) and record trigger events
    let ok = if physics_mode {
        let mut all_landed = true;
        for (i, &body) in game.ball_bodies.iter().enumerate() {
            let y = engine.physics().unwrap().bodies[body].transform.position.y;
            if !(GROUND_Y - 30.0..GROUND_Y).contains(&y) {
                log_warn!("ball {} did not land: y={}", i, y);
                all_landed = false;
            }
        }
        let sprites_follow = game.ball_entities.iter().all(|&entity| {
            let Some(link) = engine.world().get::<PhysicsLink>(entity) else {
                return false;
            };
            let body_pos = engine.physics().unwrap().bodies[link.body]
                .transform
                .position;
            let sprite_pos = engine.world().get::<Transform>(entity).unwrap().position;
            (sprite_pos - body_pos).length() < 1.0
        });
        all_landed && sprites_follow && game.trigger_events > 0
    } else {
        result.frames > 0
    };

    log_info!(
        "sandbox done: frames={} updates={} trigger_events={} ok={}",
        result.frames,
        result.updates,
        game.trigger_events,
        ok
    );
    if !ok {
        std::process::exit(1);
    }
}
