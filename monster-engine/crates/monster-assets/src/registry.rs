//! [`Assets`] — the resource registry: decode once, cache, hand out typed
//! handles.

use std::collections::HashMap;
use std::fmt;
use std::path::Path;

use monster_core::graphics::Texture;
use monster_core::handle::Handle;

use crate::bmp::decode_bmp;
use crate::png::decode_png;

/// A decoded image (PNG or BMP) before it becomes a [`Texture`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImageData {
    pub width: u32,
    pub height: u32,
    /// Row-major RGBA8.
    pub rgba: Vec<u8>,
}

impl ImageData {
    /// Wrap into an engine texture.
    pub fn into_texture(self) -> Texture {
        Texture::from_rgba(self.width, self.height, self.rgba)
    }
}

/// Asset load/decode failures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AssetError {
    /// File could not be read.
    Io(String),
    /// Extension not recognized.
    UnsupportedFormat(String),
    /// PNG decode failure.
    Png(crate::png::PngError),
    /// BMP decode failure.
    Bmp(crate::bmp::BmpError),
}

impl fmt::Display for AssetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AssetError::Io(what) => write!(f, "asset IO failure: {what}"),
            AssetError::UnsupportedFormat(ext) => write!(f, "unsupported asset format '{ext}'"),
            AssetError::Png(e) => write!(f, "PNG asset failure: {e}"),
            AssetError::Bmp(e) => write!(f, "BMP asset failure: {e}"),
        }
    }
}

/// Registry and cache for textures.
///
/// `load_file` decodes a file once and remembers it by path; later loads of
/// the same path return the same [`Handle`] without re-decoding. Procedural
/// textures register through [`Assets::insert_rgba`] under any key.
#[derive(Debug, Default)]
pub struct Assets {
    /// Key (usually a path) -> storage slot.
    entries: HashMap<String, u32>,
    /// Slot storage; never removed (textures live for the session).
    storage: Vec<Texture>,
}

impl Assets {
    pub fn new() -> Assets {
        Assets::default()
    }

    fn insert(&mut self, key: &str, texture: Texture) -> Handle<Texture> {
        if let Some(&slot) = self.entries.get(key) {
            // replace contents, keep the handle stable
            self.storage[slot as usize] = texture;
            return Handle::from_index(slot);
        }
        let slot = self.storage.len() as u32;
        self.storage.push(texture);
        self.entries.insert(key.to_string(), slot);
        Handle::from_index(slot)
    }

    /// Register a procedural RGBA8 texture under `key`.
    pub fn insert_rgba(
        &mut self,
        key: &str,
        width: u32,
        height: u32,
        rgba: Vec<u8>,
    ) -> Handle<Texture> {
        assert_eq!(rgba.len(), (width as usize) * (height as usize) * 4);
        self.insert(key, Texture::from_rgba(width, height, rgba))
    }

    /// Load an image file (`.png` or `.bmp`), decode it and cache by path.
    /// Repeated loads of the same path return the cached handle.
    pub fn load_file(&mut self, path: &Path) -> Result<Handle<Texture>, AssetError> {
        let key = path.to_string_lossy().into_owned();
        if let Some(&slot) = self.entries.get(&key) {
            return Ok(Handle::from_index(slot));
        }
        let bytes = std::fs::read(path).map_err(|e| AssetError::Io(format!("{path:?}: {e}")))?;
        let ext = path
            .extension()
            .map(|e| e.to_string_lossy().to_ascii_lowercase())
            .unwrap_or_default();
        let image = match ext.as_str() {
            "png" => {
                let img = decode_png(&bytes).map_err(AssetError::Png)?;
                ImageData {
                    width: img.width,
                    height: img.height,
                    rgba: img.rgba,
                }
            }
            "bmp" | "dib" => {
                let img = decode_bmp(&bytes).map_err(AssetError::Bmp)?;
                ImageData {
                    width: img.width,
                    height: img.height,
                    rgba: img.rgba,
                }
            }
            other => return Err(AssetError::UnsupportedFormat(other.to_string())),
        };
        Ok(self.insert(&key, image.into_texture()))
    }

    /// Texture backing a handle.
    pub fn texture(&self, handle: Handle<Texture>) -> Option<&Texture> {
        self.storage.get(handle.index() as usize)
    }

    /// Number of cached textures.
    pub fn len(&self) -> usize {
        self.storage.len()
    }

    pub fn is_empty(&self) -> bool {
        self.storage.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures::FixtureBuilder;

    #[test]
    fn insert_and_retrieve_procedural() {
        let mut assets = Assets::new();
        let h = assets.insert_rgba(
            "checker",
            2,
            2,
            vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
        );
        let tex = assets.texture(h).expect("texture exists");
        assert_eq!(tex.width(), 2);
        assert_eq!(tex.height(), 2);
        assert_eq!(tex.sample(0, 0), [1, 2, 3, 4]);
        assert_eq!(assets.len(), 1);
    }

    #[test]
    fn same_key_returns_stable_handle() {
        let mut assets = Assets::new();
        let h1 = assets.insert_rgba("a", 1, 1, vec![9, 9, 9, 9]);
        let h2 = assets.insert_rgba("a", 1, 1, vec![1, 1, 1, 1]);
        assert_eq!(h1, h2);
        assert_eq!(assets.len(), 1);
        assert_eq!(assets.texture(h1).unwrap().sample(0, 0), [1, 1, 1, 1]);
    }

    #[test]
    fn loads_png_file_and_caches() {
        let png = FixtureBuilder::solid(3, 2, 6, &[12, 34, 56, 255]).to_png();
        let dir = std::env::temp_dir().join("monster_assets_tests");
        std::fs::create_dir_all(&dir).expect("mkdir");
        let path = dir.join("fixture.png");
        std::fs::write(&path, &png).expect("write fixture");

        let mut assets = Assets::new();
        let h = assets.load_file(&path).expect("load png");
        let h2 = assets.load_file(&path).expect("load png again");
        assert_eq!(h, h2);
        let tex = assets.texture(h).expect("texture");
        assert_eq!((tex.width(), tex.height()), (3, 2));
        assert_eq!(tex.sample(2, 1), [12, 34, 56, 255]);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn load_errors_are_typed() {
        let mut assets = Assets::new();
        let missing = assets.load_file(Path::new("definitely/missing/file.png"));
        assert!(matches!(missing, Err(AssetError::Io(_))));

        let bad = Path::new("whatever.unknown-ext");
        let dir = std::env::temp_dir().join("monster_assets_tests");
        std::fs::create_dir_all(&dir).expect("mkdir");
        let path = dir.join("x.unknown-ext");
        std::fs::write(&path, b"junk").expect("write");
        let err = assets.load_file(&path).unwrap_err();
        assert_eq!(
            err,
            AssetError::UnsupportedFormat("unknown-ext".to_string())
        );
        let _ = std::fs::remove_file(&path);
        let _ = bad;
    }
}
