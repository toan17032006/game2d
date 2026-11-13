//! `tiny-swords` — the game built on the extracted Tiny Swords asset set.
//!
//! Window mode (`--window`): opens a 960x540 window showing the data-driven
//! demo scene; the player (blue warrior) is controlled with WASD or the
//! arrow keys; ESC or closing the window exits.
//!
//! Headless mode (automated verification):
//! `tiny-swords --headless [frames]` runs the full engine loop without an
//! OS window, validates and loads the entire asset manifest, then exits 0
//! on success, 1 on any validation/load/spawn failure.
//!
//! `tiny-swords --headless [frames] --autodrive` additionally drives the
//! player through a scripted input sequence (right into the east wall,
//! down, left into the west wall, up) and requires real displacement in
//! the run result — automated proof of the control pipeline.

mod anim;
mod defs;
mod game;
mod manifest;
mod player;

use std::path::Path;

use monster_engine::prelude::{
    log_error, log_info, set_max_level, EngineBuilder, Level, Vec2, WindowOptions,
};

use game::{run_succeeded, TinySwordsGame};

const DEFAULT_HEADLESS_FRAMES: u64 = 120;

fn main() {
    set_max_level(Level::Info);

    let args: Vec<String> = std::env::args().collect();
    let mut open_window = false;
    let mut headless_frames: Option<u64> = None;
    let mut autodrive = false;
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
            "--autodrive" => autodrive = true,
            other => {
                log_error!("unknown argument '{other}' (use --window, --headless [frames] or --autodrive)");
                std::process::exit(2);
            }
        }
        i += 1;
    }
    if open_window && headless_frames.is_some() {
        log_error!("--window and --headless are mutually exclusive");
        std::process::exit(2);
    }
    if autodrive && open_window {
        log_error!("--autodrive only works headless (it scripts the input)");
        std::process::exit(2);
    }

    // Asset/data root: the crate directory, independent of the CWD.
    let game_root = Path::new(env!("CARGO_MANIFEST_DIR"));

    let data = match game::load_game_data(game_root) {
        Ok(data) => data,
        Err(e) => {
            log_error!("game data load FAIL: {e}");
            std::process::exit(1);
        }
    };
    log_info!(
        "game data loaded: {} manifest entries, {} unit defs, {} scene rows",
        data.manifest.entries.len(),
        data.units.len(),
        data.scene.len()
    );

    let mut builder = if open_window {
        EngineBuilder::new().with_window(WindowOptions {
            width: 960,
            height: 540,
            title: "Tiny Swords — monster-engine demo".to_string(),
        })
    } else {
        EngineBuilder::new().headless()
    };
    builder = builder
        .with_fixed_fps(60)
        // Physics world drives the player character (gravity-free top-down).
        .with_physics(Vec2::ZERO);

    let mut engine = builder.build().expect("engine build");
    let mut app = TinySwordsGame::new(data, game_root, headless_frames, autodrive);

    let result = engine.run(&mut app);

    let ok = run_succeeded(&app, result.frames, headless_frames);
    log_info!(
        "tiny-swords done: frames={} updates={} elapsed={:.1}s avg_fps={:.1} ok={}",
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
