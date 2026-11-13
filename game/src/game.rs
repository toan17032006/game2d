//! The Tiny Swords demo game: data-driven scene over `monster-engine`.
//!
//! All content comes from `game/data/` (manifest, unit defs, scene rows)
//! and `game/assets/` (the extracted runtime asset set). Init validates
//! the whole manifest against the real PNG headers and loads every asset,
//! so a successful headless run proves the extracted set is usable.
//!
//! The player character is a physics `Character` body (engine physics
//! bridge from main): keyboard input sets body velocity, the engine steps
//! the physics world and syncs the body back into the scene `Transform`;
//! static colliders (buildings + world bounds) block movement.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use monster_engine::prelude::{
    log_error, log_info, movement_system, physics_attach, BodyKind2, ColliderShape2, EntityId,
    GameApp, Handle, InitContext, KeyCode, PhysicsBody2, PhysicsWorld2, Rect, RenderContext,
    Sprite, Texture, Transform, UpdateContext, Vec2, Velocity, ZOrder,
};

use crate::anim::{animation_system, Animation};
use crate::defs::{load_scene, load_units, SceneKind, SceneRow, UnitDef};
use crate::manifest::{Manifest, ValidationReport};
use crate::player::{player_input_from_state, player_system, Autodrive, Player};

/// Tileset cell size in texture pixels (`Tilemap_color*.png` grid,
/// verified visually and against the 576x384 tileset dimensions).
pub const TILE_SIZE: u32 = 64;

/// Walking units wrap at this horizontal bound (beyond the 480px viewport
/// half so they enter/leave off-screen).
const WRAP_X: f32 = 520.0;

/// Viewport half extents (world fits the 960x540 view).
const WORLD_HALF_X: f32 = 480.0;
const WORLD_HALF_Y: f32 = 270.0;

/// Player collider (feet box, smaller than the 192px sprite).
const PLAYER_HALF: Vec2 = Vec2::new(20.0, 14.0);

/// Static collider wall thickness around the world bounds.
const WALL: f32 = 16.0;

/// Everything loaded from `game/data` before the engine starts.
pub struct GameData {
    pub manifest: Manifest,
    pub units: HashMap<String, UnitDef>,
    pub scene: Vec<SceneRow>,
}

impl GameData {
    pub fn load(game_root: &Path) -> Result<GameData, String> {
        let manifest = Manifest::load(game_root)?;
        let units = load_units(game_root)?;
        let scene = load_scene(game_root)?;
        Ok(GameData {
            manifest,
            units,
            scene,
        })
    }
}

/// Run summary for exit-code decisions.
#[derive(Debug, Default)]
pub struct RunSummary {
    pub validation: Option<ValidationReport>,
    pub textures_loaded: usize,
    pub load_errors: usize,
    pub entities_spawned: usize,
    pub animated_entities: usize,
    pub walking_units: usize,
    pub player_spawned: bool,
    pub physics_bodies: usize,
}

pub struct TinySwordsGame {
    headless_frame_limit: Option<u64>,
    autodrive: Option<Autodrive>,
    frames_rendered: u64,
    data: GameData,
    game_root: PathBuf,
    summary: RunSummary,
    player_entity: Option<EntityId>,
    player_start: Vec2,
    autodrive_max_x: f32,
    autodrive_min_x: f32,
    autodrive_max_y: f32,
}

impl TinySwordsGame {
    pub fn new(
        data: GameData,
        game_root: &Path,
        headless_frame_limit: Option<u64>,
        autodrive: bool,
    ) -> TinySwordsGame {
        TinySwordsGame {
            headless_frame_limit,
            autodrive: if autodrive { Some(Autodrive::new()) } else { None },
            frames_rendered: 0,
            data,
            game_root: game_root.to_path_buf(),
            summary: RunSummary::default(),
            player_entity: None,
            player_start: Vec2::ZERO,
            autodrive_max_x: f32::MIN,
            autodrive_min_x: f32::MAX,
            autodrive_max_y: f32::MIN,
        }
    }

    pub fn summary(&self) -> &RunSummary {
        &self.summary
    }

