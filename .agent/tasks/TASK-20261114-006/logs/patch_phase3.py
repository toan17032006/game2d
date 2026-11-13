import io

p = 'crates/monster-engine/examples/sandbox.rs'
with io.open(p, 'r', encoding='utf-8') as f:
    c = f.read()
changes = 0

# 1. update game branch: phase machine wrapping
old1 = '''        if self.game_mode {
            // input: headless auto-walks right; window uses A/D + arrows
            let move_x = if self.headless_frame_limit.is_some() {'''
new1 = '''        if self.game_mode {
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
            let move_x = if self.headless_frame_limit.is_some() {'''
if old1 in c:
    c = c.replace(old1, new1, 1)
    changes += 1

# 2. animation switch: insert before sprite_animation_system call in game branch
old2 = '''            // animation drives the player sprite
            sprite_animation_system(ctx.world, ctx.dt);
            if let Some(player) = self.player_entity {
                let frame = ctx
                    .world
                    .get::<Animator>(player)
                    .map(|a| a.frame)
                    .unwrap_or(0);'''
new2 = '''            // animation switch: Run while moving, Idle while standing.
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
                    .unwrap_or(0);'''
if old2 in c:
    c = c.replace(old2, new2, 1)
    changes += 1

# 3. win transition after score
old3 = '''                        self.coin_collected[i] = true;
                        self.score += 1;'''
new3 = '''                        self.coin_collected[i] = true;
                        self.score += 1;
                        if self.coin_collected.iter().all(|&collected| collected) {
                            self.phase = GamePhase::Won;
                            self.won_count += 1;
                            self.phase_steps = 0;
                        }'''
if old3 in c:
    c = c.replace(old3, new3, 1)
    changes += 1

# 4. exit condition extended
old4 = '''            self.sim_steps += 1;
            if self.sim_steps >= 300 {
                ctx.exit();
            }
            return;
        }

        if self.physics_mode {'''
new4 = '''            self.sim_steps += 1;
            if self.sim_steps >= 400 || (self.won_count >= 1 && self.phase_steps >= 30) {
                ctx.exit();
            }
            return;
        }

        if self.physics_mode {'''
if old4 in c:
    c = c.replace(old4, new4, 1)
    changes += 1

# 5. HUD per phase
old5 = '''        if self.game_mode && self.hud_enabled {
            ctx.renderer
                .draw_text_screen(16.0, 12.0, &format!("SCORE {}", self.score), Color::WHITE, 3);
        }'''
new5 = '''        if self.game_mode && self.hud_enabled {
            let line = match self.phase {
                GamePhase::Menu => "MONSTER ENGINE - PRESS ENTER".to_string(),
                GamePhase::Playing => format!("SCORE {}", self.score),
                GamePhase::Won => {
                    format!("YOU WIN - SCORE {} - ENTER TO REPLAY", self.score)
                }
            };
            ctx.renderer.draw_text_screen(16.0, 12.0, &line, Color::WHITE, 3);
        }'''
if old5 in c:
    c = c.replace(old5, new5, 1)
    changes += 1

# 6. verify won-based
old6 = '''        let moved = player_x > game.player_start_x + 150.0;
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
new6 = '''        let moved = player_x > game.player_start_x + 150.0;
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
if old6 in c:
    c = c.replace(old6, new6, 1)
    changes += 1

with io.open(p, 'w', encoding='utf-8') as f:
    f.write(c)
print('changes applied:', changes)
