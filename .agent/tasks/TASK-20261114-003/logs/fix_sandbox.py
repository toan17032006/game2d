import io

p = 'crates/monster-engine/examples/sandbox.rs'
with io.open(p, 'r', encoding='utf-8') as f:
    c = f.read()

# fix init_game texture usage (ground)
c = c.replace('''            let ground_entity = ctx
                .world
                .spawn()
                .with(Transform::at(Vec2::new(0.0, 240.0)))
                .with(Sprite { texture: handles_placeholder(self), size: Vec2::new(2000.0, 20.0), src: None })
                .build();''', '''            let ground_tex = Self::make_sprite_texture(ctx.assets, "game_ground", Color::opaque(90, 130, 90));
            let ground_entity = ctx
                .world
                .spawn()
                .with(Transform::at(Vec2::new(0.0, 240.0)))
                .with(Sprite { texture: ground_tex, size: Vec2::new(2000.0, 20.0), src: None })
                .build();''')

# player start bookkeeping
c = c.replace('''        physics_attach(&mut ctx.world, player_entity, player);
        self.player_entity = Some(player_entity);''', '''        physics_attach(&mut ctx.world, player_entity, player);
        self.player_entity = Some(player_entity);
        self.player_start_x = -200.0;''')

# coin texture + collected flag push
c = c.replace('''            let coin_entity = ctx
                .world
                .spawn()
                .with(Transform::at(Vec2::new(x, 170.0)))
                .with(Sprite { texture: handles_texture(self), size: Vec2::splat(24.0), src: None })
                .build();''', '''            let coin_tex = Self::make_sprite_texture(
                ctx.assets,
                &format!("game_coin_{}", x as i32),
                Color::opaque(230, 200, 60),
            );
            let coin_entity = ctx
                .world
                .spawn()
                .with(Transform::at(Vec2::new(x, 170.0)))
                .with(Sprite { texture: coin_tex, size: Vec2::splat(24.0), src: None })
                .build();''')
c = c.replace('''                self.coin_bodies.push(coin);
            let coin_entity''', '''                self.coin_bodies.push(coin);
                self.coin_collected.push(false);
            let coin_entity''')
c = c.replace('''                self.coin_bodies.push(coin);
            let coin_entity''', '''                self.coin_bodies.push(coin);
                self.coin_collected.push(false);
            let coin_entity''')

# remove placeholder helpers
c = c.replace('''fn handles_placeholder(_: &SandboxGame) -> Handle<Texture> {
    Handle::from_index(0)
}

fn handles_texture(_: &SandboxGame) -> Handle<Texture> {
    Handle::from_index(0)
}''', '')

# update(): game branch — insert before the physics_mode branch's closing return
c = c.replace('''            return; // sprites follow physics via the bridge (no manual motion)
        }''', '''            return; // sprites follow physics via the bridge (no manual motion)
        }

        if self.game_mode {
            // input: headless auto-walks right; window uses A/D + arrows
            let move_x = if self.headless_frame_limit.is_some() {
                140.0
            } else if ctx.input.is_down(KeyCode::KeyA) || ctx.input.is_down(KeyCode::Left) {
                -140.0
            } else if ctx.input.is_down(KeyCode::KeyD) || ctx.input.is_down(KeyCode::Right) {
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
                        if let Some(entity) = self.coin_entities.get(i) {
                            if let Some(t) = ctx.world.get_mut::<Transform>(*entity) {
                                t.position = Vec2::new(0.0, -5000.0);
                            }
                        }
                    }
                }
            }

            // camera follows the player smoothly
            if let Some(player) = self.player_entity {
                camera_follow_system(ctx.camera, ctx.world, player, Vec2::new(0.0, -30.0), 6.0, ctx.dt);
                if let Some(t) = ctx.world.get::<Transform>(player) {
                    if (ctx.camera.position - t.position).length() < 120.0 {
                        self.camera_moved = true;
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

            self.sim_steps += 1;
            if self.sim_steps >= 300 {
                ctx.exit();
            }
            return;
        }''')

# struct fields
c = c.replace('''    /// Mini-game mode (--game): player + coins + camera follow.
    game_mode: bool,
    player_entity: Option<EntityId>,
    player_body: usize,
    player_start_x: f32,
    coin_bodies: Vec<usize>,
    coin_entities: Vec<EntityId>,
    score: u32,
    anim_frames_seen: u32,
    last_anim_frame: usize,
    camera_moved: bool,
}''', '''    /// Mini-game mode (--game): player + coins + camera follow.
    game_mode: bool,
    player_entity: Option<EntityId>,
    player_body: usize,
    player_start_x: f32,
    coin_bodies: Vec<usize>,
    coin_collected: Vec<bool>,
    coin_entities: Vec<EntityId>,
    score: u32,
    anim_frames_seen: u32,
    last_anim_frame: usize,
    camera_moved: bool,
}''')
c = c.replace('''                self.coin_bodies.push(coin);
            let coin_entity''', '''                self.coin_bodies.push(coin);
                self.coin_collected.push(false);
            let coin_entity''')

# main: --game flag
c = c.replace('''    let mut open_window = false;
    let mut headless_frames: Option<u64> = None;
    let mut physics_mode = false;
    let mut i = 1;''', '''    let mut open_window = false;
    let mut headless_frames: Option<u64> = None;
    let mut physics_mode = false;
    let mut game_mode = false;
    let mut i = 1;''')
c = c.replace('''            "--window" => open_window = true,
            "--physics" => physics_mode = true,''', '''            "--window" => open_window = true,
            "--physics" => physics_mode = true,
            "--game" => {
                game_mode = true;
                physics_mode = true;
                open_window = false;
                if headless_frames.is_none() {
                    headless_frames = Some(u64::MAX);
                }
            }''')
c = c.replace('''    let mut game = SandboxGame {
        headless_frame_limit: if physics_mode { None } else { headless_frames },
        frames_rendered: 0,
        physics_mode,
        sim_steps: 0,
        ball_bodies: Vec::new(),
        trigger_events: 0,
        ball_entities: Vec::new(),
    };''', '''    let mut game = SandboxGame {
        headless_frame_limit: if physics_mode || game_mode { None } else { headless_frames },
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
        anim_frames_seen: 0,
        last_anim_frame: 0,
        camera_moved: false,
    };''')

# main verification: game branch
c = c.replace('''    let ok = if physics_mode {''', '''    let ok = if game_mode {
        let player_x = engine
            .world()
            .get::<Transform>(game.player_entity.unwrap_or(EntityId {
                index: u32::MAX,
                generation: u32::MAX,
            }))
            .map(|t| t.position.x)
            .unwrap_or(f32::MIN);
        let camera_x = engine.camera().position.x;
        let moved = player_x > game.player_start_x + 150.0;
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
        moved && animated && scored && followed
    } else if physics_mode {''')

with io.open(p, 'w', encoding='utf-8') as f:
    f.write(c)
print('sandbox game mode ok')
