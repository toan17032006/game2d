//! Asset manifest: parse `data/assets.csv`, validate geometry against the
//! real PNG headers, resolve keys to paths.
//!
//! CSV dialect: plain comma-separated, no quoting, no comments — the
//! generator (task extraction script) guarantees simple fields.

use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

/// One runtime asset from `data/assets.csv`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssetEntry {
    /// Stable id, e.g. `units/blue/warrior/warrior_idle`.
    pub key: String,
    /// Top category: units | buildings | terrain | fx | ui.
    pub category: String,
    /// Path relative to the game root, e.g. `assets/units/...png`.
    pub path: String,
    /// Width of one frame in texture pixels.
    pub frame_w: u32,
    /// Height of one frame in texture pixels.
    pub frame_h: u32,
    /// Number of frames in the strip (1 = static image).
    pub frames: u32,
}

/// Parsed manifest with a key lookup index.
#[derive(Debug, Default)]
pub struct Manifest {
    pub entries: Vec<AssetEntry>,
    index: HashMap<String, usize>,
}

/// Result of validating every manifest row against the files on disk.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ValidationReport {
    pub files_checked: usize,
    pub frames_total: u64,
}

impl Manifest {
    /// Parse manifest CSV text (used by `load` and unit tests).
    pub fn parse(text: &str) -> Result<Manifest, String> {
        let mut entries = Vec::new();
        for (line_no, raw) in text.lines().enumerate() {
            if raw.trim().is_empty() {
                continue;
            }
            let fields: Vec<&str> = raw.split(',').collect();
            if line_no == 0 {
                if fields.len() != 6 || fields[0] != "key" || fields[1] != "category" {
                    return Err(format!("assets.csv header mismatch at line 1: {raw}"));
                }
                continue;
            }
            if fields.len() != 6 {
                return Err(format!("assets.csv line {}: expected 6 fields", line_no + 1));
            }
            let frame_w = parse_u32(fields[3], line_no + 1, "frame_w")?;
            let frame_h = parse_u32(fields[4], line_no + 1, "frame_h")?;
            let frames = parse_u32(fields[5], line_no + 1, "frames")?;
            if frame_w == 0 || frame_h == 0 || frames == 0 {
                return Err(format!(
                    "assets.csv line {}: geometry values must be > 0",
                    line_no + 1
                ));
            }
            entries.push(AssetEntry {
                key: fields[0].trim().to_string(),
                category: fields[1].trim().to_string(),
                path: fields[2].trim().to_string(),
                frame_w,
                frame_h,
                frames,
            });
        }
        if entries.is_empty() {
            return Err("assets.csv has no asset rows".to_string());
        }
        let mut index = HashMap::new();
        for (i, e) in entries.iter().enumerate() {
            if e.key.is_empty() {
                return Err("assets.csv contains an empty key".to_string());
            }
            if index.insert(e.key.clone(), i).is_some() {
                return Err(format!("assets.csv duplicate key: {}", e.key));
            }
        }
        Ok(Manifest { entries, index })
    }

    /// Load and parse `data/assets.csv` from the game root.
    pub fn load(game_root: &Path) -> Result<Manifest, String> {
        let path = game_root.join("data").join("assets.csv");
        let text = std::fs::read_to_string(&path)
            .map_err(|e| format!("cannot read {}: {e}", path.display()))?;
        Manifest::parse(&text)
    }

    /// Look up an asset by key.
    pub fn get(&self, key: &str) -> Option<&AssetEntry> {
        self.index.get(key).map(|&i| &self.entries[i])
    }

    /// Absolute path of an entry's file on disk.
    pub fn file_path(&self, entry: &AssetEntry, game_root: &Path) -> PathBuf {
        game_root.join(&entry.path)
    }

    /// Validate every row against the PNG headers on disk:
    /// file exists, is a PNG, `frames * frame_w == width`, `frame_h == height`.
    pub fn validate(&self, game_root: &Path) -> Result<ValidationReport, String> {
        let mut frames_total: u64 = 0;
        for entry in &self.entries {
            let path = self.file_path(entry, game_root);
            let (width, height) = png_dimensions(&path)
                .map_err(|e| format!("{} ({}): {e}", entry.key, path.display()))?;
            if entry.frame_h != height {
                return Err(format!(
                    "{}: manifest frame_h {} != PNG height {height}",
                    entry.key, entry.frame_h
                ));
            }
            if width % entry.frame_w != 0 || width / entry.frame_w != entry.frames {
                return Err(format!(
                    "{}: PNG width {width} does not match frames={} x frame_w={}",
                    entry.key, entry.frames, entry.frame_w
                ));
            }
            frames_total += entry.frames as u64;
        }
        Ok(ValidationReport {
            files_checked: self.entries.len(),
            frames_total,
        })
    }
}

fn parse_u32(text: &str, line: usize, field: &str) -> Result<u32, String> {
    text.trim()
        .parse::<u32>()
        .map_err(|e| format!("assets.csv line {line}: bad {field} '{text}': {e}"))
}

