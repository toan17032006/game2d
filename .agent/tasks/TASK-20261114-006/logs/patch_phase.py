import io

p = 'crates/monster-engine/examples/sandbox.rs'
with io.open(p, 'r', encoding='utf-8') as f:
    c = f.read()

# 1. GamePhase enum + struct fields
c = c.replace('''struct SandboxGame {
    /// Headless frame budget: exit after this many rendered frames.
    headless_frame_limit: Option<u64>,''', '''/// Game flow state machine: Menu -> Playing -> Won -> (replay) Playing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GamePhase {
    Menu,
    Playing,
    Won,
}

struct SandboxGame {
    /// Headless frame budget: exit after this many rendered frames.
    headless_frame_limit: Option<u64>,''')

c = c.replace('''    /// Mini-game mode (--game): player + coins + camera follow.
    game_mode: bool,''', '''    /// Mini-game mode (--game): player + coins + camera follow.
    game_mode: bool,
    /// Game flow state machine (game mode only).
    phase: GamePhase,
    phase_steps: u32,
    won_count: u32,
    /// Idle/Run clips for animation switching (game mode).
    idle_clip: Option<AnimationClip>,
    run_clip: Option<AnimationClip>,''')

# 2. init: load idle+run clips in init_game
c = c.replace('''        let (clip, tex) = match ctx.assets.load_file(&sheet_path) {
            Ok(handle) => {
                let tex = ctx.assets.texture(handle).cloned().unwrap();
                let dims = (tex.width(), tex.height());
                log_info!("game: Tiny Swords sheet {}x{}", dims.0, dims.1);
                let clip =
                    AnimationClip::from_grid(dims.0 as f32, dims.1 as f32, 192.0, 192.0, (dims.0 as usize) / 192, 10.0, true);
                (clip, handle)
            }
            Err(e) => {
                log_warn!("game: Tiny Swords sheet unavailable ({e}) — procedural fallback");
                let handle = ctx.assets.insert_rgba(
                    "player_fallback",
                    32,
                    32,
                    vec![146, 110, 205, 255].repeat(32 * 32),
                );
                (AnimationClip::from_grid(128.0, 32.0, 32.0, 32.0, 4, 8.0, true), handle)
            }
        };''', '''        // Run sheet (4 frames) + Idle sheet (8 frames) for animation switching
        let idle_sheet = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(
            "../../../assest/Tiny Swords (Free Pack)/Units/Blue Units/Warrior/Warrior_Idle.png",
        );
        let (run_clip, idle_clip, tex) = match (
            ctx.assets.load_file(&sheet_path),
            ctx.assets.load_file(&idle_sheet),
        ) {
            (Ok(run_handle), Ok(idle_handle)) => {
                let run_tex = ctx.assets.texture(run_handle).unwrap();
                let idle_tex = ctx.assets.texture(idle_handle).unwrap();
                let run_cols = (run_tex.width() / 192.0) as usize;
                let idle_cols = (idle_tex.width() / 192.0) as usize;
                log_info!(
                    "game: Tiny Swords sheets run {}x{} ({} frames), idle {}x{} ({} frames)",
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
            (e, _) | (_, Err(e)) => {
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
        self.idle_clip = Some(idle_clip.clone());''')

# player spawns idle
c = c.replace('''            .with(Sprite { texture: tex, size: Vec2::new(96.0, 96.0), src: None })
            .with(Animator::new(clip))
            .build();
        physics_attach(&mut ctx.world, player_entity, player);
        self.player_entity = Some(player_entity);
        self.player_start_x = -200.0;''', '''            .with(Sprite { texture: tex, size: Vec2::new(96.0, 96.0), src: None })
            .with(Animator::new(idle_clip))
            .build();
        physics_attach(&mut ctx.world, player_entity, player);
        self.player_entity = Some(player_entity);
        self.player_start_x = -200.0;
        self.phase = GamePhase::Menu;''')

# 3. coin start positions stored
c = c.replace('''            self.coin_bodies.push(coin);
                self.coin_collected.push(false);
            let coin_entity = ctx
                .world
                .spawn()
                .with(Transform::at(Vec2::new(x, 200.0)))''', '''            self.coin_bodies.push(coin);
                self.coin_collected.push(false);
                self.coin_start_positions.push(Vec2::new(x, 200.0));
            let coin_entity = ctx
                .world
                .spawn()
                .with(Transform::at(Vec2::new(x, 200.0)))''')

c = c.replace('''    coin_bodies: Vec<usize>,
    coin_collected: Vec<bool>,
    coin_entities: Vec<EntityId>,''', '''    coin_bodies: Vec<usize>,
    coin_collected: Vec<bool>,
    coin_start_positions: Vec<Vec2>,
    coin_entities: Vec<EntityId>,''')

# 4. reset_level helper (append into impl SandboxGame, before closing of make_sprite_texture impl? put inside impl SandboxGame via replace of init_game end)
c = c.replace('''        ctx.camera.position = Vec2::new(-200.0, 150.0);
        log_info!("game: player + 3 coins + camera follow ready");
    }''', '''        ctx.camera.position = Vec2::new(-200.0, 150.0);
        log_info!("game: player + 3 coins + camera follow ready");
    }

    /// Reset the playing field: player back to start, coins restored.
    fn reset_level(&mut self, ctx: &mut UpdateContext<'_>, physics: &mut PhysicsWorld2) {
        if let Some(player) = self.player_entity {
            if let Some(t) = ctx.world.get_mut::<Transform>(player) {
                t.position = Vec2::new(self.player_start_x, 180.0);
            }
            let body = &mut physics.bodies[self.player_body];
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
            if let Some(body) = physics.bodies.get_mut(body) {
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
    }''')

with io.open(p, 'w', encoding='utf-8') as f:
    f.write(c)
print('phase 1 ok')