    /// Spawn one scene row into the world.
    /// Returns `(is_animated, is_walking)` for the run summary.
    fn spawn_row(
        &mut self,
        ctx: &mut InitContext<'_>,
        handles: &HashMap<String, Handle<Texture>>,
        row: &SceneRow,
    ) -> Result<(bool, bool), String> {
        let world = &mut ctx.world;
        let physics = ctx
            .physics
            .as_mut()
            .ok_or_else(|| "physics world not enabled (missing with_physics)".to_string())?;
        let is_walking = row.kind == SceneKind::Unit;
        let is_player = row.kind == SceneKind::Player;
        let is_animated = matches!(
            row.kind,
            SceneKind::Anim | SceneKind::Unit | SceneKind::UnitIdle | SceneKind::Player
        );

        match row.kind {
            SceneKind::Tile | SceneKind::Prop | SceneKind::Anim => {
                let entry = self
                    .data
                    .manifest
                    .get(&row.reference)
                    .ok_or_else(|| format!("scene ref '{}' not in manifest", row.reference))?;
                let texture = *handles
                    .get(&row.reference)
                    .ok_or_else(|| format!("scene ref '{}' has no texture", row.reference))?;
                let (size_x, size_y, src) = match row.kind {
                    SceneKind::Tile => (
                        TILE_SIZE as f32 * row.scale_x,
                        TILE_SIZE as f32 * row.scale_y,
                        Some(Rect::new(
                            row.src_x as f32,
                            row.src_y as f32,
                            TILE_SIZE as f32,
                            TILE_SIZE as f32,
                        )),
                    ),
                    // Strips (frames > 1) and statics: frame geometry is the
                    // full image for statics, so this is the correct dst size.
                    _ => (
                        entry.frame_w as f32 * row.scale_x,
                        entry.frame_h as f32 * row.scale_y,
                        None,
                    ),
                };
                let mut builder = world
                    .spawn()
                    .with(Transform::at(Vec2::new(row.x, row.y)))
                    .with(Sprite {
                        texture,
                        size: Vec2::new(size_x, size_y),
                        src,
                    })
                    .with(ZOrder(row.z));
                if row.kind == SceneKind::Anim {
                    builder = builder.with(Animation::new(
                        entry.frame_w,
                        entry.frame_h,
                        entry.frames,
                        row.fps,
                    ));
                }
                let entity = builder.build();

                // Buildings are solid: static collider following the scene
                // (authored kind) so the player bumps into them.
                if row.kind == SceneKind::Prop && row.reference.starts_with("buildings/") {
                    let body = physics.add_body(
                        PhysicsBody2::new(
                            BodyKind2::Static,
                            Vec2::new(row.x, row.y),
                            ColliderShape2::Box {
                                half_extents: Vec2::new(
                                    entry.frame_w as f32 * 0.5 * row.scale_x,
                                    entry.frame_h as f32 * 0.5 * row.scale_y,
                                ),
                            },
                        ),
                    );
                    physics_attach(world, entity, body);
                    self.summary.physics_bodies += 1;
                }
            }
            SceneKind::Unit | SceneKind::UnitIdle | SceneKind::Player => {
                let def = self
                    .data
                    .units
                    .get(&row.reference)
                    .ok_or_else(|| format!("scene ref '{}' not in units.csv", row.reference))?;
                let anim_key = if is_walking { &def.run } else { &def.idle };
                let entry = self
                    .data
                    .manifest
                    .get(anim_key)
                    .ok_or_else(|| format!("unit anim '{}' not in manifest", anim_key))?;
                let texture = *handles
                    .get(anim_key)
                    .ok_or_else(|| format!("unit anim '{}' has no texture", anim_key))?;

                let mut builder = world
                    .spawn()
                    .with(Transform::at(Vec2::new(row.x, row.y)))
                    .with(Sprite {
                        texture,
                        size: Vec2::new(
                            def.frame_w as f32 * row.scale_x,
                            def.frame_h as f32 * row.scale_y,
                        ),
                        src: None,
                    })
                    .with(ZOrder(row.z))
                    .with(Animation::new(
                        entry.frame_w,
                        entry.frame_h,
                        entry.frames,
                        def.fps,
                    ));

                if is_player {
                    // Idle texture starts the player; player_system swaps to
                    // the run sheet when movement input arrives.
                    let run_key = &def.run;
                    let run_entry = self.data.manifest.get(run_key).ok_or_else(|| {
                        format!("player run anim '{}' not in manifest", run_key)
                    })?;
                    let run_sheet = *handles
                        .get(run_key)
                        .ok_or_else(|| format!("player run anim '{}' has no texture", run_key))?;
                    let body = physics.add_body(
                        PhysicsBody2::new(
                            BodyKind2::Character,
                            Vec2::new(row.x, row.y),
                            ColliderShape2::Box {
                                half_extents: PLAYER_HALF,
                            },
                        )
                        .with_gravity_scale(0.0),
                    );
                    self.summary.physics_bodies += 1;
                    builder = builder
                        .with(Player {
                            body,
                            speed: def.speed,
                            idle_sheet: texture,
                            run_sheet,
                            idle_geom: (entry.frame_w, entry.frame_h, entry.frames),
                            run_geom: (
                                run_entry.frame_w,
                                run_entry.frame_h,
                                run_entry.frames,
                            ),
                            fps: def.fps,
                            moving: false,
                        });
                    let entity = builder.build();
                    physics_attach(world, entity, body);
                    log_info!(
                        "player spawned: entity {entity}, physics body {body} at ({}, {})",
                        row.x,
                        row.y
                    );
                } else {
                    if is_walking {
                        builder = builder.with(Velocity(Vec2::new(def.speed * row.dir, 0.0)));
                    }
                    builder.build();
                }
            }
        }
        Ok((is_animated, is_walking))
    }

