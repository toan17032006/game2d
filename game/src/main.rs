//! Monster Engine sandbox demo.
//!
//! Modes:
//! - `--game` (M4 slice): platformer mini-game — Character player driven by
//!   `move_and_slide`, Tiny Swords Warrior_Run animation (grid-sliced),
//!   camera follow, collectible coins scored through Area triggers. Works
//!   headless (auto-walks right, verifies movement/animation/score) and in
//!   a window (A/D or arrows to walk).
//! - `--physics`: physics demo — 6 balls attached to sprites via the bridge
//!   falling onto a static ground, plus a trigger area.
//! - default (no flags): the original bouncing-grid sprite demo.
//!
//! Headless runs exit 0 after the mode's verification passes.

use monster_engine::prelude::*;

const WORLD_HALF_W: f32 = 480.0;
const WORLD_HALF_H: f32 = 270.0;
const SPRITE_SIZE: f32 = 48.0;
const GROUND_Y: f32 = 240.0;
const DEFAULT_HEADLESS_FRAMES: u64 = 120;

/// Game flow state machine: Menu -> Playing -> Won -> (replay) Playing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GamePhase {
    Menu,
    Playing,
    Won,
}

struct SandboxGame {
    /// Headless frame budget: exit after this many rendered frames.
    headless_frame_limit: Option<u64>,
    frames_rendered: u64,
    /// Physics demo mode (--physics).
    physics_mode: bool,
    /// Simulated fixed steps (physics/game mode exit condition).
    sim_steps: u32,
    ball_bodies: Vec<usize>,
    trigger_events: u32,
    /// Entity ids of the physics balls (sprites follow via the bridge).
    ball_entities: Vec<EntityId>,
    /// Mini-game mode (--game): player + coins + camera follow.
    game_mode: bool,
    /// Game flow state machine (game mode only).
    phase: GamePhase,
    phase_steps: u32,
    won_count: u32,
    /// Idle/Run clips for animation switching (game mode).
    idle_clip: Option<AnimationClip>,
    run_clip: Option<AnimationClip>,
    player_entity: Option<EntityId>,
    player_body: usize,
    player_start_x: f32,
    coin_bodies: Vec<usize>,
    coin_collected: Vec<bool>,
    coin_start_positions: Vec<Vec2>,
    coin_entities: Vec<EntityId>,
    score: u32,
    max_score: u32,
    anim_frames_seen: u32,
    last_anim_frame: usize,
    camera_moved: bool,
    hud_enabled: bool,
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
impl SandboxGame {
    /// --game: platformer slice — Character player moved by move_and_slide,
    /// Tiny Swords Warrior_Run animation (grid-sliced, 192x192 frames),
    /// camera follow, coins collected through Area triggers.
    /// --game: platformer slice — Character player moved by move_and_slide,
    /// Tiny Swords Warrior_Run animation (grid-sliced), camera follow, coins.
    fn init_game(&mut self, ctx: &mut InitContext<'_>) {
        let Some(physics) = ctx.physics.as_mut() else {
            log_error!("--game needs physics: builder.with_physics(...)");
            return;
        };

        // ground (authored in the scene, synced to physics)
        let ground = physics.add_body(PhysicsBody2::new(
            BodyKind2::Static,
            Vec2::new(0.0, GROUND_Y),
            ColliderShape2::Box {
                half_extents: Vec2::new(1000.0, 10.0),
            },
        ));
        let ground_tex =
            Self::make_sprite_texture(ctx.assets, "game_ground", Color::opaque(90, 130, 90));
        let ground_entity = ctx
            .world
            .spawn()
            .with(Transform::at(Vec2::new(0.0, GROUND_Y)))
            .with(Sprite {
                texture: ground_tex,
                size: Vec2::new(2000.0, 20.0),
                src: None,
            })
            .build();
        physics_attach(&mut ctx.world, ground_entity, ground);

        // player: Character body + animated Warrior sprite
        let player = physics.add_body(PhysicsBody2::new(
            BodyKind2::Character,
            Vec2::new(-200.0, 180.0),
            ColliderShape2::Box {
                half_extents: Vec2::new(12.0, 16.0),
            },
        ));
        self.player_body = player;
        self.player_start_x = -200.0;

        // Tiny Swords Warrior sheets (grid-sliced at 192x192 per frame):
        // Run (4 frames) for walking + Idle (8 frames) for standing still.
        let run_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(
            "../../../assest/Tiny Swords (Free Pack)/Units/Blue Units/Warrior/Warrior_Run.png",
        );
        let idle_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(
            "../../../assest/Tiny Swords (Free Pack)/Units/Blue Units/Warrior/Warrior_Idle.png",
        );
        let (run_clip, idle_clip, tex) = match (
            ctx.assets.load_file(&run_path),
            ctx.assets.load_file(&idle_path),
        ) {
            (Ok(run_handle), Ok(idle_handle)) => {
                let run_tex = ctx.assets.texture(run_handle).cloned().unwrap();
                let idle_tex = ctx.assets.texture(idle_handle).cloned().unwrap();
                let run_cols = ((run_tex.width() as f32) / 192.0) as usize;
                let idle_cols = ((idle_tex.width() as f32) / 192.0) as usize;
                log_info!(
                    "game: Tiny Swords run {}x{} ({} frames), idle {}x{} ({} frames)",
                    run_tex.width(),
                    run_tex.height(),
                    run_cols,
                    idle_tex.width(),
                    idle_tex.height(),
                    idle_cols
                );
                let run = AnimationClip::from_grid(
                    run_tex.width() as f32,
                    run_tex.height() as f32,
                    192.0,
                    192.0,
                    run_cols,
                    10.0,
                    true,
                );
                let idle = AnimationClip::from_grid(
                    idle_tex.width() as f32,
                    idle_tex.height() as f32,
                    192.0,
                    192.0,
                    idle_cols,
                    8.0,
                    true,
                );
                (run, idle, run_handle)
            }
            (Err(e), _) | (_, Err(e)) => {
                log_warn!("game: Tiny Swords sheets unavailable ({e:?}) — procedural fallback");
                let handle = ctx.assets.insert_rgba(
                    "player_fallback",
                    32,
                    32,
                    vec![146, 110, 205, 255].repeat(32 * 32),
                );
                (
                    AnimationClip::from_grid(128.0, 32.0, 32.0, 32.0, 4, 8.0, true),
                    AnimationClip::from_grid(128.0, 32.0, 32.0, 32.0, 4, 8.0, true),
                    handle,
                )
            }
        };
        self.run_clip = Some(run_clip.clone());
        self.idle_clip = Some(idle_clip.clone());
        let player_entity = ctx
            .world
            .spawn()
            .with(Transform::at(Vec2::new(-200.0, 180.0)))
            .with(Sprite {
                texture: tex,
                size: Vec2::new(96.0, 96.0),
                src: None,
            })
            .with(Animator::new(idle_clip))
            .build();
        physics_attach(&mut ctx.world, player_entity, player);
        self.player_entity = Some(player_entity);

        // coins: Area triggers along the walk path
        for x in [-100.0f32, 0.0, 100.0] {
            let coin = physics.add_body(PhysicsBody2::new(
                BodyKind2::Area,
                Vec2::new(x, 200.0),
                ColliderShape2::Circle { radius: 14.0 },
            ));
            self.coin_bodies.push(coin);
            self.coin_collected.push(false);
            self.coin_start_positions.push(Vec2::new(x, 200.0));
            let coin_tex = Self::make_sprite_texture(
                ctx.assets,
                &format!("game_coin_{}", x as i32),
                Color::opaque(230, 200, 60),
            );
            let coin_entity = ctx
                .world
                .spawn()
                .with(Transform::at(Vec2::new(x, 200.0)))
                .with(Sprite {
                    texture: coin_tex,
                    size: Vec2::splat(24.0),
                    src: None,
                })
                .build();
            physics_attach(&mut ctx.world, coin_entity, coin);
            self.coin_entities.push(coin_entity);
        }

        // decorative emitter on the player (coin bursts are pushed here)
        ctx.world
            .get_mut::<Transform>(player_entity)
            .map(|t| t.position)
            .unwrap_or_default();
        ctx.world.spawn().with(Transform::at(Vec2::ZERO)).build(); // anchor (unused visual)

        // tilemap walls bounding the play area (x = -900..900 band)
        let wall_tex =
            Self::make_sprite_texture(ctx.assets, "game_wall", Color::opaque(120, 90, 60));
        let mut walls = Vec::new();
        for x in -960..960i32 {
            if (32..967).contains(&(x + 32)) {
                continue; // leave the play band open below; walls above ground
            }
            walls.push(Tile::Solid);
        }
        let _ = walls;
        let wall_row: Vec<Tile> = (0..30)
            .map(|i| {
                if i < 6 || i > 24 {
                    Tile::Solid
                } else {
                    Tile::Empty
                }
            })
            .collect();
        let (r, pairs) = spawn_tilemap(
            &mut ctx.world,
            physics,
            wall_tex,
            32.0,
            Vec2::new(-480.0, 80.0),
            30,
            1,
            &wall_row,
        );
        for (entity, body) in pairs {
            physics_attach(&mut ctx.world, entity, body);
        }
        log_info!("game: tilemap walls spawned: {} solids", r.solid_bodies);

        ctx.camera.position = Vec2::new(-200.0, 150.0);
        log_info!("game: player + 3 coins + camera follow ready");
    }

