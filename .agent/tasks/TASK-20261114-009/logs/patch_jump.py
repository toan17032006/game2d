import io

p = 'main.rs'
with io.open(p, 'r', encoding='utf-8') as f:
    c = f.read()

# 1. struct: add player_vy + on_floor
c = c.replace(
    '''    camera_moved: bool,
    hud_enabled: bool,
}''',
    '''    camera_moved: bool,
    hud_enabled: bool,
    /// Platformer character vertical velocity (managed by game, not physics).
    player_vy: f32,
    /// Whether the player is standing on a floor surface.
    on_floor: bool,
}'''
)

# 2. main init: add fields
c = c.replace(
    '''        camera_moved: false,
    };''',
    '''        camera_moved: false,
        player_vy: 0.0,
        on_floor: false,
    };'''
)

# 3. update: replace move_and_slide + add jump/gravity
old_move = '''            // drive the character (engine already stepped physics bodies)
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
            }'''
new_move = '''            // platformer character controller
            // gravity
            self.player_vy += 980.0 * ctx.dt;
            // jump
            let jump_pressed = ctx.input.was_pressed(KeyCode::Space)
                || ctx.input.was_pressed(KeyCode::W)
                || ctx.input.was_pressed(KeyCode::Up);
            if jump_pressed && self.on_floor {
                self.player_vy = -400.0;
            }
            // clamp terminal velocity (anti-tunneling)
            self.player_vy = self.player_vy.min(500.0);

            if let Some(physics) = ctx.physics.as_mut() {
                let res = physics.move_and_slide(
                    self.player_body,
                    Vec2::new(move_x, self.player_vy),
                    ctx.dt,
                );
                self.on_floor = res.floor;

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
            }'''
assert old_move in c, 'move block not found'
c = c.replace(old_move, new_move, 1)

# 4. platforms in tilemap: add solid tiles at different heights
old_walls = '''        // tilemap walls bounding the play area (x = -900..900 band)
        let wall_tex = Self::make_sprite_texture(ctx.assets, "game_wall", Color::opaque(120, 90, 60));
        let mut walls = Vec::new();
        for x in -960..960i32 {
            if (32..967).contains(&(x + 32)) {
                continue; // leave the play band open below; walls above ground
            }
            walls.push(Tile::Solid);
        }
        let _ = walls;
        let wall_row: Vec<Tile> = (0..30).map(|i| if i < 6 || i > 24 { Tile::Solid } else { Tile::Empty }).collect();
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
        log_info!("game: tilemap walls spawned: {} solids", r.solid_bodies);'''
new_walls = '''        // tilemap: walls + platforms at different heights
        let wall_tex = Self::make_sprite_texture(ctx.assets, "game_wall", Color::opaque(120, 90, 60));
        let plat_tex = Self::make_sprite_texture(ctx.assets, "game_plat", Color::opaque(140, 110, 70));

        // side walls (block player from leaving the play area)
        let wall_row: Vec<Tile> = (0..30)
            .map(|i| if i < 4 || i > 26 { Tile::Solid } else { Tile::Empty })
            .collect();
        let (wr, wpairs) = spawn_tilemap(
            &mut ctx.world,
            physics,
            wall_tex,
            32.0,
            Vec2::new(-480.0, GROUND_Y - 32.0),
            30,
            1,
            &wall_row,
        );
        for (entity, body) in wpairs {
            physics_attach(&mut ctx.world, entity, body);
        }

        // platforms the player can jump onto (3 heights)
        let platforms: Vec<(f32, f32, usize)> = vec![
            (-300.0, GROUND_Y - 64.0, 4), // low platform
            (50.0, GROUND_Y - 96.0, 3),   // mid platform
            (350.0, GROUND_Y - 128.0, 3), // high platform
        ];
        let mut plat_solids = 0;
        for (px, py, tiles_wide) in platforms {
            let tiles: Vec<Tile> = (0..tiles_wide).map(|_| Tile::Solid).collect();
            let (pr, ppairs) = spawn_tilemap(
                &mut ctx.world,
                physics,
                plat_tex,
                32.0,
                Vec2::new(px, py),
                tiles_wide,
                1,
                &tiles,
            );
            for (entity, body) in ppairs {
                physics_attach(&mut ctx.world, entity, body);
            }
            plat_solids += pr.solid_bodies;
        }

        log_info!(
            "game: tilemap walls {} + platforms {} solids",
            wr.solid_bodies,
            plat_solids
        );'''
assert old_walls in c, 'walls block not found'
c = c.replace(old_walls, new_walls, 1)

with io.open(p, 'w', encoding='utf-8') as f:
    f.write(c)
print('jump + platforms ok')