    /// Add unlinked static walls at the world bounds (scene-less bodies:
    /// the bridge never touches them).
    fn add_world_walls(physics: &mut PhysicsWorld2) {
        let walls = [
            (Vec2::new(0.0, -WORLD_HALF_Y - WALL), Vec2::new(WORLD_HALF_X + WALL, WALL)),
            (Vec2::new(0.0, WORLD_HALF_Y + WALL), Vec2::new(WORLD_HALF_X + WALL, WALL)),
            (Vec2::new(-WORLD_HALF_X - WALL, 0.0), Vec2::new(WALL, WORLD_HALF_Y)),
            (Vec2::new(WORLD_HALF_X + WALL, 0.0), Vec2::new(WALL, WORLD_HALF_Y)),
        ];
        for (position, half_extents) in walls {
            physics.add_body(PhysicsBody2::new(
                BodyKind2::Static,
                position,
                ColliderShape2::Box { half_extents },
            ));
        }
    }
}

impl GameApp for TinySwordsGame {
    fn init(&mut self, ctx: &mut InitContext<'_>) {
        ctx.camera.position = Vec2::ZERO;
        ctx.camera.zoom = 1.0;

        if ctx.physics.is_none() {
            log_error!("physics world missing: build the engine with with_physics(gravity)");
            self.summary.load_errors += 1;
        }

        // 1. Validate the manifest against the real files on disk.
        match self.data.manifest.validate(&self.game_root) {
            Ok(report) => {
                log_info!(
                    "manifest validation PASS: {} assets, {} animation frames total",
                    report.files_checked,
                    report.frames_total
                );
                self.summary.validation = Some(report);
            }
            Err(e) => {
                log_error!("manifest validation FAIL: {e}");
                self.summary.load_errors += 1;
            }
        }

        // 2. Load every extracted asset (proves the whole set decodes).
        let mut handles = HashMap::new();
        let mut load_errors = 0usize;
        for entry in &self.data.manifest.entries {
            let path = self.data.manifest.file_path(entry, &self.game_root);
            match ctx.assets.load_file(&path) {
                Ok(handle) => {
                    handles.insert(entry.key.clone(), handle);
                }
                Err(e) => {
                    log_error!("asset load FAIL: {} -> {e}", entry.key);
                    load_errors += 1;
                }
            }
        }
        self.summary.textures_loaded = handles.len();
        self.summary.load_errors += load_errors;
        log_info!(
            "assets loaded: {}/{} textures",
            handles.len(),
            self.data.manifest.entries.len()
        );

        // 3. Spawn the data-driven scene + physics bodies.
        let rows = self.data.scene.clone();
        let mut spawned = 0usize;
        let mut animated = 0usize;
        let mut walking = 0usize;
        for row in &rows {
            match self.spawn_row(ctx, &handles, row) {
                Ok((row_animated, row_walking)) => {
                    spawned += 1;
                    if row_animated {
                        animated += 1;
                    }
                    if row_walking {
                        walking += 1;
                    }
                }
                Err(e) => {
                    log_error!("scene spawn FAIL: {e}");
                    self.summary.load_errors += 1;
                }
            }
        }
        self.summary.entities_spawned = spawned;
        self.summary.animated_entities = animated;
        self.summary.walking_units = walking;
        log_info!(
            "scene spawned: {} entities ({} animated, {} walking units, {} physics bodies)",
            spawned,
            animated,
            walking,
            self.summary.physics_bodies
        );

        // 4. World bounds + remember the player for the update loop.
        if let Some(physics) = ctx.physics.as_mut() {
            Self::add_world_walls(physics);
        }
        self.player_entity = ctx
            .world
            .entities()
            .find_map(|id| ctx.world.get::<Player>(id).map(|_| id));
        self.summary.player_spawned = self.player_entity.is_some();
        if let Some(player_id) = self.player_entity {
            if let Some(t) = ctx.world.get::<Transform>(player_id) {
                self.player_start = t.position;
                self.autodrive_max_x = t.position.x;
                self.autodrive_min_x = t.position.x;
                self.autodrive_max_y = t.position.y;
            }
        } else {
            log_error!("no player entity spawned (scene.csv needs a 'player' row)");
            self.summary.load_errors += 1;
        }
    }