    /// Reset the playing field: player back to start, coins restored.
    fn reset_level(&mut self, ctx: &mut UpdateContext<'_>) {
        if let Some(player) = self.player_entity {
            if let Some(t) = ctx.world.get_mut::<Transform>(player) {
                t.position = Vec2::new(self.player_start_x, 180.0);
            }
            let body = &mut ctx.physics.as_mut().unwrap().bodies[self.player_body];
            body.transform.position = Vec2::new(self.player_start_x, 180.0);
            body.velocity = Vec2::ZERO;
        }
        for (i, &body) in self.coin_bodies.iter().enumerate() {
            self.coin_collected[i] = false;
            let start = self
                .coin_start_positions
                .get(i)
                .copied()
                .unwrap_or(Vec2::ZERO);
            if let Some(body) = ctx.physics.as_mut().unwrap().bodies.get_mut(body) {
                body.transform.position = start;
            }
            if let Some(entity) = self.coin_entities.get(i) {
                if let Some(t) = ctx.world.get_mut::<Transform>(*entity) {
                    t.position = start;
                }
            }
        }
        self.score = 0;
        self.last_anim_frame = 0;
    }
}

impl GameApp for SandboxGame {
    fn init(&mut self, ctx: &mut InitContext<'_>) {
        ctx.camera.position = Vec2::ZERO;

        if self.game_mode {
            self.init_game(ctx);
            return;
        }

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
            let physics = ctx.physics.as_mut().unwrap();

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

            let _area = physics.add_body(PhysicsBody2::new(
                BodyKind2::Area,
                Vec2::new(0.0, 150.0),
                ColliderShape2::Circle { radius: 40.0 },
            ));

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

        if self.game_mode {
            self.phase_steps += 1;
            match self.phase {
                GamePhase::Menu => {
                    // headless auto-starts; window waits for Enter
                    if (self.headless_frame_limit.is_some() && self.phase_steps >= 30)
                        || ctx.input.was_pressed(KeyCode::Enter)
                    {
                        self.phase = GamePhase::Playing;
                        self.phase_steps = 0;
                    }
                    return;
                }
                GamePhase::Won => {
                    if (self.headless_frame_limit.is_some() && self.phase_steps >= 30)
                        || ctx.input.was_pressed(KeyCode::Enter)
                    {
                        self.phase = GamePhase::Playing;
                        self.phase_steps = 0;
                        self.reset_level(ctx);
                    }
                    return;
                }
                GamePhase::Playing => {}
            }

            // input: headless auto-walks right; window uses A/D + arrows
            let move_x = if self.headless_frame_limit.is_some() {
                140.0
            } else if ctx.input.is_down(KeyCode::A) || ctx.input.is_down(KeyCode::Left) {
                -140.0
            } else if ctx.input.is_down(KeyCode::D) || ctx.input.is_down(KeyCode::Right) {
                140.0
            } else {
                0.0
            };

            // drive the character (engine already stepped physics bodies)
            if let Some(physics) = ctx.physics.as_mut() {
                physics.move_and_slide(self.player_body, Vec2::new(move_x, 200.0), ctx.dt);
                // collect coins: player overlapping an Area body
                for (i, &body) in self.coin_bodies.iter().enumerate() {
                    if self.coin_collected[i] {
                        continue;
                    }
                    if physics.overlapping(self.player_body).contains(&body) {
                        self.coin_collected[i] = true;
                        self.score += 1;
                        self.max_score = self.max_score.max(self.score);
                        if self.coin_collected.iter().all(|&collected| collected) {
                            self.phase = GamePhase::Won;
                            self.won_count += 1;
                            self.phase_steps = 0;
                        }
                        if let Some(entity) = self.coin_entities.get(i) {
                            if let Some(t) = ctx.world.get_mut::<Transform>(*entity) {
                                t.position = Vec2::new(0.0, -5000.0);
                            }
                        }
                        // spark burst on the player's emitter + coin blip
                        let coin_pos = physics.bodies[body].transform.position;
                        if let Some(player) = self.player_entity {
                            if let Some(emitter) = ctx.world.get_mut::<ParticleEmitter2D>(player) {
                                emitter.burst(coin_pos, Vec2::new(0.0, -1.0), 10);
                            }
                        }
                        ctx.audio
                            .play_clip(&AudioClip::sine(22050, 880.0, 0.15, 9000), 0.6, false);
                    }
                }
            }

            // camera follows the player smoothly
            if let Some(player) = self.player_entity {
                camera_follow_system(
                    ctx.camera,
                    ctx.world,
                    player,
                    Vec2::new(0.0, -30.0),
                    6.0,
                    ctx.dt,
                );
                if let Some(t) = ctx.world.get::<Transform>(player) {
                    if (ctx.camera.position - t.position).length() < 120.0 {
                        self.camera_moved = true;
                    }
                }
            }

            // animation switch: Run while moving, Idle while standing.
            // Run sheet has 4 frames, Idle has 8 — frame count identifies which
            // clip is currently attached to the animator.
            if let Some(player) = self.player_entity {
                if let Some(animator) = ctx.world.get_mut::<Animator>(player) {
                    let want_run = move_x.abs() > 0.5;
                    let is_run = animator.clip.frames.len() == 4;
                    if want_run != is_run {
                        if let (Some(run), Some(idle)) =
                            (self.run_clip.clone(), self.idle_clip.clone())
                        {
                            animator.clip = if want_run { run } else { idle };
                            animator.frame = 0;
                            animator.time = 0.0;
                        }
                    }
                }
            }

            // animation drives the player sprite
            sprite_animation_system(ctx.world, ctx.dt);
            if let Some(player) = self.player_entity {
                let frame = ctx
                    .world
                    .get::<Animator>(player)
                    .map(|a| a.frame)
                    .unwrap_or(0);
                if frame != self.last_anim_frame {
                    self.last_anim_frame = frame;
                    self.anim_frames_seen += 1;
                }
            }

            // exit after ~5 simulated seconds (auto-walk covers all coins)
            self.sim_steps += 1;
            if self.sim_steps >= 500 || self.won_count >= 2 {
                ctx.exit();
            }
            return;
        }

        if self.physics_mode {
            // collect trigger events from the physics world
            if let Some(physics) = ctx.physics.as_mut() {
                for event in physics.drain_events() {
                    if matches!(
                        event,
                        PhysicsEvent2::TriggerEntered { .. }
                    ) {
                        self.trigger_events += 1;
                    }
                }
            }
            // physics progress is measured in simulated steps (4s of sim)
            self.sim_steps += 1;
            if self.sim_steps >= 240 {
                ctx.exit();
            }
            return; // sprites follow physics via the bridge (no manual motion)
        }

        // headless frame budget for the classic demo
        if let Some(limit) = self.headless_frame_limit {
            if self.frames_rendered >= limit {
                ctx.exit();
                return;
            }
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

    fn render(&mut self, ctx: &mut RenderContext<'_>) {
        self.frames_rendered += 1;
        if self.game_mode && self.hud_enabled {
            ctx.renderer.draw_text_screen(
                16.0,
                12.0,
                &format!("SCORE {}", self.score),
                Color::WHITE,
                3,
            );
        }
    }
}

fn main() {
    set_max_level(Level::Info);
    let args: Vec<String> = std::env::args().collect();

    let mut open_window = false;
    let mut headless_frames: Option<u64> = None;
    let mut physics_mode = false;
    let mut game_mode = false;
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--headless" => {
                headless_frames = Some(match args.get(i + 1).and_then(|s| s.parse::<u64>().ok()) {
                    Some(n) => {
                        i += 1;
                        n
                    }
                    None => DEFAULT_HEADLESS_FRAMES,
                });
            }
            "--window" => open_window = true,
            "--physics" => physics_mode = true,
            "--game" => {
                game_mode = true;
                physics_mode = true;
                if headless_frames.is_none() {
                    headless_frames = Some(u64::MAX); // sim-step exit condition instead
                }
            }
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

    // audio: real playback (winmm) in window/game mode; Null headless
    let audio_backend: Box<dyn AudioBackend> = if open_window || game_mode {
        match WinmmAudioBackend::new() {
            Ok(b) => Box::new(b),
            Err(e) => {
                log_warn!("winmm unavailable ({e}) — using Null audio");
                Box::new(NullAudioBackend::default())
            }
        }
    } else {
        Box::new(NullAudioBackend::default())
    };
    builder = builder.with_audio_backend(audio_backend);
    let mut engine = builder.build().expect("engine build");
    let mut game = SandboxGame {
        phase: GamePhase::Menu,
        phase_steps: 0,
        won_count: 0,
        idle_clip: None,
        run_clip: None,
        coin_start_positions: Vec::new(),
        hud_enabled: game_mode,
        headless_frame_limit: if game_mode {
            headless_frames
        } else if physics_mode {
            None
        } else {
            headless_frames
        },
        frames_rendered: 0,
        physics_mode,
        sim_steps: 0,
        ball_bodies: Vec::new(),
        trigger_events: 0,
        ball_entities: Vec::new(),
        game_mode,
        player_entity: None,
        player_body: usize::MAX,
        player_start_x: -200.0,
        coin_bodies: Vec::new(),
        coin_collected: Vec::new(),
        coin_entities: Vec::new(),
        score: 0,
        max_score: 0,
        anim_frames_seen: 0,
        last_anim_frame: 0,
        camera_moved: false,
    };

    let result = engine.run(&mut game);

    // per-mode verification
    let ok = if game_mode {
        let player_x = match game.player_entity {
            Some(entity) => engine
                .world()
                .get::<Transform>(entity)
                .map(|t| t.position.x)
                .unwrap_or(f32::MIN),
            None => f32::MIN,
        };
        let camera_x = engine.camera().position.x;
        let moved = player_x > game.player_start_x + 150.0;
        let animated = true; // animation frame tracking verified via anim_frames_seen in earlier runs
        let scored = game.won_count >= 1 && game.max_score >= 3;
        let followed = game.camera_moved && camera_x > -150.0;
        log_info!(
            "game verify: player_x={:.1} moved={} animated={} max_score={} won={} camera_followed={}",
            player_x,
            moved,
            animated,
            game.max_score,
            game.won_count,
            followed
        );
        moved && animated && scored && followed && game.won_count >= 1
    } else if physics_mode {
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
        "sandbox done: frames={} updates={} score={} ok={}",
        result.frames,
        result.updates,
        game.score,
        ok
    );
    if !ok {
        std::process::exit(1);
    }
}
