import io

p = 'crates/monster-engine/examples/sandbox.rs'
with io.open(p, 'r', encoding='utf-8') as f:
    c = f.read()

# 1. dual sheets: replace single-sheet match block (fmt-formatted)
old_sheets = '''        // Tiny Swords Warrior_Run sheet (grid-sliced at 192x192 per frame)
        let sheet_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(
            "../../../assest/Tiny Swords (Free Pack)/Units/Blue Units/Warrior/Warrior_Run.png",
        );
        let (clip, tex) = match ctx.assets.load_file(&sheet_path) {
            Ok(handle) => {
                let tex = ctx.assets.texture(handle).cloned().unwrap();
                let dims = (tex.width(), tex.height());
                log_info!("game: Tiny Swords sheet {}x{}", dims.0, dims.1);
                let clip = AnimationClip::from_grid(
                    dims.0 as f32,
                    dims.1 as f32,
                    192.0,
                    192.0,
                    (dims.0 as usize) / 192,
                    10.0,
                    true,
                );
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
        };'''
new_sheets = '''        // Tiny Swords Warrior sheets (grid-sliced at 192x192 per frame):
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
                let run_cols = (run_tex.width() / 192.0) as usize;
                let idle_cols = (idle_tex.width() / 192.0) as usize;
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
        self.idle_clip = Some(idle_clip.clone());'''
assert old_sheets in c, 'sheets block not found'
c = c.replace(old_sheets, new_sheets)

# 2. player spawns idle clip
old_player = '''            .with(Sprite {
                texture: tex,
                size: Vec2::new(96.0, 96.0),
                src: None,
            })
            .with(Animator::new(clip))
            .build();'''
new_player = '''            .with(Sprite {
                texture: tex,
                size: Vec2::new(96.0, 96.0),
                src: None,
            })
            .with(Animator::new(idle_clip))
            .build();'''
assert old_player in c, 'player animator not found'
c = c.replace(old_player, new_player)

# 3. phase starts at Menu after camera positioning
old_cam = '''        ctx.camera.position = Vec2::new(-200.0, 150.0);
        log_info!("game: player + 3 coins + camera follow ready");
    }'''
new_cam = '''        ctx.camera.position = Vec2::new(-200.0, 150.0);
        self.phase = GamePhase::Menu;
        self.phase_steps = 0;
        log_info!("game: player + 3 coins + camera follow ready (press Enter)");
    }'''
assert old_cam in c, 'camera block not found'
c = c.replace(old_cam, new_cam)

# 4. coin start positions
old_coins = '''            self.coin_bodies.push(coin);
            self.coin_collected.push(false);'''
new_coins = '''            self.coin_bodies.push(coin);
            self.coin_collected.push(false);
            self.coin_start_positions.push(Vec2::new(x, 200.0));'''
assert old_coins in c, 'coin push not found'
c = c.replace(old_coins, new_coins)

# 5. update game branch: phase machine wrapping the existing playing logic
old_game = '''        if self.game_mode {
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
'''
new_game = '''        if self.game_mode {
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
                        if let Some(physics) = ctx.physics.as_mut() {
                            self.reset_level(ctx, physics);
                        }
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
'''
assert old_game in c, 'game branch head not found'
c = c.replace(old_game, new_game)

# 6. animation switch after move_and_slide: swap clips by movement
old_anim = '''            // animation drives the player sprite
            sprite_animation_system(ctx.world, ctx.dt);'''
new_anim = '''            // animation switch: Run while moving, Idle while standing
            let moving = move_x.abs() > 0.5;
            if let Some(player) = self.player_entity {
                if let Some(animator) = ctx.world.get_mut::<Animator>(player) {
                    let want_run = moving;
                    let is_run = animator.clip.frames.len() == 4; // run sheet has 4 frames
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
            sprite_animation_system(ctx.world, ctx.dt);'''
assert old_anim in c, 'anim section not found'
c = c.replace(old_anim, new_anim)

# 7. win transition after coin collect (inside the collect loop, after score += 1)
old_score = '''                        self.coin_collected[i] = true;
                        self.score += 1;'''
new_score = '''                        self.coin_collected[i] = true;
                        self.score += 1;
                        if self.coin_collected.iter().all(|&c| c) {
                            self.phase = GamePhase::Won;
                            self.won_count += 1;
                            self.phase_steps = 0;
                        }'''
assert old_score in c, 'score line not found'
c = c.replace(old_score, new_score)

# 8. exit condition: after at least one win cycle
old_exit = '''            self.sim_steps += 1;
            if self.sim_steps >= 300 {
                ctx.exit();
            }
            return;
        }

        if self.physics_mode {'''
new_exit = '''            self.sim_steps += 1;
            if self.sim_steps >= 400 || (self.won_count >= 1 && self.phase_steps >= 30) {
                ctx.exit();
            }
            return;
        }

        if self.physics_mode {'''
assert old_exit in c, 'exit block not found'
c = c.replace(old_exit, new_exit)

# 9. render HUD per phase
old_hud = '''        if self.game_mode && self.hud_enabled {
            ctx.renderer.draw_text_screen(
                16.0,
                12.0,
                &format!("SCORE {}", self.score),
                Color::WHITE,
                3,
            );
        }'''
new_hud = '''        if self.game_mode && self.hud_enabled {
            let line = match self.phase {
                GamePhase::Menu => "MONSTER ENGINE - PRESS ENTER".to_string(),
                GamePhase::Playing => format!("SCORE {}", self.score),
                GamePhase::Won => format!("YOU WIN - SCORE {} - ENTER TO REPLAY", self.score),
            };
            ctx.renderer.draw_text_screen(16.0, 12.0, &line, Color::WHITE, 3);
        }'''
assert old_hud in c, 'hud block not found'
c = c.replace(old_hud, new_hud)

# 10. main verification: won-based
old_verify = '''        let moved = player_x > game.player_start_x + 150.0;
        let animated = game.anim_frames_seen > 2;
        let scored = game.score >= 2;
        let followed = game.camera_moved && camera_x > -150.0;
        log_info!(
            "game verify: player_x={:.1} moved={} animated={} score={} camera_followed={}",
            player_x,
            moved,
            animated,
            game.score,
            followed
        );
        moved && animated && scored && followed'''
new_verify = '''        let moved = player_x > game.player_start_x + 150.0;
        let animated = game.anim_frames_seen > 2;
        let scored = game.won_count >= 1 && game.score >= 2;
        let followed = game.camera_moved && camera_x > -150.0;
        log_info!(
            "game verify: player_x={:.1} moved={} animated={} score={} won={} camera_followed={}",
            player_x,
            moved,
            animated,
            game.score,
            game.won_count,
            followed
        );
        moved && animated && scored && followed && game.won_count >= 1'''
assert old_verify in c, 'verify block not found'
c = c.replace(old_verify, new_verify)

# 11. main: game struct init fields
old_init = '''        game_mode,
        player_entity: None,'''
new_init = '''        game_mode,
        phase: GamePhase::Menu,
        phase_steps: 0,
        won_count: 0,
        idle_clip: None,
        run_clip: None,
        player_entity: None,'''
assert old_init in c, 'struct init not found'
c = c.replace(old_init, new_init)

# 12. coin reset uses stored start positions (reset_level uses coin_start_positions)
# reset_level already references self.coin_start_positions — struct field added above.

with io.open(p, 'w', encoding='utf-8') as f:
    f.write(c)
print('phase machine ok')
