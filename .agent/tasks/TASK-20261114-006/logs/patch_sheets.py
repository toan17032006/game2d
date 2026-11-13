import io

p = 'crates/monster-engine/examples/sandbox.rs'
with io.open(p, 'r', encoding='utf-8') as f:
    c = f.read()

# locate region: start comment .. end of Animator::new(clip) line
start_marker = '// Tiny Swords Warrior_Run sheet'
start = c.find(start_marker)
assert start != -1, 'start marker missing'
end_marker = '.with(Animator::new(clip))\n            .build();'
end = c.find(end_marker)
assert end != -1, 'end marker missing'
end += len(end_marker)

new_block = '''// Tiny Swords Warrior sheets (grid-sliced at 192x192 per frame):
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
                log_warn!(
                    "game: Tiny Swords sheets unavailable ({e:?}) — procedural fallback"
                );
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
            .build();'''

c = c[:start] + new_block + c[end:]

with io.open(p, 'w', encoding='utf-8') as f:
    f.write(c)
print('sheets block replaced')
