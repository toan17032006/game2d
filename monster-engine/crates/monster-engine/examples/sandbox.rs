//! Monster Engine sandbox demo.
//!
//! Window mode (default): opens a 960x540 window with a grid of bouncing
//! sprites; ESC or closing the window exits.
//!
//! Headless mode (used by automated verification):
//! `cargo run --example sandbox -- --headless [frames]` runs the full
//! engine loop without an OS window for `[frames]` rendered frames
//! (default 120) and exits 0.

use monster_engine::prelude::*;

const WORLD_HALF_W: f32 = 480.0;
const WORLD_HALF_H: f32 = 270.0;
const SPRITE_SIZE: f32 = 48.0;
const DEFAULT_HEADLESS_FRAMES: u64 = 120;

struct SandboxGame {
    /// Headless frame budget: exit after this many rendered frames.
    headless_frame_limit: Option<u64>,
    frames_rendered: u64,
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

        // Grid of sprites with velocities that bounce off the world bounds.
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

        // Exercise the audio backend (Null backend in headless/CI: mute).
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
        // Headless budget: once enough frames were rendered, stop.
        if let Some(limit) = self.headless_frame_limit {
            if self.frames_rendered >= limit {
                ctx.exit();
                return;
            }
        }

        movement_system(ctx.world, ctx.dt);

        // Bounce sprites off the world bounds: read state first, compute,
        // then write back — keeps the ECS borrows clean.
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
            _ => {}
        }
        i += 1;
    }

    let mut builder = if open_window {
        EngineBuilder::new().with_window(WindowOptions {
            width: 960,
            height: 540,
            title: "Monster Engine Sandbox".to_string(),
        })
    } else {
        EngineBuilder::new().headless()
    };
    builder = builder.with_fixed_fps(60);

    let mut engine = builder.build().expect("engine build");
    let mut game = SandboxGame {
        headless_frame_limit: headless_frames,
        frames_rendered: 0,
    };

    let result = engine.run(&mut game);

    // Headless runs must hit their frame budget; window runs need > 0 frames
    // (they end when the user closes the window).
    let ok = match headless_frames {
        Some(n) => result.frames >= n,
        None => result.frames > 0,
    };
    log_info!(
        "sandbox done: frames={} updates={} elapsed={:.1}s avg_fps={:.1} ok={}",
        result.frames,
        result.updates,
        result.elapsed_secs,
        result.avg_fps,
        ok
    );
    if !ok {
        std::process::exit(1);
    }
}