/// Read IHDR dimensions directly (std only; full decode happens via the
/// engine's asset loader at runtime).
pub fn png_dimensions(path: &Path) -> Result<(u32, u32), String> {
    let mut header = [0u8; 24];
    File::open(path)
        .and_then(|mut f| f.read_exact(&mut header))
        .map_err(|e| e.to_string())?;
    const PNG_SIGNATURE: [u8; 8] = [137, 80, 78, 71, 13, 10, 26, 10];
    if header[0..8] != PNG_SIGNATURE {
        return Err("not a PNG file".to_string());
    }
    let width = u32::from_be_bytes([header[16], header[17], header[18], header[19]]);
    let height = u32::from_be_bytes([header[20], header[21], header[22], header[23]]);
    if width == 0 || height == 0 {
        return Err("PNG dimensions are zero".to_string());
    }
    Ok((width, height))
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "\
key,category,path,frame_w,frame_h,frames
units/blue/warrior/warrior_idle,units,assets/units/blue/warrior/warrior_idle.png,192,192,8
fx/fire_01,fx,assets/fx/fire_01.png,64,64,8
ui/icons/icon_01,ui,assets/ui/icons/icon_01.png,64,64,1
";

    #[test]
    fn parses_manifest_and_indexes_keys() {
        let m = Manifest::parse(SAMPLE).expect("parse");
        assert_eq!(m.entries.len(), 3);
        let e = m.get("units/blue/warrior/warrior_idle").expect("key");
        assert_eq!(e.category, "units");
        assert_eq!(e.path, "assets/units/blue/warrior/warrior_idle.png");
        assert_eq!((e.frame_w, e.frame_h, e.frames), (192, 192, 8));
        assert_eq!(m.get("missing"), None);
    }

    #[test]
    fn rejects_bad_rows() {
        assert!(Manifest::parse("key,category,path,frame_w,frame_h,frames\nbad\n").is_err());
        assert!(Manifest::parse("key,category,path,frame_w,frame_h,frames\na,fx,p.png,64,64,0\n").is_err());
        assert!(Manifest::parse("key,category,path,frame_w,frame_h,frames\na,fx,p.png,64,64,8\na,fx,p.png,64,64,8\n").is_err());
        assert!(Manifest::parse("").is_err());
        assert!(Manifest::parse("wrong,header\n").is_err());
    }

    #[test]
    fn validates_geometry_against_real_png() {
        // A 2x2 solid PNG built inline (IHDR only is read by validation).
        let dir = std::env::temp_dir().join("game_manifest_tests");
        std::fs::create_dir_all(&dir).expect("mkdir");
        let png_path = dir.join("fixture.png");
        std::fs::write(&png_path, minimal_png(2, 2)).expect("write");

        let text = format!(
            "key,category,path,frame_w,frame_h,frames\nfx/fixture,fx,{},2,2,1\n",
            png_path.to_string_lossy().replace('\\', "/")
        );
        let m = Manifest::parse(&text).expect("parse");
        let report = m.validate(Path::new("/")).expect("validate");
        assert_eq!(report.files_checked, 1);
        assert_eq!(report.frames_total, 1);

        // Wrong frame count must fail.
        let bad = format!(
            "key,category,path,frame_w,frame_h,frames\nfx/fixture,fx,{},2,2,2\n",
            png_path.to_string_lossy().replace('\\', "/")
        );
        let m = Manifest::parse(&bad).expect("parse");
        assert!(m.validate(Path::new("/")).is_err());
        let _ = std::fs::remove_file(&png_path);
    }

    /// Minimal valid 8-bit RGBA PNG of `w`x`h` pixels (one IDAT, no CRC
    /// shortcuts: real CRC32 computed so a strict decoder accepts it).
    fn minimal_png(w: u32, h: u32) -> Vec<u8> {
        fn chunk(tag: &[u8; 4], data: &[u8]) -> Vec<u8> {
            let mut out = Vec::new();
            out.extend_from_slice(&(data.len() as u32).to_be_bytes());
            out.extend_from_slice(tag);
            out.extend_from_slice(data);
            let mut crc = 0xFFFF_FFFFu32;
            for b in tag.iter().chain(data.iter()) {
                crc ^= *b as u32;
                for _ in 0..8 {
                    crc = if crc & 1 != 0 {
                        (crc >> 1) ^ 0xEDB8_8320
                    } else {
                        crc >> 1
                    };
                }
            }
            out.extend_from_slice(&(!crc).to_be_bytes());
            out
        }
        let mut ihdr = Vec::new();
        ihdr.extend_from_slice(&w.to_be_bytes());
        ihdr.extend_from_slice(&h.to_be_bytes());
        ihdr.extend_from_slice(&[8, 6, 0, 0, 0]); // depth 8, RGBA

        let mut raw = Vec::new();
        for _ in 0..h {
            raw.push(0); // filter none
            for _ in 0..w {
                raw.extend_from_slice(&[255, 0, 0, 255]);
            }
        }
        let mut zlib = vec![0x78, 0x01];
        // stored deflate blocks
        let mut offset = 0;
        loop {
            let take = (raw.len() - offset).min(65_535);
            let last = offset + take == raw.len();
            zlib.push(if last { 1 } else { 0 });
            zlib.extend_from_slice(&(take as u16).to_le_bytes());
            zlib.extend_from_slice(&(!(take as u16)).to_le_bytes());
            zlib.extend_from_slice(&raw[offset..offset + take]);
            offset += take;
            if last {
                break;
            }
        }
        let mut out = Vec::new();
        out.extend_from_slice(&[137, 80, 78, 71, 13, 10, 26, 10]);
        out.extend_from_slice(&chunk(b"IHDR", &ihdr));
        out.extend_from_slice(&chunk(b"IDAT", &zlib));
        out.extend_from_slice(&chunk(b"IEND", &[]));
        out
    }
}