    fn update(&mut self, ctx: &mut UpdateContext<'_>) {
        if ctx.input.was_pressed(KeyCode::Escape) {
            ctx.exit();
            return;
        }
        // Track autodrive displacement while the pattern runs.
        if self.autodrive.is_some() {
            if let Some(player_id) = self.player_entity {
                if let Some(t) = ctx.world.get::<Transform>(player_id) {
                    self.autodrive_max_x = self.autodrive_max_x.max(t.position.x);
                    self.autodrive_min_x = self.autodrive_min_x.min(t.position.x);
                    self.autodrive_max_y = self.autodrive_max_y.max(t.position.y);
                }
            }
        }
        // Headless budget: exit after enough frames AND (autodrive) after
        // the scripted pattern has finished.
        if let Some(limit) = self.headless_frame_limit {
            let pattern_done = self.autodrive.as_ref().map_or(true, |a| a.done);
            if self.frames_rendered >= limit && pattern_done {
                if let (Some(player_id), Some(_)) = (self.player_entity, self.autodrive.as_ref()) {
                    if let Some(t) = ctx.world.get::<Transform>(player_id) {
                        log_info!(
                            "autodrive result: start=({:.0}, {:.0}) min_x={:.0} max_x={:.0} max_y={:.0} end=({:.0}, {:.0})",
                            self.player_start.x,
                            self.player_start.y,
                            self.autodrive_min_x,
                            self.autodrive_max_x,
                            self.autodrive_max_y,
                            t.position.x,
                            t.position.y
                        );
                    }
                }
                ctx.exit();
                return;
            }
        }

        // Player: input -> Character body velocity (physics-owned body; the
        // engine syncs its transform into the scene every fixed step).
        match ctx.physics.as_mut() {
            Some(physics) => {
                let input = match self.autodrive.as_mut() {
                    Some(ad) => ad.tick(ctx.dt),
                    None => player_input_from_state(ctx.input),
                };
                player_system(ctx.world, physics, input);
            }
            None => {
                log_error!("physics world missing at update: player cannot move");
                self.summary.load_errors += 1;
            }
        }

        // NPCs: ECS movement + wrap + animations (player has no Velocity,
        // so movement_system leaves it alone).
        movement_system(ctx.world, ctx.dt);
        ctx.world.for_each2::<Transform, Velocity, _>(
            |_id, transform, velocity| {
                if velocity.0.x != 0.0 {
                    if transform.position.x > WRAP_X {
                        transform.position.x = -WRAP_X;
                    } else if transform.position.x < -WRAP_X {
                        transform.position.x = WRAP_X;
                    }
                }
            },
        );
        animation_system(ctx.world, ctx.dt);
    }

    fn render(&mut self, _ctx: &mut RenderContext<'_>) {
        self.frames_rendered += 1;
    }
}

/// Decide whether the run proved the asset set works — and (autodrive)
/// that the player really moves.
pub fn run_succeeded(app: &TinySwordsGame, frames: u64, headless_limit: Option<u64>) -> bool {
    let summary = app.summary();
    let frames_ok = match headless_limit {
        Some(n) => frames >= n,
        None => frames > 0,
    };
    let validated = summary.validation.map_or(0, |r| r.files_checked);
    let base = frames_ok
        && validated > 0
        && summary.load_errors == 0
        && summary.textures_loaded == validated
        && summary.entities_spawned > 0
        && summary.player_spawned;
    // Autodrive evidence: the player reached both east and west walls and
    // moved meaningfully down from its start row.
    let autodrive_ok = match app.autodrive {
        Some(_) => {
            app.autodrive_max_x > 400.0
                && app.autodrive_min_x < -400.0
                && (app.autodrive_max_y - app.player_start.y) > 40.0
        }
        None => true,
    };
    let ok = base && autodrive_ok;
    if !ok {
        log_error!(
            "run FAILED: frames_ok={frames_ok} validated={validated} load_errors={} textures={} spawned={} player={} autodrive(min_x={:.1} max_x={:.1} max_y={:.1} start_y={:.1})",
            summary.load_errors,
            summary.textures_loaded,
            summary.entities_spawned,
            summary.player_spawned,
            app.autodrive_min_x,
            app.autodrive_max_x,
            app.autodrive_max_y,
            app.player_start.y
        );
    }
    ok
}

/// Load all `game/data` inputs (convenience for `main`).
pub fn load_game_data(game_root: &Path) -> Result<GameData, String> {
    GameData::load(game_root)
}
