//! Game data rows: `data/units.csv` and `data/scene.csv`.

use std::collections::HashMap;
use std::path::Path;

/// One unit definition from `data/units.csv`.
#[derive(Debug, Clone, PartialEq)]
pub struct UnitDef {
    pub faction: String,
    pub unit: String,
    /// Manifest key of the idle strip.
    pub idle: String,
    /// Manifest key of the run strip.
    pub run: String,
    /// Demo walk speed in world units/second (not gameplay-balanced).
    pub speed: f32,
    pub frame_w: u32,
    pub frame_h: u32,
    pub fps: f32,
}

impl UnitDef {
    /// Lookup id used by `scene.csv`: `<faction>:<unit>`.
    pub fn id(&self) -> String {
        format!("{}:{}", self.faction, self.unit)
    }
}

/// One spawn row from `data/scene.csv`.
#[derive(Debug, Clone, PartialEq)]
pub struct SceneRow {
    pub kind: SceneKind,
    /// Manifest key (tile/prop/anim) or `<faction>:<unit>` id (unit kinds).
    pub reference: String,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub scale_x: f32,
    pub scale_y: f32,
    pub fps: f32,
    pub src_x: u32,
    pub src_y: u32,
    /// Walk direction for `unit` rows: 1 (right) or -1 (left).
    pub dir: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SceneKind {
    /// Static tile cut from a tileset via `src_x/src_y` (64px grid).
    Tile,
    /// Static full-texture sprite.
    Prop,
    /// Animated strip (loops) at `fps`.
    Anim,
    /// Walking unit: run animation + velocity from `units.csv`.
    Unit,
    /// Idle unit: idle animation, no movement.
    UnitIdle,
    /// Keyboard-controlled player: physics `Character` body via the engine
    /// bridge, WASD/arrow movement, idle/run animation switch.
    Player,
}

impl SceneKind {
    fn parse(text: &str) -> Result<SceneKind, String> {
        match text {
            "tile" => Ok(SceneKind::Tile),
            "prop" => Ok(SceneKind::Prop),
            "anim" => Ok(SceneKind::Anim),
            "unit" => Ok(SceneKind::Unit),
            "unit_idle" => Ok(SceneKind::UnitIdle),
            "player" => Ok(SceneKind::Player),
            other => Err(format!("unknown scene kind '{other}'")),
        }
    }
}

/// Parse `data/units.csv` text.
pub fn parse_units(text: &str) -> Result<Vec<UnitDef>, String> {
    let mut out = Vec::new();
    for (line_no, raw) in text.lines().enumerate() {
        if raw.trim().is_empty() {
            continue;
        }
        let f: Vec<&str> = raw.split(',').collect();
        if line_no == 0 {
            if f.len() != 8 || f[0] != "faction" || f[1] != "unit" {
                return Err(format!("units.csv header mismatch at line 1: {raw}"));
            }
            continue;
        }
        if f.len() != 8 {
            return Err(format!("units.csv line {}: expected 8 fields", line_no + 1));
        }
        let speed = parse_f32(f[4], line_no + 1, "speed")?;
        let frame_w = parse_u32(f[5], line_no + 1, "frame_w")?;
        let frame_h = parse_u32(f[6], line_no + 1, "frame_h")?;
        let fps = parse_f32(f[7], line_no + 1, "fps")?;
        out.push(UnitDef {
            faction: f[0].trim().to_string(),
            unit: f[1].trim().to_string(),
            idle: f[2].trim().to_string(),
            run: f[3].trim().to_string(),
            speed,
            frame_w,
            frame_h,
            fps,
        });
    }
    if out.is_empty() {
        return Err("units.csv has no unit rows".to_string());
    }
    Ok(out)
}

/// Parse `data/scene.csv` text.
pub fn parse_scene(text: &str) -> Result<Vec<SceneRow>, String> {
    let mut out = Vec::new();
    for (line_no, raw) in text.lines().enumerate() {
        if raw.trim().is_empty() {
            continue;
        }
        let f: Vec<&str> = raw.split(',').collect();
        if line_no == 0 {
            if f.len() != 11 || f[0] != "kind" || f[1] != "ref" {
                return Err(format!("scene.csv header mismatch at line 1: {raw}"));
            }
            continue;
        }
        if f.len() != 11 {
            return Err(format!("scene.csv line {}: expected 11 fields", line_no + 1));
        }
        let kind = SceneKind::parse(f[0].trim())
            .map_err(|e| format!("scene.csv line {}: {e}", line_no + 1))?;
        out.push(SceneRow {
            kind,
            reference: f[1].trim().to_string(),
            x: parse_f32(f[2], line_no + 1, "x")?,
            y: parse_f32(f[3], line_no + 1, "y")?,
            z: parse_f32(f[4], line_no + 1, "z")?,
            scale_x: parse_f32(f[5], line_no + 1, "scale_x")?,
            scale_y: parse_f32(f[6], line_no + 1, "scale_y")?,
            fps: parse_f32(f[7], line_no + 1, "fps")?,
            src_x: parse_u32(f[8], line_no + 1, "src_x")?,
            src_y: parse_u32(f[9], line_no + 1, "src_y")?,
            dir: parse_f32(f[10], line_no + 1, "dir")?,
        });
    }
    if out.is_empty() {
        return Err("scene.csv has no scene rows".to_string());
    }
    Ok(out)
}

/// Load unit defs from the game root, indexed by `<faction>:<unit>`.
pub fn load_units(game_root: &Path) -> Result<HashMap<String, UnitDef>, String> {
    let path = game_root.join("data").join("units.csv");
    let text = std::fs::read_to_string(&path)
        .map_err(|e| format!("cannot read {}: {e}", path.display()))?;
    let defs = parse_units(&text)?;
    let mut map = HashMap::new();
    for d in defs {
        if map.insert(d.id(), d.clone()).is_some() {
            return Err(format!("units.csv duplicate unit id: {}", d.id()));
        }
    }
    Ok(map)
}

/// Load scene rows from the game root.
pub fn load_scene(game_root: &Path) -> Result<Vec<SceneRow>, String> {
    let path = game_root.join("data").join("scene.csv");
    let text = std::fs::read_to_string(&path)
        .map_err(|e| format!("cannot read {}: {e}", path.display()))?;
    parse_scene(&text)
}

fn parse_f32(text: &str, line: usize, field: &str) -> Result<f32, String> {
    text.trim()
        .parse::<f32>()
        .map_err(|e| format!("data line {line}: bad {field} '{text}': {e}"))
}

fn parse_u32(text: &str, line: usize, field: &str) -> Result<u32, String> {
    text.trim()
        .parse::<u32>()
        .map_err(|e| format!("data line {line}: bad {field} '{text}': {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_units() {
        let text = "\
faction,unit,idle,run,speed,frame_w,frame_h,fps
blue,warrior,units/blue/warrior/warrior_idle,units/blue/warrior/warrior_run,120,192,192,10
";
        let defs = parse_units(text).expect("parse");
        assert_eq!(defs.len(), 1);
        assert_eq!(defs[0].id(), "blue:warrior");
        assert_eq!(defs[0].idle, "units/blue/warrior/warrior_idle");
        assert_eq!(defs[0].speed, 120.0);
    }

    #[test]
    fn parses_scene_and_kinds() {
        let text = "\
kind,ref,x,y,z,scale_x,scale_y,fps,src_x,src_y,dir
tile,terrain/tileset/tilemap_color1,-448,-238,0,1,1,0,64,64,0
prop,ui/banners/banner,370,-170,100,0.5,0.5,0,0,0,0
anim,fx/fire_01,-330,-55,7,1,1,8,0,0,0
unit,blue:warrior,-400,120,10,1,1,0,0,0,1
unit_idle,purple:archer,250,-150,10,1,1,0,0,0,0
";
        let rows = parse_scene(text).expect("parse");
        assert_eq!(rows.len(), 5);
        assert_eq!(rows[0].kind, SceneKind::Tile);
        assert_eq!((rows[0].src_x, rows[0].src_y), (64, 64));
        assert_eq!(rows[1].scale_x, 0.5);
        assert_eq!(rows[3].kind, SceneKind::Unit);
        assert_eq!(rows[3].dir, 1.0);
        assert_eq!(rows[4].kind, SceneKind::UnitIdle);
    }

    #[test]
    fn parses_player_kind() {
        let text = "\
kind,ref,x,y,z,scale_x,scale_y,fps,src_x,src_y,dir
player,blue:warrior,-200,0,10,1,1,0,0,0,0
";
        let rows = parse_scene(text).expect("parse");
        assert_eq!(rows[0].kind, SceneKind::Player);
        assert_eq!(rows[0].reference, "blue:warrior");
    }

    #[test]
    fn rejects_bad_data() {
        assert!(parse_units("bad,header\n").is_err());
        assert!(parse_units("faction,unit,idle,run,speed,frame_w,frame_h,fps\nblue,warrior,a,b,120,192,192\n").is_err());
        assert!(parse_scene("kind,ref\nunit,blue:warrior\n").is_err());
        assert!(parse_scene(
            "kind,ref,x,y,z,scale_x,scale_y,fps,src_x,src_y,dir\nmagic,x,0,0,0,1,1,0,0,0,0\n"
        )
        .is_err());
    }
}
